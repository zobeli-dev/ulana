//! `ulana serve`: run every view in a folder, reload one when its file
//! changes, and hand frames to apps over HTTP.
//!
//! One thread owns every view (a [`View`] is not `Send`). Requests that wait
//! for a change (long polling) are parked and answered when the view's
//! revision moves, or after [`WAIT`].
//!
//! With `--edit`, a client may also write a view's script
//! (`PUT /views/<name>/source`), which is how an app lets a model change
//! its own interface. Without it the folder is read-only.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use serde::{Deserialize, Serialize};
use tiny_http::{Header, Method, Request, Response, Server};
use ulana::{Error, Frame, Host, Limits, View};

/// How long a request for a change waits before answering with what there is.
const WAIT: Duration = Duration::from_secs(25);
/// How often the folder is checked for changes.
const SCAN: Duration = Duration::from_millis(250);
/// Lines of `print` kept per view.
const LOG_LINES: usize = 50;
/// The largest event body read.
const BODY_LIMIT: u64 = 64 * 1024;
/// The largest script a client may write.
const SOURCE_LIMIT: u64 = 256 * 1024;
/// The API a view sees, for clients that ask a model to write one.
const SPEC: &str = include_str!("../../../spec/ulana.md");

pub struct Options {
    pub dir: PathBuf,
    pub bind: String,
    pub host: Host,
    /// Let clients write scripts.
    pub edit: bool,
}

/// What an app gets for one view.
#[derive(Serialize)]
struct Snapshot<'a> {
    name: &'a str,
    /// Goes up whenever the frame, the error, or the logs change.
    revision: u64,
    /// The last frame that rendered. It stays while the script has an error,
    /// so the app keeps showing something.
    frame: Option<&'a Frame>,
    error: Option<&'a str>,
    logs: &'a [String],
}

#[derive(Serialize)]
struct Listed<'a> {
    name: &'a str,
    revision: u64,
}

/// What a client needs to ask a model for a view.
#[derive(Serialize)]
struct HostInfo<'a> {
    spec: &'a str,
    blocks: Vec<&'a str>,
    actions: Vec<&'a str>,
    /// Image addresses must start with one of these.
    images: Vec<&'a str>,
    edit: bool,
}

#[derive(Serialize)]
struct Source<'a> {
    name: &'a str,
    source: String,
    data: &'a serde_json::Value,
}

#[derive(Deserialize)]
struct NewSource {
    source: String,
}

#[derive(Deserialize)]
struct Event {
    version: u64,
    id: String,
    #[serde(default)]
    value: serde_json::Value,
}

struct Entry {
    script: PathBuf,
    data_file: PathBuf,
    stamp: (Option<SystemTime>, Option<SystemTime>),
    view: Option<View>,
    data: serde_json::Value,
    revision: u64,
    frame: Option<Frame>,
    error: Option<String>,
    logs: Vec<String>,
}

struct Waiter {
    request: Request,
    name: String,
    since: u64,
    until: Instant,
}

pub fn serve(options: Options) -> Result<(), String> {
    let server = Server::http(&options.bind).map_err(|e| format!("{}: {e}", options.bind))?;
    let mut state = Served {
        dir: options.dir,
        host: options.host,
        edit: options.edit,
        entries: BTreeMap::new(),
        waiting: Vec::new(),
    };
    state.scan();
    let names: Vec<_> = state.entries.keys().cloned().collect();
    eprintln!(
        "ulana: serving {} on http://{}{} ({})",
        state.dir.display(),
        options.bind,
        if state.edit {
            ", clients may write scripts"
        } else {
            ""
        },
        if names.is_empty() {
            "no views yet".to_string()
        } else {
            names.join(", ")
        }
    );
    let mut scanned = Instant::now();
    loop {
        match server.recv_timeout(SCAN) {
            Ok(Some(request)) => state.handle(request),
            Ok(None) => {}
            Err(e) => eprintln!("ulana: {e}"),
        }
        if scanned.elapsed() >= SCAN {
            state.scan();
            scanned = Instant::now();
        }
        state.answer_waiting();
    }
}

struct Served {
    dir: PathBuf,
    host: Host,
    edit: bool,
    entries: BTreeMap<String, Entry>,
    waiting: Vec<Waiter>,
}

impl Entry {
    fn new(script: PathBuf) -> Self {
        Entry {
            data_file: script.with_extension("json"),
            script,
            stamp: (None, None),
            view: None,
            data: serde_json::Value::Null,
            revision: 0,
            frame: None,
            error: None,
            logs: Vec::new(),
        }
    }
}

impl Served {
    /// Load new views, reload changed ones, and forget deleted ones.
    fn scan(&mut self) {
        let mut found = Vec::new();
        if let Ok(dir) = std::fs::read_dir(&self.dir) {
            for item in dir.flatten() {
                let path = item.path();
                if path.extension().is_some_and(|e| e == "luau") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        if valid_name(name) {
                            found.push((name.to_string(), path));
                        }
                    }
                }
            }
        }
        self.entries
            .retain(|name, _| found.iter().any(|(n, _)| n == name));
        for (name, script) in found {
            let stamp = (modified(&script), modified(&script.with_extension("json")));
            let entry = self
                .entries
                .entry(name.clone())
                .or_insert_with(|| Entry::new(script));
            if entry.stamp != stamp {
                let script_changed = entry.stamp.0 != stamp.0;
                entry.stamp = stamp;
                entry.refresh(&name, &self.host, script_changed);
            }
        }
    }

    fn handle(&mut self, mut request: Request) {
        if *request.method() == Method::Options {
            return respond(request, 204, None);
        }
        let url = request.url().to_string();
        let (path, query) = url.split_once('?').unwrap_or((&url, ""));
        let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
        match (request.method(), parts.as_slice()) {
            (Method::Get, ["host"]) => {
                let info = HostInfo {
                    spec: SPEC,
                    blocks: self.host.blocks().collect(),
                    actions: self.host.actions().collect(),
                    images: self.host.images().collect(),
                    edit: self.edit,
                };
                respond(request, 200, Some(json(&info)));
            }
            (Method::Get, ["views", name, "source"]) => {
                let Some(entry) = self.entries.get(*name) else {
                    return respond(request, 404, Some(message("no such view")));
                };
                match std::fs::read_to_string(&entry.script) {
                    Ok(source) => {
                        let body = json(&Source {
                            name,
                            source,
                            data: &entry.data,
                        });
                        respond(request, 200, Some(body));
                    }
                    Err(e) => respond(request, 500, Some(message(&e.to_string()))),
                }
            }
            (Method::Put, ["views", name, "source"]) => {
                let name = name.to_string();
                if !self.edit {
                    let text =
                        "this server does not let clients write scripts; start it with --edit";
                    return respond(request, 403, Some(message(text)));
                }
                if !valid_name(&name) {
                    let text = "a view's name is 1 to 64 letters, digits, '-', or '_'";
                    return respond(request, 400, Some(message(text)));
                }
                let new: NewSource = match read_json(&mut request, SOURCE_LIMIT) {
                    Ok(new) => new,
                    Err(e) => return respond(request, 400, Some(message(&e))),
                };
                let body = match self.write(&name, &new.source) {
                    Ok(body) => body,
                    Err(e) => return respond(request, 500, Some(message(&e))),
                };
                respond(request, 200, Some(body));
            }
            (Method::Get, ["views"]) => {
                let list: Vec<_> = self
                    .entries
                    .iter()
                    .map(|(name, e)| Listed {
                        name,
                        revision: e.revision,
                    })
                    .collect();
                respond(request, 200, Some(json(&list)));
            }
            (Method::Get, ["views", name]) => {
                let Some(entry) = self.entries.get(*name) else {
                    return respond(request, 404, Some(message("no such view")));
                };
                let since = query
                    .split('&')
                    .find_map(|kv| kv.strip_prefix("since="))
                    .and_then(|v| v.parse::<u64>().ok());
                match since {
                    Some(since) if since >= entry.revision => self.waiting.push(Waiter {
                        request,
                        name: name.to_string(),
                        since,
                        until: Instant::now() + WAIT,
                    }),
                    _ => {
                        let body = json(&entry.snapshot(name));
                        respond(request, 200, Some(body));
                    }
                }
            }
            (Method::Post, ["views", name, "events"]) => {
                let name = name.to_string();
                let event: Event = match read_json(&mut request, BODY_LIMIT) {
                    Ok(event) => event,
                    Err(e) => return respond(request, 400, Some(message(&e))),
                };
                let Some(entry) = self.entries.get_mut(&name) else {
                    return respond(request, 404, Some(message("no such view")));
                };
                entry.event(&name, event);
                let body = json(&entry.snapshot(&name));
                respond(request, 200, Some(body));
            }
            _ => respond(request, 404, Some(message("not found"))),
        }
    }

    /// Write a view's script, load it at once, and return its snapshot as JSON.
    fn write(&mut self, name: &str, source: &str) -> Result<String, String> {
        let script = self.dir.join(format!("{name}.luau"));
        std::fs::write(&script, source).map_err(|e| format!("{}: {e}", script.display()))?;
        let entry = self
            .entries
            .entry(name.to_string())
            .or_insert_with(|| Entry::new(script.clone()));
        // Load now rather than on the next scan, so the answer has the result
        // even when the file's time did not move.
        entry.stamp = (modified(&script), modified(&entry.data_file));
        entry.refresh(name, &self.host, true);
        Ok(json(&entry.snapshot(name)))
    }

    fn answer_waiting(&mut self) {
        let now = Instant::now();
        let mut still = Vec::new();
        for w in self.waiting.drain(..) {
            match self.entries.get(&w.name) {
                None => respond(w.request, 404, Some(message("no such view"))),
                Some(e) if e.revision > w.since || now >= w.until => {
                    let body = json(&e.snapshot(&w.name));
                    respond(w.request, 200, Some(body));
                }
                Some(_) => still.push(w),
            }
        }
        self.waiting = still;
    }
}

impl Entry {
    /// Read the files again: reload the script if it changed (keeping state),
    /// re-read the data, and render.
    fn refresh(&mut self, name: &str, host: &Host, script_changed: bool) {
        self.data = match std::fs::read_to_string(&self.data_file) {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(data) => data,
                Err(e) => return self.fail(name, format!("{}: {e}", self.data_file.display())),
            },
            Err(_) => serde_json::Value::Null,
        };
        if script_changed || self.view.is_none() {
            let source = match std::fs::read_to_string(&self.script) {
                Ok(source) => source,
                Err(e) => return self.fail(name, format!("{}: {e}", self.script.display())),
            };
            let loaded = match self.view.as_mut() {
                Some(view) => view.reload(&source),
                None => View::new(&source, host.clone(), Limits::default()).map(|v| {
                    self.view = Some(v);
                }),
            };
            if let Err(e) = loaded {
                return self.fail(name, e.to_string());
            }
        }
        self.render(name);
    }

    fn render(&mut self, name: &str) {
        let Some(view) = self.view.as_mut() else {
            return;
        };
        let result = view.render(&self.data);
        self.take_logs(name);
        match result {
            Ok(frame) => {
                if self.error.take().is_some() {
                    eprintln!("ulana: {name} renders again");
                }
                self.frame = Some(frame);
                self.revision += 1;
            }
            Err(e) => self.fail(name, e.to_string()),
        }
    }

    fn event(&mut self, name: &str, event: Event) {
        let Some(view) = self.view.as_mut() else {
            return;
        };
        match view.event(event.version, &event.id, &event.value) {
            Ok(()) => self.render(name),
            // The app has an old frame; the snapshot it gets back is the latest.
            Err(Error::Stale { .. }) => {}
            Err(e) => {
                self.take_logs(name);
                self.fail(name, e.to_string());
            }
        }
    }

    fn fail(&mut self, name: &str, error: String) {
        eprintln!("ulana: {name}: {error}");
        self.error = Some(error);
        self.revision += 1;
    }

    fn take_logs(&mut self, name: &str) {
        let Some(view) = self.view.as_mut() else {
            return;
        };
        for line in view.take_logs() {
            eprintln!("ulana: {name}: print: {line}");
            self.logs.push(line);
        }
        let extra = self.logs.len().saturating_sub(LOG_LINES);
        self.logs.drain(..extra);
    }

    fn snapshot<'a>(&'a self, name: &'a str) -> Snapshot<'a> {
        Snapshot {
            name,
            revision: self.revision,
            frame: self.frame.as_ref(),
            error: self.error.as_deref(),
            logs: &self.logs,
        }
    }
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Read a JSON body of at most `limit` bytes.
fn read_json<T: serde::de::DeserializeOwned>(
    request: &mut Request,
    limit: u64,
) -> Result<T, String> {
    let mut body = String::new();
    request
        .as_reader()
        .take(limit + 1)
        .read_to_string(&mut body)
        .map_err(|e| e.to_string())?;
    if body.len() as u64 > limit {
        return Err(format!("the body is over {limit} bytes"));
    }
    serde_json::from_str(&body).map_err(|e| e.to_string())
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

fn json(value: &impl Serialize) -> String {
    serde_json::to_string(value).expect("JSON")
}

fn message(text: &str) -> String {
    json(&serde_json::json!({ "error": text }))
}

/// Answer with JSON and open CORS, so the web build of an app can call it.
fn respond(request: Request, status: u16, body: Option<String>) {
    let mut response = Response::from_string(body.unwrap_or_default()).with_status_code(status);
    for (k, v) in [
        ("Content-Type", "application/json"),
        ("Access-Control-Allow-Origin", "*"),
        ("Access-Control-Allow-Methods", "GET, POST, PUT, OPTIONS"),
        ("Access-Control-Allow-Headers", "content-type"),
        ("Cache-Control", "no-store"),
    ] {
        if let Ok(h) = Header::from_bytes(k, v) {
            response.add_header(h);
        }
    }
    let _ = request.respond(response);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::time::Duration;

    struct Folder(PathBuf);

    impl Folder {
        fn new(name: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("ulana-serve-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            Folder(dir)
        }

        /// Write a file and move its time forward, so a change is seen however
        /// fine the file system's clock is.
        fn write(&self, file: &str, text: &str, tick: u64) {
            let path = self.0.join(file);
            std::fs::write(&path, text).unwrap();
            let when = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000 + tick);
            File::options()
                .write(true)
                .open(&path)
                .unwrap()
                .set_modified(when)
                .unwrap();
        }

        fn served(&self) -> Served {
            Served {
                dir: self.0.clone(),
                host: Host::new(),
                edit: true,
                entries: BTreeMap::new(),
                waiting: Vec::new(),
            }
        }
    }

    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    const COUNTER: &str = r#"
        return function(data)
            local n = ui.state("n", 0)
            return ui.stack{ ui.text((data and data.label or "Count") .. " " .. n:get()),
                ui.button{ "+", on_press = function() n:set(n:get() + 1) end } }
        end
    "#;

    fn text(entry: &Entry) -> String {
        let frame = entry.frame.as_ref().expect("a frame");
        match &frame.root.children()[0] {
            ulana::Block::Text { text, .. } => text.clone(),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_saved_script_reloads_and_keeps_state() {
        let folder = Folder::new("reload");
        folder.write("counter.luau", COUNTER, 1);
        let mut s = folder.served();
        s.scan();
        let e = s.entries.get_mut("counter").unwrap();
        assert_eq!((e.revision, text(e).as_str()), (1, "Count 0"));
        let version = e.frame.as_ref().unwrap().version;
        e.event(
            "counter",
            Event {
                version,
                id: "h1".into(),
                value: serde_json::Value::Null,
            },
        );
        assert_eq!((e.revision, text(e).as_str()), (2, "Count 1"));

        folder.write(
            "counter.luau",
            &COUNTER.replace("\"Count\"", "\"Total\""),
            2,
        );
        s.scan();
        let e = &s.entries["counter"];
        assert_eq!((e.revision, text(e).as_str()), (3, "Total 1"));

        // Nothing changed: no new revision.
        s.scan();
        assert_eq!(s.entries["counter"].revision, 3);
    }

    #[test]
    fn an_error_keeps_the_last_frame_until_it_is_fixed() {
        let folder = Folder::new("error");
        folder.write("counter.luau", COUNTER, 1);
        let mut s = folder.served();
        s.scan();

        folder.write("counter.luau", "return function( end", 2);
        s.scan();
        let e = &s.entries["counter"];
        assert!(
            e.error.as_deref().unwrap().starts_with("syntax error"),
            "{:?}",
            e.error
        );
        assert_eq!(text(e), "Count 0");

        folder.write("counter.luau", COUNTER, 3);
        s.scan();
        let e = &s.entries["counter"];
        assert_eq!(e.error, None);
        assert_eq!(e.revision, 3);
    }

    #[test]
    fn data_files_new_views_and_deleted_views_are_seen() {
        let folder = Folder::new("data");
        folder.write("counter.luau", COUNTER, 1);
        let mut s = folder.served();
        s.scan();

        folder.write("counter.json", r#"{ "label": "Cups" }"#, 2);
        folder.write("other.luau", "return ui.divider()", 2);
        folder.write("bad name.luau", "return ui.divider()", 2);
        s.scan();
        assert_eq!(text(&s.entries["counter"]), "Cups 0");
        assert_eq!(s.entries.keys().collect::<Vec<_>>(), ["counter", "other"]);

        folder.write("counter.json", "{ not json", 3);
        s.scan();
        assert!(s.entries["counter"]
            .error
            .as_deref()
            .unwrap()
            .contains("counter.json"));

        std::fs::remove_file(folder.0.join("other.luau")).unwrap();
        s.scan();
        assert!(!s.entries.contains_key("other"));
    }

    #[test]
    fn a_failing_handler_is_shown_and_a_stale_press_is_ignored() {
        let folder = Folder::new("event");
        folder.write(
            "v.luau",
            r#"return ui.button{ "x", on_press = function() error("nope") end }"#,
            1,
        );
        let mut s = folder.served();
        s.scan();
        let e = s.entries.get_mut("v").unwrap();
        e.event(
            "v",
            Event {
                version: 99,
                id: "h1".into(),
                value: serde_json::Value::Null,
            },
        );
        assert_eq!((e.revision, e.error.as_deref()), (1, None));
        e.event(
            "v",
            Event {
                version: 1,
                id: "h1".into(),
                value: serde_json::Value::Null,
            },
        );
        assert_eq!(e.revision, 2);
        assert!(e.error.as_deref().unwrap().contains("nope"));
    }

    #[test]
    fn a_written_script_loads_at_once_and_keeps_state() {
        let folder = Folder::new("write");
        folder.write("counter.luau", COUNTER, 1);
        let mut s = folder.served();
        s.scan();
        let e = s.entries.get_mut("counter").unwrap();
        let version = e.frame.as_ref().unwrap().version;
        e.event(
            "counter",
            Event {
                version,
                id: "h1".into(),
                value: serde_json::Value::Null,
            },
        );

        let body = s
            .write("counter", &COUNTER.replace("\"Count\"", "\"Clicks\""))
            .unwrap();
        let snap: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(snap["error"], serde_json::Value::Null);
        assert_eq!(snap["frame"]["root"]["children"][0]["text"], "Clicks 1");
        assert_eq!(
            std::fs::read_to_string(folder.0.join("counter.luau")).unwrap(),
            COUNTER.replace("\"Count\"", "\"Clicks\"")
        );

        // A broken script is written and reported; the last frame stays.
        let body = s.write("counter", "return function( end").unwrap();
        let snap: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(snap["error"].as_str().unwrap().starts_with("syntax error"));
        assert_eq!(snap["frame"]["root"]["children"][0]["text"], "Clicks 1");

        // A new name makes a new view.
        let body = s.write("fresh", "return ui.text('new')").unwrap();
        let snap: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(snap["frame"]["root"]["text"], "new");
        assert!(s.entries.contains_key("fresh"));
    }
}
