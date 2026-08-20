# luadot

A dotfiles manager configured in Lua.

One git repository mirrors your machines: `home/` for your home directory,
`root/` for the rest of the filesystem. A Lua configuration decides how each
file is placed: linked hard, symbolic or copied, ignored, encrypted, or
generated per machine by a template.

```lua
ld.rules({
  { match = "home/.ssh/id_*", encrypt = true },
  { match = "home/.config/mako/**", on_change = "makoctl reload" },
  { match = "home/.cache/**", ignore = true },
})

if ld.sys.has_battery() then
  ld.rules({ { match = "home/.config/tlp/**", link = "symbolic" } })
end
```

Secrets go through `age` or `gpg` and never reach the repository as plaintext.
Files under `/etc` are written with `sudo` only when the filesystem demands
it. Everything luadot overwrites is backed up first.

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
- [The repository](docs/repository.md): layout, gitignore, system files.
- [The ld interface](docs/ld.md): rules, options, classes, the Lua toolbox.
- [Templates](docs/templates.md): files that differ per machine.
- [Encrypted files](docs/secrets.md): age and gpg secrets.
- [Backups](docs/backups.md): what a run saved, and restore.

Notes on packaging and internals live in [internal/](internal/).

## License

MIT; see [LICENSE](LICENSE). The bundled LPeg keeps its own MIT notice in
[vendor/lpeg/LICENSE](vendor/lpeg/LICENSE).
