#!/bin/sh
# Generate the Swift bridge and test that their deck pile shows the undealt shoe.
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
TARGET="$ROOT/target"
SHOE="$ROOT/ios/CardGame/PushShoe"
GEN="$SHOE/Generated"
FFI_SRC="$SHOE/Sources/PushFfi"

cargo build -p push_ffi --release --target-dir "$TARGET"
mkdir -p "$GEN" "$FFI_SRC"
cargo run -p push_ffi --release --target-dir "$TARGET" --bin uniffi-bindgen -- generate \
    --library "$TARGET/release/libpush_ffi.dylib" \
    --language swift \
    --out-dir "$GEN"
cp "$GEN/push_ffi.swift" "$FFI_SRC/push_ffi.swift"
cp "$GEN/push_ffiFFI.modulemap" "$GEN/module.modulemap"

cd "$SHOE"
swift test \
    -Xlinker -force_load -Xlinker "$TARGET/release/libpush_ffi.a"
