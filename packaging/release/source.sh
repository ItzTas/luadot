#!/usr/bin/env bash
set -euo pipefail

die() {
	echo "release: $1" >&2
	exit 1
}

[ $# -eq 1 ] || die "usage: $0 <tag>"

tag=$1
[ -n "$tag" ] || die 'no tag given; nothing was released'

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$root"

git rev-parse --verify --quiet "$tag^{commit}" >/dev/null || die "$tag is not in this repository"

pkgver=${tag#v}
pkgver=${pkgver//-/.}

dist="$root/dist"
mkdir -p "$dist"

archive="$dist/luadot-$pkgver-src.tar.gz"
git archive --format=tar --prefix="luadot-$pkgver/" "$tag" | gzip -9n >"$archive"

echo "release: packed luadot-$pkgver-src.tar.gz"
