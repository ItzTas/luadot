# luadot

A dotfiles manager configured in Lua.

luadot keeps your dotfiles in a git repository and puts them back on every
machine you clone it to. The configuration is a Lua script instead of a static
file, so one repository answers for a laptop, a desktop and a server without a
branch or a copy per machine.

The repository mirrors your home directory, path for path: `.zshrc` in the
repository is `~/.zshrc` on the machine. Rules decide how each file is placed:
linked hard, symbolic or copied, ignored, encrypted, kept in Git LFS, or
generated per machine by a template.

```lua
ld.rules({
  { match = ".ssh/id_*", encrypt = true },
  { match = ".config/mako/**", on_change = "makoctl reload" },
  { match = ".cache/**", ignore = true },
})

if ld.sys.has_battery() then
  ld.rules({ { match = ".config/tlp/**", link = "symbolic" } })
end
```

## Quick start

```
luadot init ~/dotfiles       # or: luadot clone git@github.com:me/dotfiles.git
luadot add ~/.zshrc
luadot git commit -m "first"
```

On the next machine, `luadot clone` and `luadot apply`. From then on,
`luadot status` says what drifted, `luadot diff` shows it, and `luadot sync`
commits and pushes it.

## Install

```
paru -S luadot                                  # Arch Linux (AUR); also luadot-bin
nix profile install github:ItzTas/luadot        # Nix
cargo install --git https://github.com/ItzTas/luadot luadot
```

Debian and Ubuntu get a `.deb` with each release. [docs/install.md](docs/install.md)
has the details.

## Documentation

- [Install](docs/install.md): AUR, deb, Nix, source.
- [Commands](docs/commands.md): what each one does.
- [The repository](docs/repository.md): layout, the rules git reads, mode and owner.
- [The ld interface](docs/ld.md): rules, options, classes, plugins, the Lua toolbox.
- [Templates](docs/templates.md): files that differ per machine.
- [Encrypted files](docs/secrets.md): age and gpg secrets.
- [Backups](docs/backups.md): what a run saved, and restore.

Notes on packaging and internals live in [internal/](internal/).
