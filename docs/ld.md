# The ld interface

`~/.config/luadot/config.lua` (or `$XDG_CONFIG_HOME/luadot/config.lua`) runs
before every command and configures luadot through the global `ld`. Without
the file, the defaults apply.

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

It is a normal Lua script: conditionals per machine, loops building pattern
lists, local helper functions. Options also take a table form, which sets only
the keys it carries: `ld.opt({ link = "symbolic" })`. The full list of calls
is [at the end of this page](#every-call).

## Rules

A rule names the files it covers through `match`, a glob, or through `regex`,
a regular expression, never both. A single rule needs no list around it.

```lua
ld.rules({
  { regex = "^home/\\.config/(nvim|zsh)/", link = "symbolic" },
  { regex = "\\.sw[po]$", ignore = true },
})
```

The expression is [Rust's regex syntax][regex], matched against the path as
written, with `/` as the separator and no anchoring of its own: `nvim` covers
every path carrying that word, `^home/\.ssh/` only what sits under `~/.ssh/`.
Lua escapes a backslash as `\\`, so a literal dot is `"\\."`. There are no
backreferences and no lookaround, which keeps every match linear in the length
of the path.

[regex]: https://docs.rs/regex/latest/regex/#syntax

Either key also takes a table of patterns, and the rule covers whatever any of
them matches:

```lua
ld.rules({
  { match = { "**/*.tmp", "home/.cache/**" }, ignore = true },
  { regex = { "^home/\\.local/state/", "\\.sw[po]$" }, ignore = true },
})
```

The other nine keys, all optional:

| Key | Values | Effect |
| --- | --- | --- |
| `link` | `"hard"`, `"symbolic"`, `"copy"` | How the matching files are placed. Files under `root/` are always copies, whatever it says. |
| `conflict` | `"overwrite"`, `"skip"`, `"error"` | Answer when the system copy differs. |
| `on_change` | a command line | Runs after `apply` or `tmpl alt` created or replaced one of those files. |
| `ignore` | `true`, `false` | Whether the matching files are left unmanaged. |
| `mode` | three or four octal digits, as a string | The permission bits a matching file under `root/` is placed with. An encrypted file carries `600` without it. |
| `owner` | `"user"` or `"user:group"` | Who owns a matching file under `root/`. |
| `encrypt` | `true`, `false` | Whether `add` stores the matching files encrypted. |
| `autocommit` | `true`, `false` | Whether `add` and `rm` commit on their own once one of those files is staged. |
| `autopush` | `true`, `false` | Whether that commit is pushed too. It commits on its own, so `autocommit` comes with it, and `autocommit = false` holds both back. |

Any other key is refused, so a typo does not pass as a rule that sets nothing.

Either syntax also matches a directory on behalf of everything under it:
`{ match = "home/.ssh" }` and `{ regex = "^home/\\.ssh$" }` both cover
`home/.ssh/keys/id_ed25519`.

`ld.rules` accumulates across calls. The last matching rule wins, key by key:
`{ match = "home/.cache/**", ignore = true }` followed by
`{ match = "home/.cache/keep/**", ignore = false }` ignores everything under
`~/.cache/` but that one directory. Keys no rule sets fall back to the
`ld.opt.link` and `ld.opt.conflict` defaults.

`on_change` commands are deduplicated and run once per run, at the end: twenty
changed files under `.config/mako/` reload mako once. A failing command stops
the run after the files are in place; `--dry-run` prints the command instead
of running it.

### Patterns

Patterns are relative to the repository root: `home/.config/nvim/init.lua` is
the pattern for `~/.config/nvim/init.lua`, `root/etc/pacman.conf` the one for
`/etc/pacman.conf`.

- `*` matches within a single path segment, `**` crosses segments.
- A pattern naming a directory covers everything under it.
- The repository's `.git` directory is always ignored.

## One interface everywhere

Every script luadot runs carries the same `ld`: `config.lua`, `bootstrap.lua`,
the setup scripts, a template's `luadot.lua` and `luadot exec`. A call does the
same thing wherever it runs, on the one configuration the command is using, so
a setup script that `config.lua` invokes and a template that `tmpl alt`
resolves both write into it.

| Call | Where it has an effect | Elsewhere |
| --- | --- | --- |
| `ld.crypt.backend`, `ld.crypt.lock` | `config.lua`, which builds the configuration | does nothing, warns |
| `ld.cmd`, `ld.git`, `ld.pkg.install`, `ld.setup`, `ld.setup.all` | everywhere | warns where it is slow |
| `ld.alt.out` | everywhere | warns in `config.lua`, where it writes its file before every command |
| everything else | everywhere | |

`ld.crypt` is the one exception. luadot reads the lock once, before it touches
any file, so a script cannot change it halfway through a run. A crypt call
elsewhere is not an error; it runs, does nothing and says so:

```
luadot: `ld.crypt.lock` in a setup script does nothing; config.lua is where it
has an effect (silence it with `ld.opt.pkg_warn(false)`)
```

The order the scripts run in decides what wins. `config.lua` runs first, then
the command, and `tmpl alt` resolves one template at a time while it is already
placing files, so an option the second template sets never reaches what the
first one produced. The same goes for what a run reads once at the start:
`ld.opt.repo_dir` picks the repository before anything else runs, and the
backup options are read when the run opens its backup directory.

`ld.alt` resolves its files against the directory the running script lives in,
which is the template directory inside a template and `ld.path.dir` anywhere
else. Outside a template `ld.alt.out` writes its file where `dest` says and
applies `mode`, `conflict` and `on_change`; it takes no backup, and a
`--dry-run` writes nothing.

Declaring a class in `config.lua` collects it for `bootstrap`, `clone` and
`class` to ask about later. Declaring one anywhere else asks straight away when
the machine has no answer yet and writes the answer to the state, so
`ld.class.get` reads it for the rest of the run.

`ld.path` carries `home` and `config` everywhere, `repo` once a repository is
set, and `dir`, the directory of the script that is running. Inside `config.lua`
itself, `ld.path.repo` is the repository known before the file ran, so it does
not answer for an `ld.opt.repo_dir` set in that same file; every script that
runs afterwards gets the resolved one.

## The machine

`ld.sys` describes the machine the script is running on:

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

`ld.sys.gpu.vendor` is a short name (`nvidia`, `amd`, `intel`), or the PCI
identifier when the vendor is not a known one. `ld.sys.gpu.name` is the model
as `lspci` reports it, empty when `lspci` is not installed. A machine with
several cards carries each of them:

```lua
for _, card in ipairs(ld.sys.gpu) do
  print(card.vendor, card.name, card.driver)
end
```

`ld.sys.has_battery()` tells a laptop from a desktop and ignores the battery
of a mouse or a keyboard. `ld.sys.ram` is the kernel's raw `MemTotal`, a
little under the installed memory (the firmware keeps a slice); rounding is up
to the configuration: `math.ceil(ld.sys.ram / 1024 ^ 3)`.

## Classes

A class is a question the machine answers once: which of two setups this one
is, what mail address its git commits carry. The configuration declares it,
`luadot class` answers it, every script reads the answer back.

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

`ld.class.get` reads the answer, the same in every script. A class nobody
answered reads as `nil`, so a fallback is plain Lua:
`ld.class.get("form-factor") or "desktop"`.

```lua
if ld.class.get("form-factor") == "laptop" then
  ld.rules({ { match = "home/.config/tlp/**", link = "symbolic" } })
end
```

The answers live in luadot's state file, next to the repository path: per
machine, out of the repository.

| Command | Effect |
| --- | --- |
| `luadot class` | Lists every declared class with the answer of this machine. |
| `luadot class set` | Asks for every class still unanswered. |
| `luadot class set <name>` | Asks that one again, its current answer being what enter accepts. |
| `luadot class set <name> <value>` | Answers it without asking; a value outside the `choices` is refused. |
| `luadot class unset <name>` | Forgets the answer. |
| `luadot class get <name>` | Prints the answer alone, for a script to read. |

```
$ luadot class set
Is this machine a desktop or a laptop?
  1) desktop
  2) laptop
form-factor [1-2, enter for laptop]: 2
Which email do you use for git?
email: me@example.com
```

A choice is answered by its number or by its own name. Asking needs a
terminal; without one the command says so and names the way out,
`luadot class set <name> <value>`.

`bootstrap` asks for whatever is still unanswered before it runs
`bootstrap.lua`, and so does `clone` when it offers to run it. The
declarations come from `~/.config/luadot/config.lua`, so a machine only
reaches the ones its repository ships after that file is in place.

## The invocation

`ld.argv.name` is the command as typed, `ld.argv.args` the list of everything
after it: `luadot apply .config/nvim` gives `"apply"` and `{ ".config/nvim" }`.

```lua
if ld.argv.name == "apply" then
  ld.opt.conflict("error")
end
```

## Running commands

`ld.cmd` runs a command and returns its standard output, trailing newline
removed. Called directly it hands the whole line to `sh`, so pipes, globs and
redirection work. Indexed by a program name it runs the program itself with
no shell in the way, so every argument stays literal:

```lua
local branch = ld.cmd("git -C " .. ld.path.repo .. " branch --show-current")
local branch = ld.cmd.git("-C", ld.path.repo, "branch", "--show-current")
ld.cmd.chsh("-s", "/usr/bin/zsh")
```

A non-zero exit stops the script:

```
luadot: `ld.cmd` `chsh -s /usr/bin/zsh` exited with status 1
```

Only standard output is captured; standard error and standard input stay on
the terminal, so diagnostics show up and a password prompt still works. When a
non-zero status is an answer rather than a failure, handle it in the shell
form: `ld.cmd("systemctl is-enabled ufw || true")`.

## Running git

`ld.git` runs git inside the managed repository, no path or directory change
needed. It behaves like `ld.cmd.git` otherwise: literal arguments, standard
output returned, non-zero status stops the script.

```lua
local branch = ld.git("branch", "--show-current")
ld.git("commit", "-m", "apply from " .. ld.sys.host.name)
```

A call before a repository is set stops instead of running git somewhere else:

```
luadot: `ld.git`: no repository set; run `luadot clone <url>` first
```

## Printing

`ld.print` writes a line the way luadot writes, styled by the table beside the
text:

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

The color names are the sixteen ANSI ones (`black` through `white`,
`bright-black` through `bright-white`). Every color is dropped when the output
is not a terminal, so a piped `luadot` stays plain text.

A `mark` given a function is called every time the line is written; `time` is
the clock already written:

```lua
ld.print("applied", { mark = "»" })                        --> » applied
ld.print("applied", { time = "%H:%M" })                    --> 14:32 applied
ld.print("applied", { time = true, mark = "»" })           --> 14:32:07 » applied
ld.print("applied", { mark = function() return "[" .. count .. "]" end })
```

The rest of the family writes the shapes luadot itself uses, same options:

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

## Customizing a command

`ld.on` replaces what a command prints. One call per command, `ld.on.status`
and `ld.on.diff`, both made of the same three pieces:

| Key | Takes | Effect |
| --- | --- | --- |
| `entry` | a function, or `false` | Runs for every file the command inspected, in place of the line it would have written and, in `status`, in place of the sections grouping them. |
| `summary` | a function, a string, or `false` | Replaces the line each side opens with. |
| `render` | a function, or `false` | Runs once, with every one of those files, and takes the whole report over. |

Whatever a function returns is written as a line; a function returning nothing
writes nothing, which suits one printing for itself with `ld.print`:

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

`false` silences its piece: `ld.on.status({ summary = false })` leaves each
side with its header alone. A second call replaces only the keys it carries,
and the two commands are customized apart.

`entry` and `render` are handed the same table, one file at a time or every
one at once:

| Field | Holds |
| --- | --- |
| `path` | The path as the repository writes it: `home/.bashrc`. |
| `system` | The absolute path of the system copy. |
| `side` | `"repository"` for a managed file, `"generated"` for one a template produced. |
| `state` | Where the file stands; the two commands answer it differently, below. |

### ld.on.status

`state` is `"synced"`, `"missing"`, `"unlinked"`, `"differs"` or
`"unreadable"`. Every inspected file reaches `entry` and `render`, synced ones
included: the built-in report leaves those out and groups the rest into
sections, a customized one gets a flat list and decides for itself.

```lua
ld.on.status({
  render = function(files)
    for _, file in ipairs(files) do
      ld.print.entry(file.state, file.path, { tone = "muted" })
    end
  end,
})
```

`summary` is handed `side`; `total`, the files that side reported;
`templates`, the templates behind them on the generated side; `default`, the
line it stands in for; and one count per state (`synced`, `missing`,
`unlinked`, `differs`, `unreadable`).

### ld.on.diff

`state` is `"missing"`, `"differs"`, `"mode"` or `"other"`. Only the drifted
files reach `entry` and `render`, and their table carries both sides:

| Field | Holds |
| --- | --- |
| `content` | `source` and `system`, the bytes of both sides; `system` is absent when the file is not there. |
| `mode` | `source` and `system`, as octal strings like `"0644"`. |

`summary` is handed `side`, `drifted`, `total` and `default`. `render` takes
the diff over completely; nothing is compared afterwards, so
`ld.on.diff({ render = false })` reports the drifted files without diffing
them at all.

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

`diff` carries two keys of its own, for the program the two sides go to:

| Key | Takes | Effect |
| --- | --- | --- |
| `tool` | a program, or a list holding it and its arguments | Compares the two sides instead of `git diff`. |
| `args` | a word or a list of them | Extra arguments for whichever program compares them. |

A tool of its own gets a private copy of the two sides as two directories, its
last two arguments. `args` lands right after `diff` when git runs:

```lua
ld.on.diff({ tool = { "difft", "--color", "always" } })
ld.on.diff({ args = { "--stat" } })
```

Exit status `0` or `1` counts as success, the way `git diff` reports a
difference; anything else stops the command.

## Regular expressions

Lua patterns have no alternation, no quantifier on a group, no named capture.
`ld.regex` is the same engine the `regex` rule key uses, available in every
script:

```lua
local _, version = ld.regex.match(ld.cmd("nvim --version"), "NVIM v([\\d.]+)")

local trimmed = ld.regex.gsub(text, "\\s+$", "")

for _, name, value in ld.regex.gmatch(env, "(\\w+)=(\\S+)") do
  ld.print.field(name, value)
end
```

Every call yields the whole match first, then each group. `find` yields the
two positions instead. `gsub` returns the new text and the number of matches
rewritten; a replacement string carries the groups as `$1` or `${name}`, and a
replacement function receives what `match` yields, returning the piece to
write, or `nil` to leave that match alone:

```lua
local bumped = ld.regex.gsub("nvim 0.11.2", "(\\d+)\\.(\\d+)", function(_, major, minor)
  return major .. "." .. (tonumber(minor) + 1)
end)
```

`ld.regex.split` cuts a text on every match; `ld.regex.escape` turns a literal
into an expression matching itself. The syntax is documented at
<https://docs.rs/regex/latest/regex/#syntax>: leading `\\` because Lua strings
eat one of them, linear time on any input, no backreferences or lookaround as
the price.

## Parsing with LPeg

LPeg 1.1.0 is compiled into the binary, so `require("lpeg")` and its `re`
companion work everywhere the configuration runs, nothing to install:

```lua
local lpeg = require("lpeg")

local digit = lpeg.R("09")
local number = digit ^ 1 / tonumber
local version = lpeg.Ct(number * ("." * number) ^ 0)

local parts = version:match("0.11.2")
```

```lua
local re = require("re")

local name = re.match("neovim@0.11.2", "{%a+}")
```

`ld.lpeg` and `ld.re` are the same two modules for a file without the
`require` lines. Neither is loaded until first reached, so a configuration
that never mentions them pays nothing. Both are the upstream modules,
documented at <https://www.inf.puc-rio.br/~roberto/lpeg/> and
<https://www.inf.puc-rio.br/~roberto/lpeg/re.html>.

## Slow calls

`ld.cmd`, `ld.git`, `ld.pkg.install`, `ld.setup` and `ld.setup.all` reach
other programs, which takes seconds to minutes. `config.lua` runs before every
command, so a call to one of them there slows every command down. They belong
in `bootstrap.lua`, which runs once; elsewhere a warning says so:

```
luadot: `ld.pkg.install` in config.lua runs before every command and will slow all
of them down; bootstrap.lua is where it belongs (silence it with
`ld.opt.pkg_warn(false)`)
```

A template pays the same cost on every `tmpl alt`, so `luadot.lua` is warned
too. The same calls in `bootstrap.lua` or a setup script are silent: those run
once. `ld.opt.pkg_warn(false)` turns every warning off, in any script; set it
before the call it should cover.

## Splitting the configuration

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
`config.lua` to pass along, or exposes functions. The same directory is on the
module path of every template, so a helper written once is reached from both.

## exec

`luadot exec` runs Lua with the same `ld` interface, from a string or a
`.lua` file:

```
luadot exec 'print(ld.sys.gpu.name)'
luadot exec ~/scripts/report.lua --json
```

The argument is a `.lua` path when it names an existing file or ends in
`.lua`, and Lua source otherwise. Everything after it reaches the script
through `ld.argv.args`. A file requires modules from the `lua/` directory next
to it, a source string from the one in your configuration directory. Neither
runs `config.lua` first: `exec` is a scratchpad, not a command that configures
anything.

## Editor support

`luadot meta install` writes `meta/ld.lua`, lua-language-server definitions of
every call on this page, and a `.luarc.json` that loads them, into
`~/.config/luadot/` and into the repository. Completion and hover text then
work in `config.lua`, `bootstrap.lua`, the setup scripts and every
`luadot.lua`, whichever of the two the editor has open.

```
luadot meta install
luadot meta install ~/scripts
```

A directory of your own takes the place of both. An existing `.luarc.json`
keeps its keys: `workspace.library` and `runtime.path` gain the entries they
lack, `runtime.version` becomes `Lua 5.4`. One that does not parse is left
alone, and the settings are printed for you to merge by hand. The definitions
come out of the binary, so run the command again after an upgrade; `luadot
meta` prints them alone.

`ld.cmd.<program>` completes for any name, since the program is only known at
the call.

## Every call

`luadot doc opt.link` writes the row of a call in the terminal, `luadot doc
opt` every row of a namespace, `luadot doc --list` the names alone.

| Call | Arguments | Effect |
| --- | --- | --- |
| `ld.opt.link(mode)` | `"hard"`, `"symbolic"`, `"copy"` | Default strategy used to link a managed file. |
| `ld.opt.backup(enabled)` | `true`, `false` | Whether a file is copied aside before luadot writes over it. Defaults to `true`. |
| `ld.opt.backup_dir(path)` | a directory | Where those copies land. `~` and a relative path resolve against your home directory. Defaults to `~/.local/share/luadot/backups`. |
| `ld.opt.backup_keep(count)` | a number of one or more | How many backups to keep; the oldest ones are dropped once there are more. Defaults to keeping every one of them. |
| `ld.opt.backup_age(span)` | a span like `"30d"`, in `s`, `m`, `h`, `d` or `w` | How long a backup is kept; the ones older than that are dropped. Defaults to keeping them forever. |
| `ld.opt.conflict(policy)` | `"overwrite"`, `"skip"`, `"error"` | Default answer when `apply` finds a differing file already on the system. |
| `ld.opt.pkg_warn(enabled)` | `true`, `false` | Whether a call is warned about where it is slow or has no effect. Defaults to `true`. |
| `ld.opt.autocommit(enabled)` | `true`, `false` | Whether `add` and `rm` commit what they staged. Defaults to `false`. |
| `ld.opt.autopush(enabled)` | `true`, `false` | Whether that commit is pushed too, committing first. Defaults to `false`. |
| `ld.opt.repo_dir(path)` | a directory | The repository luadot manages, winning over the one `clone` left behind. `~` and a relative path resolve against your home directory. |
| `ld.opt(options)` | a table of options | Sets several options at once; only the keys it carries. |
| `ld.crypt.backend(name)` | `"age"`, `"gpg"` | Tool used to encrypt and decrypt managed files. Defaults to `"age"`. |
| `ld.crypt.lock(lock)` | `"passphrase"`, or a table of `recipients` and `identity` | How secrets are locked: the word locks with a passphrase, the table with keys. The `identity` takes a path or a command. |
| `ld.opt.passphrase_warn(enabled)` | `true`, `false` | Whether passphrase mode says it is weaker than keys. Defaults to `true`. |
| `ld.crypt(options)` | a table of options | Sets several crypt options at once; only the keys it carries. |
| `ld.rules(rules)` | a rule or a list of them | Overrides `link` and `conflict` for the files a glob or a regular expression matches, names an `on_change` command for them, sets the `mode` and `owner` of system files, marks them as never managed, marks them as encrypted, and commits and pushes them on their own. |
| `ld.class(class)` | a table declaring a class | Declares a question this machine answers once. In `config.lua` it waits for `bootstrap`, `clone` or `luadot class` to ask; anywhere else it asks straight away and saves the answer. |
| `ld.class.get(name)` | a class name | The answer this machine gave, `nil` when it gave none. |
| `ld.pkg.install(packages)` | a package name or a list of them | Installs packages through the system package manager. |
| `ld.setup(name)` | a setup name | Runs one setup script: `<name>.lua`, `<name>.sh`, or a `<name>/` directory holding an `init.lua` or an `init.sh`. |
| `ld.setup.all(options)` | a table with an optional `order` list | Runs every setup script. |
| `ld.setup.list()` | none | The names of the available setup scripts, directories included. |
| `ld.cmd(line)` | a command line | Runs it through `sh` and returns what it printed. |
| `ld.cmd.<program>(args...)` | the arguments of that program | Runs the program itself and returns what it printed. |
| `ld.git(args...)` | the arguments of a git command | Runs git inside the repository and returns what it printed. |
| `ld.argv` | none | `name` and `args` of the command being run. |
| `ld.sys` | none | `host`, `gpu`, `ram` and `has_battery()` of the machine. |
| `ld.path` | none | `home`, `config`, `repo` and `dir`, where they exist. |
| `ld.on.status(options)` | a table of options | Says what `status` prints, line by line. |
| `ld.on.diff(options)` | a table of options | Says what `diff` prints and which program compares the two sides. |
| `ld.print(text, options)` | a string and a table of options | Writes a line to the terminal, styled the way the options ask. |
| `ld.print.note(text)`, `ld.print.warn(text)`, `ld.print.error(text)` | a string and a table of options | The same line carrying `luadot:`; the last two on the error stream. |
| `ld.print.section(title)`, `ld.print.entry(label, text)`, `ld.print.field(name, value)` | strings and a table of options | A title over a blank line, a labelled line, a named value. |
| `ld.regex.test(text, pattern)` | a text and an expression | Whether the expression matches anywhere in the text. |
| `ld.regex.match(text, pattern)` | a text and an expression | The whole match, then each of its groups; nothing when the expression does not match. |
| `ld.regex.find(text, pattern)` | a text and an expression | Where the match starts and where it ends; nothing when the expression does not match. |
| `ld.regex.gmatch(text, pattern)` | a text and an expression | An iterator walking every match, each one yielding the whole match then its groups. |
| `ld.regex.gsub(text, pattern, replacement, limit)` | a text, an expression, a string or a function, and an optional count | The text with the matches rewritten and how many were; `$1` and `${name}` carry the groups. |
| `ld.regex.split(text, pattern, limit)` | a text, an expression and an optional count | The pieces the expression cuts the text into. |
| `ld.regex.escape(text)` | a string | The text as an expression matching itself, every special character quoted. |

The template calls (`ld.alt.out`, `ld.alt.file`, `ld.alt.render`,
`ld.alt.expand`, `ld.alt.read`, `ld.alt.exists`, `ld.alt.glob`, `ld.alt.json`)
are documented in [templates.md](templates.md); the crypt calls in detail in
[secrets.md](secrets.md).
