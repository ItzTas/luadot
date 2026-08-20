# The repository

The repository mirrors the machine under two directories: `home/` holds what
lives in your home directory, `root/` the rest of the filesystem, path for
path. `add` chooses between them by the path it is given:

```
luadot add ~/.zshrc          -- lands in home/.zshrc
luadot add /etc/pacman.conf  -- lands in root/etc/pacman.conf
```

Anything at the top level outside the two directories (the repository's own
README, a license) is never applied anywhere.

## What git refuses to keep

`add` reads the repository's `.gitignore` before it writes anything: a file
git would never track would sit outside every commit and be gone on the next
clone. A path named on the command line that lands on an excluded destination
stops the run:

```
$ luadot add ~/.cache
add: /home/u/.cache lands on home/.cache, which the repository's .gitignore excludes
```

Walking a directory is quieter: the excluded files are left out and the rest
is added, so `luadot add ~/.config/nvim` brings the configuration in and
leaves the logs behind. Nested `.gitignore` files, negated patterns,
`.git/info/exclude` and the global excludes file all count, exactly as git
reads them; a repository git does not track excludes nothing.

## System files

Files under `root/` go through the same commands, with two differences.

They are always plain copies, never links: a hard link would leave the
repository's copy owned by root, and a symlink into your home directory breaks
while your home is unavailable.

Writing them usually takes privilege: every operation is tried as you first,
and only when the filesystem answers "permission denied" does luadot run
`sudo` for that one file (`install` to place it, `cat` to read it). A run that
never touches a privileged path never asks for a password.

Mode and owner come from the rules:

```lua
ld.rules({
  { match = "root/etc/**", mode = "0644", owner = "root:root" },
  { match = "root/etc/sudoers.d/**", mode = "0440" },
})
```

Without a `mode`, the repository file's own permission bits are applied.
Without an `owner`, the file belongs to whoever wrote it: you when no
privilege was needed, root when sudo was. `status` reports a system file it
cannot read as `unreadable` and moves on; `apply` and `diff` read it through
`sudo cat`.
