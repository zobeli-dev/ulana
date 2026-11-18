//! Running a Luau view in a sandbox.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

use mlua::{Function, Lua, LuaOptions, StdLib, Table, Value, Variadic, VmState};

use crate::block::Frame;
use crate::check::{check, read, Host, Limits};
use crate::convert::{from_json, to_json};
use crate::error::{Error, Result};

const PRELUDE: &str = include_str!("prelude.luau");

/// Lines of `print` kept between [`View::take_logs`] calls, and characters per line.
const LOG_LINES: usize = 200;
const LOG_LINE_CHARS: usize = 500;

/// Globals a view never gets: loading code, changing environments, and the collector.
const REMOVED: &[&str] = &[
    "require",
    "loadstring",
    "getfenv",
    "setfenv",
    "collectgarbage",
    "gcinfo",
];

/// What a view keeps between renders, by the name given to `ui.state`.
///
/// It is plain JSON, so a host can save it and pass it back later
/// with [`View::with_state`].
pub type State = BTreeMap<String, serde_json::Value>;

/// One Luau view: its script, its state, and the latest frame's handlers.
///
/// A view is not `Send`. Make one where it is used (a server can make one
/// per request, from the saved state).
pub struct View {
    engine: Engine,
    host: Host,
    limits: Limits,
    state: Rc<RefCell<State>>,
    logs: Rc<RefCell<Vec<String>>>,
    version: u64,
}

impl View {
    /// Load a script with empty state.
    pub fn new(source: &str, host: Host, limits: Limits) -> Result<Self> {
        Self::with_state(source, host, limits, State::new())
    }

    /// Load a script with state saved earlier.
    pub fn with_state(source: &str, host: Host, limits: Limits, state: State) -> Result<Self> {
        let state = Rc::new(RefCell::new(state));
        let logs = Rc::new(RefCell::new(Vec::new()));
        let engine = Engine::load(source, &limits, state.clone(), logs.clone())?;
        Ok(View {
            engine,
            host,
            limits,
            state,
            logs,
            version: 0,
        })
    }

    /// Swap in a new script and keep the state. On an error the old script stays.
    ///
    /// Events for frames from before the reload are refused as stale.
    pub fn reload(&mut self, source: &str) -> Result<()> {
        let engine = Engine::load(source, &self.limits, self.state.clone(), self.logs.clone())?;
        self.engine = engine;
        self.version += 1;
        Ok(())
    }

    /// Run the view and check the tree it returns. `data` is passed to the
    /// view's function; use `null` for none.
    pub fn render(&mut self, data: &serde_json::Value) -> Result<Frame> {
        let value = self.engine.render(data, self.limits.time)?;
        let mut json = to_json(&value, "root")?;
        if json.is_array() {
            json = serde_json::json!({ "type": "stack", "children": json });
        }
        let block = read(json)?;
        let engine = &self.engine;
        let root = check(block, &self.host, &self.limits, &|id| {
            engine.has_handler(id)
        })?;
        self.version += 1;
        Ok(Frame {
            version: self.version,
            root,
        })
    }

    /// Run the handler an `event` action named in frame `version`, with the
    /// value the host sends (the new value of a toggle, or `null`). Render
    /// again afterwards to see the change.
    pub fn event(&mut self, version: u64, id: &str, value: &serde_json::Value) -> Result<()> {
        if version != self.version {
            return Err(Error::Stale {
                latest: self.version,
            });
        }
        self.engine.event(id, value, self.limits.time)
    }

    /// The version of the latest frame, or 0 before the first render.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// A copy of the view's state, to save.
    pub fn state(&self) -> State {
        self.state.borrow().clone()
    }

    /// What the script printed since the last call.
    pub fn take_logs(&mut self) -> Vec<String> {
        std::mem::take(&mut *self.logs.borrow_mut())
    }
}

/// When the running call must stop.
#[derive(Default)]
struct Clock {
    deadline: Cell<Option<Instant>>,
    expired: Cell<bool>,
}

struct Engine {
    lua: Lua,
    /// The view's function, or `None` when the script returned a tree.
    render: Option<Function>,
    tree: Option<Table>,
    begin: Function,
    handler: Function,
    clock: Rc<Clock>,
}

impl Engine {
    fn load(
        source: &str,
        limits: &Limits,
        state: Rc<RefCell<State>>,
        logs: Rc<RefCell<Vec<String>>>,
    ) -> Result<Self> {
        let libs =
            StdLib::TABLE | StdLib::STRING | StdLib::UTF8 | StdLib::BIT | StdLib::MATH | StdLib::OS;
        let lua = Lua::new_with(libs, LuaOptions::new()).map_err(setup)?;
        let globals = lua.globals();
        for name in REMOVED {
            globals.raw_set(*name, Value::Nil).map_err(setup)?;
        }
        globals
            .raw_set("print", print(&lua, logs).map_err(setup)?)
            .map_err(setup)?;

        let (get, set, init) = state_functions(&lua, state, limits.state).map_err(setup)?;
        let (ui, internal): (Table, Table) = lua
            .load(PRELUDE)
            .set_name("=ulana")
            .call((get, set, init))
            .map_err(setup)?;
        globals.raw_set("ui", ui).map_err(setup)?;
        let begin: Function = internal.raw_get("begin").map_err(setup)?;
        let handler: Function = internal.raw_get("handler").map_err(setup)?;

        // From here on, globals and libraries are read-only and the script's
        // own globals live in a table of its own.
        lua.sandbox(true).map_err(setup)?;
        let clock = Rc::new(Clock::default());
        let watch = clock.clone();
        lua.set_interrupt(move |_| match watch.deadline.get() {
            Some(deadline) if Instant::now() >= deadline => {
                watch.expired.set(true);
                Err(mlua::Error::RuntimeError("time limit".into()))
            }
            _ => Ok(VmState::Continue),
        });
        lua.set_memory_limit(limits.memory).map_err(setup)?;

        let mut engine = Engine {
            lua,
            render: None,
            tree: None,
            begin,
            handler,
            clock,
        };
        let value: Value = engine.timed(limits.time, |lua| {
            lua.load(source).set_name("=view").eval::<Value>()
        })?;
        match value {
            Value::Function(f) => engine.render = Some(f),
            Value::Table(t) => engine.tree = Some(t),
            other => {
                return Err(Error::invalid(
                    "",
                    format!(
                        "the script must return a function or a block, not {}",
                        other.type_name()
                    ),
                ))
            }
        }
        Ok(engine)
    }

    fn render(&self, data: &serde_json::Value, time: Duration) -> Result<Value> {
        self.timed(time, |lua| match (&self.render, &self.tree) {
            (Some(render), _) => {
                self.begin.call::<()>(())?;
                render.call::<Value>(from_json(lua, data)?)
            }
            // A tree returned at load keeps the handlers it made then.
            (None, Some(tree)) => Ok(Value::Table(tree.clone())),
            (None, None) => unreachable!("load sets one"),
        })
    }

    fn has_handler(&self, id: &str) -> bool {
        matches!(self.handler.call::<Value>(id), Ok(Value::Function(_)))
    }

    fn event(&self, id: &str, value: &serde_json::Value, time: Duration) -> Result<()> {
        let Ok(Value::Function(f)) = self.handler.call::<Value>(id) else {
            return Err(Error::UnknownEvent(id.to_string()));
        };
        self.timed(time, |lua| f.call::<()>(from_json(lua, value)?))
    }

    /// Run `f` with the clock armed, and turn what went wrong into an [`Error`].
    fn timed<T>(&self, time: Duration, f: impl FnOnce(&Lua) -> mlua::Result<T>) -> Result<T> {
        self.clock.expired.set(false);
        self.clock.deadline.set(Some(Instant::now() + time));
        let result = f(&self.lua);
        self.clock.deadline.set(None);
        result.map_err(|e| {
            if self.clock.expired.get() {
                Error::Timeout
            } else {
                from_lua(e)
            }
        })
    }
}

fn setup(e: mlua::Error) -> Error {
    Error::Runtime(format!("could not set up the sandbox: {e}"))
}

fn from_lua(e: mlua::Error) -> Error {
    match e {
        mlua::Error::SyntaxError { message, .. } => Error::Syntax(message),
        mlua::Error::MemoryError(_) => Error::Memory,
        mlua::Error::RuntimeError(m) if m.contains("not enough memory") => Error::Memory,
        mlua::Error::RuntimeError(m) => Error::Runtime(m),
        mlua::Error::CallbackError { cause, .. } => from_lua((*cause).clone()),
        mlua::Error::ExternalError(ref inner) => match inner.downcast_ref::<Error>() {
            Some(ours) => ours.clone(),
            None => Error::Runtime(inner.to_string()),
        },
        other => Error::Runtime(other.to_string()),
    }
}

fn print(lua: &Lua, logs: Rc<RefCell<Vec<String>>>) -> mlua::Result<Function> {
    lua.create_function(move |_, args: Variadic<Value>| {
        let line: Vec<String> = args
            .iter()
            .map(|v| match v {
                Value::Nil => "nil".to_string(),
                Value::Boolean(b) => b.to_string(),
                Value::Integer(i) => i.to_string(),
                Value::Number(n) => n.to_string(),
                Value::String(s) => s.to_string_lossy(),
                other => other.type_name().to_string(),
            })
            .collect();
        let line: String = line.join("\t").chars().take(LOG_LINE_CHARS).collect();
        let mut logs = logs.borrow_mut();
        if logs.len() < LOG_LINES {
            logs.push(line);
        }
        Ok(())
    })
}

fn state_functions(
    lua: &Lua,
    state: Rc<RefCell<State>>,
    limit: usize,
) -> mlua::Result<(Function, Function, Function)> {
    let read = state.clone();
    let get = lua.create_function(move |lua, name: String| match read.borrow().get(&name) {
        Some(v) => from_json(lua, v),
        None => Ok(Value::Nil),
    })?;

    let save = move |state: &RefCell<State>, name: String, value: Value| -> mlua::Result<()> {
        let json = to_json(&value, &format!("state.{name}")).map_err(mlua::Error::external)?;
        let mut state = state.borrow_mut();
        let old = state.insert(name.clone(), json);
        let size = serde_json::to_vec(&*state)
            .map(|v| v.len())
            .unwrap_or(usize::MAX);
        if size > limit {
            match old {
                Some(old) => state.insert(name.clone(), old),
                None => state.remove(&name),
            };
            return Err(mlua::Error::external(Error::invalid(
                format!("state.{name}"),
                format!("state may hold at most {limit} bytes of JSON"),
            )));
        }
        Ok(())
    };

    let write = state.clone();
    let set =
        lua.create_function(move |_, (name, value): (String, Value)| save(&write, name, value))?;

    let first = state;
    let init = lua.create_function(move |_, (name, value): (String, Value)| {
        if first.borrow().contains_key(&name) || value.is_nil() {
            return Ok(());
        }
        save(&first, name, value)
    })?;
    Ok((get, set, init))
}
