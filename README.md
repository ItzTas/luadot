# luadot

A dotfiles manager configured in Lua.

## Commands

| Command | Effect |
| --- | --- |
| `luadot clone <url> [dir]` | Clones a dotfiles repository and makes it the managed one. |
| `luadot add <path>...` | Starts managing a file or directory, mirroring it into the repository. |
| `luadot rm [-y] [-n] <path>...` | Stops managing a file or directory, leaving the system copy in place. |
| `luadot status [path]` | Lists the managed files whose system copy is not in sync. |
| `luadot apply [-n] [path]` | Puts the repository's files back on the system. |
| `luadot alt [-n] [path]` | Runs the templates and puts the files they produce on the system. |
| `luadot new [-f] <path>` | Creates an empty template in the repository, for the file that path names. |
| `luadot restore [-l] [-y] [-n] [backup]` | Puts back the files an earlier `apply` or `alt` replaced. |
| `luadot edit <path>` | Opens the repository's copy of a file in `$VISUAL`/`$EDITOR`. |
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
cannot read as `unreadable` and moves on; `apply` reads it through `sudo cat`
before deciding what to do.

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
there. Without it the directory grows on every run and pruning is yours to do.

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

A rule carries six more keys, all optional next to `match` or `regex`:

| Key | Values | Effect |
| --- | --- | --- |
| `link` | `"hard"`, `"symbolic"`, `"copy"` | How the matching files are placed. Files under `root/` are always copies, whatever it says. |
| `conflict` | `"overwrite"`, `"skip"`, `"error"` | Answer when the system copy differs. |
| `on_change` | a command line | Runs after `apply` or `alt` created or replaced one of those files. |
| `ignore` | `true`, `false` | Whether the matching files are left unmanaged. |
| `mode` | three or four octal digits, as a string | The permission bits a matching file under `root/` is placed with. |
| `owner` | `"user"` or `"user:group"` | Who owns a matching file under `root/`. |

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
| `ld.opt.conflict(policy)` | `"overwrite"`, `"skip"`, `"error"` | Default answer when `apply` finds a differing file already on the system. |
| `ld.opt.pkg_warn(enabled)` | `true`, `false` | Whether a call is warned about where it is slow or has no effect. Defaults to `true`. |
| `ld.opt.repo_dir(path)` | a directory | The repository luadot manages, winning over the one `clone` left behind. `~` and a relative path resolve against your home directory. |
| `ld.opt(options)` | a table of options | Sets several options at once; only the keys it carries. |
| `ld.rules(rules)` | a rule or a list of them | Overrides `link` and `conflict` for the files a glob or a regular expression matches, names an `on_change` command for them, sets the `mode` and `owner` of system files, and marks them as never managed. |
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
| `ld.rules`, `ld.opt.link`, `ld.opt.backup`, `ld.opt.backup_dir`, `ld.opt.backup_keep`, `ld.opt.conflict`, `ld.opt.repo_dir`, `ld.class` | `config.lua`, which builds the configuration | does nothing, warns |
| `ld.alt.out`, `ld.alt.file`, `ld.alt.render`, `ld.alt.expand`, `ld.alt.read`, `ld.alt.exists`, `ld.alt.glob` | `luadot.lua`, which produces a template's files | does nothing and yields `nil` (`false` for `ld.alt.exists`), warns |
| `ld.cmd`, `ld.git`, `ld.pkg.install`, `ld.setup`, `ld.setup.all` | everywhere | warns where it is slow |
| `ld.opt.pkg_warn`, `ld.setup.list`, `ld.class.get`, `ld.alt.json`, `ld.argv`, `ld.sys`, `ld.path` | everywhere | — |

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
`luadot alt` is what runs both: `apply` and `status` walk past them instead of
mirroring them.

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
