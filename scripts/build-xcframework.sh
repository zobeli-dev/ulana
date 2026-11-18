#!/usr/bin/env bash
# Build UlanaCore.xcframework: the Rust runtime (crates/ulana-ffi) as a static
# library for iPhone, the iOS simulator (Apple silicon and Intel), and macOS,
# with its C header, into packages/swift-local, where the UlanaLocal package
# uses it to run views on the device.
#
#   scripts/build-xcframework.sh            # release build
#   scripts/build-xcframework.sh --debug    # faster to build, larger
set -euo pipefail
cd "$(dirname "$0")/.."

profile=release
flag=--release
if [[ "${1:-}" == "--debug" ]]; then
  profile=debug
  flag=
fi

targets=(aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios aarch64-apple-darwin)
installed=$(rustup target list --installed)
for t in "${targets[@]}"; do
  grep -qx "$t" <<<"$installed" || rustup target add "$t"
done

export IPHONEOS_DEPLOYMENT_TARGET=17.0
export MACOSX_DEPLOYMENT_TARGET=14.0
for t in "${targets[@]}"; do
  echo "building $t ($profile)"
  cargo build -p ulana-ffi $flag --target "$t"
done

headers=target/xcframework/headers
rm -rf target/xcframework
mkdir -p "$headers"
cp crates/ulana-ffi/include/ulana.h crates/ulana-ffi/include/module.modulemap "$headers/"

out=packages/swift-local/UlanaCore.xcframework
rm -rf "$out"
# The simulator slice holds both simulator architectures in one library.
sim=target/xcframework/ios-simulator
mkdir -p "$sim"
lipo -create \
  "target/aarch64-apple-ios-sim/$profile/libulana_ffi.a" \
  "target/x86_64-apple-ios/$profile/libulana_ffi.a" \
  -output "$sim/libulana_ffi.a"
args=()
for lib in "target/aarch64-apple-ios/$profile/libulana_ffi.a" "$sim/libulana_ffi.a" "target/aarch64-apple-darwin/$profile/libulana_ffi.a"; do
  args+=(-library "$lib" -headers "$headers")
done
xcodebuild -create-xcframework "${args[@]}" -output "$out" >/dev/null
echo "wrote $out"
du -sh "$out"/*/libulana_ffi.a
