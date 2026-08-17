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

Some dotfiles hold secrets — SSH private keys, API tokens, `~/.netrc`, mail
credentials — and today they can only be ignored, which leaves them outside the
repository and outside `apply`. Managers like chezmoi and yadm solve this by
encrypting the files chosen by the user, so the repository can stay public while
the plaintext only ever exists in the home directory.

### Prior art

- **chezmoi** encrypts file by file. The backend is `age` or `gpg`, chosen in
  its configuration; each encrypted file is stored with an `encrypted_` prefix
  in its name and decrypted on apply.
- **yadm** keeps a list of patterns in `~/.config/yadm/encrypt`, and
  `yadm encrypt` bundles everything matching it into a single encrypted archive
  committed to the repository; `yadm decrypt` restores them. It also documents
  `transcrypt` as an alternative, which encrypts per file through git filters.

Per-file encryption is the better fit here: it keeps `git diff` meaningful at
the file level, allows partial decryption, and avoids the "re-encrypt
everything" step yadm's archive needs on every change.

### Proposed configuration

Encryption is a per-file property, so it belongs with the existing per-file
properties — `link` and `conflict` — and follows the same defaults-plus-rules
model:

```lua
ld.crypt.backend("age")
ld.crypt.recipients({ "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p" })

ld.rules({
  { match = ".ssh/id_*", encrypt = true },
  { match = ".config/*/secrets.toml", encrypt = true },
})
```

| Call | Arguments | Effect |
| --- | --- | --- |
| `ld.crypt.backend(name)` | `"age"`, `"gpg"` | Tool used to encrypt and decrypt managed files. |
| `ld.crypt.recipients(keys)` | a key or a list of them | Public keys or key ids the files are encrypted to. |
| `ld.crypt.identity(path)` | a path | Private key used to decrypt; defaults to the backend's own default location. |
| `ld.crypt(options)` | a table of options | Sets several options at once; only the keys it carries. |

The last-rule-wins and accumulate semantics stay exactly as they are today, and
`encrypt` defaults to `false`, so nothing changes for a configuration that
never mentions it.

### Behavior

- `add` on a file matching an encrypt rule encrypts it into the repository
  instead of copying it, and never writes the plaintext there.
- `apply` decrypts to the home directory. Linking is bypassed for these files —
  a hard link or symlink to ciphertext would be useless — so they are always
  written as a plain copy, whatever `link` says.
- `edit` decrypts to a temporary file, opens the editor, then re-encrypts and
  removes the temporary file, so the plaintext never lands in the repository.
- Conflict policies apply as usual, comparing the decrypted content against
  what is on the system.
- `diff` compares the decrypted content, staged into the same private mirror it
  already builds, so no plaintext outlives the command.
- The stored file keeps its path and gains an extension (`.age`, `.gpg`), so
  the mapping to the home directory stays obvious and the repository shows at a
  glance what is encrypted.

### Open questions

- Whether to shell out to the `age`/`gpg` binaries or link a Rust
  implementation. Shelling out ships nothing extra and matches what the user
  already has configured; a library removes the dependency on an external tool
  and gives better errors. Any crate considered has to be checked for current
  maintenance before it is added.
- What to do when decryption fails during `apply` — skip the file with a
  warning, or abort the whole run.
- Whether a passphrase-only mode (no key pair) is worth supporting for people
  who do not keep a key on the machine.
- Whether re-encrypting everything after a recipient list change deserves its
  own command.
- Temporary plaintext during `edit` should live somewhere that is not
  world-readable, and be removed even when the editor exits badly.
