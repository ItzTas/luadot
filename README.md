# luadot

A dotfiles manager configured in Lua.

## Install

On Arch Linux, from the AUR:

```
paru -S luadot
```

`luadot-nightly` follows the pre-release tags instead, and conflicts with
`luadot` — install one or the other. Both build from the tagged source, run the
test suite, and ship the shell completions.

Anywhere else, from source:

```
cargo install --git https://github.com/ItzTas/luadot luadot
```

## Commands

| Command | Effect |
| --- | --- |
| `luadot init [dir]` | Creates an empty dotfiles repository and makes it the managed one. |
| `luadot clone <url> [dir]` | Clones a dotfiles repository and makes it the managed one. |
| `luadot add <path>...` | Starts managing a file or directory, mirroring it into the repository. |
| `luadot rm [-y] [-n] <path>...` | Stops managing a file or directory, leaving the system copy in place. |
| `luadot status [-t] [path]` | Lists the managed files whose system copy is not in sync, `-t` the files the templates produce too. |
| `luadot diff [-t] [path]` | Shows what the repository holds and the system does not, `-t` what the templates produce too. |
| `luadot apply [-n] [path]` | Puts the repository's files back on the system. |
| `luadot alt [-n] [path]` | Runs the templates and puts the files they produce on the system. |
| `luadot new [-f] <path>` | Creates an empty template in the repository, for the file that path names. |
| `luadot restore [-l] [-y] [-n] [backup]` | Puts back the files an earlier `apply` or `alt` replaced. |
| `luadot edit <path>` | Opens the repository's copy of a file, or the script of the template producing it, in `$VISUAL`/`$EDITOR`. |
| `luadot rekey [-n] [path]` | Re-encrypts the repository's secrets for the recipients set now. |
| `luadot exec <source\|file.lua> [args]...` | Runs Lua with `ld` installed, from a string or a `.lua` file. |
| `luadot config [show\|path\|edit]` | Shows the resolved configuration, prints its path, or opens it. |
| `luadot class [list\|set\|unset\|get]` | Lists the declared classes and answers them for this machine. |
| `luadot cd` | Starts a shell in the repository. |
| `luadot git <args>...` | Runs git inside the repository. |
| `luadot push [args]...` | Shorthand for `luadot git push`. |
| `luadot completions <shell>` | Prints a completion script for that shell. |

`luadot --help` explains any command in place (`luadot rm --help`), and
`luadot --version` prints the version.

### Where the repository lives

`clone` puts it in `~/.local/share/luadot/repo` (or `$XDG_DATA_HOME/luadot/repo`)
and remembers the path, so nothing else has to be told about it. A directory of
your own is given to `clone` directly, resolved against the directory you are
in, the way `git clone` does it:

```
luadot clone git@github.com:me/dotfiles.git ~/dotfiles
```

`init` is the same thing without a repository to start from: it creates an empty
git repository, in the same default place or in the directory you name, and
remembers it too. The directory has to be empty or not exist yet, and nothing is
committed for you — `luadot add` fills it and `luadot git` commits it:

```
luadot init ~/dotfiles
luadot add ~/.zshrc
luadot git add -A
luadot git commit -m "first"
```

`ld.opt.repo_dir` says it in the configuration instead, which is what a
repository luadot did not clone needs — the one already sitting in `~/dotfiles`
from before, or a checkout shared with another tool:

```lua
ld.opt.repo_dir("~/dotfiles")
```

It wins over what `clone` remembered, so it is also how a machine follows a
repository that moved. The path is read on every command; luadot never moves the
directory for you, and points at it where it stands.

`status` reports one line per file that `apply` would touch, and a count for
everything else:

- `missing` — the file is in the repository but not on the system.
- `unlinked` — the contents match, but the system copy is not the link the
  configuration asks for.
- `differs` — the system copy holds something else.
- `unreadable` — a system file luadot may not read; `status` never asks for
  privilege, `apply` does.

Those lines and the count closing them are the configuration's to replace, in
`ld.on.status` — [Customizing a command](#customizing-a-command) is what it
takes.

`diff` picks up where `status` stops, and shows the content behind a `differs`:

```
$ luadot diff
diff --git repository/home/.vimrc system/home/.vimrc
--- repository/home/.vimrc
+++ system/home/.vimrc
@@ -1,2 +1,2 @@
 set number
-set ruler
+set paste
luadot: 1 of 12 managed file(s) differ
```

The repository is the left side and the system the right side, so what the
diff adds is what `apply` would overwrite and what it removes is what `add`
would bring in. A path narrows the report to that file or to everything below
that directory. A file the system does not have shows its whole content as
absent from the right side; one reported `unlinked` holds the same content and
has nothing to show.

The diff itself is `git diff`, run over a private copy of the two sides, so
your pager, your colors and your `diff.*` settings are the ones that apply, and
binary files are reported as differing instead of printed. Since git only
records the executable bit, a system file whose content matches but whose mode
drifted gets a line of its own:

```
mode       root/etc/sudoers.d/wheel 0644 -> 0440
```

Every line of that report is the configuration's to replace, down to the
program the two sides are handed to — `ld.on.diff` is where it is said, and
[Customizing a command](#customizing-a-command) is what it takes.

Templates are left out of both reports, their side not existing until they are
resolved; the summary says how many were. `-t` (or `--templates`) resolves
them and reports the files they produce, without writing any of them:

```
$ luadot status --templates
missing    home/.config/nvim/init.lua
luadot: 12 managed file(s) (12 synced, 0 missing, 0 unlinked, 0 differs)
luadot: 2 template(s) into 3 file(s) (2 synced, 1 missing, 0 unlinked, 0 differs)
```

`diff --templates` shows the same files as a diff, the generated side under
`generated/` rather than `repository/`. Both run the template's own
`luadot.lua`, which is why the flag exists — the Templates section says more
about it.

`rm` is the inverse of `add`: it removes the file from the repository and leaves
your home directory with a plain, unmanaged copy of it. When the system copy is
a symlink into the repository, the content is written out before the repository
entry goes away; when it is a hard link or a file of its own, it is left alone.
Directories that become empty are pruned from the repository.

Removing more than one file asks first, listing what is about to go:

```
  home/.config/nvim/init.lua
  home/.config/nvim/lua/plugins.lua
Stop managing 2 file(s)? [y/N]
```

`-y` (or `--yes`) answers it upfront, which is also what a script needs: without
a terminal to ask on, `rm` refuses rather than assuming an answer.

### The layout

The repository mirrors the machine under two directories: `home/` holds what
lives in your home directory, `root/` holds the rest of the filesystem, path
for path. `add` chooses between them by the path it is given:

```
luadot add ~/.zshrc          -- lands in home/.zshrc
luadot add /etc/pacman.conf  -- lands in root/etc/pacman.conf
```

Anything at the top level outside the two directories — the repository's own
README, a license — is the repository's own and is never applied anywhere.

### What git refuses to keep

`add` reads the repository's `.gitignore` before it writes anything: a file git
would never track has no business being mirrored there, where it would sit
outside every commit and be gone on the next clone. A path named on the command
line that lands on an excluded destination stops the run:

```
$ luadot add ~/.cache
add: /home/u/.cache lands on home/.cache, which the repository's .gitignore excludes
```

Walking a directory is quieter: the excluded files are left out and the rest is
added, so `luadot add ~/.config/nvim` brings the configuration in and leaves the
logs behind. Nested `.gitignore` files, negated patterns, `.git/info/exclude`
and the global excludes file all count, exactly as git reads them; a repository
git does not track excludes nothing.

### System files

Files under `root/` go through the same commands as everything else, with two
differences. They are always placed as plain copies, never links: a hard link
would leave the repository's copy owned by root, and a symlink into your home
directory breaks for anything that runs while your home is unavailable. And
writing them usually takes privilege: every operation is tried as you first,
and only when the filesystem answers "permission denied" does luadot run
`sudo` for that one file (`install` to place it, `cat` to read it), so a run
that never touches a privileged path never asks for a password.

The mode and owner a system file should carry come from the rules:

```lua
ld.rules({
  { match = "root/etc/**", mode = "0644", owner = "root:root" },
  { match = "root/etc/sudoers.d/**", mode = "0440" },
})
```

Without a `mode`, the repository file's own permission bits are applied;
without an `owner`, the file belongs to whoever wrote it — you when no
privilege was needed, root when sudo was. `status` reports a system file it
cannot read as `unreadable` and moves on; `apply` and `diff` read it through
`sudo cat` before deciding what to do.

### Encrypted files

Some dotfiles hold secrets — SSH private keys, API tokens, `~/.netrc`,
`/etc/wireguard/wg0.conf` — and a public repository is no place for their
plaintext. An `encrypt` rule keeps the file managed anyway: the repository
stores only ciphertext, and the plaintext only ever exists where the file is
meant to live.

```lua
ld.crypt.recipients("age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p")
ld.crypt.identity("~/.keys/age.txt")

ld.rules({
  { match = "home/.ssh/id_*", encrypt = true },
  { match = "home/.netrc", encrypt = true },
  { match = "root/etc/wireguard/**", encrypt = true, owner = "root:root" },
})
```

Encryption runs through the `age` or `gpg` binary on your `PATH` — luadot
ships no cryptography of its own. `age` is the default;
`ld.crypt.backend("gpg")` switches. The stored file keeps its path and gains
the backend's extension, so `add ~/.netrc` lands in `home/.netrc.age` and the
repository shows at a glance what is encrypted; the extension is what marks a
file as encrypted from then on, whatever the rules say later.

- `add` encrypts the file into the repository instead of linking it, and never
  writes the plaintext there.
- `apply` decrypts to its place on the system. Linking is bypassed — a link to
  ciphertext would be useless — so the file is always a plain copy, written
  with mode `600`, whatever `link` says.
- `status` compares the decrypted content against the system copy and reports
  the file `unreadable` when it cannot decrypt.
- `edit` decrypts to a private temporary directory (`0700`, under
  `$XDG_RUNTIME_DIR` when it exists), opens the editor there, re-encrypts and
  removes the plaintext, even when the editor exits badly. An unchanged file is
  left alone, so the ciphertext only churns when the content did change.
- `rm` deletes the ciphertext from the repository; when the system copy is
  missing it decrypts one last time to leave the plaintext behind.
- Conflict policies apply as usual, comparing the decrypted content against
  what is on the system.

Encrypting is done to the `recipients` — age public keys, or key ids for gpg —
and decrypting with age needs the `identity`, the private key file. gpg ignores
the identity and uses its own keyring for both directions. A failed decryption
stops `apply` with the tool's own error rather than skipping the file.

`ld.crypt.identity_command` names a command that hands the identity over
instead, for a key that lives in a password manager rather than on disk. A
string runs through `sh`, a list runs the program itself, and what it prints is
the private key. It runs once per command, never per file, and its output is
written to a `600` file inside the same private temporary directory `edit`
uses, removed when the command ends — the key never touches your dotfiles.

```lua
ld.crypt.identity_command("pass show age/key")
ld.crypt.identity_command({ "op", "read", "op://vault/age/key" })
```

age plugins work as they do for age itself: point `identity` (or
`identity_command`) at the plugin identity and use the plugin's recipients.
luadot only checks that the plugin binary a key names is on your `PATH` —
`AGE-PLUGIN-YUBIKEY-1…` and `age1yubikey1…` both need `age-plugin-yubikey` —
and says which one is missing instead of letting age fail obscurely.

`ld.crypt.passphrase(true)` encrypts to a passphrase instead of keys: `age
--passphrase` or `gpg --symmetric`, with the tool doing the asking, so nothing
about the passphrase passes through luadot. **It is weaker than keys** — the
one passphrase opens every secret, everyone sharing the repository shares it,
and changing it means re-encrypting everything — so every command touching a
secret says so once. `ld.crypt.passphrase_warn(false)` silences that line and
nothing else. age asks per file, and only gpg's agent caches the answer, so
expect one prompt per secret with age.

Changing the recipients does not reach the files already stored: `luadot rekey`
decrypts each secret and encrypts it again for the recipients set now, in
place, one staging file at a time so a failure never leaves a half-written
secret. `-n` reports what it would touch. Switching `ld.crypt.backend` and
running it moves each secret to the other extension (`home/.netrc.age` becomes
`home/.netrc.gpg`), and the repository is yours to commit afterwards.

Files under `root/` can be encrypted too, and the plaintext never reaches the
disk on the way. Reading one to `add` it and writing one back on `apply` are
tried as you first; when the filesystem answers "permission denied", the
plaintext is handed to the backend and to `sudo install` on their standard
input, so it exists only in the memory of the two processes that need it — no
temporary file, no repository copy. A `mode` rule sets the bits, `600` when
there is none, and an `owner` rule sets who owns it, as for any system file.

| Call | Arguments | Effect |
| --- | --- | --- |
| `ld.crypt.backend(name)` | `"age"`, `"gpg"` | Tool used to encrypt and decrypt managed files. Defaults to `"age"`. |
| `ld.crypt.recipients(keys)` | a key or a list of them | Public keys or key ids the files are encrypted to. |
| `ld.crypt.identity(path)` | a path | Private key used to decrypt with age; gpg uses its keyring. `~` and a relative path resolve against your home directory. |
| `ld.crypt.identity_command(command)` | a command line, or a list of a program and its arguments | Command printing the identity, run once per command instead of reading a key file. |
| `ld.crypt.passphrase(enabled)` | `true`, `false` | Whether secrets are locked with a passphrase the backend asks for instead of the recipients. Defaults to `false`. |
| `ld.crypt.passphrase_warn(enabled)` | `true`, `false` | Whether passphrase mode says it is weaker than keys. Defaults to `true`. |
| `ld.crypt(options)` | a table of options | Sets several options at once; only the keys it carries. |

### Seeing it first

`-n` (or `--dry-run`) makes `apply`, `alt` and `rm` report what they would do
and touch nothing — no file written, no backup taken:

```
$ luadot apply --dry-run
create   home/.config/nvim/init.lua
replace  home/.zshrc
luadot: would apply 12 file(s) (1 created, 1 replaced, 10 unchanged, 0 skipped)
```

Only the files that would change are listed, one line each, the same way
`status` reports them.

### Backups

Every file luadot destroys is copied first, so nothing it overwrites is lost:
`apply` and `alt` save what they replace, and `rm` saves both the repository
entry it deletes and the system symlink it writes over.

```
luadot: applied 12 file(s) (0 created, 1 replaced, 11 unchanged, 0 skipped)
luadot: backed up 1 file(s) in ~/.local/share/luadot/backups/1786677956412
```

One directory per run, named after the millisecond it ran, holding the saved
files under the same `home/` and `root/` layout the repository uses, so a
replaced `/etc` file is saved and restored like any other. A symlink is kept
as a symlink; nothing is written for a file that was created rather than
replaced, and `add` takes no backup because it never writes over anything.

`luadot restore` puts the most recent one back, asking first and listing what
is about to land:

```
$ luadot restore --list
1786677956412  2 minutes ago  1 file(s)
1786590012773  1 day ago      4 file(s)

$ luadot restore
  home/.zshrc
Put 1 file(s) of backup 1786677956412 back? [y/N]
```

A backup of your own is reached by its name (`luadot restore 1786590012773`),
`-y` answers the question upfront and `-n` reports what would be put back.
Restoring writes plain copies, so the files it touches stop being linked to the
repository until the next `apply`.

Backups live next to the repository, in `~/.local/share/luadot/backups` (or
`$XDG_DATA_HOME/luadot/backups`), and `ld.opt.backup_dir` moves them anywhere
you like. `ld.opt.backup(false)` turns the whole thing off.

Nothing is pruned until you set a limit. `ld.opt.backup_keep(n)` keeps the `n`
most recent backups and drops the older ones at the end of every run that takes
one:

```lua
ld.opt.backup_keep(10)
```

```
luadot: backed up 1 file(s) in ~/.local/share/luadot/backups/1786677956412
luadot: dropped 1 backup(s), keeping the 10 most recent
```

The limit counts whole backups, not the files inside them — a run either keeps
everything it saved or is dropped as a whole, so a backup is never left half
there.

`ld.opt.backup_age(span)` prunes by age instead: a backup older than the span
is dropped at the end of every run that takes one, however many there are. The
span is a number and a unit — `s`, `m`, `h`, `d` or `w`:

```lua
ld.opt.backup_age("30d")
```

```
luadot: backed up 1 file(s) in ~/.local/share/luadot/backups/1786677956412
luadot: dropped 2 backup(s), keeping the ones taken in the last 30 days
```

Both limits can be set at once, and each one drops what it reaches:
`ld.opt.backup_keep(10)` with `ld.opt.backup_age("30d")` leaves the 10 most
recent backups taken in the last 30 days. Without either the directory grows on
every run and pruning is yours to do.

`exec` runs Lua with the same `ld` interface the configuration gets, which is
how you ask a machine what luadot sees on it:

```
luadot exec 'print(ld.sys.gpu.name)'
luadot exec ~/scripts/report.lua --json
```

The argument is a `.lua` path when it names an existing file or ends in `.lua`,
and Lua source otherwise. Everything after it reaches the script through
`ld.argv.args`. A file requires modules from the `lua/` directory next to it, a
source string from the one in your configuration directory, and neither runs
`config.lua` first: `exec` is a scratchpad, not a command that configures anything.

## Configuration

luadot runs `~/.config/luadot/config.lua` (or `$XDG_CONFIG_HOME/luadot/config.lua`)
before every command. The script configures luadot through the global `ld`
interface; without the file, the defaults apply.

```lua
ld.opt.link("hard")

ld.opt.conflict("overwrite")

ld.rules({
  { match = "home/.ssh/**", link = "symbolic", conflict = "skip" },
  { match = "home/.config/nvim/**", conflict = "error" },
  { match = "home/.config/mako/**", on_change = "makoctl reload" },
  { match = "**/*.swp", ignore = true },
  { match = "home/.cache/**", ignore = true },
})
```

A single rule needs no list around it — `ld.rules({ match = "home/.ssh/**",
link = "symbolic" })` is the same call carrying one entry.

A rule names the files it covers through `match`, a glob, or through `regex`, a
regular expression; a rule carries one of the two, never both.

```lua
ld.rules({
  { regex = "^home/\\.config/(nvim|zsh)/", link = "symbolic" },
  { regex = "\\.sw[po]$", ignore = true },
})
```

The expression is [Rust's regex syntax][regex], matched against the path as
written, with `/` as the separator and no anchoring of its own: `nvim` covers
every path carrying that word, `^home/\.ssh/` only what sits under `~/.ssh/`. Lua
escapes a backslash as `\\`, so a literal dot is `"\\."` inside the script.
Neither backreferences nor lookaround exist there, which is what keeps every
match linear in the length of the path.

[regex]: https://docs.rs/regex/latest/regex/#syntax

A rule carries seven more keys, all optional next to `match` or `regex`:

| Key | Values | Effect |
| --- | --- | --- |
| `link` | `"hard"`, `"symbolic"`, `"copy"` | How the matching files are placed. Files under `root/` are always copies, whatever it says. |
| `conflict` | `"overwrite"`, `"skip"`, `"error"` | Answer when the system copy differs. |
| `on_change` | a command line | Runs after `apply` or `alt` created or replaced one of those files. |
| `ignore` | `true`, `false` | Whether the matching files are left unmanaged. |
| `mode` | three or four octal digits, as a string | The permission bits a matching file under `root/` is placed with. An encrypted file carries `600` without it. |
| `owner` | `"user"` or `"user:group"` | Who owns a matching file under `root/`. |
| `encrypt` | `true`, `false` | Whether `add` stores the matching files encrypted. |

Either syntax also matches a directory on behalf of everything under it, so
`{ match = "home/.ssh" }` and `{ regex = "^home/\\.ssh$" }` both cover
`home/.ssh/keys/id_ed25519`.

The last matching rule wins, key by key, so a general rule is narrowed by a
later one and never merged with it — `{ match = "home/.cache/**", ignore = true }`
followed by `{ match = "home/.cache/keep/**", ignore = false }` ignores
everything under `~/.cache/` but that one directory. `on_change` runs once per command line
and per run, at the end: twenty files under `.config/mako/` changing reload mako
once, not twenty times. A failing command stops the run after the files are in
place, and `--dry-run` prints the command instead of running it.

| Call | Arguments | Effect |
| --- | --- | --- |
| `ld.opt.link(mode)` | `"hard"`, `"symbolic"`, `"copy"` | Default strategy used to link a managed file. |
| `ld.opt.backup(enabled)` | `true`, `false` | Whether a file is copied aside before luadot writes over it. Defaults to `true`. |
| `ld.opt.backup_dir(path)` | a directory | Where those copies land. `~` and a relative path resolve against your home directory. Defaults to `~/.local/share/luadot/backups`. |
| `ld.opt.backup_keep(count)` | a number of one or more | How many backups to keep; the oldest ones are dropped once there are more. Defaults to keeping every one of them. |
| `ld.opt.backup_age(span)` | a span like `"30d"`, in `s`, `m`, `h`, `d` or `w` | How long a backup is kept; the ones older than that are dropped. Defaults to keeping them forever. |
| `ld.opt.conflict(policy)` | `"overwrite"`, `"skip"`, `"error"` | Default answer when `apply` finds a differing file already on the system. |
| `ld.opt.pkg_warn(enabled)` | `true`, `false` | Whether a call is warned about where it is slow or has no effect. Defaults to `true`. |
| `ld.opt.repo_dir(path)` | a directory | The repository luadot manages, winning over the one `clone` left behind. `~` and a relative path resolve against your home directory. |
| `ld.opt(options)` | a table of options | Sets several options at once; only the keys it carries. |
| `ld.crypt.backend(name)` | `"age"`, `"gpg"` | Tool used to encrypt and decrypt managed files. Defaults to `"age"`. |
| `ld.crypt.recipients(keys)` | a key or a list of them | Public keys or key ids encrypted files are encrypted to. |
| `ld.crypt.identity(path)` | a path | Private key used to decrypt with age. |
| `ld.crypt.identity_command(command)` | a command line, or a list of a program and its arguments | Command printing the identity, for a key kept in a password manager. |
| `ld.crypt.passphrase(enabled)` | `true`, `false` | Whether secrets are locked with a passphrase instead of the recipients. Defaults to `false`. |
| `ld.crypt.passphrase_warn(enabled)` | `true`, `false` | Whether passphrase mode says it is weaker than keys. Defaults to `true`. |
| `ld.crypt(options)` | a table of options | Sets several crypt options at once; only the keys it carries. |
| `ld.rules(rules)` | a rule or a list of them | Overrides `link` and `conflict` for the files a glob or a regular expression matches, names an `on_change` command for them, sets the `mode` and `owner` of system files, marks them as never managed, and marks them as encrypted. |
| `ld.class(class)` | a table declaring a class | Declares a question this machine answers once, through `luadot class`. |
| `ld.class.get(name)` | a class name | The answer this machine gave, `nil` when it gave none. |
| `ld.pkg.install(packages)` | a package name or a list of them | Installs packages through the system package manager. |
| `ld.setup(name)` | a setup name | Runs one setup script. |
| `ld.setup.all(options)` | a table with an optional `order` list | Runs every setup script. |
| `ld.setup.list()` | — | The names of the available setup scripts. |
| `ld.cmd(line)` | a command line | Runs it through `sh` and returns what it printed. |
| `ld.cmd.<program>(args...)` | the arguments of that program | Runs the program itself and returns what it printed. |
| `ld.git(args...)` | the arguments of a git command | Runs git inside the repository and returns what it printed. |
| `ld.argv` | — | `name` and `args` of the command being run. |
| `ld.on.status(options)` | a table of options | Says what `status` prints, line by line. |
| `ld.on.diff(options)` | a table of options | Says what `diff` prints and which program compares the two sides. |
| `ld.print(text, options)` | a string and a table of options | Writes a line to the terminal, styled the way the options ask. |
| `ld.print.note(text)`, `ld.print.warn(text)`, `ld.print.error(text)` | a string and a table of options | The same line carrying `luadot:`; the last two on the error stream. |
| `ld.print.section(title)`, `ld.print.entry(label, text)`, `ld.print.field(name, value)` | strings and a table of options | A title over a blank line, a labelled line, a named value. |

Options also take the table form, which sets whatever keys it carries and leaves
the rest alone:

```lua
ld.opt({ link = "symbolic" })
```

It is a normal Lua script, so anything the language offers is available:
conditionals per machine, loops building pattern lists, local helper functions.

### One interface everywhere

`ld` is the same interface in every script luadot runs — `config.lua`,
`bootstrap.lua`, the setup scripts, a template's `luadot.lua` and
`luadot exec` all carry every call. What changes is what a call *does* once it runs, and luadot says it
instead of hiding the call:

| Call | Where it has an effect | Elsewhere |
| --- | --- | --- |
| `ld.rules`, `ld.opt.link`, `ld.opt.backup`, `ld.opt.backup_dir`, `ld.opt.backup_keep`, `ld.opt.backup_age`, `ld.opt.conflict`, `ld.opt.repo_dir`, `ld.crypt.backend`, `ld.crypt.recipients`, `ld.crypt.identity`, `ld.crypt.identity_command`, `ld.crypt.passphrase`, `ld.crypt.passphrase_warn`, `ld.class`, `ld.on.status`, `ld.on.diff` | `config.lua`, which builds the configuration | does nothing, warns |
| `ld.alt.out`, `ld.alt.file`, `ld.alt.render`, `ld.alt.expand`, `ld.alt.read`, `ld.alt.exists`, `ld.alt.glob` | `luadot.lua`, which produces a template's files | does nothing and yields `nil` (`false` for `ld.alt.exists`), warns |
| `ld.cmd`, `ld.git`, `ld.pkg.install`, `ld.setup`, `ld.setup.all` | everywhere | warns where it is slow |
| `ld.opt.pkg_warn`, `ld.setup.list`, `ld.class.get`, `ld.alt.json`, `ld.argv`, `ld.sys`, `ld.path`, `ld.print` | everywhere | — |

A call away from where it has an effect is not an error; it runs, does nothing
and says so:

```
luadot: `ld.rules` in a setup script does nothing; config.lua is where it has an
effect (silence it with `ld.opt.pkg_warn(false)`)
```

`ld.path` carries `home` and `config` everywhere, `repo` once a repository is
set, and `dir` inside a template. Inside `config.lua` itself, `ld.path.repo` is the
repository luadot knew about before the file ran, so it does not answer for an
`ld.opt.repo_dir` set in that same file; every script luadot runs afterwards —
`bootstrap.lua`, a setup script, a template — gets the resolved one.

### The machine

`ld.sys` describes the machine the script is running on, so one repository can
answer for every one of them:

```lua
if ld.sys.gpu.vendor == "nvidia" and ld.sys.ram > 24 * 1024 ^ 3 then
  ld.pkg.install({ "nvidia-utils", "cuda" })
end
```

| Value | Holds |
| --- | --- |
| `ld.sys.host` | `name`, `os` and `arch` of the machine. |
| `ld.sys.gpu` | `vendor`, `name` and `driver` of the first card, and every card as a list. |
| `ld.sys.ram` | The memory of the machine, in bytes, exactly as the kernel reports it. |
| `ld.sys.has_battery()` | `true` on a machine with a battery of its own, `false` on one without. |

`ld.sys.gpu.vendor` is a short name — `nvidia`, `amd`, `intel` — or the PCI
identifier when the vendor is not a known one, and `ld.sys.gpu.name` is the
model as `lspci` reports it, empty when `lspci` is not installed. A machine with
several cards carries each of them:

```lua
for _, card in ipairs(ld.sys.gpu) do
  print(card.vendor, card.name, card.driver)
end
```

`ld.sys.has_battery()` is what tells a laptop from a desktop without naming
either of them, and it ignores the battery of a mouse or a keyboard:

```lua
if ld.sys.has_battery() then
  ld.pkg.install("tlp")
end
```

`ld.sys.ram` is the raw `MemTotal` of the kernel, so it is a little under what
the machine has installed — the firmware keeps a slice of it. Rounding it to a
size a person would recognize is up to the configuration:

```lua
local gb = math.ceil(ld.sys.ram / 1024 ^ 3)
```

### Classes

Some things about a machine cannot be read from it — which of two setups this
one is, what mail address its git commits carry, whether it is the one that
holds the work configuration. A class is that kind of question: the
configuration declares it, the machine answers it once through `luadot class`,
and every script reads the answer back.

```lua
ld.class({
  name = "form-factor",
  prompt = "Is this machine a desktop or a laptop?",
  choices = { "desktop", "laptop" },
  default = "laptop",
})

ld.class({ name = "email", prompt = "Which email do you use for git?" })
```

| Key | Values | Effect |
| --- | --- | --- |
| `name` | a name without spaces | How the class is read and answered. Required. |
| `prompt` | a question | What the machine is asked. Defaults to `define the class <name>`. |
| `choices` | a value or a list of them | Restricts the answer to that list; without it the answer is free text. |
| `default` | one of the choices | The answer pressing enter accepts. It only fills the prompt: an unanswered class still reads as `nil`. |

Declaring the same name twice replaces the first declaration, so a required
module can declare a class the configuration then refines.

`ld.class.get` reads the answer, and reads it the same in `config.lua`,
`bootstrap.lua`, a setup script, a template and `luadot exec`:

```lua
if ld.class.get("form-factor") == "laptop" then
  ld.rules({ { match = "home/.config/tlp/**", link = "symbolic" } })
end
```

```lua
-- home/.zshrc.luadot/luadot.lua
return ld.alt.file(ld.class.get("form-factor") .. ".zsh")
```

A class nobody answered reads as `nil`, so a fallback is plain Lua —
`ld.class.get("form-factor") or "desktop"`.

The answers live in luadot's state file, next to the repository path, which
makes them per machine and keeps them out of the repository.

```
$ luadot class
form-factor laptop
email       (none)
```

| Command | Effect |
| --- | --- |
| `luadot class` | Lists every declared class with the answer of this machine. |
| `luadot class set` | Asks for every class still unanswered. |
| `luadot class set <name>` | Asks that one again, its current answer being what enter accepts. |
| `luadot class set <name> <value>` | Answers it without asking; a value outside the `choices` is refused. |
| `luadot class unset <name>` | Forgets the answer. |
| `luadot class get <name>` | Prints the answer alone, for a script to read. |

Asking is what `set` does when no value follows the name:

```
$ luadot class set
Is this machine a desktop or a laptop?
  1) desktop
  2) laptop
form-factor [1-2, enter for laptop]: 2
Which email do you use for git?
email: me@example.com
```

A choice is answered by its number or by its own name. Asking needs a terminal:
without one the command says so and names the way out,
`luadot class set <name> <value>`.

`bootstrap` asks for whatever is still unanswered before it runs
`bootstrap.lua`, and so does `clone` when it offers to run it. The declarations
come from `~/.config/luadot/config.lua`, so a machine only reaches the ones its
repository ships after that file is in place.

### The invocation

`ld.argv` carries the invocation the script is running under, so the
configuration can answer differently depending on what was asked:

```lua
if ld.argv.name == "apply" then
  ld.opt.conflict("error")
end
```

`ld.argv.name` is the command as typed and `ld.argv.args` the list of everything
after it, so `luadot apply .config/nvim` gives `"apply"` and
`{ ".config/nvim" }`.

### Running commands

`ld.cmd` runs a command and returns what it printed, in two forms. Called
directly it takes a whole command line and hands it to `sh`, so pipes, globs and
redirection work:

```lua
local branch = ld.cmd("git -C " .. ld.path.repo .. " branch --show-current")
```

Indexed by a program name it runs that program itself, with no shell in the
way, so every argument stays exactly as written even when it holds spaces:

```lua
local branch = ld.cmd.git("-C", ld.path.repo, "branch", "--show-current")
ld.cmd.chsh("-s", "/usr/bin/zsh")
```

Both return the command's standard output with its trailing newline removed,
and both stop the script when the command exits with anything but zero:

```
luadot: `ld.cmd` `chsh -s /usr/bin/zsh` exited with status 1
```

Only standard output is captured; standard error and standard input stay
attached to the terminal, so a command's diagnostics still show up and one
asking for a password still gets it.

When a non-zero status is an answer rather than a failure, the shell form is
where it is handled: `ld.cmd("systemctl is-enabled ufw || true")` returns the
answer instead of stopping.

### Running git

`ld.git` is the same thing for the repository itself: called with the arguments
of a git command, it runs git there, so nothing has to name the path or change
directory first:

```lua
local branch = ld.git("branch", "--show-current")
ld.git("commit", "-m", "apply from " .. ld.sys.host.name)
```

It behaves like `ld.cmd.git` in every other way — every argument stays literal,
standard output comes back without its trailing newline, and a non-zero status
stops the script:

```
luadot: `ld.git` `git commit -m one` exited with status 1
```

The repository is where the command runs, so a call before `luadot clone <url>`
stops instead of running git somewhere else:

```
luadot: `ld.git`: no repository set; run `luadot clone <url>` first
```

### Printing

`ld.print` writes to the terminal the way luadot writes to it. Called on its
own it is one line, and the table beside the text is what it is styled with:

```lua
ld.print("nothing drifted")
ld.print("applied", { tone = "good" })
ld.print("applied", { fg = "#ff8800", bold = true, indent = 2 })
```

| Option | Values | Effect |
| --- | --- | --- |
| `tone` | `"good"`, `"warning"`, `"bad"`, `"strong"`, `"muted"` | The palette luadot's own output uses. |
| `fg`, `bg` | a color name, a number from 0 to 255, `"#ff8800"` | The color of the text and of what sits behind it, over whatever the tone carries. |
| `bold`, `dim`, `italic`, `underline` | `true`, `false` | Adds an attribute, or takes back one the tone carries. |
| `mark` | a string, or a function returning one | What opens the line, one space before the text. |
| `time` | `true`, or a strftime format like `"%H:%M"` | A timestamp opening the line, before the `mark`. |
| `indent` | a whole number | Spaces before everything else. |
| `width` | a whole number | The column the styled part is padded to. |
| `stream` | `"stdout"`, `"stderr"` | Where the line goes. Defaults to `"stdout"`. |
| `newline` | `true`, `false` | Whether the line ends; `false` leaves the cursor where the text stopped. |

The color names are the sixteen ANSI ones — `black`, `red`, … `white`, and
`bright-black` through `bright-white` — and every color is dropped when the
output is not a terminal, so a piped or redirected `luadot` stays plain text.

A `mark` given a function is called every time the line is written, which is
what a clock or a counter of your own needs; `time` is the clock already
written:

```lua
ld.print("applied", { mark = "»" })                        --> » applied
ld.print("applied", { time = "%H:%M" })                    --> 14:32 applied
ld.print("applied", { time = true, mark = "»" })           --> 14:32:07 » applied
ld.print("applied", { mark = function() return "[" .. count .. "]" end })
```

The rest of the family writes the shapes luadot itself uses and takes those
same options:

| Call | Writes |
| --- | --- |
| `ld.print.note(text)` | `luadot: text` |
| `ld.print.warn(text)` | the same line, in yellow, on the error stream |
| `ld.print.error(text)` | the same line, in red, on the error stream |
| `ld.print.section(title)` | a blank line and the title, in bold |
| `ld.print.entry(label, text)` | the label in a column of its own and the text beside it |
| `ld.print.field(name, value)` | the same, for a name and the value it holds |

```lua
ld.print.section("Repository")
ld.print.field("path", ld.path.repo)
ld.print.entry("create", "~/.bashrc", { tone = "good" })
```

It runs in every script luadot runs, and it is what a customized command writes
with.

### Customizing a command

`ld.on` is where a command of luadot's own is told what to say. One call per
command — `ld.on.status`, `ld.on.diff` — and the three pieces every report is
made of are the same in both:

| Key | Takes | Effect |
| --- | --- | --- |
| `entry` | a function, or `false` | Runs for every file the command inspected, in place of the line it would have written. |
| `summary` | a function, a string, or `false` | Replaces the count line closing each side. |
| `render` | a function, or `false` | Runs once, with every one of those files, and takes the whole report over. |

Each one takes a function, and the one that is only a line takes a string too.
Whatever a function returns is written as a line, and a function returning
nothing writes nothing — which is what one printing for itself with `ld.print`
wants:

```lua
ld.on.status({
  entry = function(file)
    ld.print.entry(file.state, file.path, { tone = "warning" })
  end,
  summary = function(counts)
    return counts.synced .. "/" .. counts.total .. " in sync"
  end,
})
```

`false` silences the piece it is given to, so `ld.on.status({ summary = false })`
drops the count lines. A second call replaces only the keys it carries, and the
two commands are customized apart — neither reads the other's call.

`entry` and `render` are handed the same table, one file at a time or every one
of them at once. `path`, `system` and `side` are in both commands' tables:

| Field | Holds |
| --- | --- |
| `path` | The path as the repository writes it: `home/.bashrc`. |
| `system` | The absolute path of the system copy. |
| `side` | `"repository"` for a managed file, `"generated"` for one a template produced. |
| `state` | Where the file stands — the two commands answer it differently, below. |

#### `ld.on.status`

`state` is the word `status` reports with: `"synced"`, `"missing"`,
`"unlinked"`, `"differs"` or `"unreadable"`. Every inspected file reaches
`entry` and `render`, the synced ones included — the built-in report leaves
those out, a customized one decides for itself:

```lua
ld.on.status({
  render = function(files)
    for _, file in ipairs(files) do
      ld.print.entry(file.state, file.path, { tone = "muted" })
    end
  end,
})
```

`summary` is handed `side`, `total` — the files that side reported — `templates`,
the templates behind them on the generated side, `default`, the line it stands
in for, and one count per state: `synced`, `missing`, `unlinked`, `differs` and
`unreadable`.

#### `ld.on.diff`

`state` says what drifted: `"missing"`, `"differs"`, `"mode"` or `"other"`. Only
the drifted files reach `entry` and `render`, and their table carries both sides
of the comparison:

| Field | Holds |
| --- | --- |
| `content` | `source` and `system`, the bytes of both sides; `system` is absent when the file is not there. |
| `mode` | `source` and `system`, as octal strings like `"0644"`. |

`summary` is handed `side`, `drifted`, `total` and `default`, the line it stands
in for. `render` takes the diff over completely — nothing is compared afterwards,
so `ld.on.diff({ render = false })` reports the drifted files without diffing
them at all:

```lua
ld.on.diff({
  render = function(files)
    for _, file in ipairs(files) do
      ld.print.section(file.path)
      ld.print(file.content.source, { fg = "green", newline = false })
      ld.print(file.content.system or "", { fg = "red", newline = false })
    end
  end,
})
```

`diff` carries two keys of its own, for the program the two sides are handed to:

| Key | Takes | Effect |
| --- | --- | --- |
| `tool` | a program, or a list holding it and its arguments | Compares the two sides instead of `git diff`. |
| `args` | a word or a list of them | Extra arguments for whichever program compares them. |

The tool runs inside the same private copy of the two sides, with the two
directories as its last two arguments, and `args` is passed to whichever program
runs — git included, where it lands before the `--`:

```lua
ld.on.diff({ tool = { "difft", "--color", "always" } })
ld.on.diff({ args = { "--stat" } })
```

Exit status `0` or `1` counts as success, the way `git diff` reports a
difference; anything else stops the command.

### Slow calls

`ld.cmd`, `ld.git`, `ld.pkg.install`, `ld.setup` and `ld.setup.all` reach other
programs, the package manager and the setup scripts, which takes seconds to
minutes. `config.lua` runs before every command, so a call to one of them there
makes `status`, `apply`, `add` and the rest pay that cost every single time.
They belong in `bootstrap.lua`, which runs once, and calling them from `config.lua`
prints a warning:

```
luadot: `ld.pkg.install` in config.lua runs before every command and will slow all
of them down; bootstrap.lua is where it belongs (silence it with
`ld.opt.pkg_warn(false)`)
```

A template pays the same cost on every `alt`, so `luadot.lua` is warned too:

```
luadot: `ld.cmd` in luadot.lua runs every time the template is resolved and
will slow `alt` down; bootstrap.lua is where it belongs (silence it with
`ld.opt.pkg_warn(false)`)
```

The same calls in `bootstrap.lua` or in a setup script are silent — those run
once. When the placement is deliberate, `ld.opt.pkg_warn(false)` turns every
warning off, in any script; set it before the call it should cover, since the
warning is emitted as the call runs.

`ld.rules` accumulates, so calling it several times adds to what came before. A
rule needs a `match` pattern and sets `link`, `conflict`, `on_change`, `ignore`,
or any mix of them. When several rules match a file, the last one wins, and whatever it leaves
out falls back to the `ld.opt.link` and `ld.opt.conflict` defaults.

### Splitting the configuration

`~/.config/luadot/lua/` is on the module path, so `require` reaches everything
under it and the configuration can be split like a Neovim one:

```
~/.config/luadot/
├── config.lua
└── lua/
    ├── patterns.lua        -- require("patterns")
    └── editors/
        └── init.lua        -- require("editors")
```

```lua
ld.rules(require("patterns"))
require("editors")
```

`ld` is a global, so a required module calls it directly, returns values for
`config.lua` to pass along, or exposes functions to be called from there.

### Patterns

Patterns are relative to the repository root, so `home/.config/nvim/init.lua`
is the pattern for `~/.config/nvim/init.lua` and `root/etc/pacman.conf` the one
for `/etc/pacman.conf`.

- `*` matches within a single path segment, `**` crosses segments.
- A pattern naming a directory covers everything under it.
- The repository's `.git` directory is always ignored.

## Templates

A path whose name ends in `.luadot` is a template, in one of two forms. A
**directory** holds a `luadot.lua` deciding what ends up on the system, next to
the files that decision picks from or renders. A plain **file** is an embedded
template rendered directly to the mirrored path, with nothing else around it.
`luadot alt` is what runs both: `apply` walks past them instead of mirroring
them, and what the rest of the commands do with a template closes this
section.

```
~/dotfiles/
├── home/
│   ├── .zshrc.luadot/                 -- produces ~/.zshrc
│   │   ├── luadot.lua
│   │   ├── laptop.zsh
│   │   └── desktop.zsh
│   ├── .config/nvim/init.lua.luadot/  -- produces ~/.config/nvim/init.lua
│   │   ├── luadot.lua
│   │   └── init.tmpl.lua
│   ├── .zprofile.luadot               -- a standalone template, produces ~/.zprofile
│   └── .vimrc                         -- a plain managed file
└── root/
    └── etc/motd.luadot                -- produces /etc/motd
```

The destination is the template's own path without the suffix, so the
repository keeps mirroring the system. Inside a directory, a `dest`
of your own overrides it.

`luadot new` creates either form, empty. It takes the path of the file the
template is for, the `.luadot` suffix being added when it is not already
there, and mirrors it into the repository the way `add` does:

```
luadot new ~/.zshrc                  -- ~/dotfiles/home/.zshrc.luadot/luadot.lua
luadot new .config/nvim/init.lua     -- ~/dotfiles/home/.config/nvim/init.lua.luadot/luadot.lua
luadot new -f ~/.zprofile            -- ~/dotfiles/home/.zprofile.luadot, a standalone template
```

A relative path is resolved against the directory you are in, and has to land
inside your home directory. The directory form gets a `luadot.lua` returning
an empty string, the file form is an empty file: both resolve to an empty file
until you write them, and neither replaces anything that is already there.

`luadot.lua` has the `ld` interface, and declares what it produces either by
calling `ld.alt.out` or by returning the same table:

```lua
-- home/.zshrc.luadot/luadot.lua
return {
  content = (ld.sys.host.name == "thinkpad") and ld.alt.file("laptop.zsh")
      or ld.alt.file("desktop.zsh"),
  link = "symbolic",
}
```

```lua
-- home/.config/nvim/init.lua.luadot/luadot.lua
ld.alt.out({ content = ld.alt.render("init.tmpl.lua", { leader = " " }) })
ld.alt.out({ dest = "~/.config/nvim/host.lua", content = "vim.g.host = ' '\n" })
```

| Call | Arguments | Effect |
| --- | --- | --- |
| `ld.alt.out(file)` | a table, a string or an `ld.alt.file` | Declares a file the template produces; repeated calls accumulate. |
| `ld.alt.file(name)` | a path | A real file, linked to the destination like a managed one. |
| `ld.alt.render(name, vars)` | a path and a table | Runs that Lua file with `vars` in scope and returns the string it returns. |
| `ld.alt.expand(name, vars)` | a path and a table | Renders that embedded template with `vars` in scope and returns the string it emits. |
| `ld.alt.read(name)` | a path | What that file holds, as a string, never run. |
| `ld.alt.exists(name)` | a path | Whether that file is there. |
| `ld.alt.glob(pattern)` | a pattern | The names of the files it matches, sorted, named the way `ld.alt.read` takes them. |
| `ld.alt.json(value)` | a table or a scalar | That value as JSON, indented, with sorted keys. |
| `ld.sys` | — | `host`, `gpu`, `ram` and `has_battery()` of the machine. |
| `ld.class.get(name)` | a class name | The answer this machine gave, `nil` when it gave none. |
| `ld.cmd(line)` | a command line | Runs it and returns what it printed; also `ld.cmd.<program>(args...)`. |
| `ld.git(args...)` | the arguments of a git command | Runs git inside the repository and returns what it printed. |
| `ld.argv` | — | `name` and `args` of the command being run. |
| `ld.print(text, options)` | a string and a table of options | Writes a styled line; also `note`, `warn`, `error`, `section`, `entry` and `field`. |
| `ld.path` | — | `home`, `config`, `repo` and `dir`, the template's own directory. |

A declared file carries what it needs and nothing else:

| Key | Values | Effect |
| --- | --- | --- |
| `content` | a string or an `ld.alt.file` | What lands on the system: a string is written, a file is linked. Required. |
| `dest` | a path | Where it lands; `~/` and a relative path both start at your home directory. Defaults to the mirrored path. |
| `link` | `"hard"`, `"symbolic"`, `"copy"` | How an `ld.alt.file` is placed. Defaults to the configured mode. A destination under `/` is always a copy. |
| `conflict` | `"overwrite"`, `"skip"`, `"error"` | Answer when the destination already holds something else. Defaults to the configured policy. |
| `mode` | three or four octal digits, as a string | The permissions of the generated file, `"600"` for one holding a secret. Defaults to what your umask gives. Only for generated content: an `ld.alt.file` is the repository's own copy and keeps its own mode. |
| `on_change` | a command line | Runs through `sh -c` after the file is created or replaced, and only then — an unchanged file runs nothing. Wins over an `on_change` rule matching the same path. |

Returning a string or an `ld.alt.file` is shorthand for a table carrying only
`content`, so a template selecting a variant fits in one line:

```lua
return ld.alt.file("laptop.zsh")
```

The rendered file is a normal Lua script that returns a string, with the
variables of the call in scope and the standard library still reachable:

```lua
-- init.tmpl.lua
return string.format("vim.g.mapleader = %q\n", leader)
```

`ld.alt.file`, `ld.alt.render` and `ld.alt.expand` take a relative path
starting at the template
directory, and an absolute path — or one climbing out with `..` — anywhere else,
so several templates can share a file kept elsewhere in the repository:

```lua
ld.alt.out({ content = ld.alt.file(ld.path.repo .. "/shared/aliases.zsh") })
```

The template's own `lua/` directory is requirable, exactly like the
configuration's.

### Building a file out of fragments

`ld.alt.glob` lists what the template holds and `ld.alt.read` hands back the
bytes, so one file can be assembled from fragments that are each versioned on
their own:

```lua
-- home/.zshrc.luadot/luadot.lua
local parts = {}
for _, name in ipairs(ld.alt.glob("conf.d/*.zsh")) do
  parts[#parts + 1] = ld.alt.read(name)
end

return table.concat(parts, "\n")
```

`*` stays inside one path segment and `**` crosses them, directories are never
listed, and the names come back sorted — `10-env.zsh` before `20-path.zsh` — so
the result never depends on the order the filesystem hands the directory over.
`ld.alt.exists` answers for a single name, which is what a fallback needs:

```lua
local name = ld.alt.exists("laptop.zsh") and "laptop.zsh" or "default.zsh"
return ld.alt.file(name)
```

A generated file that is JSON is built by `ld.alt.json` instead of by
`string.format` and hope:

```lua
return ld.alt.json({
  editor = ld.class.get("editor") or "nvim",
  gpu = ld.sys.gpu.vendor,
})
```

It takes a table of names or a list, never both in the same table, and its keys
come out sorted, so the file changes only when the data does. It is the one
`ld.alt` call that needs no template, so a standalone `.luadot` file has it too.

### Secrets and reloads

A generated file inherits your umask, which is wrong for one holding a secret,
and a daemon reading a file has to be told the file changed:

```lua
ld.alt.out({
  dest = "~/.netrc",
  content = ld.alt.expand("netrc.tmpl", { token = ld.cmd("pass show api") }),
  mode = "600",
})

ld.alt.out({
  dest = "~/.config/mako/config",
  content = ld.alt.read("mako.conf"),
  on_change = "makoctl reload",
})
```

`mode` is compared as well as written, so a file that already holds the right
content with the wrong permissions is reported as differing and put back at
`600`. `on_change` runs only when the file was actually created or replaced —
`alt` over an unchanged file runs nothing — and `--dry-run` prints the command
instead of running it.

The same command belongs in `config.lua` when it is about a path rather than about
one template, and there it covers plain managed files under `apply` too:

```lua
ld.rules({ { match = ".config/mako/**", on_change = "makoctl reload" } })
```

Both forms end up in the same list: every command is deduplicated and runs once,
at the end of the run. A `on_change` declared by the template wins over a rule
matching the same destination.

### Embedded templates

`ld.alt.expand` renders an ERB-style embedded template: text emitted as it
stands, Lua between `<%` and `%>`:

```zsh
export EDITOR=<%= ld.class.get("editor") or "nvim" %>
<% for _, dir in ipairs({ "~/bin", "~/.local/bin" }) do -%>
path+=(<%= dir %>)
<% end -%>
```

| Tag | Effect |
| --- | --- |
| `<% ... %>` | Lua statements, emits nothing |
| `<%_ ... %>` | the same, and strips all whitespace before the tag, newlines included |
| `<%= expr %>` | emits `tostring(expr)`, raw |
| `<%- expr %>` | an alias of `<%=` |
| `<%# ... %>` | a comment, reaches no output |
| `... -%>` | trims the newline after the tag, except on `<%#` |
| `... _%>` | removes all whitespace after the tag, newlines included, except on `<%#` |
| `<%%` | a literal `<%` |
| `%%>` | a literal `%>`, inside a tag |

The tags are EJS's, with EJS's meanings. There is one output tag and it is
always raw — a dotfile is not HTML, so nothing is escaped. Beware the
slurping pair: `<%_` and `_%>` eat newlines too, welding the previous or the
next line onto the tag's own, so an unindented `<%` closed by `-%>` is the
everyday recipe. A comment closes on `%>` alone, so the newline after it
survives and a `<%# ... %>` on a line of its own leaves a blank line behind;
`<% --[[ ... ]] -%>` is the comment that takes its line with it. Errors report
the template's own line, an output tag holding `nil` is an error rather than
the word `nil` in the file, and a `%>` inside a Lua string, comment or long
bracket does not close the tag.

A template reaches the whole interface, `ld.alt.expand` included, so a partial
is an ordinary call — each one keeps its own variables and emits into its own
buffer:

```zsh
<%= ld.alt.expand("header.tmpl.zsh", { title = "zsh" }) -%>
export EDITOR=<%= editor %>
```

### The standalone form

A `.luadot` **file** is an embedded template that needs no directory: `alt`
renders it and places the result at the mirrored path, following the
configured `link` and `conflict` rules. `ld.sys`, `ld.class`, `ld.cmd` and the
rest of the interface are all there; what it does not have is what needs the
directory:

| Missing | Reason |
| --- | --- |
| several outputs | there is no `ld.alt.out` to call |
| `dest`, `link`, `conflict` | no table to carry them; `ld.rules` in `config.lua` still applies |
| `require` of a `lua/` directory | there is no directory |
| `ld.alt.*` | inert, warned; `ld.alt.json` is the exception, it needs no directory |

`ld.path.dir` is `nil` — there is no template directory; `ld.path.repo` still
reaches a file shared elsewhere in the repository.

### The other commands

A template is one thing, not the files it holds, and every command treats it
as one:

| Command | On a template |
| --- | --- |
| `apply` | Walks past it; `alt` is what resolves it. |
| `status -t`, `diff -t` | Resolves it and reports the files it produces; without the flag both say how many templates they left out. |
| `add` | Refuses a file a template already produces, and leaves it out when it walks a directory. |
| `rm` | Takes the whole template out and leaves what it produced on the system. |
| `edit` | Opens the `luadot.lua` of a directory template, the file itself of a standalone one. |

`status --templates` and `diff --templates` resolve the templates the way
`alt --dry-run` does: the `luadot.lua` runs, `ld.cmd` and the rest of the
interface run with it, and nothing is written — what comes out is compared
against what the system holds and then dropped. That is why they take a flag
instead of resolving on their own: a template asking a password store for a
token has no business doing it on every `luadot status`.

`diff --templates` puts the generated file under `generated/`, against the
`system/` side it is compared to, rather than under the `repository/` side a
managed file uses: what a template produces is not in the repository, it is
built again on every run.

`rm`, `edit`, `status` and `diff` reach a template through the path it
produces — `luadot rm ~/.zshrc` takes out `home/.zshrc.luadot/` — and the
template's own path works as well. `rm` backs up every file the template
holds before removing it, and leaves the system alone: a file the template
generated stays where it is, and a symlink pointing into the template becomes
a file of its own so nothing is left dangling.

### Editor support

An editor sees `.luadot`, not `.zsh`, so highlighting needs a nudge. Neovim
already ships the grammar this format parses as —
`tree-sitter-embedded-template`, the `eruby`/`ejs` one:

```lua
vim.filetype.add({ pattern = { [".*%.luadot"] = "luadot" } })
vim.treesitter.language.register("embedded_template", "luadot")
```

plus an `injections.scm` sending `(code)` to `lua` and `(content)` to the
language of the file being generated. A `<%# luadot: zsh %>` comment on the
first line names that language; the renderer ignores it.

## Benchmarks

`cargo bench` measures what every command pays for, over repositories of 16,
128 and 1024 files, so a result reads as a cost per file instead of a single
number.

| Bench | Measures |
| --- | --- |
| `files` | Walking the repository, the status of each file against the system copy, and placing every file as a hard link, as a symlink, or over one that differs. |
| `config` | `link_mode`, `conflict_policy` and `is_ignored` against 4, 32 and 256 rules, plus the path math each managed file goes through. |
| `lua` | Resolving a template, which pays for the Lua runtime, the `ld` interface and the script's own work. |

Comparing two runs is what the numbers are for:

```
cargo bench -- --save-baseline main
cargo bench -- --baseline main
```

The second run reports the change against the saved one. A single area is
reached by target, and a single measurement by name:

```
cargo bench --bench files
cargo bench --bench files -- walk/collect_entries/1024
```
