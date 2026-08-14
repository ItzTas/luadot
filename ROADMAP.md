# Roadmap

Planned work, not implemented yet. Nothing here is settled: the shape of each
item can change before it lands.

## Templates

Templates exist: a `*.luadot/` directory with a `luadot.lua` inside, resolved by
`luadot alt`. What was left out of that first pass:

- **A mini-syntax for `ld.alt.render`.** Today a rendered file is a Lua script
  returning a string, which is fine for a handful of substitutions and awkward
  for a long configuration file. A `{{ var }}` form would read better; whether
  it earns its own parser is the open question.
- **`status` for what a template produces.** Templates only run under `alt`, so
  `status` says nothing about the files they generate. Reporting them means
  either running the templates outside `alt` or recording the last resolution.
- **`add`, `rm` and `edit` on a template.** They all work on plain managed
  files; a template directory is reached by path only through `alt`.

## Diff

`status` says a managed file `differs`, and stops there. Knowing *what* differs
means running a diff by hand against the repository path, which is the one step
between noticing a divergence and deciding whether to `apply` over it or bring
the change back in with `add`. A `luadot diff [path...]` closes that loop.

- No path diffs every managed file that is not synced; a path narrows it to that
  file or to everything under that directory.
- The repository is the left side and the system the right side, so the output
  reads as what `apply` would overwrite.
- Files reported `unlinked` have identical content and produce no diff; only
  `differs` and `missing` have anything to show.

### Open questions

- Whether to shell out to `git diff --no-index` — the repository is a git
  repository already, so the user's pager, colors and `diff.*` settings come for
  free — or to compute the diff in process with a crate, which works without git
  and gives control over the output. Any crate has to be checked for current
  maintenance before it is added.
- Binary files, which have no useful textual diff and should be reported as
  "binary files differ" plus their sizes.
- Templates, whose repository side does not exist until `alt` runs them, so a
  diff means rendering into a temporary file first.
- Encrypted files, once they exist, have to be decrypted before comparison and
  must never leave plaintext behind.
- Whether a `--stat` summary is worth having, and whether this belongs as
  `status --diff` instead of its own command.

## Backups and restore

`apply` and `alt` already save every file they replace, into
`~/.local/share/luadot/backups/<unix-timestamp>/`, mirroring the path below the
home directory; `ld.opt.backup(false)` turns it off. What is still missing:

- **Wiring `restore` into the CLI.** The command exists as a module and is not
  reachable yet.
- **The other commands that overwrite.** `add` and `rm` move files around
  without taking a backup, so the safety net only covers half of the writes.
- **Retention.** Nothing ever prunes the backup directory, so it grows on every
  run. A keep-the-last-N option, an age limit, or a `restore --prune` are the
  obvious answers; which one is not settled.
- **Where the backups live.** They are recreatable state rather than data, so
  `$XDG_STATE_HOME` (`~/.local/state/luadot/backups/`) is arguably the right
  place. Moving them later orphans the backups already taken, so this is worth
  deciding before the directory has real content in it.
- **Timestamp collisions.** The directory name is the unix second, so two runs
  within the same second share a backup directory and the second one overwrites
  what the first saved.

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
