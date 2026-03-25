#!/usr/bin/env bash
# Linux x86_64 archive: dist/release/keyprobe-<version>-linux-x86_64.tar.gz
set -euo pipefail
# shellcheck source=packaging/env.sh
. "$(dirname "$0")/../env.sh"
cd "$ROOT"

cargo build --release --locked -p keyprobe

name=$APP-$VERSION-linux-x86_64
stage=$ROOT/dist/stage/$name
rm -rf "${stage:?}" "${OUT:?}/${name:?}.tar.gz"
mkdir -p "$stage/licenses" "$OUT"
install -m 755 target/release/keyprobe "$stage/keyprobe"
cp README.md LICENSE-MIT LICENSE-APACHE "$stage/"
cp assets/fonts/UFL.txt assets/fonts/Hack-Regular.txt "$stage/licenses/"
tar -C "$ROOT/dist/stage" -czf "$OUT/$name.tar.gz" "$name"
echo "done: $OUT/$name.tar.gz"
