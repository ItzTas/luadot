# The repository

The repository mirrors your home directory, path for path: `add` puts
`~/.zshrc` at `.zshrc` and `~/.config/nvim/init.lua` at
`.config/nvim/init.lua`, and `apply` puts them back. A path outside your home
directory is refused:

```
$ luadot add /etc/pacman.conf
add: cannot manage /etc/pacman.conf: outside your home directory /home/u
```

## The repository's own files

A few files at the top level belong to the repository rather than to your
home directory, and are never applied: `.git/`, `.gitignore`,
`.gitattributes`, `.gitmodules`, and the `.luarc.json` that `init`, `clone`
and `meta install` write. Anything else at the top level is a dotfile, so a
README or a license needs an `ignore` rule:

```lua
ld.rules({ match = { "README.md", "LICENSE" }, ignore = true })
```

`.gitattributes` is partly luadot's own: `add` keeps its `# luadot:lfs` block
in step with the rules carrying `lfs = true`, and stages it along with what it
mirrored. Lines outside that block are yours. See
[the ld interface](ld.md#git-lfs).

## What git refuses to keep

`add` reads the repository's `.gitignore` before it writes anything: a file
git would never track would sit outside every commit and be gone on the next
clone. A path named on the command line that lands on an excluded destination
stops the run:

```
$ luadot add ~/.cache
add: /home/u/.cache lands on .cache, which the repository's .gitignore excludes
```

Walking a directory is quieter: the excluded files are left out and the rest
is added, so `luadot add ~/.config/nvim` brings the configuration in and
leaves the logs behind. Nested `.gitignore` files, negated patterns,
`.git/info/exclude` and the global excludes file all count, exactly as git
reads them; a repository git does not track excludes nothing.

## Mode and owner

Two rule keys decide what a placed file looks like on disk, whatever `link`
says:

```lua
ld.rules({
  { match = ".ssh/**", mode = "0600" },
  { match = ".local/bin/**", mode = "0755", owner = "me:wheel" },
})
```

`mode` is the permission bits, three or four octal digits. A file carrying
other bits reports as `differs`, `diff` prints both modes, and `apply` puts
the bits back. Git keeps only the executable bit, so this is how a key stays
`600` after a clone. A hard link or a symlink shares its inode with the
repository's copy, so the bits land there too. Without a `mode`, a copy gets
the repository file's own bits and nothing is compared.

`owner` is `"user"` or `"user:group"`, set through `chown` once the file is
placed. luadot never asks for privilege: a user you cannot chown to stops the
run with chown's own message, a group you belong to works. Without an
`owner`, the file belongs to you.
