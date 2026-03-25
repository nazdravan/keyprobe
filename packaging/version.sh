#!/usr/bin/env bash
# KeyProbe's version (one place: [workspace.package] version in Cargo.toml).
#   packaging/version.sh              print it
#   packaging/version.sh set 0.2.0    change it in Cargo.toml and Cargo.lock
#   packaging/version.sh set 0.2.0-rc.1
set -euo pipefail
# shellcheck source=packaging/env.sh
. "$(dirname "$0")/env.sh"
cd "$ROOT"

case "${1:-}" in
    "")
        echo "$VERSION"
        ;;
    set)
        v="${2:?missing version, e.g. 0.2.0}"
        v="${v#v}"
        if ! printf '%s' "$v" | grep -Eq '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$'; then
            echo "\"$v\" is not a version like 1.2.3 or 1.2.3-rc.1" >&2
            exit 1
        fi
        sed -i.bak "/^\[workspace\.package\]/,/^\[/s/^version = \".*\"/version = \"$v\"/" Cargo.toml
        rm Cargo.toml.bak
        cargo update --workspace --quiet
        echo "$VERSION -> $v"
        ;;
    *)
        echo "usage: $0 [set <version>]" >&2
        exit 2
        ;;
esac
