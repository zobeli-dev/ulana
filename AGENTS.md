# Ulana

Luau scripts that draw native UI. Read [README.md](README.md) for what it is and [spec/ulana.md](spec/ulana.md) for the API a view sees.

## Layout

| Path | What it is |
| --- | --- |
| `crates/ulana/src/block.rs` | The blocks, actions, and tokens. The source of truth for the tree. |
| `crates/ulana/src/check.rs` | `Host`, `Limits`, `parse`, and the checker every tree passes. |
| `crates/ulana/src/runtime.rs` | `View`: the sandbox, the clock, state, events, reload. |
| `crates/ulana/src/prelude.luau` | The `ui` library, loaded before the sandbox turns on. |
| `crates/ulana/src/convert.rs` | Luau values to JSON and back, with the checks a script cannot skip. |
| `crates/ulana/tests/sandbox.rs` | Tests that try to get out of the sandbox. |
| `crates/ulana-cli/src/main.rs` | The `ulana` binary: `render`, `check`, `serve`, `schema`. |
| `crates/ulana-cli/src/serve.rs` | `ulana serve`: one thread owns every view, rescans the folder every 250 ms, and answers long polls when a view's revision moves. |
| `packages/react-native` | The renderer. `src/types.ts` mirrors `block.rs` by hand; `test/sync.test.ts` fails when they drift. |
| `packages/react-native/test/render.test.tsx` | Draws every block through react-native-web (`test/setup.ts` swaps the module), no device needed. |
| `examples/views` | Views `ulana serve examples/views` runs. |
| `examples/expo` | The example app. A Bun workspace with `packages/*`, so there is one copy of React; install from the root. |
| `spec/frame.schema.json` | Generated. `ULANA_BLESS=1 cargo test -p ulana --test schema` rewrites it. |

## Rules

- A view is untrusted code. Never give it a way to reach the network, files, the host's tools, or another view. A new capability goes through a host action the host names, not a new global.
- Every tree is checked in Rust before it is drawn, wherever it came from. Do not move a check into a renderer.
- Changing a block means changing `block.rs`, the checker if it has rules, `prelude.luau` if the Luau call changes, `spec/ulana.md`, `types.ts`, `UlanaView.tsx`, the sync test's field list, and the schema (bless it).
- Errors are read by models. Name the path and say what would be accepted.

## Check

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
bun install
(cd packages/react-native && bun run typecheck && bun test)
(cd examples/expo && bun run typecheck && CI=1 bunx expo export --platform ios --platform web --output-dir /tmp/ulana-export)
```

Commits are conventional commits (`feat:`, `fix:`, `docs:`, `chore:`). One branch, one change.
