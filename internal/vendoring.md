# Vendored sources

`vendor/` holds the third-party sources compiled into the binary, one directory
per library. `vendor/sources.toml` declares where each of them came from, and
`packaging/vendor/update.sh` is what puts them there. Only LPeg is vendored
today, in `vendor/lpeg/`, compiled by `build/lpeg/`.

## Why they are committed

`cargo build` reaches no network. A build script that downloads its own sources
breaks every build that is meant to be offline:

- the AUR packages run `cargo fetch --locked` in `prepare()` and
  `cargo build --frozen` in `build()`, so the build stage is expected to touch
  nothing;
- docs.rs builds are network-isolated;
- a release stops being reproducible once it depends on a tag still being
  reachable years later.

Committing the sources also drops the build dependencies the download needed
(`ureq`, `flate2`, `tar`, `sha2`), which every consumer of the crate had to
fetch and compile.

## The manifest

One section per library, named after the directory it lands in:

```toml
[lpeg]
version = "1.1.0"
url = "https://gitlab.digitalventura.com.br/luadot/deps/-/raw/d3dfdc67bf203e4b5c1fdc66c45dcdbf85ebf46e/opt/lpeg-{version}.tar.gz"
upstream = "https://www.inf.puc-rio.br/~roberto/lpeg/lpeg-{version}.tar.gz"
sha256 = "4b155d67d2246c1ffa7ad7bc466c1ea899bbc40fef0257cc9c03cecbaed4352a"
files = ["*.c", "*.h", "re.lua", "README.md"]
license = "lpeg.html#license"
```

- `url` is what the script downloads: a byte-for-byte copy of the release in
  `luadot/deps`, at a path pinned to the commit that added it. A commit cannot
  be moved, so those bytes cannot change under the recorded checksum — which a
  branch, a tag or a registry entry all can. `{version}` still resolves in the
  file name, but the commit in the path does not follow a bump, so it is edited
  by hand when one happens. Neovim mirrors the same LPeg tarball the same way,
  in `neovim/deps`, and it hashes to the value recorded here.
- `upstream` is where that copy was taken from, so a bump knows what to
  download and mirror. Nothing reads it. It names the release the author
  publishes, not the archive a forge generates from a tag: the first is an
  artifact with a fixed hash, the second is produced on demand and can change
  bytes without the tag moving.
- `sha256` is the tarball the vendored tree was taken from, recorded by the
  script itself.
- `files` are globs against the root of the extracted tarball, with the paths
  they match preserved. Anything not listed — documentation, tests, makefiles,
  images — stays out.
- `license` is the file carrying the notice. LPeg publishes no license file, so
  the form `<file>#<anchor>` says the text lives in an HTML page and is taken
  from the section that anchor names. A plain file name is copied as it is.

The notice has to travel with the sources: LPeg is MIT, which allows the copy
as long as the copyright and permission notices come along. That is also why
the AUR packages install it as `LICENSE.lpeg` beside luadot's own.

## Updating a library

A version is mirrored before it is vendored: download what `upstream` resolves
to, commit it into `luadot/deps` under `opt/` — files are added there, never
rewritten — and point `url` at the commit that added it. Then:

```
./packaging/vendor/update.sh lpeg 1.2.0
```

It downloads the new tarball, replaces `vendor/lpeg/`, and only then rewrites
`version` and `sha256` in the manifest — a release that turns out to be missing
a declared file leaves both the tree and the manifest untouched. `url` is not
one of the fields it rewrites, which is why the mirror comes first.

Without a version, the recorded checksum is verified instead of replaced, which
is how the tree is rebuilt from the manifest:

```
./packaging/vendor/update.sh          # every library
./packaging/vendor/update.sh lpeg     # one of them
```

`--check` does the same work into a temporary directory and compares, writing
nothing and exiting non-zero on any difference:

```
./packaging/vendor/update.sh --check
```

That covers both halves of the drift: a checksum that no longer matches, and a
vendored file edited in place.

A mismatch is worth reading before it is re-recorded. The tarball is the one
the author published, so its bytes are not supposed to change at all: a
checksum that stopped matching means the release was replaced under the same
version, or what answered the request was not the release. Neither is something
to paper over with a new hash.

What the recorded hash cannot cover on its own is the address staying up, which
is why `url` points at the mirror instead of at the author's site. The sources
are committed either way, so a page that disappears breaks no build — what it
takes down is the ability to download the tarball again and compare, and that
is the whole of what `--check` does.

## Adding a library

Four things, in order: the release tarball committed into `luadot/deps` under
`opt/`, a section in the manifest whose `url` names that commit and whose
`upstream` names where it was downloaded from, then
`./packaging/vendor/update.sh <name>`, then a `build/<name>/` calling
`vendor::compile(NAME, headers)` from `build/main.rs`.

The build script takes no file list. It compiles every `.c` under the vendored
directory and includes the directories they sit in, so a release that adds a
source needs nothing but the manifest. What stays library-specific is what the
Rust side does with the rest: `build/lpeg/compile.rs` points `LPEG_RE_PATH` at
`re.lua` so `src/lua/bundled/lpeg/` can embed it.
