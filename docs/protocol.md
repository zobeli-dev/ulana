# The frame format and the `ulana serve` protocol

## A frame

What a view renders, and what an app draws:

```json
{
  "version": 3,
  "root": {
    "type": "card", "title": "Counter", "tone": "accent",
    "children": [
      { "type": "text", "text": "2", "style": "display" },
      { "type": "button", "label": "More", "icon": "add", "action": { "kind": "event", "id": "h1" } },
      { "type": "button", "label": "Ask", "action": { "kind": "host", "name": "send", "value": "What's next?" } }
    ]
  }
}
```

- `version` goes up with every render. A press names the version it saw, and a press on an older frame is refused, so a quick double tap never runs against a frame that is already gone.
- `root` is a block. Every block has a `type`; containers (`stack`, `row`, `grid`, `card`, `list`) have `children`. [`spec/ulana.md`](../spec/ulana.md) lists every block and field, and [`spec/frame.schema.json`](../spec/frame.schema.json) is the JSON Schema, generated from the Rust types.
- An action is `{ "kind": "event", "id" }`, which runs the view's own handler, or `{ "kind": "host", "name", "value" }`, which goes to the app.
- The value sent back with an event is `null` for a press, a boolean for a toggle, and a string for a field or a choice.

## `ulana serve`

`ulana serve <folder>` runs every `<name>.luau` in the folder, with `<name>.json` as its data when there is one. It reloads a view when either file changes, keeping its state.

```sh
cargo install --git https://github.com/zobeli-dev/ulana ulana-cli
ulana serve views --block calendar --action send --image https://images.example.com/
```

| Flag | |
| --- | --- |
| `--bind <address>` | Default `127.0.0.1:8790`. Use `0.0.0.0:8790` for a phone on the same network. |
| `--block <name>` | Accept `ui.custom(name, …)`. Repeatable. |
| `--action <name>` | Accept `ui.host(name, …)`. Repeatable. |
| `--image <prefix>` | Show images whose address starts with this. Repeatable. |
| `--edit` | Let clients write scripts (`PUT …/source`). Only on a network you trust. |

Every answer is JSON, with open CORS so a web app can call it. An error is `{ "error": "…" }` with a 4xx or 5xx status.

| Request | Answer |
| --- | --- |
| `GET /views` | `[{ "name", "revision" }]` |
| `GET /views/<name>` | A snapshot, at once. |
| `GET /views/<name>?since=<revision>` | A snapshot once the view's revision is past `since`, or after 25 seconds with the current one (long polling). |
| `POST /views/<name>/events` with `{ "version", "id", "value" }` | The snapshot after the handler ran. A press on an old frame changes nothing and gets the latest snapshot. |
| `GET /views/<name>/source` | `{ "name", "source", "data" }` |
| `PUT /views/<name>/source` with `{ "source" }` | Only with `--edit`, else 403. Writes `<name>.luau`, loads it at once, and answers with the snapshot. A new name makes a new view. |
| `GET /host` | `{ "spec", "blocks", "actions", "images", "edit" }`: the view API and what the server accepts, for asking a model to write a view. |

A snapshot:

```json
{ "name": "counter", "revision": 7, "frame": { "version": 4, "root": { … } }, "error": null, "logs": ["printed line"] }
```

- `revision` goes up whenever the frame, the error, or the logs change. Follow a view by asking `?since=` with the last revision you saw.
- `frame` is the last frame that rendered. It stays while the script has an error, so the app keeps showing something.
- `error` is the script's current error, or `null`.
- `logs` holds the last 50 lines the script printed.

`ServerSession` (Swift) and `useServerView` (React Native) speak this protocol. Your own server can speak it too, with the [Rust crate](rust.md).
