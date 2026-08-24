#!/usr/bin/env bash
set -euo pipefail

repo='ItzTas/luadot'
api='https://api.github.com'

die() {
	echo "release: $1" >&2
	exit 1
}

[ $# -eq 1 ] || die "usage: $0 <tag>"

tag=$1
[ -n "$tag" ] || die 'no tag given; nothing was released'
[ -n "${GITHUB_TOKEN:-}" ] || die 'GITHUB_TOKEN is not set'
[ -n "${CI_JOB_TOKEN:-}" ] || die 'CI_JOB_TOKEN is not set'
[ -n "${CI_API_V4_URL:-}" ] || die 'CI_API_V4_URL is not set'
[ -n "${CI_PROJECT_ID:-}" ] || die 'CI_PROJECT_ID is not set'

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
changelog="$root/CHANGELOG.md"

[ -f "$changelog" ] || die "no changelog at $changelog"

github() {
	local method=$1 path=$2
	shift 2
	curl -fsS -X "$method" \
		-H "Authorization: Bearer $GITHUB_TOKEN" \
		-H 'Accept: application/vnd.github+json' \
		-H 'X-GitHub-Api-Version: 2022-11-28' \
		"$@" "$api/repos/$repo$path"
}

github GET "/git/ref/tags/$tag" >/dev/null 2>&1 ||
	die "$tag has not reached github.com/$repo; the mirror has to run first"

release=$(curl -fsS -H "JOB-TOKEN: $CI_JOB_TOKEN" \
	"$CI_API_V4_URL/projects/$CI_PROJECT_ID/releases/$tag") ||
	die "$tag has no release on GitLab; run upload.sh first"

notes=$(awk -v head="## [$tag]" '
	index($0, head) == 1 { inside = 1; next }
	inside && ($0 == "- - -" || index($0, "## [") == 1) { exit }
	inside { print }
' "$changelog")

[ -n "$notes" ] || die "the changelog holds no section for $tag"

table=$(jq -r '
	.assets.links
	| if length == 0 then empty
	  else
		["| File | Download |", "| --- | --- |"]
		+ (sort_by(.name) | map("| `\(.name)` | [download](\(.url)) |"))
		| join("\n")
	  end
' <<<"$release")

web=$(jq -r '._links.self' <<<"$release")

body=$notes
[ -z "$table" ] || body=$(printf '%s\n\n## Downloads\n\n%s\n\nThe files are served by the GitLab package registry, listed on the [%s release](%s).\n' \
	"$notes" "$table" "$tag" "$web")

case ${tag#v} in
*-*) prerelease=true ;;
*) prerelease=false ;;
esac

payload=$(jq -n --arg tag "$tag" --arg body "$body" --argjson prerelease "$prerelease" \
	'{tag_name: $tag, name: $tag, body: $body, prerelease: $prerelease}')

existing=$(github GET "/releases/tags/$tag" 2>/dev/null || true)

if [ -z "$existing" ]; then
	github POST '/releases' \
		-H 'Content-Type: application/json' \
		-d "$payload" >/dev/null
	echo "release: created the GitHub release $tag"
	exit 0
fi

github PATCH "/releases/$(jq -r '.id' <<<"$existing")" \
	-H 'Content-Type: application/json' \
	-d "$payload" >/dev/null

echo "release: updated the GitHub release $tag"
