# Backups

Every file luadot writes over is copied aside first: `apply` and `tmpl alt`
save what they replace, `rm` saves the repository entry it deletes and the
system symlink it writes over. `add` takes no backup.

```
luadot: applied 12 file(s) (0 created, 1 replaced, 11 unchanged, 0 skipped)
luadot: backed up 1 file(s) in ~/.local/share/luadot/backups/1786677956412
```

One directory per run, named after the millisecond it ran, holding each saved
file under its absolute path: `/home/u/.zshrc` is kept at
`<run>/home/u/.zshrc`, and the repository entry `rm` deletes under the
repository's own path. A symlink is kept as a symlink; nothing is written for
a file that was created rather than replaced.

## restore

`luadot restore` puts the most recent backup back, asking first and listing
what is about to land. A backup is reached by its name, `-l` (or `--list`)
lists them, `-l <backup>` lists the files one holds, `-y` answers the question
upfront, `-n` reports what would be put back.

Each file is printed with the system path it lands on and what happens there,
`create` when the path is gone and `replace` when it is still in place:

```
$ luadot restore --list
1786677956412  2 minutes ago  2 file(s)
1786590012773  1 day ago      4 file(s)

$ luadot restore --list 1786677956412
1786677956412  2 minutes ago  2 file(s)
  /home/u/.zshrc
  /home/u/.vimrc

$ luadot restore
replace    /home/u/.zshrc
create     /home/u/.vimrc
Put 2 file(s) of backup 1786677956412 back? [y/N] y
replaced   /home/u/.zshrc
created    /home/u/.vimrc
luadot: restored 2 file(s) from backup 1786677956412 (1 created, 1 replaced)
```

Restoring writes plain copies, so the files it touches stop being linked to
the repository until the next `apply`.

A saved file only goes back where luadot manages: under your home directory,
or under the repository when it sits outside the home. A backup directory
holding any other absolute path is refused by name, so nothing is put back
past those two roots.

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

The limits count whole backups, not the files inside them: a run is kept
entirely or dropped entirely. Without a limit nothing is pruned.
