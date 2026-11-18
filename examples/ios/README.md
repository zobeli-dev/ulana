# Ulana example iPhone app

The same views two ways, switched at the top of the screen:

- **On this iPhone**: the views in `examples/views` are bundled with the app and run in the Rust runtime on the device. The menu edits a script in the app (it reloads as soon as you press Run, keeping its state), resets a view's state, or undoes your edits. Edits and state are kept across launches.
- **On a server**: the app draws whatever `ulana serve` runs and redraws a view as soon as its file is saved on the computer.

```sh
# from the repository root
brew install xcodegen
scripts/build-xcframework.sh                      # the runtime for "On this iPhone"
cargo run -p ulana-cli -- serve examples/views --block calendar --action send --image https://picsum.photos/   # for "On a server"
cd examples/ios && xcodegen && open UlanaExample.xcodeproj
```

Run it on a simulator, which reaches the computer at `http://127.0.0.1:8790`. On an iPhone, serve with `--bind 0.0.0.0:8790`, type the computer's address on the network in the app, and allow local network access when asked.

`calendar` is a stand-in for an app's own block, and a host action (`ui.host("send", …)`) shows what it would send at the bottom of the screen.
