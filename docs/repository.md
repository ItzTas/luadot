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

The mirror goes all the way to the top, so the rules git needs for the
repository do not sit there. They live in `~/.local/share/luadot/git/`, a
directory managed like any other: `ignore` holds what a `.gitignore` would,
`attributes` what a `.gitattributes` would, and `luadot add` on either puts it
in the repository like any dotfile. Every command copies the repository's copy
of each into `.git/info/exclude` and `.git/info/attributes`, between a
`# luadot` and a `# /luadot` line, where git reads them for that clone with no
file in the tree; whatever you wrote in those two files outside the markers
stays. `add` keeps the `# luadot:lfs` block of `attributes` in step with the
rules carrying `lfs = true` and stages it; the lines outside that block are
yours too.

`clone` copies both before anything else and pulls the LFS contents the
attributes name, so a fresh clone needs nothing more. A clone made with plain
`git` reads neither until a luadot command runs in it.

A `.gitignore` or `.gitattributes` at the top of the repository is a dotfile
like any other and lands in `~`; git reads it for the repository all the same,
so keep the repository's rules in `~/.local/share/luadot/git/` instead.
`.gitmodules` has no other place: git reads it at the top only, so a
repository with submodules keeps it there and `apply` puts a copy in `~`,
where git ignores it. Anything else at the top level, a README or a license,
needs an `ignore` rule:

```lua
ld.rules({ match = { "README.md", "LICENSE" }, ignore = true })
```

Only `.git/` stays out, whatever the rules say. luadot writes nothing of its
own at the top level: the `.luarc.json` that `init`, `clone` and `meta
install` produce goes to `~/.config/luadot/`, since one in the repository
would land in `~` and make your whole home directory a Lua workspace.

## What git refuses to keep

`add` reads the repository's ignore rules before it writes anything: a file
git would never track would sit outside every commit and be gone on the next
clone. A path named on the command line that lands on an excluded destination
stops the run:

```
$ luadot add ~/.cache
add: /home/u/.cache lands on .cache, which the repository's ignore rules exclude
```

Walking a directory is quieter: the excluded files are left out and the rest
is added, so `luadot add ~/.config/nvim` brings the configuration in and
leaves the logs behind. The repository's `ignore`, nested `.gitignore` files,
negated patterns, `.git/info/exclude` and the global excludes file all count,
exactly as git reads them; a repository git does not track excludes nothing.

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
