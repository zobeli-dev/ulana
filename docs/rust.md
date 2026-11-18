# Running views in your own server

The `ulana` crate runs views: it loads a Luau script in a sandbox, renders it into a checked `Frame`, runs the handlers a press names, and keeps the view's state as JSON. Use it when views should run on your server, so you can change screens without an app update, and send the frames to an app that draws them ([SwiftUI](swift.md), [React Native](react-native.md)).

```sh
cargo add ulana --git https://github.com/zobeli-dev/ulana
cargo add serde_json
```

## A view, start to finish

```rust
use serde_json::json;
use ulana::{Host, Limits, View};

// What your app accepts beyond the built-in blocks.
let host = Host::new()
    .block("calendar")                          // ui.custom("calendar", …)
    .action("send")                             // ui.host("send", …)
    .image("https://images.example.com/");      // image addresses must start with this

let mut view = View::new(&source, host, Limits::default())?;
let frame = view.render(&json!({ "events": events }))?;   // send serde_json::to_string(&frame)? to the app

// The app reports a press on { "kind": "event", "id": "h1" } in frame.version:
view.event(frame.version, "h1", &serde_json::Value::Null)?;
let next = view.render(&json!({ "events": events }))?;

// The state is plain JSON. Save it with the user's account.
let saved: ulana::State = view.state();
```

- `View::with_state(source, host, limits, saved)` starts from saved state.
- `view.reload(source)` swaps the script and keeps the state. On an error the old script stays, which is what you want when a model sends a broken one.
- `view.take_logs()` returns what the script printed.
- A `View` is not `Send`. Make one where it is used, such as one per request from the saved state, or keep each one on its own thread.

## A web server

A sketch with [axum](https://github.com/tokio-rs/axum) (`Shared` is axum's `State` extractor, renamed so it does not clash with `ulana::State`). Each request builds the view from the script and the user's saved state, does its work, and saves the state again. Rendering takes well under a millisecond, so this is cheap.

```rust
async fn render(Shared(app): Shared<App>, Path(name): Path<String>, user: User) -> Result<Json<ulana::Frame>, AppError> {
    let source = app.scripts.get(&name).await?;
    let saved = app.states.get(&user, &name).await?;
    let (frame, state) = tokio::task::spawn_blocking(move || {
        let mut view = View::with_state(&source, host(), Limits::default(), saved)?;
        let frame = view.render(&data_for(&user))?;
        Ok::<_, ulana::Error>((frame, view.state()))
    })
    .await??;
    app.states.put(&user, &name, state).await?;
    Ok(Json(frame))
}
```

An event handler builds the view the same way, renders once so the frame's handlers exist, runs the press, renders again, saves the state, and answers with the new frame:

```rust
let mut view = View::with_state(&source, host(), Limits::default(), saved)?;
view.render(&data)?;                                // handler ids come from this render
view.event(view.version(), &press.id, &press.value)?;
let frame = view.render(&data)?;
```

A fresh `View` counts frame versions from 1, so pass `view.version()` rather than the version the app sent. Handler ids are the same for the same script, state, and data. If the state might have changed since the app's frame (another device, another tab), store a revision with the state and refuse a press made against an older one, as `ulana serve` does. To have the app follow changes live, use the long-poll protocol of `ulana serve` ([protocol.md](protocol.md)); the SwiftUI and React Native packages already speak it.

## Limits

`Limits::default()` allows 16 MB of memory and 100 ms per call, 2000 blocks nested at most 32 deep, 4000 characters per piece of text (longer text is cut), 64 KB of state, and 16 KB per custom block's props or host action's value. Change any field:

```rust
let limits = Limits { time: std::time::Duration::from_millis(50), ..Limits::default() };
```

## Errors

`ulana::Error` messages are written to be read by a person or handed back to a model:

| Variant | When |
| --- | --- |
| `Syntax`, `Runtime` | The script does not compile, or raised an error (`view:3: attempt to index nil`). |
| `Memory`, `Timeout` | It went over a limit. The view still works afterwards. |
| `Invalid { path, message }` | The tree or a stored value breaks a rule: `at root.children[2].src: this app only shows images from …`. |
| `Stale { latest }`, `UnknownEvent` | The press names an old frame, or a handler that does not exist. Answer with the latest frame. |

## Trees that are not scripts

`ulana::parse(json, &host, &limits)` checks a tree written as JSON, by a model or by your server, with the same rules. It cannot use `event` actions, because there is no script to handle them. Use host actions instead. `ulana::frame_schema()` returns the JSON Schema of a frame.

## The sandbox

A view gets Luau's `string`, `table`, `math`, `utf8`, and `bit32` libraries, plus `os.time`, `os.clock`, and `os.date`. It does not get `require`, `loadstring`, `getfenv`, `setfenv`, `collectgarbage`, `io`, `debug`, or files and network of any kind. Libraries and the `ui` table are read-only, and each view has its own Lua state. A call that runs too long is stopped even inside `pcall`. `crates/ulana/tests/sandbox.rs` tries each of these.
