# Publishing

For maintainers. Ulana ships as four things: the `ulana` and `ulana-cli` crates on crates.io, `@zobeli/ulana-react-native` on npm, and the `Ulana` Swift package through git tags. The versions move together; this guide uses `0.1.0`.

## Before every release

```sh
git checkout main && git pull
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
swift test
(cd packages/react-native && bun run typecheck && bun test)
```

Set the version in `Cargo.toml` (`[workspace.package] version`, and the `version` on `ulana-cli`'s `ulana` dependency) and in `packages/react-native/package.json`. Merge that change first.

## crates.io

Once: create a token at <https://crates.io/settings/tokens> with the `publish-new` and `publish-update` scopes, then:

```sh
cargo login            # paste the token
```

Each release, the library first (the CLI depends on it):

```sh
cargo publish -p ulana --dry-run && cargo publish -p ulana
cargo publish -p ulana-cli --dry-run && cargo publish -p ulana-cli
```

`ulana-cli`'s dry run fails until that version of `ulana` is on crates.io, so run it after the first publish. `ulana-ffi` is not published (`publish = false`).

## npm

Once:

```sh
npm login              # the account that owns the zobeli scope
npm whoami             # should print zobeli
```

Each release:

```sh
cd packages/react-native
npm pack --dry-run     # check the files: src, README.md, LICENSE, package.json
npm publish --access public
```

A scoped package is private by default, which needs a paid plan; `--access public` (also set in `publishConfig`) makes it free and public. With two-factor authentication on, npm asks for a code.

## Swift

SwiftPM installs from git tags, so a release is a tag:

```sh
git tag 0.1.0 && git push origin 0.1.0
gh release create 0.1.0 --title "Ulana 0.1.0" --generate-notes
```

Apps can then use `.package(url: "https://github.com/zobeli-dev/ulana", from: "0.1.0")`. SwiftPM tags have no `v` prefix.

`UlanaLocal` still needs a clone and `scripts/build-xcframework.sh`. To install it by URL too, a release would attach `UlanaCore.xcframework.zip`, and the root `Package.swift` would point a `binaryTarget(url:checksum:)` at it.

## After the first release

Change the install lines in `README.md`, `docs/react-native.md`, `docs/rust.md`, and `docs/protocol.md` from the git forms (`cargo add ulana --git …`, copying the React Native package, `cargo install --git …`, `branch: "main"`) to the registry forms: `cargo add ulana`, `npm install @zobeli/ulana-react-native`, `cargo install ulana-cli`, and `from: "0.1.0"`.
