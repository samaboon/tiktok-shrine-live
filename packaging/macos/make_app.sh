#!/usr/bin/env bash
# Build "TikTok Shrine Live.app" (universal arm64 + x86_64) and a .dmg. Run on macOS.
set -euo pipefail
cd "$(dirname "$0")/../.."
VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
APP="dist/TikTok Shrine Live.app"
rm -rf dist && mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
lipo -create -output "$APP/Contents/MacOS/tiktok-shrine-live" \
  target/aarch64-apple-darwin/release/tiktok-shrine-live \
  target/x86_64-apple-darwin/release/tiktok-shrine-live
sed "s/__VERSION__/$VERSION/g" packaging/macos/Info.plist > "$APP/Contents/Info.plist"
# icon
ICONSET=dist/AppIcon.iconset; mkdir -p $ICONSET
for s in 16 32 64 128 256 512; do
  sips -z $s $s assets/icon/icon.png --out $ICONSET/icon_${s}x${s}.png >/dev/null
  d=$((s*2)); sips -z $d $d assets/icon/icon.png --out $ICONSET/icon_${s}x${s}@2x.png >/dev/null
done
iconutil -c icns $ICONSET -o "$APP/Contents/Resources/AppIcon.icns"
# bundled Node.js runtime for the TikTok connector: the Mach-O goes next to the main binary (code location),
# its licence/version under Resources so nothing but code sits in Contents/MacOS
RT=dist/rt-mac
packaging/fetch_node.sh mac "$RT"
cp "$RT/node" "$APP/Contents/MacOS/node"
mkdir -p "$APP/Contents/Resources/runtime"
cp "$RT/LICENSE-node.txt" "$RT/NODE_VERSION" "$APP/Contents/Resources/runtime/"
# ad-hoc signature (not notarized -> first open needs approval, see docs/ติดตั้ง-Mac.md)
# sign the nested node first, then the bundle (seals it as part of the app)
codesign --force -s - "$APP/Contents/MacOS/node"
codesign --force -s - "$APP"
codesign --verify --deep --strict --verbose=2 "$APP"
# dmg
mkdir -p dist/dmg && cp -R "$APP" dist/dmg/ && ln -s /Applications dist/dmg/Applications
cp docs/คู่มือ.md docs/ติดตั้ง-Mac.md dist/dmg/ 2>/dev/null || true
hdiutil create -volname "TikTok Shrine Live" -srcfolder dist/dmg -ov -format UDZO "dist/TikTokShrineLive-macOS-universal.dmg"
echo "built: dist/TikTokShrineLive-macOS-universal.dmg"
# zip of the .app for the in-app updater (keeps signature/permissions)
(cd dist && ditto -c -k --sequesterRsrc --keepParent "TikTok Shrine Live.app" "TikTokShrineLive-macOS-universal.zip")
echo "built: dist/TikTokShrineLive-macOS-universal.zip"
