# Ulana for Swift

Draw Ulana views with SwiftUI, and run them either on a server or on the device.

| Product | What it is | Needs |
| --- | --- | --- |
| `Ulana` | The frame types, `UlanaView` (SwiftUI), `UlanaTheme`, and `ServerSession`, which follows views on a server speaking the `ulana serve` protocol. | Nothing: pure Swift. |
| `UlanaLocal` | `LocalView` and `LocalSession`: the Rust runtime with Luau, on the device. The same sandbox, limits, and checks as on a server. | `UlanaCore.xcframework`, built by `scripts/build-xcframework.sh`. Until it exists the product is left out and `Ulana` still builds. |

iOS 17 and macOS 14 or later.

## Which one

- **On a server** (`ServerSession`): scripts can change without an app update, several devices share one view's state, and the app carries no runtime. Every press that runs a script's handler is a round trip.
- **On the device** (`LocalSession`): works offline, a press is instant, and nothing about the user's screen leaves the phone. The app carries the runtime (a few MB once linked).

Both give the screen the same thing, a `Frame`, and both conform to `UlanaSession`, so the drawing code is the same.

## Draw a frame

```swift
import Ulana

UlanaView(
    frame: frame,
    theme: .system,  // the app's accent colour, Dynamic Type, dark mode
    onEvent: { version, id, value in session.send(version: version, id: id, value: value) },
    onHostAction: { name, value in if name == "send" { chat.send(value.display) } },
    blocks: ["calendar": { props in AnyView(CalendarView(props.props)) }]
)
```

Blocks map to SwiftUI's own views: `VStack`, `HStack`, `Text` with Dynamic Type fonts, `Button` (`.borderedProminent`, `.bordered`, `.borderless`), `Toggle`, `ProgressView`, `Divider`. A block type this version does not know shows a short note instead of failing.

## On a server

```swift
let session = ServerSession(baseURL: URL(string: "http://127.0.0.1:8790")!)
session.start()            // lists views and follows `session.name`
session.name = "counter"
// session.frame, session.error, session.logs, session.views, session.offline
```

It long-polls `GET /views/<name>?since=<revision>`, so a saved script shows up at once. Plain `http` to a computer on the local network needs `NSAllowsLocalNetworking` in the app's `Info.plist`.

## On the device

```sh
scripts/build-xcframework.sh   # from the repository root; needs Rust and Xcode
```

```swift
import UlanaLocal

let session = LocalSession(
    source: script,
    host: HostSpec(blocks: ["calendar"], actions: ["send"]),
    data: .object(["events": .array([])]),
    state: saved                       // [String: JSONValue], or [:]
)
session.onState = { state in save(state) }
session.reload(source: edited)         // keeps the state; an error keeps the last frame
```

`LocalView` is the lower level: `render(data:)`, `event(version:id:value:)`, `reload(source:)`, `state()`, `takeLogs()`. It is not thread-safe; `LocalSession` runs it on the main actor, where a call is capped at 100 ms and usually takes well under a millisecond.

## Tests

```sh
swift test
```

`SchemaTests` builds every block and action in `spec/frame.schema.json` with every field set, decodes it in Swift, encodes it again, and fails if a field was lost, so these hand-written types cannot drift from the Rust ones. `UlanaLocalTests` (with the XCFramework built) runs the runtime: events, reload, saved state, the sandbox, and the example views.
