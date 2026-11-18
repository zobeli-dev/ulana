# Ulana in SwiftUI

Draw Ulana views with SwiftUI's own views, from a server or from the device itself. iOS 17 and macOS 14 or later.

| Package | Product | What it does | Install |
| --- | --- | --- | --- |
| `Ulana` (this repository's root) | `Ulana` | The frame types, `UlanaView`, `UlanaSessionView`, `UlanaTheme`, and `ServerSession`. Pure Swift. | The repository URL. |
| `UlanaLocal` ([`packages/swift-local`](../packages/swift-local)) | `UlanaLocal` | `LocalSession` and `LocalView`: the Rust runtime on the device. | A clone, after building the runtime. |

## Install

**Views on a server.** In Xcode, choose **File › Add Package Dependencies…**, enter `https://github.com/zobeli-dev/ulana` (rule: *Up to Next Major Version* from `0.1.0`), and add the `Ulana` library to your app. In a `Package.swift`:

```swift
dependencies: [
    .package(url: "https://github.com/zobeli-dev/ulana", from: "0.1.0"),
],
targets: [
    .target(name: "MyApp", dependencies: [.product(name: "Ulana", package: "ulana")]),
]
```

To reach `ulana serve` over plain `http` on your computer or local network, add `NSAppTransportSecurity › NSAllowsLocalNetworking = YES` to the app's `Info.plist`. On a real iPhone, also add an `NSLocalNetworkUsageDescription`.

**Views on the device.** You need Rust (`rustup`) and Xcode. Clone the repository and build the runtime once:

```sh
git clone https://github.com/zobeli-dev/ulana
cd ulana && scripts/build-xcframework.sh   # about two minutes; writes packages/swift-local/UlanaCore.xcframework
```

Then in Xcode choose **File › Add Package Dependencies… › Add Local…**, pick `packages/swift-local`, and add the `UlanaLocal` library. It brings `Ulana` with it. The example app's release build, runtime included, is under 4 MB.

## Views on a server

```swift
import SwiftUI
import Ulana

struct Screen: View {
    @State private var session = ServerSession(baseURL: URL(string: "http://127.0.0.1:8790")!)

    var body: some View {
        ScrollView {
            UlanaSessionView(session: session).padding()
        }
        .onAppear {
            session.start()          // lists the server's views and follows `name`
            session.name = "counter"
        }
        .onDisappear { session.stop() }
    }
}
```

`ServerSession` is `@Observable`. It has `views` (the names the server has), `name` (the one it follows; setting it switches), `frame`, `error`, `logs`, and `offline` (set while the server cannot be reached; it keeps trying). It long-polls the server, so a script saved there shows up at once, and a press goes back with `send(version:id:value:)`. The server is `ulana serve` or anything that speaks [its protocol](protocol.md).

## Views on the device

```swift
import SwiftUI
import Ulana
import UlanaLocal

struct Screen: View {
    @State private var session = LocalSession(
        source: try! String(contentsOf: Bundle.main.url(forResource: "counter", withExtension: "luau")!, encoding: .utf8),
        host: HostSpec(blocks: ["calendar"], actions: ["send"], images: ["https://images.example.com/"]),
        data: .object(["name": .string("Ana")]),
        state: Saved.load("counter")          // [String: JSONValue], or [:]
    )

    var body: some View {
        ScrollView { UlanaSessionView(session: session).padding() }
            .onAppear { session.onState = { Saved.store("counter", $0) } }
    }
}
```

- `LocalSession(source:host:data:state:)` loads and renders at once. A script that does not load leaves `frame` empty and `error` set.
- `session.data = …` renders again with new data.
- `session.reload(source:)` swaps the script and keeps the state. On an error the last good frame stays and `error` says what went wrong. This is how a model's new script goes in.
- `session.reset(source:)` forgets the state.
- `session.onState` is called with the state after every change. It is JSON (`[String: JSONValue]`), so store it anywhere: `UserDefaults`, a file, your server.
- `HostSpec` lists what the app accepts beyond the built-in blocks: `custom` block names, host action names, and image address prefixes. Without an image prefix, `image` blocks are refused.

Calls run on the main actor, capped at 100 ms and 16 MB each. A typical call takes well under a millisecond. `LocalView` is the lower level (`render(data:)`, `event(version:id:value:)`, `reload(source:)`, `state()`, `takeLogs()`) if you want your own threading. It is not thread-safe, so use it from one thread or actor.

## Drawing a frame yourself

`UlanaSessionView` covers most apps. To draw a `Frame` you got another way:

```swift
UlanaView(
    frame: frame,
    theme: .system,
    onEvent: { version, id, value in /* run the handler, then draw the next frame */ },
    onHostAction: { name, value in if name == "send" { chat.send(value.display) } },
    blocks: ["calendar": { props in AnyView(CalendarView(range: props.props["range"]?.display ?? "today")) }]
)
```

Blocks become SwiftUI's own views:

| Block | SwiftUI |
| --- | --- |
| `stack`, `row`, `grid` | `VStack`, `HStack` (or a wrapping layout), `LazyVGrid` |
| `text` | `Text` with Dynamic Type fonts; `display` is 52 pt rounded bold and grows with Dynamic Type |
| `button` | `Button` with `.borderedProminent`, `.bordered`, or `.borderless`, and an SF Symbol when it has an `icon` |
| `toggle`, `field`, `choice` | `Toggle`, `TextField`, `Picker(.segmented)` |
| `icon` | `Image(systemName:)`; `Token.Icon.systemName` maps each name to its SF Symbol |
| `image` | `AsyncImage` at the block's aspect ratio |
| `progress`, `divider` | `ProgressView`, `Divider` |

A block type this version does not know shows a short note instead of failing.

## Theme

`UlanaTheme.system` follows the system: the app's accent colour, semantic text colours, Dynamic Type, and dark mode. To change it:

```swift
var theme = UlanaTheme.system
theme.tones[.accent] = .init(fg: .orange, soft: .orange.opacity(0.15))
theme.space[.md] = 20
theme.fonts[.title] = .system(.largeTitle, design: .serif).bold()
theme.cardRadius = 20
UlanaSessionView(session: session, theme: theme)
```

## Tests

```sh
swift test                                  # in the repository root: Ulana
cd packages/swift-local && swift test       # UlanaLocal, after scripts/build-xcframework.sh
```

`SchemaTests` builds every block in `spec/frame.schema.json` with every field set and round-trips it through Swift, so the hand-written types cannot drift from the Rust ones.
