#!/usr/bin/env bash
set -euo pipefail

aur_user_name='ItzTas'
aur_user_email='ts.aur@imts.aleeas.com'
gitlab_registry='https://gitlab.digitalventura.com.br/api/v4/projects/luadot%2Fluadot/packages/generic/luadot'
download_attempts=30
download_delay=20

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

version=${tag#v}
pkgver=${version//-/.}
asset="$gitlab_registry/$version/luadot-$pkgver"

declare -A package_assets=(
	["luadot"]="SHA256=$asset-src.tar.gz"
	["luadot-nightly"]="SHA256=$asset-src.tar.gz"
	["luadot-bin"]="SHA256_X86_64=$asset-x86_64.tar.gz
SHA256_AARCH64=$asset-aarch64.tar.gz"
)

[ -n "${package_assets[$pkgname]+set}" ] || die "no assets declared for $pkgname"

download() {
	local url=$1 out=$2 attempt
	for ((attempt = 1; attempt <= download_attempts; attempt++)); do
		if curl -fsSL -o "$out" "$url"; then
			return 0
		fi
		echo "aur: $url not published yet ($attempt/$download_attempts)" >&2
		sleep "$download_delay"
	done
	die "$url never became available"
}

sed_args=(-e "s|@PKGVER@|$pkgver|g" -e "s|@VERSION@|$version|g" -e "s|@TAG@|$tag|g")
index=0
while IFS='=' read -r placeholder url; do
	index=$((index + 1))
	download "$url" "$work/asset-$index"
	sha256=$(sha256sum "$work/asset-$index" | cut -d ' ' -f 1)
	sed_args+=(-e "s|@$placeholder@|$sha256|g")
done <<<"${package_assets[$pkgname]}"

sed "${sed_args[@]}" "$templates/PKGBUILD.in" >"$work/PKGBUILD"

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
