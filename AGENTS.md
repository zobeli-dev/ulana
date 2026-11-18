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
| `crates/ulana-cli` | The `ulana` binary. |
| `packages/react-native` | The renderer. `src/types.ts` mirrors `block.rs` by hand; `test/sync.test.ts` fails when they drift. |
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
cd packages/react-native && bun install && bun run typecheck && bun test
```

Commits are conventional commits (`feat:`, `fix:`, `docs:`, `chore:`). One branch, one change.
