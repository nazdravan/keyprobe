#!/usr/bin/env bash
# Linux x86_64 AppImage: dist/release/keyprobe-<version>-linux-x86_64.AppImage
# One file: chmod +x and run, nothing to install. appimagetool is downloaded to .cache/ once.
set -euo pipefail
# shellcheck source=packaging/env.sh
. "$(dirname "$0")/../env.sh"
cd "$ROOT"

TOOL_URL=https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
tool=$ROOT/.cache/appimagetool-x86_64.AppImage
if [ ! -x "$tool" ]; then
    mkdir -p "$ROOT/.cache"
    curl -fsSL -o "$tool.part" "$TOOL_URL"
    chmod +x "$tool.part"
    mv "$tool.part" "$tool"
fi

cargo build --release --locked -p keyprobe

name=$APP-$VERSION-linux-x86_64
appdir=$ROOT/dist/stage/KeyProbe.AppDir
rm -rf "${appdir:?}" "${OUT:?}/${name:?}.AppImage"
mkdir -p "$appdir/usr/bin" "$appdir/usr/share/applications" \
    "$appdir/usr/share/icons/hicolor/256x256/apps" "$appdir/usr/share/doc/keyprobe" "$OUT"
install -m 755 target/release/keyprobe "$appdir/usr/bin/keyprobe"
cp packaging/linux/keyprobe.desktop "$appdir/usr/share/applications/"
cp assets/icon.png "$appdir/usr/share/icons/hicolor/256x256/apps/keyprobe.png"
cp README.md LICENSE-MIT LICENSE-APACHE assets/fonts/UFL.txt assets/fonts/Hack-Regular.txt \
    "$appdir/usr/share/doc/keyprobe/"
# appimagetool wants the .desktop file and the icon in the AppDir root too.
ln -s usr/share/applications/keyprobe.desktop "$appdir/keyprobe.desktop"
ln -s usr/share/icons/hicolor/256x256/apps/keyprobe.png "$appdir/keyprobe.png"
ln -s keyprobe.png "$appdir/.DirIcon"
ln -s usr/bin/keyprobe "$appdir/AppRun"

# No FUSE in containers and CI: appimagetool extracts itself and runs from a folder.
APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 VERSION=$VERSION \
    "$tool" --no-appstream "$appdir" "$OUT/$name.AppImage"
echo "done: $OUT/$name.AppImage"
