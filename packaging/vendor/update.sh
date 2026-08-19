#!/usr/bin/env bash
set -euo pipefail

die() {
	echo "vendor: $1" >&2
	exit 1
}

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
manifest="$root/vendor/sources.toml"

[ -f "$manifest" ] || die "no manifest at $manifest"

check=0
if [ "${1-}" = '--check' ]; then
	check=1
	shift
fi

[ $# -le 2 ] || die "usage: $0 [--check] [name] [version]"

wanted=${1-}
bumped=${2-}

[ -z "$bumped" ] || [ "$check" -eq 0 ] || die 'check mode cannot bump a version'
[ -z "$bumped" ] || [ -n "$wanted" ] || die 'a version needs the library it belongs to'

fields() {
	awk '
		/^[[:space:]]*#/ { next }
		open {
			buffer = buffer " " $0
			if ($0 ~ /\]/) { open = 0; emit(section, key, buffer) }
			next
		}
		/^[[:space:]]*\[/ {
			section = $0
			sub(/^[^[]*\[/, "", section)
			sub(/\].*/, "", section)
			next
		}
		/=/ {
			key = $0
			sub(/[[:space:]]*=.*/, "", key)
			gsub(/[[:space:]]/, "", key)
			value = $0
			sub(/^[^=]*=[[:space:]]*/, "", value)
			if (value ~ /^\[/ && value !~ /\]/) {
				open = 1
				buffer = value
				next
			}
			emit(section, key, value)
		}
		function emit(section, key, value) {
			sub(/^[[:space:]]*\[/, "", value)
			sub(/\][[:space:]]*$/, "", value)
			gsub(/"/, "", value)
			gsub(/,/, " ", value)
			gsub(/[[:space:]]+/, " ", value)
			gsub(/^[[:space:]]|[[:space:]]$/, "", value)
			print section "\t" key "\t" value
		}
	' "$manifest"
}

record() {
	local name=$1 version=$2 sha256=$3 temp
	temp=$(mktemp)
	awk -v name="$name" -v version="$version" -v sha256="$sha256" '
		/^[[:space:]]*\[/ {
			section = $0
			sub(/^[^[]*\[/, "", section)
			sub(/\].*/, "", section)
		}
		section == name && /^[[:space:]]*version[[:space:]]*=/ { print "version = \"" version "\""; next }
		section == name && /^[[:space:]]*sha256[[:space:]]*=/ { print "sha256 = \"" sha256 "\""; next }
		{ print }
	' "$manifest" >"$temp"
	mv "$temp" "$manifest"
}

license_section() {
	local file=$1 anchor=$2 text
	[ -f "$file" ] || die "no $file to take the license from"

	text=$(sed -nE "/<a name=\"$anchor\"/,/<h2|<\/div/p" "$file" |
		sed -e '1d' -e '$d' -e 's/<[^>]*>//g' \
			-e 's/&copy;/©/g' -e 's/&quot;/"/g' -e 's/&nbsp;/ /g' \
			-e 's/&lt;/</g' -e 's/&gt;/>/g' -e 's/&amp;/\&/g' \
			-e 's/[[:space:]]*$//' |
		cat -s |
		sed -e '/./,$!d')

	[ -n "$text" ] || die "no section named $anchor in $file"
	if printf '%s' "$text" | grep -q '&[a-zA-Z]\{1,\};'; then
		die "unresolved html entity in the license of $file"
	fi

	printf '%s\n' "$text"
}

declare -A field=()
declare -a names=()

while IFS=$'\t' read -r section key value; do
	[ -n "${field["$section.name"]+set}" ] || names+=("$section")
	field["$section.name"]=$section
	field["$section.$key"]=$value
done < <(fields)

[ "${#names[@]}" -gt 0 ] || die "$manifest declares no library"

if [ -n "$wanted" ]; then
	[ -n "${field["$wanted.name"]+set}" ] || die "no library named $wanted in the manifest"
	names=("$wanted")
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

vendor() {
	local name=$1 key version url recorded license file anchor
	local src="$work/$name-source" out="$work/$name"
	local -a patterns=()
	local pattern match count=0

	for key in version url sha256 files license; do
		[ -n "${field["$name.$key"]-}" ] || die "$name: the manifest declares no $key"
	done

	version=${field["$name.version"]}
	url=${field["$name.url"]}
	recorded=${field["$name.sha256"]}
	license=${field["$name.license"]}
	read -r -a patterns <<<"${field["$name.files"]}"

	[ -z "$bumped" ] || version=$bumped
	url=${url//\{version\}/$version}

	curl -fsSL -o "$work/$name.tar.gz" "$url" || die "$name: $url could not be downloaded"

	local sha256
	sha256=$(sha256sum "$work/$name.tar.gz" | cut -d ' ' -f 1)

	if [ -z "$bumped" ] && [ "$sha256" != "$recorded" ]; then
		die "$name: checksum mismatch: expected $recorded, got $sha256"
	fi

	mkdir -p "$src" "$out"
	tar -xzf "$work/$name.tar.gz" --strip-components=1 -C "$src"

	for pattern in "${patterns[@]}"; do
		local -a matches=()
		while IFS= read -r match; do
			matches+=("$match")
		done < <(cd "$src" && compgen -G "$pattern" || true)

		[ "${#matches[@]}" -gt 0 ] || die "$name: no file matches $pattern"

		for match in "${matches[@]}"; do
			[ -f "$src/$match" ] || continue
			install -Dm644 "$src/$match" "$out/$match"
			count=$((count + 1))
		done
	done

	file=${license%%#*}
	anchor=${license#*#}
	if [ "$anchor" = "$license" ]; then
		install -Dm644 "$src/$file" "$out/LICENSE"
	else
		license_section "$src/$file" "$anchor" >"$out/LICENSE"
		chmod 644 "$out/LICENSE"
	fi

	if [ "$check" -eq 1 ]; then
		[ -d "$root/vendor/$name" ] || die "$name: nothing vendored at vendor/$name"
		diff -ru "$root/vendor/$name" "$out" || die "$name: vendor/$name does not match the manifest"
		echo "vendor: $name $version matches vendor/$name"
		return
	fi

	rm -rf "${root:?}/vendor/$name"
	mv "$out" "$root/vendor/$name"
	[ -z "$bumped" ] || record "$name" "$version" "$sha256"
	echo "vendor: $name $version into vendor/$name ($count file(s) and the license)"
}

for name in "${names[@]}"; do
	vendor "$name"
done
