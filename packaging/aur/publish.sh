#!/usr/bin/env bash
set -euo pipefail

aur_user_name='ItzTas'
aur_user_email='ts.aur@imts.aleeas.com'
github_url='https://github.com/ItzTas/luadot'
tarball_attempts=30
tarball_delay=20

die() {
	echo "aur: $1" >&2
	exit 1
}

[ $# -eq 2 ] || die "usage: $0 <pkgname> <tag>"

pkgname=$1
tag=$2

[ -n "$tag" ] || die 'no tag given; nothing was released'

templates=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/$pkgname" && pwd)
[ -f "$templates/PKGBUILD.in" ] || die "no template for $pkgname"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

pkgver=${tag#v}
pkgver=${pkgver//-/.}
tarball="$github_url/archive/refs/tags/$tag.tar.gz"

fetch_tarball() {
	local attempt
	for ((attempt = 1; attempt <= tarball_attempts; attempt++)); do
		if curl -fsSL -o "$work/source.tar.gz" "$tarball"; then
			return 0
		fi
		echo "aur: $tag not on GitHub yet ($attempt/$tarball_attempts)" >&2
		sleep "$tarball_delay"
	done
	die "$tarball never became available"
}

fetch_tarball
sha256=$(sha256sum "$work/source.tar.gz" | cut -d ' ' -f 1)

sed \
	-e "s|@PKGVER@|$pkgver|g" \
	-e "s|@TAG@|$tag|g" \
	-e "s|@SHA256@|$sha256|g" \
	"$templates/PKGBUILD.in" >"$work/PKGBUILD"

id -u builder >/dev/null 2>&1 || useradd -m builder
chown -R builder "$work"
runuser -u builder -- bash -c "cd '$work' && makepkg --printsrcinfo" >"$work/.SRCINFO"

git clone "ssh://aur@aur.archlinux.org/$pkgname.git" "$work/aur"
cp "$work/PKGBUILD" "$work/.SRCINFO" "$work/aur/"

git -C "$work/aur" add PKGBUILD .SRCINFO
if git -C "$work/aur" diff --cached --quiet; then
	echo "aur: $pkgname is already at $pkgver"
	exit 0
fi

git -C "$work/aur" \
	-c "user.name=$aur_user_name" \
	-c "user.email=$aur_user_email" \
	commit -m "$pkgname $pkgver"
git -C "$work/aur" push origin HEAD:master

echo "aur: pushed $pkgname $pkgver"
