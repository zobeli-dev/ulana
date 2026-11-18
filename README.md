# Ulana

**Luau scripts that draw native UI.** A model (or a person) writes a small [Luau](https://luau.org) script; Ulana runs it in a sandbox and returns a checked tree of blocks; your app draws that tree with its own native components. *Ulana* is Hawaiian for "to weave".

```luau
return function(data)
	local count = ui.state("count", 0)
	return ui.card{ title = "Counter",
		ui.text{ tostring(count:get()), style = "title" },
		ui.button{ "More", on_press = function()
			count:update(function(n) return n + 1 end)
		end },
	}
end
```

Status: **0.1, early.** The API will change.

## Why

Apps that let a model change their interface usually pick one of two paths. JSON layouts are safe but cannot hold logic: no state, no filters, no "tick this off". Web views can do anything, including look like your app's own approval screen. Ulana sits between them:

- **Real logic, in a sandbox.** Luau was built to run untrusted code. Views get no network, no files, no `require`, read-only libraries, and limits on memory and time. A view that loops forever or allocates without end stops with an error, and the app keeps running.
- **Native, not a web view.** The output is a small set of blocks (`stack`, `row`, `text`, `card`, `list`, `item`, `button`, `toggle`, `progress`, `badge`, `divider`, `spacer`) plus the host's own `custom` blocks. Your app draws them with its components and its theme.
- **Checked before drawn.** Every tree goes through the same Rust checker, whether it came from a script or from JSON. Unknown fields, unknown custom blocks, and unknown host actions are refused with the path, so a model can fix its own mistake.
- **The host stays in charge.** A view can only ask the host to do the things the host named (`ui.host("send", …)`). The host decides what happens, including asking the user first.
- **Made for models to write.** [`spec/ulana.md`](spec/ulana.md) is the whole API on one page, written to go into a prompt.

## What is here

| Path | What it is |
| --- | --- |
| [`crates/ulana`](crates/ulana) | The Rust library: blocks, checker, and the sandboxed Luau runtime (through [mlua](https://github.com/mlua-rs/mlua)). |
| [`crates/ulana-cli`](crates/ulana-cli) | `ulana render`, `ulana check`, `ulana schema`. |
| [`packages/react-native`](packages/react-native) | `@zobeli/ulana-react-native`: draws a frame with React Native's own components. |
| [`spec/ulana.md`](spec/ulana.md) | The API, for models and people. |
| [`spec/frame.schema.json`](spec/frame.schema.json) | JSON Schema of a frame, generated from the Rust types. |
| [`examples`](examples) | `counter.luau`, `todo.luau`, and `agenda.luau` with `agenda.json`. |

## How it fits together

```
 Luau view ──► ulana (Rust): sandbox → tree → check ──► Frame (JSON) ──► <UlanaView> (React Native)
     ▲                                                                         │
     └──────────────── event { version, id, value } ◄──────── press / change ──┘
```

1. The host loads a view: `View::new(source, host, limits)`. `Host` lists the custom blocks and host actions the app accepts.
2. `view.render(&data)` runs the script and returns a `Frame { version, root }`.
3. The app draws it. A press on an `event` action calls `view.event(version, id, value)`; the app renders again. A press on a `host` action goes to the app.
4. `view.state()` is plain JSON. Save it and pass it back with `View::with_state` to pick up where the user left off. `view.reload(source)` swaps the script and keeps the state.

For now the runtime is Rust, so it runs on a server or in a desktop app, and a phone receives frames. Running it on the phone itself (iOS, Android, and WebAssembly) is planned, using the same crate.

### Rust

```rust
use ulana::{Host, Limits, View};

let host = Host::new().block("calendar").action("send");
let mut view = View::new(source, host, Limits::default())?;
let frame = view.render(&serde_json::json!({ "events": [] }))?;
send_to_app(&frame);
// later, when the app reports a press:
view.event(frame.version, "h1", &serde_json::Value::Null)?;
let next = view.render(&data)?;
```

A tree written as JSON (by a model, say) goes through the same checks with `ulana::parse(json, &host, &limits)`. It cannot use `event` actions, because there is no script to handle them.

### React Native

```tsx
import { UlanaView, makeTheme } from "@zobeli/ulana-react-native";

<UlanaView
  frame={frame}
  theme={makeTheme({ tones: { accent: { fg: "#C2410C", soft: "#FFEDD5" } } })}
  onEvent={(version, id, value) => api.event(version, id, value).then(setFrame)}
  onHostAction={(name, value) => name === "send" && sendMessage(String(value))}
  blocks={{ calendar: CalendarBlock }}
/>
```

## Try it

```sh
cargo test --workspace
cargo run -p ulana-cli -- render examples/counter.luau --event h2 --event h2
cargo run -p ulana-cli -- render examples/agenda.luau --data examples/agenda.json --block calendar --action send
cd packages/react-native && bun install && bun run typecheck && bun test
```

## Not yet

- The runtime on iOS, Android, and the web (WebAssembly).
- An example Expo app with hot reload from `ulana serve`.
- Text fields, images, and charts.
- Publishing to crates.io and npm.

## License

[MIT](LICENSE).
