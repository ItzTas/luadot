# luadot

Manage your dotfiles with lua

luadot keeps your dotfiles in a git repository and puts them back on every
machine you clone it to. The configuration is a Lua script instead of a static
file, evaluated on each machine, so one repository serves a laptop, a desktop
and a server without a branch or a copy for each.

The repository mirrors your home directory, path for path: `.zshrc` in the
repository is `~/.zshrc` on the machine. Rules decide how each file is placed:
linked hard, symbolic or copied, left alone, taken in on its own, encrypted,
kept in Git LFS, or generated per machine by a template.

```lua
ld.rules({
  { match = ".ssh/id_*", encrypt = true },
  { match = ".config/mako/**", on_change = "makoctl reload" },
  { match = ".cache/**", track = "never" },
})
```

## Quick start

```
luadot init                 # or: luadot clone git@github.com:me/dotfiles.git
luadot add ~/.bashrc
luadot git commit -m "first"
```

On the next machine, `luadot clone` then `luadot apply`. After that,
`luadot status` lists what drifted, `luadot diff` shows it, `luadot apply` and
`luadot take` settle it either way, and `luadot sync` commits and pushes it.

## Install

```
paru -S luadot                                  # Arch Linux (AUR); also luadot-bin
nix profile install github:ItzTas/luadot        # Nix
cargo install --git https://github.com/ItzTas/luadot luadot
```

Debian and Ubuntu get a `.deb` with each release. See [docs/install.md](docs/install.md).

## Documentation

- [Install](docs/install.md): AUR, deb, Nix, source.
- [Commands](docs/commands.md): what each one does.
- [Templates](docs/templates.md): files that differ per machine.
- [The ld interface](docs/ld.md): rules, options, classes, plugins, the Lua toolbox.
- [The repository](docs/repository.md): layout, the rules git reads, mode and owner.
- [Encrypted files](docs/secrets.md): age and gpg secrets.
- [Backups](docs/backups.md): what a run saved, and restore.

Notes on packaging and internals live in [internal/](internal/).
