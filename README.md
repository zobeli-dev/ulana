# Ulana

**Let a model change your app's screens, safely.** A model (or a person) writes a small [Luau](https://luau.org) script. Ulana runs it in a sandbox and checks the result, and your app draws it with its own native components: SwiftUI on iOS, React Native on iOS, Android, and the web. *Ulana* is Hawaiian for "to weave".

```luau
return function(data)
	local count = ui.state("count", 0)
	return ui.card{ title = "Counter", tone = "accent",
		ui.text{ tostring(count:get()), style = "display" },
		ui.button{ "More", icon = "add", on_press = function()
			count:update(function(n) return n + 1 end)
		end },
	}
end
```

Status: **0.1, early.** The API may still change. MIT licensed.

## Why Ulana

Apps that let a model change their interface usually pick one of two paths. JSON layouts are safe but hold no logic: no state, no filters, no "tick this off". Web views can do anything, including look like your app's own approval screen. Ulana sits between them.

- **Real logic, in a sandbox.** Views get no network, no files, no `require`, read-only libraries, and 100 ms and 16 MB per call. A view that loops forever stops with an error, and the app keeps running.
- **Native, not a web view.** A view builds from 18 blocks: `stack`, `row`, `grid`, `card`, `list`, `item`, `text`, `icon`, `image`, `button`, `toggle`, `field`, `choice`, `progress`, `badge`, `divider`, `spacer`, plus your app's own `custom` blocks. Colours, spacing, and sizes are tokens, and your theme decides how they look.
- **Checked before drawn.** Every tree goes through the same Rust checker. A mistake is refused with its path (`at root.children[2]: unknown field 'colour'`), so a model can fix it itself.
- **Your app stays in charge.** A view can only ask your app to do things your app named (`ui.host("send", …)`), and only shows images from addresses you allow.
- **Runs where you want.** On a server, so you can change screens without an app update. Or on the device, for offline use and instant taps. The same checks apply in both places.

## Try it in two minutes

You need [Rust](https://rustup.rs) and [Bun](https://bun.sh).

```sh
git clone https://github.com/zobeli-dev/ulana && cd ulana
bun install
cargo run -p ulana-cli -- serve examples/views --edit --block calendar --action send --image https://picsum.photos/
# in another terminal
cd examples/expo && cp .env.example .env && bun run web
```

Open the views on the left. Edit `examples/views/counter.luau`, save, and the screen changes without losing its count. In the chat panel, paste an [OpenRouter key](https://openrouter.ai/keys) and ask for a change: "turn this into a dashboard with tinted cards and icons". The model (`z-ai/glm-5.3-flash` by default) rewrites the script, and a failing script goes back to it to fix.

On a Mac with Xcode, [`examples/ios`](examples/ios) draws the same views in SwiftUI, either running on the iPhone itself (edit a script in the app and it reloads) or following the server. It has no chat yet.

## Add it to your app

Pick where views run, then the platform.

| | Views run on a server | Views run on the device |
| --- | --- | --- |
| Good for | Changing screens without an app update; several devices sharing a view's state; no runtime in the app | Offline use; instant taps; nothing leaving the phone |
| SwiftUI | `Ulana` package, `ServerSession` | `UlanaLocal` package, `LocalSession` |
| React Native | `@zobeli/ulana-react-native`, `useServerView` | Not yet |
| Server | `ulana` Rust crate, or `ulana serve` | — |

### SwiftUI

In Xcode, choose **File › Add Package Dependencies…** and enter `https://github.com/zobeli-dev/ulana`, or in `Package.swift`:

```swift
.package(url: "https://github.com/zobeli-dev/ulana", branch: "main")
```

```swift
import SwiftUI
import Ulana

struct CounterScreen: View {
    @State private var session = ServerSession(baseURL: URL(string: "http://127.0.0.1:8790")!)

    var body: some View {
        ScrollView { UlanaSessionView(session: session).padding() }
            .onAppear { session.start(); session.name = "counter" }
    }
}
```

Guide: [docs/swift.md](docs/swift.md), which also covers running views on the device with `UlanaLocal`.

### React Native and Expo

The package is not on npm yet: copy [`packages/react-native`](packages/react-native) into your repository (or add it as a workspace) until it is.

```tsx
import { UlanaServerView } from "@zobeli/ulana-react-native";

export function CounterScreen() {
  return <UlanaServerView url="http://127.0.0.1:8790" name="counter" />;
}
```

Guide: [docs/react-native.md](docs/react-native.md): your theme, your own blocks and icons, host actions, and drawing frames you got some other way.

### Your server (Rust)

```sh
cargo add ulana --git https://github.com/zobeli-dev/ulana
```

```rust
use ulana::{Host, Limits, View};

let host = Host::new().block("calendar").action("send");
let mut view = View::new(&source, host, Limits::default())?;
let frame = view.render(&serde_json::json!({ "events": [] }))?; // send this JSON to the app
view.event(frame.version, "h1", &serde_json::Value::Null)?;     // when the app reports a press
let next = view.render(&data)?;
let saved = view.state();                                       // plain JSON, for View::with_state later
```

Guide: [docs/rust.md](docs/rust.md). Or skip the code and run `ulana serve` (`cargo install --git https://github.com/zobeli-dev/ulana ulana-cli`), which serves a folder of views over HTTP: [docs/protocol.md](docs/protocol.md).

### Let a model write views

Give the model [`spec/ulana.md`](spec/ulana.md), the current script, the data it reads, and the request. Then check what it sends back, and return the error if there is one. [docs/models.md](docs/models.md) has the prompt, the repair loop, and the safety rules. The Expo example's [`src/ai.ts`](examples/expo/src/ai.ts) is a working version in about 200 lines.

## How it fits together

```
 Luau view ──► ulana runtime: sandbox → tree → check ──► Frame (JSON) ──► your app draws it natively
     ▲                                                                            │
     └──────────── event { version, id, value } ◄──────────── press, type, pick ──┘
```

1. Load a view with the blocks and actions your app accepts (`Host`).
2. `render(data)` runs the script and returns a `Frame { version, root }`.
3. The app draws it. A press on one of the view's handlers comes back as `event(version, id, value)`, and the view renders again. A press on a host action goes to your app.
4. State (`ui.state`) is plain JSON. Save it to pick up where the user left off; `reload(source)` swaps the script and keeps it.

## Documentation

| | |
| --- | --- |
| [spec/ulana.md](spec/ulana.md) | The view API: every block, token, and pattern. Written to go into a prompt. |
| [docs/swift.md](docs/swift.md) | SwiftUI: install, server and on-device sessions, theme, your own blocks. |
| [docs/react-native.md](docs/react-native.md) | React Native and Expo: install, `UlanaServerView`, hooks, theme, icons, your own blocks. |
| [docs/rust.md](docs/rust.md) | Running views in your own server: `View`, `Host`, `Limits`, state, errors. |
| [docs/models.md](docs/models.md) | Letting a model write views: prompt, repair loop, safety. |
| [docs/protocol.md](docs/protocol.md) | The frame format and the `ulana serve` HTTP API. |
| [docs/publishing.md](docs/publishing.md) | For maintainers: releasing to crates.io, npm, and SwiftPM. |
| [AGENTS.md](AGENTS.md) | For contributors and coding agents: layout, rules, checks. |

## What is in this repository

| Path | What it is |
| --- | --- |
| [`crates/ulana`](crates/ulana) | The Rust library: blocks, checker, and the sandboxed Luau runtime (through [mlua](https://github.com/mlua-rs/mlua)). |
| [`crates/ulana-cli`](crates/ulana-cli) | `ulana render`, `ulana check`, `ulana serve`, `ulana schema`. |
| [`crates/ulana-ffi`](crates/ulana-ffi) | A C interface to the runtime, built into `UlanaCore.xcframework` by `scripts/build-xcframework.sh`. |
| [`Package.swift`](Package.swift), [`packages/swift`](packages/swift) | The `Ulana` Swift package. |
| [`packages/swift-local`](packages/swift-local) | The `UlanaLocal` Swift package (views on the device). |
| [`packages/react-native`](packages/react-native) | `@zobeli/ulana-react-native`. |
| [`spec`](spec) | The view API and the frame's JSON Schema, generated from the Rust types. |
| [`examples/views`](examples/views) | Example views: counter, to-do list, agenda, weather. |
| [`examples/expo`](examples/expo), [`examples/ios`](examples/ios) | Example apps. |

## Not yet

- Publishing to npm, crates.io, and a prebuilt `UlanaCore.xcframework` with each release (until then `UlanaLocal` needs a clone and `scripts/build-xcframework.sh`).
- Running views on the device on Android and on the web (WebAssembly).
- A stable identity per block, so changes can animate; navigation between screens.

## License

[MIT](LICENSE).
