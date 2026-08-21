# Backups

Every file luadot writes over is copied aside first: `apply` and `tmpl alt`
save what they replace, `rm` saves the repository entry it deletes and the
system symlink it writes over. `add` takes no backup, since it never replaces
anything.

```
luadot: applied 12 file(s) (0 created, 1 replaced, 11 unchanged, 0 skipped)
luadot: backed up 1 file(s) in ~/.local/share/luadot/backups/1786677956412
```

One directory per run, named after the millisecond it ran, holding each saved
file under its absolute path: `/home/u/.zshrc` is kept at
`<run>/home/u/.zshrc`, and the repository entry `rm` deletes under the
repository's own path, wherever it lives. A symlink is kept as a symlink;
nothing is written for a file that was created rather than replaced.

## restore

`luadot restore` puts the most recent backup back, asking first and listing
what is about to land. A backup is reached by its name, `-l` (or `--list`)
lists them, `-y` answers the question upfront, `-n` reports what would be put
back:

```
$ luadot restore --list
1786677956412  2 minutes ago  1 file(s)
1786590012773  1 day ago      4 file(s)

$ luadot restore
  home/u/.zshrc
Put 1 file(s) of backup 1786677956412 back? [y/N]
```

Restoring writes plain copies, so the files it touches stop being linked to
the repository until the next `apply`.

## Location and pruning

Backups live in `~/.local/share/luadot/backups` (or
`$XDG_DATA_HOME/luadot/backups`); `ld.opt.backup_dir` moves them,
`ld.opt.backup(false)` turns backups off.

Nothing is pruned until a limit is set. `ld.opt.backup_keep(n)` keeps the `n`
most recent backups; `ld.opt.backup_age(span)` drops the ones older than the
span, a number and a unit (`s`, `m`, `h`, `d` or `w`). Both prune at the end
of every run that takes a backup, and both can be set at once, each dropping
what it reaches:

```lua
ld.opt.backup_keep(10)
ld.opt.backup_age("30d")
```

```
luadot: backed up 1 file(s) in ~/.local/share/luadot/backups/1786677956412
luadot: dropped 1 backup(s), keeping the 10 most recent
```

The limits count whole backups, not the files inside them: a run either keeps
everything it saved or is dropped as a whole, never left half there. Without a
limit the directory grows on every run and pruning is yours to do.
