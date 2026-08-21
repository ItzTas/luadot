#!/usr/bin/env bash
set -euo pipefail

die() {
	echo "meta: $1" >&2
	exit 1
}

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
target="$root/meta/ld.lua"

check=0
if [ "${1-}" = '--check' ]; then
	check=1
	shift
fi

[ $# -eq 0 ] || die "usage: $0 [--check]"

generated=$(mktemp)
trap 'rm -f "$generated"' EXIT

(cd "$root" && cargo run --quiet --features meta --bin luadot-meta >"$generated") ||
	die 'the generator failed'
[ -s "$generated" ] || die 'the generator wrote nothing'

if [ "$check" -eq 1 ]; then
	[ -f "$target" ] || die 'nothing generated at meta/ld.lua'
	diff -u "$target" "$generated" ||
		die 'meta/ld.lua does not match the ld surface; run packaging/meta/update.sh'
	echo 'meta: meta/ld.lua is current'
	exit 0
fi

install -Dm644 "$generated" "$target"
echo "meta: wrote meta/ld.lua ($(wc -l <"$target") lines)"
