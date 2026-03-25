#!/usr/bin/env bash
# Debian / Ubuntu package: dist/release/keyprobe-<version>-linux-x86_64.deb
# Install with `sudo apt install ./keyprobe-*.deb`. Needs dpkg-deb.
set -euo pipefail
# shellcheck source=packaging/env.sh
. "$(dirname "$0")/../env.sh"
cd "$ROOT"

cargo build --release --locked -p keyprobe

name=$APP-$VERSION-linux-x86_64
pkg=$ROOT/dist/stage/deb
rm -rf "${pkg:?}" "${OUT:?}/${name:?}.deb"
mkdir -p "$pkg/DEBIAN" "$pkg/usr/bin" "$pkg/usr/share/applications" \
    "$pkg/usr/share/icons/hicolor/256x256/apps" "$pkg/usr/share/doc/keyprobe" "$OUT"
install -m 755 target/release/keyprobe "$pkg/usr/bin/keyprobe"
install -m 644 packaging/linux/keyprobe.desktop "$pkg/usr/share/applications/keyprobe.desktop"
install -m 644 assets/icon.png "$pkg/usr/share/icons/hicolor/256x256/apps/keyprobe.png"
install -m 644 README.md "$pkg/usr/share/doc/keyprobe/README.md"
{
    echo "Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/"
    echo "Upstream-Name: keyprobe"
    echo "Source: https://github.com/nazdravan/keyprobe"
    echo
    echo "Files: *"
    echo "License: MIT or Apache-2.0"
    echo
    echo "License: MIT"
    sed 's/^/ /; s/^ $/ ./' LICENSE-MIT
    echo
    echo "License: Apache-2.0"
    echo " On Debian systems, the full text is in /usr/share/common-licenses/Apache-2.0."
    echo
    echo "Files: fonts built into the binary"
    echo "License: Ubuntu-Font-Licence-1.0 and Hack"
    sed 's/^/ /; s/^ $/ ./' assets/fonts/UFL.txt assets/fonts/Hack-Regular.txt
} >"$pkg/usr/share/doc/keyprobe/copyright"

# Debian versions use ~ for pre-releases: 0.2.0-rc.1 -> 0.2.0~rc.1.
debver=${VERSION/-/\~}
size=$(du -sk "$pkg/usr" | cut -f1)
cat >"$pkg/DEBIAN/control" <<CONTROL
Package: keyprobe
Version: $debver
Architecture: amd64
Maintainer: nazdravan <nazdravan@users.noreply.github.com>
Installed-Size: $size
Depends: libc6 (>= 2.35), libxkbcommon0
Recommends: libvulkan1 | libegl1, libxkbcommon-x11-0, libwayland-client0
Section: utils
Priority: optional
Homepage: https://github.com/nazdravan/keyprobe
Description: keyboard tester
 KeyProbe lights up every key you press on an on-screen keyboard and shows
 the codes your system sent for it: W3C code, Windows virtual-key code,
 PS/2 scan code and USB HID usage. Detects key chatter and stuck keys.
CONTROL

dpkg-deb --root-owner-group -Zxz --build "$pkg" "$OUT/$name.deb"
echo "done: $OUT/$name.deb"
