# Roadmap

Planned work, not implemented yet. Nothing here is settled: the shape of each
item can change before it lands.

## Templates

Templates exist: a `*.luadot/` directory with a `luadot.lua` inside, resolved by
`luadot alt`. What was left out of that first pass:

- **`status` and `diff` for what a template produces.** Templates only run under
  `alt`, so both say nothing about the files they generate and skip them.
  Reporting them means either running the templates outside `alt`, or recording
  the last resolution, or rendering into the temporary copy `diff` already
  builds.
- **`add`, `rm` and `edit` on a template.** They all work on plain managed
  files; a template directory is reached by path only through `alt`.

## Diff

`luadot diff` exists: it stages both sides of every file that is not synced into
a private temporary mirror and hands the two directories to `git diff
--no-index`. What was left out of that first pass:

- **A `--stat` summary**, and whether the same thing belongs on `status` as
  `status --diff`.
- **Diffing without git**, computing the hunks in process instead of shelling
  out. It would work on a machine without git and give control over the output,
  at the cost of the user's pager, colors and `diff.*` settings. Any crate
  considered has to be checked for current maintenance before it is added.

## Backups and restore

`apply`, `alt` and `rm` save every file they destroy, into
`~/.local/share/luadot/backups/<unix-millisecond>/`, under the same `home/` and
`root/` layout the repository uses; `ld.opt.backup(false)` turns it off,
`ld.opt.backup_dir(path)` moves the directory elsewhere,
`ld.opt.backup_keep(n)` keeps only the `n` most recent, and `restore` puts a
backup back. `add` takes none because it writes over nothing: it refuses a
destination that already exists and leaves the home copy where it is. What is
still missing:

- **Retention beyond a count.** The limit is a number of backups and nothing
  else: no age limit, and no way to prune by hand short of removing the
  directories. A `restore --prune` and an `ld.opt.backup_age` are the obvious
  answers; neither is settled, and a count may well be enough.

## Encrypted files

Encrypted files exist: an `encrypt` rule makes `add` store ciphertext under a
`.age`/`.gpg` extension, `apply`, `status`, `edit` and `rm` decrypt through the
`age` or `gpg` binary, configured by `ld.crypt.backend`, `ld.crypt.recipients`
and `ld.crypt.identity`. What was left out of that first pass:

- **`diff` over an encrypted file.** It still compares the stored ciphertext;
  comparing the decrypted content means staging it into the same private mirror
  `diff` already builds, so no plaintext outlives the command.
- **Files under `root/`.** Their apply path runs through `sudo` and staged
  writes; encrypting them means deciding where the plaintext may flow during
  escalation, so `add` and `apply` refuse the combination for now.
- **A passphrase-only mode.** Both backends run non-interactively against a
  key; someone who keeps no key on the machine has no way in.
- **Re-encrypting after a recipient change.** New recipients only reach a file
  when its content changes through `edit`; a command re-encrypting everything
  in place is still missing.
- **A Rust implementation as a fallback.** Everything shells out to the
  `age`/`gpg` binaries; a machine without them cannot decrypt. Any crate
  considered has to be checked for current maintenance before it is added.
- **Skipping over a failed decryption.** `apply` aborts with the backend's
  error; whether a warn-and-continue mode is worth having is unsettled.
