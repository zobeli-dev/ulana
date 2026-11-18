# ulana

Run [Ulana](https://github.com/zobeli-dev/ulana) views: Luau scripts that build native UI, in a sandbox with limits on memory and time, checked before an app draws them.

```rust
use ulana::{Host, Limits, View};

let mut view = View::new(source, Host::new().action("send"), Limits::default())?;
let frame = view.render(&serde_json::json!({}))?; // send this JSON to the app
```

Guide: [docs/rust.md](https://github.com/zobeli-dev/ulana/blob/main/docs/rust.md). The view API: [spec/ulana.md](https://github.com/zobeli-dev/ulana/blob/main/spec/ulana.md). MIT licensed.
