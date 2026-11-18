# Ulana example app

Draws the views `ulana serve` runs, and redraws one as soon as its file is saved. Its state (a count, a to-do list) survives the reload.

```sh
# from the repository root
bun install
cargo run -p ulana-cli -- serve examples/views --block calendar --action send
# in another terminal
cd examples/expo
bun run web        # or: bun run ios, bun run android (Expo Go works; no native modules)
```

The app talks to `http://127.0.0.1:8790` (`http://10.0.2.2:8790` on the Android emulator). For a phone on the same network, serve with `--bind 0.0.0.0:8790` and set `EXPO_PUBLIC_ULANA_URL=http://<this machine's address>:8790`.

- A tab per view in the folder; a new `.luau` file shows up within two seconds.
- A view with an error keeps its last good frame, with the error above it.
- `calendar` is a stand-in for an app's own block (`ui.custom("calendar", …)`).
- A host action (`ui.host("send", …)`) shows what it would send at the bottom of the screen.
- `print` output from the view is under "Show print output".
