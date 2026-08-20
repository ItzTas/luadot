# Publishing to the AUR

Three packages are kept in sync automatically:

| Package | Channel | Tag it follows | Example `pkgver` | Built by |
| --- | --- | --- | --- | --- |
| `luadot` | stable | `main`, e.g. `v0.2.0` | `0.2.0` | the user, from source |
| `luadot-bin` | stable | `main`, e.g. `v0.2.0` | `0.2.0` | CI, prebuilt binary |
| `luadot-nightly` | nightly | `nightly`, e.g. `v0.2.0-nightly.1` | `0.2.0.nightly.1` | the user, from source |

There is no `luadot-nightly-bin`: only the stable channel ships binaries.

`luadot-bin` and `luadot-nightly` both declare `provides=(luadot=$pkgver)` and
`conflicts=(luadot)`, so only one of the three can be installed at a time.
Pacman matches the conflict against what the others provide.

A `pkgver` cannot hold a `-`, so every `-` in the tag becomes a `.`. This makes
`0.2.0.nightly.1` sort *above* `0.2.0` for pacman, which is why the nightly
lives in a package of its own rather than in the stable one.

The package registry keeps the tag's own spelling, `<version>` —
`0.2.0-nightly.1` — as the path segment, while the file inside it is named with
`<pkgver>`. Both spellings appear in the same URL; a stable tag has no `-`, so
there they are the same string.

## What runs, and when

The pipeline in `.gitlab/workflows/ci.yml` does it, in this order:

1. `bump` (stage `release`) runs `cog bump`, pushes the new commit and tag to
   GitLab, and exports the tag as `LUADOT_TAG` through a dotenv artifact.
2. `push-to-github` (stage `mirror`) mirrors that tag to GitHub. Nothing
   downstream waits for it — it is the contribution mirror, not a source.
3. `source-tarball` (stage `binaries`, `main` and `nightly`) calls
   `packaging/release/source.sh` and `packaging/release/upload.sh`.
4. `binaries` (stage `binaries`, `main` only) checks out the tag and calls
   `packaging/release/build.sh`, `packaging/release/deb.sh` and
   `packaging/release/upload.sh`. See [Release assets](#release-assets).
5. `aur-stable` / `aur-bin` / `aur-nightly` (stage `publish`) call
   `packaging/aur/publish.sh`, which:
   - waits for every asset the package needs (30 tries, 20 seconds apart) and
     hashes it: the source tarball for `luadot` and `luadot-nightly`, the two
     binary tarballs for `luadot-bin`;
   - renders `packaging/aur/<pkgname>/PKGBUILD.in`, replacing `@PKGVER@`,
     `@VERSION@`, `@TAG@` and the `@SHA256*@` placeholders that package
     declares;
   - generates `.SRCINFO` with `makepkg --printsrcinfo`, as a non-root user;
   - clones `ssh://aur@aur.archlinux.org/<pkgname>.git`, commits both files and
     pushes. Nothing is committed when the rendered files are unchanged.

`aur-stable` and `aur-nightly` `need` `bump` and `source-tarball`; `aur-bin`
needs `bump` and `binaries`. `source-tarball` and `binaries` share the
`release-assets` resource group, so the two never write to the same release at
once.

## Release assets

Every asset an installed package fetches is a file this pipeline uploaded, and
they all live in the GitLab generic package registry. Nothing is fetched from a
forge's on-the-fly archive endpoint: those are regenerated per request, and the
day the compression or the git version behind one changes, every pinned
`sha256sums` breaks at once. An uploaded file's bytes never move.

GitHub carries the mirror, for contributions. No package fetches from it.

`packaging/release/source.sh <tag>` writes `dist/luadot-<pkgver>-src.tar.gz`
with `git archive` at the tag, under a `luadot-<pkgver>/` prefix, piped through
`gzip -9n` so the same tag always produces the same bytes. That is what the two
from-source packages build from, and their `_srcdir` is that prefix.

`packaging/release/build.sh <tag>` builds `x86_64-unknown-linux-gnu` natively
and cross-compiles `aarch64-unknown-linux-gnu` with `gcc-aarch64-linux-gnu`,
strips both through `-C strip=symbols`, and writes to `dist/`:

```
luadot-<pkgver>-x86_64.tar.gz
luadot-<pkgver>-aarch64.tar.gz
```

Each holds a `luadot-<pkgver>-<arch>/` directory with the binary, `LICENSE`,
`LICENSE.lpeg` and `completions/`. The completions come from the native binary,
since the cross-built one cannot be run on the builder.

`packaging/release/deb.sh <tag>` turns each of those tarballs into a Debian
package, `dist/luadot_<version>-1_<amd64|arm64>.deb`, with the completions in
the paths Debian uses and the licenses under `/usr/share/doc/luadot/`. It reads
the `libc6` floor out of the binary itself (the highest `GLIBC_*` symbol it
imports), so the dependency follows whatever image built it. No packaging
tooling beyond `dpkg-deb` and `readelf`.

`packaging/release/upload.sh <tag>` then `PUT`s every `dist/` asset into the
GitLab generic package registry, under `luadot/<version>/`, and points the
GitLab release for the tag at them as asset links — creating the release when it
does not exist yet, and replacing links of the same name when it does, so a
re-run is safe. It needs `CI_JOB_TOKEN` and `jq`.

The project is public, so those URLs are anonymous downloads, which is what the
three packages fetch:

```
https://gitlab.digitalventura.com.br/api/v4/projects/luadot%2Fluadot/packages/generic/luadot/<version>/<file>
```

## Setup

- An AUR account whose SSH public key is registered under *My Account*. The
  package repository is created by the AUR on the first push, so no package
  has to be submitted by hand.
- A CI variable `AUR_SSH_KEY_B64` in the GitLab project: the **private** key,
  base64-encoded, masked, protected.

  ```
  base64 -w0 < ~/.ssh/aur
  ```

- No extra variable for the release assets: `binaries` writes to the package
  registry and to the release with the job's own `CI_JOB_TOKEN`.

The host key of `aur.archlinux.org` is fetched with `ssh-keyscan` at job time,
which trusts it on first use. Pin it in the job instead if that is not good
enough.

## Changing the packaging

`packaging/aur/*/PKGBUILD.in` is the source of truth; the AUR repositories hold
only what the pipeline renders, and a change made directly there is overwritten
by the next release. To test one by hand:

```
sed -e 's|@PKGVER@|0.2.0|g' -e 's|@VERSION@|0.2.0|g' -e 's|@TAG@|v0.2.0|g' \
  -e "s|@SHA256@|$(sha256sum tarball | cut -d' ' -f1)|g" \
  packaging/aur/luadot/PKGBUILD.in > PKGBUILD
makepkg --printsrcinfo
```

`luadot-bin` takes `@SHA256_X86_64@` and `@SHA256_AARCH64@` instead of
`@SHA256@`; a package's placeholders and the assets they hash are declared
together in the `package_assets` map in `packaging/aur/publish.sh`.
