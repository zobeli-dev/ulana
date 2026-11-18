# ulana-cli

The `ulana` command for [Ulana](https://github.com/zobeli-dev/ulana): Luau scripts that draw native UI.

```sh
cargo install ulana-cli
ulana serve views --block calendar --action send   # serve a folder of views, reloaded when saved
ulana render views/counter.luau --event h1         # print a frame as JSON
ulana check tree.json                              # check a tree written as JSON
ulana schema                                       # the JSON Schema of a frame
```

`ulana serve` is what the SwiftUI and React Native packages follow; its protocol is in [docs/protocol.md](https://github.com/zobeli-dev/ulana/blob/main/docs/protocol.md). MIT licensed.
