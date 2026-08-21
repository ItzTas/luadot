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

pkgver=${tag#v}
pkgver=${pkgver//-/.}

declare -A rust_targets=(
	[x86_64]=x86_64-unknown-linux-gnu
	[aarch64]=aarch64-unknown-linux-gnu
)

declare -A completion_files=(
	[bash]=luadot.bash
	[zsh]=_luadot
	[fish]=luadot.fish
)

host=$(uname -m)
[ -n "${rust_targets[$host]+set}" ] || die "$host is not a released architecture"

export RUSTUP_TOOLCHAIN=stable
export RUSTFLAGS="${RUSTFLAGS:-} -C strip=symbols"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar

dist="$root/dist"
rm -rf "$dist"
mkdir -p "$dist"

for arch in "${!rust_targets[@]}"; do
	echo "release: building $pkgver for $arch"
	cargo build --release --locked --target "${rust_targets[$arch]}"
done

completions="$dist/completions"
mkdir -p "$completions"
for shell in "${!completion_files[@]}"; do
	"target/${rust_targets[$host]}/release/luadot" completions "$shell" \
		>"$completions/${completion_files[$shell]}"
done

manual="$dist/luadot.1"
"target/${rust_targets[$host]}/release/luadot" man >"$manual"

for arch in "${!rust_targets[@]}"; do
	stage="$dist/luadot-$pkgver-$arch"
	install -Dm0755 "target/${rust_targets[$arch]}/release/luadot" "$stage/luadot"
	install -Dm0644 LICENSE "$stage/LICENSE"
	install -Dm0644 vendor/lpeg/LICENSE "$stage/LICENSE.lpeg"
	install -Dm0644 "$manual" "$stage/luadot.1"
	install -d "$stage/completions"
	install -m0644 "$completions"/* "$stage/completions/"
	tar -C "$dist" -czf "$dist/luadot-$pkgver-$arch.tar.gz" "luadot-$pkgver-$arch"
	rm -rf "$stage"
	echo "release: packed luadot-$pkgver-$arch.tar.gz"
done

rm -rf "$completions" "$manual"
