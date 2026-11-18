# Ulana example app

Draws the views `ulana serve` runs, with a chat that asks a model to change them. A view also redraws as soon as its file is saved by hand. Either way its state (a count, a to-do list) survives.

```sh
# from the repository root
bun install
cargo run -p ulana-cli -- serve examples/views --edit --block calendar --action send
# in another terminal
cd examples/expo
cp .env.example .env
bun run web        # or: bun run ios, bun run android (Expo Go works; no native modules)
```

`.env` sets where the server is (`EXPO_PUBLIC_ULANA_URL`, default `http://127.0.0.1:8790`, or `http://10.0.2.2:8790` on the Android emulator) and the model (`EXPO_PUBLIC_ULANA_MODEL`, default `z-ai/glm-5.3-flash`). Restart Expo after changing it. For a phone on the same network, serve with `--bind 0.0.0.0:8790` and set the URL to this machine's address.

## The chat

1. Paste an OpenRouter key in the chat panel and press Connect. The app checks it with OpenRouter and keeps it on the device (the keychain on a phone, this browser's storage on the web). It is sent only to OpenRouter.
2. Ask for a change to the view that is open: "make the count huge and orange", "show each event as a card", "add a summary at the top".
3. The model gets the view API ([`spec/ulana.md`](../../spec/ulana.md), served at `GET /host`), the script, the data the script reads, the colours of the app's tones, and the request. It answers with a sentence and the whole new script, which goes to `PUT /views/<name>/source`. The server loads it at once. When it fails, the error goes back to the model to fix, up to twice.
4. Each change shows its time and cost, and Undo puts the old script back.

`+ New view` adds an empty view for the chat to build. The server only takes scripts when it runs with `--edit`; without it the chat says so. Changes are written into the served folder, so `git checkout examples/views` puts the examples back.

## Also

- A tab per view in the folder; a new `.luau` file shows up within two seconds.
- A view with an error keeps its last good frame, with the error above it.
- `calendar` is a stand-in for an app's own block (`ui.custom("calendar", …)`).
- A host action (`ui.host("send", …)`) shows what it would send at the bottom of the screen.
- `print` output from the view is under "Show print output".
