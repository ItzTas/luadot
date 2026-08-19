# The Nix flake

`flake.nix` at the root is the whole packaging; `nix/package.nix` is the
derivation it calls, written so `pkgs.callPackage` works on it directly.

## What it exports

| Output | What it is |
| --- | --- |
| `packages.<system>.luadot` | the binary with the three shell completions |
| `packages.<system>.default` | the same derivation |
| `overlays.default` | adds `luadot` to a nixpkgs instance |
| `devShells.<system>.default` | build inputs plus cargo, clippy, rustfmt, rust-analyzer, cocogitto and shellcheck |
| `formatter.<system>` | `nixfmt-rfc-style`, for `nix fmt` |

The systems are `x86_64-linux`, `aarch64-linux`, `x86_64-darwin` and
`aarch64-darwin`. Only the two Linux ones are what the release pipeline builds
elsewhere; Darwin is declared because nothing in the build is Linux-specific,
not because it is exercised.

## Using it

In a flake of your own, through the overlay:

```nix
{
  inputs.luadot.url = "github:ItzTas/luadot";

  outputs = { nixpkgs, luadot, ... }: {
    # ...
    nixpkgs.overlays = [ luadot.overlays.default ];
    environment.systemPackages = [ pkgs.luadot ];
  };
}
```

Or without the overlay, from the package output:

```nix
home.packages = [ luadot.packages.${pkgs.system}.default ];
```

There is no home-manager or NixOS module. The configuration is a Lua file
luadot reads at runtime, and a store path is read-only — leaving it out of the
Nix world keeps the dotfiles repository the only thing that writes it.

## Why the build is offline and unchecked

`cargoLock.lockFile` points at the committed `Cargo.lock`, which holds no git
dependency, so the fixed-output vendor derivation needs no hash of its own.
Everything else the build compiles is already in the tree: LPeg under `vendor/`
and Lua 5.4 from the `lua-src` crate. See [Vendored sources](vendoring.md).
Nothing reaches the network past the vendoring step, and no system library is
needed — `gix` talks TLS through rustls, not OpenSSL.

`doCheck = false`. The test suite is what CI runs on every push; running it
again inside every installation only makes `nix build` slow and gives the same
answer.

`build/hooks.rs` points `core.hooksPath` at `.githooks`, which is for the
working tree, not for a build. It looks for a git work tree first and returns
when there is none, so the sandbox never sees it try.

## Version

`pname` and `version` are read from `Cargo.toml` with `lib.importTOML`, so
`cog bump` moves the flake with the crate and there is no second place to
forget.
