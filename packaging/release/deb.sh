#!/usr/bin/env bash
set -euo pipefail

maintainer='ItzTas <ts.aur@imts.aleeas.com>'
homepage='https://github.com/ItzTas/luadot'
revision=1

die() {
	echo "release: $1" >&2
	exit 1
}

[ $# -eq 1 ] || die "usage: $0 <tag>"

tag=$1
[ -n "$tag" ] || die 'no tag given; nothing was released'

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
dist="$root/dist"
[ -d "$dist" ] || die 'no binaries to package; run build.sh first'

pkgver=${tag#v}
version=${pkgver//-/\~}

declare -A debian_archs=(
	["x86_64"]=amd64
	["aarch64"]=arm64
)

glibc_floor() {
	readelf --wide --dyn-syms "$1" |
		grep -o 'GLIBC_[0-9]\+\.[0-9]\+' |
		sort -u -V |
		tail -1 |
		cut -d _ -f 2
}

for arch in "${!debian_archs[@]}"; do
	tarball="$dist/luadot-$pkgver-$arch.tar.gz"
	[ -f "$tarball" ] || die "missing $(basename "$tarball"); run build.sh first"

	debian_arch=${debian_archs[$arch]}
	source="$dist/luadot-$pkgver-$arch"
	tree="$dist/deb-$debian_arch"
	rm -rf "$source" "$tree"
	tar -C "$dist" -xzf "$tarball"

	install -Dm0755 "$source/luadot" "$tree/usr/bin/luadot"
	install -Dm0644 "$source/LICENSE" "$tree/usr/share/doc/luadot/copyright"
	install -Dm0644 "$source/LICENSE.lpeg" "$tree/usr/share/doc/luadot/LICENSE.lpeg"
	install -Dm0644 "$source/completions/luadot.bash" \
		"$tree/usr/share/bash-completion/completions/luadot"
	install -Dm0644 "$source/completions/_luadot" \
		"$tree/usr/share/zsh/vendor-completions/_luadot"
	install -Dm0644 "$source/completions/luadot.fish" \
		"$tree/usr/share/fish/vendor_completions.d/luadot.fish"

	installed_size=$(du -ks "$tree" | cut -f 1)
	glibc=$(glibc_floor "$source/luadot")
	rm -rf "$source"

	install -d "$tree/DEBIAN"
	cat >"$tree/DEBIAN/control" <<-EOF
		Package: luadot
		Version: $version-$revision
		Architecture: $debian_arch
		Maintainer: $maintainer
		Installed-Size: $installed_size
		Depends: libc6 (>= $glibc), libgcc-s1 (>= 3.0)
		Section: utils
		Priority: optional
		Homepage: $homepage
		Description: A dotfiles manager configured in Lua
		 luadot mirrors the files a machine is configured with into a git repository
		 and puts them back anywhere, the whole configuration written in Lua.
	EOF

	(cd "$tree" && find . -type f -not -path './DEBIAN/*' -printf '%P\n' |
		sort |
		xargs -r md5sum >DEBIAN/md5sums)

	deb="$dist/luadot_$version-${revision}_$debian_arch.deb"
	dpkg-deb --build --root-owner-group "$tree" "$deb" >/dev/null
	rm -rf "$tree"
	echo "release: packed $(basename "$deb")"
done
