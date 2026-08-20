# Install

Arch Linux, from the AUR:

```
paru -S luadot
```

`luadot-bin` and `luadot-nightly` are the prebuilt and pre-release variants.

Debian or Ubuntu, from the `.deb` each release publishes for `amd64` and
`arm64`:

```
curl -fLO https://gitlab.digitalventura.com.br/api/v4/projects/luadot%2Fluadot/packages/generic/luadot/0.2.0/luadot_0.2.0-1_amd64.deb
sudo apt install ./luadot_0.2.0-1_amd64.deb
```

There is no apt repository behind it: `apt upgrade` will not move it, the next
version is another download.

Nix, from the flake in the repository:

```
nix run github:ItzTas/luadot -- status
nix profile install github:ItzTas/luadot
```

The flake exports `packages.default` for the four Linux and Darwin systems, an
`overlays.default` that adds `luadot` to a nixpkgs instance, and a dev shell.
Details in [internal/nix.md](../internal/nix.md).

Anywhere else, from source:

```
cargo install --git https://github.com/ItzTas/luadot luadot
```

A source install has no packaged completions; `luadot completions <shell>`
prints the script for your shell.
