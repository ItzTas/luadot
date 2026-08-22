#!/usr/bin/env bash
set -euo pipefail

crate='luadot'
registry='https://crates.io/api/v1/crates'
user_agent='luadot-release (https://github.com/ItzTas/luadot)'

die() {
	echo "release: $1" >&2
	exit 1
}

[ $# -eq 1 ] || die "usage: $0 <tag>"

tag=$1
[ -n "$tag" ] || die 'no tag given; nothing was released'

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$root"

version=${tag#v}

case $version in
*-*)
	echo "release: $version is a pre-release; crates.io carries stable releases only"
	exit 0
	;;
esac

manifest=$(sed -n '0,/^version = /s/^version = "\(.*\)"/\1/p' Cargo.toml)
[ "$manifest" = "$version" ] || die "Cargo.toml is at $manifest, not $version"

if curl -fsS -A "$user_agent" "$registry/$crate/$version" >/dev/null 2>&1; then
	echo "release: $crate $version is already on crates.io"
	exit 0
fi

[ -n "${CARGO_REGISTRY_TOKEN:-}" ] || die 'CARGO_REGISTRY_TOKEN is not set'

cargo publish --locked

echo "release: published $crate $version to crates.io"
