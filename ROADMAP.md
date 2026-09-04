# Roadmap

Planned work, not implemented yet. Nothing here is settled: the shape of each
item can change before it lands.

## Diff

`luadot diff` exists: it stages both sides of every file that is not synced into
a private temporary mirror and hands the two directories to `git diff
--no-index`. What was left out of that first pass:

- **A `--stat` summary on the command line.** The configuration reaches it
  already, through `ld.on.diff({ args = { "--stat" } })`; whether a flag of its
  own is worth having, and whether the same thing belongs on `status` as
  `status --diff`, is unsettled.
- **Diffing without git**, computing the hunks in process instead of shelling
  out. `ld.on.diff({ tool = ... })` names another program, but every one of them
  still has to be installed; doing it in process would work on a machine with
  none, at the cost of the user's pager, colors and `diff.*` settings. Any crate
  considered has to be checked for current maintenance before it is added.

## Backups and restore

`apply`, `tmpl alt` and `rm` save every file they destroy, into
`~/.local/share/luadot/backups/<unix-millisecond>/`, under the file's absolute
path; `ld.opt.backup(false)` turns it off,
`ld.opt.backup_dir(path)` moves the directory elsewhere,
`ld.opt.backup_keep(n)` keeps only the `n` most recent, `ld.opt.backup_age(span)`
drops the ones older than the span, and `restore` puts a backup back. `add`
takes none because it writes over nothing: it refuses a destination that already
exists and leaves the home copy where it is. What is still missing:

- **Pruning by hand.** Both limits are applied at the end of a run that takes a
  backup, so a machine that stops running `apply` keeps whatever it has. A
  `restore --prune` running the same retention on demand is the obvious answer;
  it is not settled.

## Encrypted files

Encrypted files exist: an `encrypt` rule makes `add` store ciphertext under a
`.age`/`.gpg` extension, `apply`, `status`, `diff`, `edit` and `rm` decrypt
through the `age` or `gpg` binary, configured by `ld.crypt.backend`,
`ld.crypt.recipients`, `ld.crypt.identity`, `ld.crypt.identity_command` and
`ld.crypt.passphrase`, and `luadot rekey` re-encrypts everything for the
recipients set now. What was left out of that first pass:

- **A Rust implementation as a fallback.** Everything shells out to the
  `age`/`gpg` binaries; a machine without them cannot decrypt. Any crate
  considered has to be checked for current maintenance before it is added.
- **Skipping over a failed decryption.** `diff` warns and leaves the file out,
  `apply` aborts with the backend's error. Whether `apply` should have a
  warn-and-continue mode of its own is unsettled.
