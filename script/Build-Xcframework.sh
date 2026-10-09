#!/bin/sh
# Build the UniFFI xcframework and the iOS app that shows the shoe and the board.
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

swift build -c release --package-path "$ROOT/ios/PushUI"
UI_PRODUCTS=$ROOT/ios/PushUI/.build/out/Products/Release
if [ ! -d "$UI_PRODUCTS" ]; then
    UI_PRODUCTS=$ROOT/ios/PushUI/.build/release
fi
swiftc -o "$CHECK/board-ui" \
    -I "$UI_PRODUCTS" \
    -L "$UI_PRODUCTS" \
    -lPushUI \
    -L "$TARGET_DIR/release" \
    -lpush_ffi \
    -Xcc -fmodule-map-file="$GEN/push_ffiFFI.modulemap" \
    -I "$GEN" \
    "$GEN/push_ffi.swift" \
    "$ROOT/ios/PushApp/LiveTable.swift" \
    "$ROOT/ios/board_ui_check.swift"
DYLD_LIBRARY_PATH="$TARGET_DIR/release" "$CHECK/board-ui"

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

# The package build is a library. The booted simulator keeps the last installed app.
UDID=$(xcrun simctl list devices booted | awk -F '[()]' '/Booted/ { print $2; exit }')
if [ -n "$UDID" ]; then
    SDK=$(xcrun --sdk iphonesimulator --show-sdk-path)
    SIM_OUT=$(mktemp -d)
    UI=$ROOT/ios/PushUI/Sources/PushUI
    swiftc -wmo -c \
        -sdk "$SDK" -target arm64-apple-ios16.0-simulator \
        -module-name PushUI \
        -emit-module -emit-module-path "$SIM_OUT/PushUI.swiftmodule" \
        -o "$SIM_OUT/PushUI.o" \
        "$UI/BoardCard.swift" \
        "$UI/CardFace.swift" \
        "$UI/CardView.swift" \
        "$UI/BoardView.swift" \
        "$UI/PublishedTable.swift" \
        "$UI/GameBoardScreen.swift"
    swiftc -parse-as-library \
        -sdk "$SDK" -target arm64-apple-ios16.0-simulator \
        -module-name PushApp \
        -I "$SIM_OUT" \
        -I "$GEN" \
        -Xcc -fmodule-map-file="$GEN/push_ffiFFI.modulemap" \
        -o "$SIM_OUT/PushApp" \
        "$GEN/push_ffi.swift" \
        "$ROOT/ios/PushApp/LiveTable.swift" \
        "$ROOT/ios/PushApp/App.swift" \
        "$SIM_OUT/PushUI.o" \
        "$ROOT/ios/PushCore.xcframework/ios-arm64-simulator/libpush_ffi.a" \
        -framework SwiftUI \
        -framework Foundation
    mkdir -p "$SIM_OUT/PushApp.app"
    cp "$SIM_OUT/PushApp" "$SIM_OUT/PushApp.app/PushApp"
    cat > "$SIM_OUT/PushApp.app/Info.plist" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDisplayName</key><string>Push</string>
  <key>CFBundleExecutable</key><string>PushApp</string>
  <key>CFBundleIdentifier</key><string>com.pushsimulator.app</string>
  <key>CFBundleName</key><string>Push</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>0.3.2</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>LSRequiresIPhoneOS</key><true/>
  <key>MinimumOSVersion</key><string>16.0</string>
  <key>UIDeviceFamily</key><array><integer>1</integer></array>
  <key>UILaunchScreen</key><dict/>
  <key>UISupportedInterfaceOrientations</key>
  <array><string>UIInterfaceOrientationPortrait</string></array>
</dict>
</plist>
EOF
    codesign --force --sign - "$SIM_OUT/PushApp.app"
    xcrun simctl install "$UDID" "$SIM_OUT/PushApp.app"
    xcrun simctl terminate "$UDID" com.pushsimulator.app || true
    xcrun simctl launch "$UDID" com.pushsimulator.app
fi

cd "$ROOT/ios"
# An empty PushApp.xcodeproj has no pbxproj. Xcode then ignores Package.swift.
PROJ=$ROOT/ios/PushApp.xcodeproj
STASH=
if [ -d "$PROJ" ] && [ ! -f "$PROJ/project.pbxproj" ]; then
    STASH=$(mktemp -d)
    mv "$PROJ" "$STASH/PushApp.xcodeproj"
fi
set +e
xcodebuild \
    -scheme PushApp \
    -destination 'generic/platform=iOS' \
    -derivedDataPath "$TARGET_DIR/ios" \
    CODE_SIGNING_ALLOWED=NO \
    build
STATUS=$?
set -e
if [ -n "$STASH" ]; then
    mv "$STASH/PushApp.xcodeproj" "$PROJ"
    rmdir "$STASH"
fi
exit "$STATUS"
