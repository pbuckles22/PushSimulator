#!/bin/sh
# Build the UniFFI xcframework and the iOS app that shows Game.get_deck_size().
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
cd "$ROOT"
TARGET_DIR=${CARGO_TARGET_DIR:-$ROOT/target}
GEN=$ROOT/ios/PushApp/Generated
HEADERS=$ROOT/ios/generated/headers

cargo build -p push_ffi --release
cargo run -p push_ffi --release --bin uniffi-bindgen -- generate \
    --library "$TARGET_DIR/release/libpush_ffi.dylib" \
    --language swift \
    --out-dir "$GEN"

CHECK=$(mktemp -d)
swiftc -o "$CHECK/deck-size" \
    -L "$TARGET_DIR/release" \
    -lpush_ffi \
    -Xcc -fmodule-map-file="$GEN/push_ffiFFI.modulemap" \
    -I "$GEN" \
    "$GEN/push_ffi.swift" \
    "$ROOT/ios/deck_size_check.swift"
DYLD_LIBRARY_PATH="$TARGET_DIR/release" "$CHECK/deck-size"

cargo build -p push_ffi --release --target aarch64-apple-ios
cargo build -p push_ffi --release --target aarch64-apple-ios-sim

rm -rf "$HEADERS" "$ROOT/ios/PushCore.xcframework"
mkdir -p "$HEADERS"
cp "$GEN/push_ffiFFI.h" "$HEADERS/push_ffiFFI.h"
cp "$GEN/push_ffiFFI.modulemap" "$HEADERS/module.modulemap"

xcodebuild -create-xcframework \
    -library "$TARGET_DIR/aarch64-apple-ios/release/libpush_ffi.a" -headers "$HEADERS" \
    -library "$TARGET_DIR/aarch64-apple-ios-sim/release/libpush_ffi.a" -headers "$HEADERS" \
    -output "$ROOT/ios/PushCore.xcframework"

cd "$ROOT/ios"
xcodebuild \
    -scheme PushApp \
    -destination 'generic/platform=iOS' \
    -derivedDataPath "$TARGET_DIR/ios" \
    CODE_SIGNING_ALLOWED=NO \
    build
