#!/usr/bin/env bash
# macOS app: dist/release/keyprobe-<version>-macos-universal.dmg
# A universal KeyProbe.app (Apple Silicon + Intel). Runs on a Mac with the Xcode Command Line Tools.
set -euo pipefail
# shellcheck source=packaging/env.sh
. "$(dirname "$0")/../env.sh"
cd "$ROOT"

# The oldest macOS supported (wgpu's Metal backend needs 10.13).
export MACOSX_DEPLOYMENT_TARGET=10.13
targets=(aarch64-apple-darwin x86_64-apple-darwin)
if command -v rustup >/dev/null; then
    rustup target add "${targets[@]}"
fi
for t in "${targets[@]}"; do
    cargo build --release --locked -p keyprobe --target "$t"
done

name=$APP-$VERSION-macos-universal
stage=$ROOT/dist/stage/$name
app=$stage/KeyProbe.app
rm -rf "${stage:?}" "${OUT:?}/${name:?}.dmg"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources" "$OUT"
lipo -create -output "$app/Contents/MacOS/keyprobe" \
    target/aarch64-apple-darwin/release/keyprobe target/x86_64-apple-darwin/release/keyprobe
sed -e "s/@VERSION@/$VERSION/g" -e "s/@MIN_MACOS@/$MACOSX_DEPLOYMENT_TARGET/g" \
    packaging/macos/Info.plist >"$app/Contents/Info.plist"
cp assets/keyprobe.icns "$app/Contents/Resources/keyprobe.icns"
cp LICENSE-MIT LICENSE-APACHE assets/fonts/UFL.txt assets/fonts/Hack-Regular.txt "$app/Contents/Resources/"

# Ad-hoc signature: without one the app doesn't start on Apple Silicon at all.
# It does not replace signing with an Apple Developer certificate (see README, macOS).
codesign --force --deep --sign - "$app"
codesign --verify --deep --strict "$app"
lipo -info "$app/Contents/MacOS/keyprobe"

# The disk image: KeyProbe.app next to an Applications shortcut, so installing is one drag.
ln -s /Applications "$stage/Applications"
hdiutil create -volname "KeyProbe $VERSION" -srcfolder "$stage" -fs HFS+ \
    -format UDZO -imagekey zlib-level=9 -ov "$OUT/$name.dmg"
hdiutil verify "$OUT/$name.dmg"
echo "done: $OUT/$name.dmg"
