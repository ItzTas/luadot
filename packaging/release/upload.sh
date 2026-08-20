#!/usr/bin/env bash
set -euo pipefail

package='luadot'

die() {
	echo "release: $1" >&2
	exit 1
}

[ $# -eq 1 ] || die "usage: $0 <tag>"

tag=$1
[ -n "$tag" ] || die 'no tag given; nothing was released'
[ -n "${CI_JOB_TOKEN:-}" ] || die 'CI_JOB_TOKEN is not set'
[ -n "${CI_API_V4_URL:-}" ] || die 'CI_API_V4_URL is not set'
[ -n "${CI_PROJECT_ID:-}" ] || die 'CI_PROJECT_ID is not set'

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
dist="$root/dist"
[ -d "$dist" ] || die 'nothing to upload; run build.sh first'

shopt -s nullglob
assets=("$dist"/*.tar.gz "$dist"/*.deb)
[ "${#assets[@]}" -gt 0 ] || die 'nothing to upload; run build.sh first'

pkgver=${tag#v}
registry="$CI_API_V4_URL/projects/$CI_PROJECT_ID/packages/generic/$package/$pkgver"
releases="$CI_API_V4_URL/projects/$CI_PROJECT_ID/releases"

gitlab() {
	local method=$1 url=$2
	shift 2
	curl -fsS -X "$method" -H "JOB-TOKEN: $CI_JOB_TOKEN" "$@" "$url"
}

links=$(jq -n '[]')
for path in "${assets[@]}"; do
	name=$(basename "$path")

	curl -fsS -X PUT \
		-H "JOB-TOKEN: $CI_JOB_TOKEN" \
		--upload-file "$path" \
		"$registry/$name" >/dev/null

	links=$(jq --arg name "$name" --arg url "$registry/$name" \
		'. + [{name: $name, url: $url, link_type: "package"}]' <<<"$links")

	echo "release: uploaded $name to the package registry"
done

existing=$(gitlab GET "$releases/$tag" 2>/dev/null || true)

if [ -z "$existing" ]; then
	gitlab POST "$releases" \
		-H 'Content-Type: application/json' \
		-d "$(jq -n --arg tag "$tag" --argjson links "$links" \
			'{tag_name: $tag, name: $tag, assets: {links: $links}}')" >/dev/null
	echo "release: created the GitLab release $tag"
	exit 0
fi

while read -r name; do
	stale=$(jq -r --arg name "$name" \
		'.assets.links[]? | select(.name == $name) | .id' <<<"$existing")
	for id in $stale; do
		gitlab DELETE "$releases/$tag/assets/links/$id" >/dev/null
	done

	gitlab POST "$releases/$tag/assets/links" \
		-H 'Content-Type: application/json' \
		-d "$(jq -n --arg name "$name" --arg url "$registry/$name" \
			'{name: $name, url: $url, link_type: "package"}')" >/dev/null
done < <(jq -r '.[].name' <<<"$links")

echo "release: linked $(jq -r 'length' <<<"$links") asset(s) on the GitLab release $tag"
