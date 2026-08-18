# Publishing to the AUR

Two packages are kept in sync automatically, one per release channel:

| Package | Channel | Tag it follows | Example `pkgver` |
| --- | --- | --- | --- |
| `luadot` | stable | `main`, e.g. `v0.2.0` | `0.2.0` |
| `luadot-nightly` | nightly | `nightly`, e.g. `v0.2.0-nightly.1` | `0.2.0.nightly.1` |

`luadot-nightly` declares `provides=(luadot=$pkgver)` and `conflicts=(luadot)`,
so only one of the two can be installed at a time.

A `pkgver` cannot hold a `-`, so every `-` in the tag becomes a `.`. This makes
`0.2.0.nightly.1` sort *above* `0.2.0` for pacman, which is why the nightly
lives in a package of its own rather than in the stable one.

## What runs, and when

The pipeline in `.gitlab/workflows/ci.yml` does it, in this order:

1. `bump` (stage `release`) runs `cog bump`, pushes the new commit and tag to
   GitLab, and exports the tag as `LUADOT_TAG` through a dotenv artifact.
2. `push-to-github` (stage `mirror`) mirrors that tag to GitHub, which is where
   the AUR sources are fetched from.
3. `aur-stable` / `aur-nightly` (stage `publish`) call
   `packaging/aur/publish.sh`, which:
   - waits for `https://github.com/ItzTas/luadot/archive/refs/tags/<tag>.tar.gz`
     to exist (30 tries, 20 seconds apart) and hashes it;
   - renders `packaging/aur/<pkgname>/PKGBUILD.in`, replacing `@PKGVER@`,
     `@TAG@` and `@SHA256@`;
   - generates `.SRCINFO` with `makepkg --printsrcinfo`, as a non-root user;
   - clones `ssh://aur@aur.archlinux.org/<pkgname>.git`, commits both files and
     pushes. Nothing is committed when the rendered files are unchanged.

Both jobs `need` `bump` and `push-to-github`, so nothing reaches the AUR before
the tarball is reachable.

## Setup

- An AUR account whose SSH public key is registered under *My Account*. The
  package repository is created by the AUR on the first push, so neither
  package has to be submitted by hand.
- A CI variable `AUR_SSH_KEY_B64` in the GitLab project: the **private** key,
  base64-encoded, masked, protected.

  ```
  base64 -w0 < ~/.ssh/aur
  ```

The host key of `aur.archlinux.org` is fetched with `ssh-keyscan` at job time,
which trusts it on first use. Pin it in the job instead if that is not good
enough.

## Changing the packaging

`packaging/aur/*/PKGBUILD.in` is the source of truth; the AUR repositories hold
only what the pipeline renders, and a change made directly there is overwritten
by the next release. To test one by hand:

```
sed -e 's|@PKGVER@|0.2.0|g' -e 's|@TAG@|v0.2.0|g' -e "s|@SHA256@|$(sha256sum tarball | cut -d' ' -f1)|g" \
  packaging/aur/luadot/PKGBUILD.in > PKGBUILD
makepkg --printsrcinfo
```
