---@meta

---How a managed file is placed on the system: a hard link, a symbolic link or a copy.
---@alias ld.LinkMode
---| "hard"
---| "symbolic"
---| "copy"

---What happens when the system copy differs: it is overwritten, the file is skipped, or the run stops.
---@alias ld.Conflict
---| "overwrite"
---| "skip"
---| "error"

---The tool that encrypts and decrypts managed files.
---@alias ld.Backend
---| "age"
---| "gpg"

---Whether an identity names a file or a command.
---@alias ld.IdentityType
---| "command"
---| "file"

---Which side reported a file: the repository, or the templates that generated it.
---@alias ld.Side
---| "repository"
---| "generated"

---Where a drifted file stands, as `diff` reports it.
---@alias ld.DiffState
---| "missing"
---| "differs"
---| "mode"
---| "other"

---Where an inspected file stands, as `status` reports it.
---@alias ld.StatusState
---| "synced"
---| "missing"
---| "unlinked"
---| "differs"
---| "unreadable"

---The palette luadot's own output uses.
---@alias ld.Tone
---| "good"
---| "warning"
---| "bad"
---| "strong"
---| "muted"

---Where a line goes.
---@alias ld.Stream
---| "stdout"
---| "stderr"

---The sixteen ANSI color names. A color is also a number from 0 to 255, or a hex color like `"#ff8800"`.
---@alias ld.Color
---| "black"
---| "red"
---| "green"
---| "yellow"
---| "blue"
---| "magenta"
---| "cyan"
---| "white"
---| "bright-black"
---| "bright-red"
---| "bright-green"
---| "bright-yellow"
---| "bright-blue"
---| "bright-magenta"
---| "bright-cyan"
---| "bright-white"

---A rule: the files it covers through `match` or `regex`, never both, and what it sets for them. A pattern naming a directory covers everything under it.
---@class ld.Rule
---@field match? string|string[] A glob relative to the repository root, or a table of them: `*` matches within a segment, `**` crosses segments.
---@field regex? string|string[] A regular expression in Rust's syntax, or a table of them, matched against the path as written with no anchoring of its own.
---@field link? ld.LinkMode How the matching files are placed.
---@field conflict? ld.Conflict Answer when the system copy differs.
---@field on_change? string A command line that runs after `apply` or `tmpl alt` created or replaced one of those files.
---@field ignore? boolean Whether the matching files are left unmanaged.
---@field mode? string Three or four octal digits, the permission bits a matching file is placed with, and put back when they drift. An encrypted file carries `600` without it.
---@field owner? string `"user"` or `"user:group"`, who owns a matching file once placed, set through `chown`.
---@field encrypt? boolean Whether `add` stores the matching files encrypted.
---@field lfs? boolean Whether the matching files are stored in Git LFS. Needs `match`, since git attributes have no regular expressions, and does not go with `encrypt`. luadot writes the patterns into the repository's `.local/share/luadot/git/attributes`, between the `# luadot:lfs` markers, and copies that file into `.git/info/attributes`.
---@field autocommit? boolean Whether `add` and `rm` commit on their own once one of those files is staged.
---@field autopush? boolean Whether that commit is pushed too. It commits on its own, so `autocommit` comes with it, and `autocommit = false` holds both back.

---A file a template produces, as `ld.alt.out` takes it or `luadot.lua` returns it.
---@class ld.Output
---@field content string|ld.File What lands on the system: a string is written, a file is linked. Required.
---@field dest? string Where it lands; `~/` and a relative path both start at your home directory. Defaults to the mirrored path.
---@field link? ld.LinkMode How an `ld.alt.file` is placed. Defaults to the configured mode.
---@field conflict? ld.Conflict Answer when the destination already holds something else. Defaults to the configured policy.
---@field mode? string Three or four octal digits, the permissions of the generated file, `"600"` for one holding a secret. Only for generated content: an `ld.alt.file` keeps its own mode.
---@field on_change? string A command line run through `sh -c` after the file is created or replaced, and only then. Wins over an `on_change` rule matching the same path.

---A file of the template as `ld.alt.file` hands it over, linked to its destination the way a managed file is.
---@class ld.File

---A class declaration.
---@class ld.Class
---@field name string How the class is read and answered; no spaces. Required.
---@field prompt? string What the machine is asked. Defaults to `define the class <name>`.
---@field choices? string|string[] Restricts the answer to that list; without it the answer is free text.
---@field default? string The answer pressing enter accepts, one of the choices. It only fills the prompt: an unanswered class still reads as `nil`.

---The table form, `ld.crypt({ backend = "gpg" })`: only the keys it carries are set.
---@class ld.CryptOptions
---@field backend? ld.Backend Tool used to encrypt and decrypt managed files. Defaults to `"age"`.
---@field lock? "passphrase"|ld.Keys How secrets are locked: the word locks with a passphrase, the table with keys. Defaults to keys with none set.

---A lock made of keys: who the files are encrypted to, and what decrypts them.
---@class ld.Keys
---@field recipients? string|string[] Public keys, or key ids for gpg, the files are encrypted to.
---@field identity? string|ld.Identity What decrypts with age; gpg uses its keyring. A path resolves `~` and a relative path against your home directory; a command line prints the key instead, and a string carrying a space is read as one.

---An identity spelled out: its words are a path, or a program and its arguments run without a shell, and `type` says which when the guess would be wrong.
---@class ld.Identity
---@field type? ld.IdentityType Says outright whether the words name a file or a command.
---@field [integer] string The path, or the program and its arguments.

---A function to run before the command and one after it. Whatever a function returns is written as a line; a function returning nothing writes nothing.
---@class ld.Around
---@field after? (fun(): string?)|false Runs once the command is done; a command that fails stops before it. Calls add up, in order; `false` drops the functions registered so far.
---@field before? (fun(): string?)|false Runs once `config.lua` ran, before the command does anything. Calls add up, in order; `false` drops the functions registered so far.

---What `diff` prints and which program compares the two sides, and a function to run before and after it. Whatever a function returns is written as a line; a function returning nothing writes nothing.
---@class ld.DiffOptions
---@field after? (fun(): string?)|false Runs once the command is done; a command that fails stops before it. Calls add up, in order; `false` drops the functions registered so far.
---@field args? string|string[] Extra arguments for whichever program compares the two sides; right after `diff` when git runs.
---@field before? (fun(): string?)|false Runs once `config.lua` ran, before the command does anything. Calls add up, in order; `false` drops the functions registered so far.
---@field entry? (fun(file: ld.DiffFile): string?)|false Runs for every drifted file, in place of the line the command would have written. `false` silences the line.
---@field render? (fun(files: ld.DiffFile[]): string?)|false Runs once, with every drifted file, and takes the whole report over; nothing is compared afterwards. `false` reports the files without diffing them.
---@field summary? (fun(counts: ld.DiffCounts): string?)|string|false Replaces the line each side opens with; a string stands as it is, `false` silences it.
---@field tool? string|string[] The program comparing the two sides instead of `git diff`, with its arguments; it gets the two sides as two directories, its last two arguments. Exit status 0 or 1 counts as success.

---What `status` prints, and a function to run before and after it. Whatever a function returns is written as a line; a function returning nothing writes nothing.
---@class ld.StatusOptions
---@field after? (fun(): string?)|false Runs once the command is done; a command that fails stops before it. Calls add up, in order; `false` drops the functions registered so far.
---@field before? (fun(): string?)|false Runs once `config.lua` ran, before the command does anything. Calls add up, in order; `false` drops the functions registered so far.
---@field entry? (fun(file: ld.StatusFile): string?)|false Runs for every inspected file, synced ones included, in place of the line and the sections the command would have written. `false` silences them.
---@field render? (fun(files: ld.StatusFile[]): string?)|false Runs once, with every inspected file, and takes the whole report over.
---@field summary? (fun(counts: ld.StatusCounts): string?)|string|false Replaces the line each side opens with; a string stands as it is, `false` silences it.

---A drifted file, as `diff` hands it to `entry` and `render`.
---@class ld.DiffFile
---@field path string The path as the repository writes it: `.bashrc`.
---@field system string The absolute path of the system copy.
---@field side ld.Side `"repository"` for a managed file, `"generated"` for one a template produced.
---@field state ld.DiffState Where the file stands.
---@field content ld.Content The bytes of both sides.
---@field mode ld.Mode The permission bits of both sides.

---An inspected file, synced or not, as `status` hands it to `entry` and `render`.
---@class ld.StatusFile
---@field path string The path as the repository writes it: `.bashrc`.
---@field system string The absolute path of the system copy.
---@field side ld.Side `"repository"` for a managed file, `"generated"` for one a template produced.
---@field state ld.StatusState Where the file stands.

---The bytes of both sides of a drifted file.
---@class ld.Content
---@field source string The repository's side.
---@field system? string The system's side; absent when the file is not there.

---The permission bits of both sides of a drifted file, as octal strings like `"0644"`.
---@class ld.Mode
---@field source string The repository's side.
---@field system? string The system's side; absent when the file is not there.

---What `diff` hands to `summary`, once per side.
---@class ld.DiffCounts
---@field side ld.Side The side the line opens.
---@field total integer The files that side reported.
---@field default string The line it stands in for.
---@field drifted integer The files that differ.

---What `status` hands to `summary`, once per side.
---@class ld.StatusCounts
---@field side ld.Side The side the line opens.
---@field total integer The files that side reported.
---@field default string The line it stands in for.
---@field templates integer The templates behind the files, on the generated side.
---@field synced integer The files in that state.
---@field missing integer The files in that state.
---@field unlinked integer The files in that state.
---@field differs integer The files in that state.
---@field unreadable integer The files in that state.

---The table form of the options, `ld.opt({ link = "symbolic" })`: only the keys it carries are set.
---@class ld.Options
---@field autocommit? boolean Whether `add` and `rm` commit what they staged. Defaults to `false`.
---@field autopush? boolean Whether that commit is pushed too, committing first. Defaults to `false`.
---@field backup? boolean Whether a file is copied aside before luadot writes over it. Defaults to `true`.
---@field backup_age? string How long a backup is kept, as a span like `"30d"` in `s`, `m`, `h`, `d` or `w`; the ones older than that are dropped. Defaults to keeping them forever.
---@field backup_dir? string Where those copies land. `~` and a relative path resolve against your home directory. Defaults to `~/.local/share/luadot/backups`.
---@field backup_keep? integer How many backups to keep, one or more; the oldest ones are dropped once there are more. Defaults to keeping every one of them.
---@field conflict? ld.Conflict Default answer when `apply` finds a differing file already on the system.
---@field lfs? boolean Whether luadot installs the Git LFS filters and writes the attributes the rules ask for. Defaults to `true`, and has no effect without `git-lfs` on your PATH.
---@field link? ld.LinkMode Default strategy used to link a managed file.
---@field passphrase_warn? boolean Whether passphrase mode says it is weaker than keys. Defaults to `true`.
---@field pkg_warn? boolean Whether a call is warned about where it is slow or has no effect. Defaults to `true`.
---@field repo_dir? string The repository luadot manages, winning over the one `clone` left behind. `~` and a relative path resolve against your home directory.

---The table beside the text, styling the line.
---@class ld.PrintOptions
---@field bg? ld.Color|integer|string The color behind the text, over whatever the tone carries.
---@field bold? boolean Adds the attribute, or takes back one the tone carries.
---@field dim? boolean Adds the attribute, or takes back one the tone carries.
---@field fg? ld.Color|integer|string The color of the text, over whatever the tone carries.
---@field indent? integer Spaces before everything else.
---@field italic? boolean Adds the attribute, or takes back one the tone carries.
---@field mark? string|(fun(): string) What opens the line, one space before the text; a function is called every time the line is written.
---@field newline? boolean Whether the line ends; `false` leaves the cursor where the text stopped.
---@field stream? ld.Stream Where the line goes. Defaults to `"stdout"`.
---@field time? boolean|string A timestamp opening the line, before the `mark`: `true` for `%H:%M:%S`, or a strftime format like `"%H:%M"`.
---@field tone? ld.Tone The palette luadot's own output uses.
---@field underline? boolean Adds the attribute, or takes back one the tone carries.
---@field width? integer The column the styled part is padded to.

---What `ld.setup.all` takes.
---@class ld.SetupOptions
---@field order? string[] The names that run first, in this order; the rest follow.

---One graphics card.
---@class ld.Card
---@field vendor string A short name (`nvidia`, `amd`, `intel`), or the PCI identifier when the vendor is not a known one.
---@field name string The model as `lspci` reports it, empty when `lspci` is not installed.
---@field driver string The kernel driver bound to the card.

---The interface luadot installs in every script it runs: `config.lua`, `bootstrap.lua`, the setup scripts, the templates and `luadot exec`. A call does the same thing wherever it runs, on the one configuration the command is using.
---@class ld
---@field lpeg table The LPeg module, the table `require("lpeg")` returns, loaded when first reached.
---@field re table LPeg's `re` module, the table `require("re")` returns, loaded when first reached.
ld = {}

---Overrides `link` and `conflict` for the files a glob or a regular expression matches, names an `on_change` command for them, sets the `mode` and `owner` they are placed with, marks them as never managed, marks them as encrypted, stores them in Git LFS, and commits and pushes them on their own. A single rule needs no list around it. Calls accumulate, and the last matching rule wins, key by key.
---@param rules ld.Rule|ld.Rule[]
function ld.rules(rules) end

---The files of a template, resolved against the directory the running script lives in: the template directory inside a template, `ld.path.dir` anywhere else. A relative name starts there; an absolute one, or one climbing out with `..`, reaches anywhere.
---@class ld.alt
ld.alt = {}

---Declares a file the template produces; repeated calls accumulate. Outside a template it writes the file where `dest` says, straight away.
---@param file ld.Output|string|ld.File
function ld.alt.out(file) end

---A real file, linked to the destination like a managed one.
---@param name string
---@return ld.File
function ld.alt.file(name) end

---Runs that Lua file with `vars` in scope and returns the string it returns.
---@param name string
---@param vars? table
---@return string
function ld.alt.render(name, vars) end

---Renders that embedded template, text as it stands and Lua between `<%` and `%>`, with `vars` in scope, and returns the string it emits.
---@param name string
---@param vars? table
---@return string
function ld.alt.expand(name, vars) end

---What that file holds, as a string, never run.
---@param name string
---@return string
function ld.alt.read(name) end

---Whether that file is there.
---@param name string
---@return boolean
function ld.alt.exists(name) end

---The names of the files it matches, sorted, named the way `ld.alt.read` takes them; directories are never listed.
---@param pattern string
---@return string[]
function ld.alt.glob(pattern) end

---That value as JSON, indented, with sorted keys. A table is a list or a table of names, never both.
---@param value any
---@return string
function ld.alt.json(value) end

---The invocation: `luadot apply .config/nvim` gives `"apply"` and `{ ".config/nvim" }`.
---@class ld.argv
---@field name string The command as typed.
---@field args string[] Everything after the command.
ld.argv = {}

---The classes of the machine: questions it answers once, read back by every script. The answers live in luadot's state, per machine, out of the repository.
---@class ld.class
---@overload fun(class: ld.Class)
ld.class = {}

---The answer this machine gave, `nil` when it gave none.
---@param name string
---@return string?
function ld.class.get(name) end

---Runs commands and returns their standard output, trailing newline removed. A non-zero exit stops the script; standard error and standard input stay on the terminal. Slow: it belongs in `bootstrap.lua` or a setup script, and warns elsewhere.
---@class ld.cmd
---@field [string] fun(...: string): string Indexed by a program name, runs the program itself with no shell in the way, every argument literal, and returns what it printed: `ld.cmd.git("status")`.
---@overload fun(line: string): string
ld.cmd = {}

---How managed secrets are encrypted. It has an effect in `config.lua` only; elsewhere a call does nothing and says so.
---@class ld.crypt
---@overload fun(options: ld.CryptOptions)
ld.crypt = {}

---Tool used to encrypt and decrypt managed files. Defaults to `"age"`.
---@param name ld.Backend
function ld.crypt.backend(name) end

---How secrets are locked: the word locks with a passphrase, the table with keys. Defaults to keys with none set.
---@param lock "passphrase"|ld.Keys
function ld.crypt.lock(lock) end

---Runs git inside the managed repository: literal arguments, standard output returned, a non-zero status stops the script. A call before a repository is set stops instead of running git somewhere else. Slow: it belongs in `bootstrap.lua` or a setup script, and warns elsewhere.
---@class ld.git
---@overload fun(...: string): string
ld.git = {}

---One call per command, taking a table: functions to run `before` and `after` the command, and what `status` and `diff` print. Every function registered for a moment runs, in the order it was registered; what `status` and `diff` print is replaced by a later call, key by key. Every command is customized apart.
---@class ld.on
ld.on = {}

---Runs a function before and after `add`.
---@param options ld.Around
function ld.on.add(options) end

---Runs a function before and after `apply`.
---@param options ld.Around
function ld.on.apply(options) end

---Runs a function before and after `bootstrap`.
---@param options ld.Around
function ld.on.bootstrap(options) end

---Runs a function before and after `cd`.
---@param options ld.Around
function ld.on.cd(options) end

---Runs a function before and after `class`.
---@param options ld.Around
function ld.on.class(options) end

---Runs a function before and after `clone`.
---@param options ld.Around
function ld.on.clone(options) end

---Runs a function before and after `config`.
---@param options ld.Around
function ld.on.config(options) end

---Says what `diff` prints and which program compares the two sides, and runs a function before and after it.
---@param options ld.DiffOptions
function ld.on.diff(options) end

---Runs a function before and after `edit`.
---@param options ld.Around
function ld.on.edit(options) end

---Runs a function before and after `exec`.
---@param options ld.Around
function ld.on.exec(options) end

---Runs a function before and after `git`.
---@param options ld.Around
function ld.on.git(options) end

---Runs a function before and after `init`.
---@param options ld.Around
function ld.on.init(options) end

---Runs a function before and after `push`.
---@param options ld.Around
function ld.on.push(options) end

---Runs a function before and after `rekey`.
---@param options ld.Around
function ld.on.rekey(options) end

---Runs a function before and after `restore`.
---@param options ld.Around
function ld.on.restore(options) end

---Runs a function before and after `rm`.
---@param options ld.Around
function ld.on.rm(options) end

---Runs a function before and after `setup`.
---@param options ld.Around
function ld.on.setup(options) end

---Says what `status` prints, line by line, and runs a function before and after it.
---@param options ld.StatusOptions
function ld.on.status(options) end

---Runs a function before and after `sync`.
---@param options ld.Around
function ld.on.sync(options) end

---The two `tmpl` actions, customized apart.
---@class ld.on.tmpl
ld.on.tmpl = {}

---Runs a function before and after `tmpl alt`.
---@param options ld.Around
function ld.on.tmpl.alt(options) end

---Runs a function before and after `tmpl new`.
---@param options ld.Around
function ld.on.tmpl.new(options) end

---The options of a run. Each setter takes one value; called with a table, `ld.opt` sets every key the table carries.
---@class ld.opt
---@overload fun(options: ld.Options)
ld.opt = {}

---Whether `add` and `rm` commit what they staged. Defaults to `false`.
---@param enabled boolean
function ld.opt.autocommit(enabled) end

---Whether that commit is pushed too, committing first. Defaults to `false`.
---@param enabled boolean
function ld.opt.autopush(enabled) end

---Whether a file is copied aside before luadot writes over it. Defaults to `true`.
---@param enabled boolean
function ld.opt.backup(enabled) end

---How long a backup is kept, as a span like `"30d"` in `s`, `m`, `h`, `d` or `w`; the ones older than that are dropped. Defaults to keeping them forever.
---@param span string
function ld.opt.backup_age(span) end

---Where those copies land. `~` and a relative path resolve against your home directory. Defaults to `~/.local/share/luadot/backups`.
---@param path string
function ld.opt.backup_dir(path) end

---How many backups to keep, one or more; the oldest ones are dropped once there are more. Defaults to keeping every one of them.
---@param count integer
function ld.opt.backup_keep(count) end

---Default answer when `apply` finds a differing file already on the system.
---@param policy ld.Conflict
function ld.opt.conflict(policy) end

---Whether luadot installs the Git LFS filters and writes the attributes the rules ask for. Defaults to `true`, and has no effect without `git-lfs` on your PATH.
---@param enabled boolean
function ld.opt.lfs(enabled) end

---Default strategy used to link a managed file.
---@param mode ld.LinkMode
function ld.opt.link(mode) end

---Whether passphrase mode says it is weaker than keys. Defaults to `true`.
---@param enabled boolean
function ld.opt.passphrase_warn(enabled) end

---Whether a call is warned about where it is slow or has no effect. Defaults to `true`.
---@param enabled boolean
function ld.opt.pkg_warn(enabled) end

---The repository luadot manages, winning over the one `clone` left behind. `~` and a relative path resolve against your home directory.
---@param path string
function ld.opt.repo_dir(path) end

---The directories of the run.
---@class ld.path
---@field home string Your home directory.
---@field config string The configuration directory, `~/.config/luadot`.
---@field repo? string The managed repository, once one is set. Inside `config.lua` it is the one known before the file ran, so it does not answer for an `ld.opt.repo_dir` set in that same file.
---@field dir? string The directory of the script that is running, the template directory inside a template.
ld.path = {}

---The system package manager: pacman, apt-get or dnf, whichever is on the `PATH`, through `sudo` when it is there.
---@class ld.pkg
ld.pkg = {}

---Installs packages through the system package manager. Slow: it belongs in `bootstrap.lua` or a setup script, and warns elsewhere.
---@param packages string|string[]
function ld.pkg.install(packages) end

---Writes lines the way luadot writes them, styled by the table beside the text. Every color is dropped when the output is not a terminal.
---@class ld.print
---@overload fun(text: string|number, options?: ld.PrintOptions)
ld.print = {}

---The label in a column of its own and the text beside it.
---@param label string|number
---@param text string|number
---@param options? ld.PrintOptions
function ld.print.entry(label, text, options) end

---`luadot: text`, in red, on the error stream.
---@param text string|number
---@param options? ld.PrintOptions
function ld.print.error(text, options) end

---A name in a column of its own and the value it holds beside it.
---@param name string|number
---@param value string|number
---@param options? ld.PrintOptions
function ld.print.field(name, value, options) end

---`luadot: text`.
---@param text string|number
---@param options? ld.PrintOptions
function ld.print.note(text, options) end

---A blank line and the title, in bold.
---@param title string|number
---@param options? ld.PrintOptions
function ld.print.section(title, options) end

---`luadot: text`, in yellow, on the error stream.
---@param text string|number
---@param options? ld.PrintOptions
function ld.print.warn(text, options) end

---Regular expressions in Rust's syntax, the engine the `regex` rule key uses: linear time, no backreferences or lookaround. Lua strings eat one backslash, so `\d` is written `"\\d"`.
---@class ld.regex
ld.regex = {}

---Whether the expression matches anywhere in the text.
---@param text string
---@param pattern string
---@return boolean
function ld.regex.test(text, pattern) end

---The whole match, then each of its groups, `nil` for a group that did not take part; nothing when the expression does not match.
---@param text string
---@param pattern string
---@return string? ...
function ld.regex.match(text, pattern) end

---Where the match starts and where it ends, counted from 1 like `string.find`; nothing when the expression does not match.
---@param text string
---@param pattern string
---@return integer?
---@return integer?
function ld.regex.find(text, pattern) end

---An iterator walking every match, each one yielding the whole match then its groups.
---@param text string
---@param pattern string
---@return fun(): string? ...
function ld.regex.gmatch(text, pattern) end

---The text with the matches rewritten, and how many were. A string carries the groups as `$1` or `${name}`; a function receives what `match` yields and returns the piece to write, or `nil` to leave that match alone.
---@param text string
---@param pattern string
---@param replacement string|(fun(...: string?): string?)
---@param limit? integer
---@return string
---@return integer
function ld.regex.gsub(text, pattern, replacement, limit) end

---The pieces the expression cuts the text into; with a limit, the last piece keeps the rest.
---@param text string
---@param pattern string
---@param limit? integer
---@return string[]
function ld.regex.split(text, pattern, limit) end

---The text as an expression matching itself, every special character quoted.
---@param text string
---@return string
function ld.regex.escape(text) end

---The directories `require` searches besides the configuration's own `lua/`: what a plugin manager registers, carried to every script the command runs.
---@class ld.rtp
ld.rtp = {}

---Puts `<dir>/lua/` on the module path of this script and of every script the command runs after it, behind the configuration's own `lua/` and in the order registered. `~` and a relative path resolve against your home directory; a directory added twice is kept once.
---@param dir string
function ld.rtp.add(dir) end

---The setup scripts of the repository, under `.config/luadot/setup/`: `<name>.lua`, `<name>.sh`, or a `<name>/` directory holding an `init.lua` or an `init.sh`. Running one is slow: it belongs in `bootstrap.lua`, and warns elsewhere.
---@class ld.setup
---@overload fun(name: string)
ld.setup = {}

---The names of the available setup scripts, directories included.
---@return string[]
function ld.setup.list() end

---Runs every setup script, the ones `order` names first.
---@param options? ld.SetupOptions
function ld.setup.all(options) end

---The machine the script is running on.
---@class ld.sys
---@field ram integer The memory of the machine, in bytes, the kernel's raw `MemTotal`: a little under the installed memory, so round it yourself: `math.ceil(ld.sys.ram / 1024 ^ 3)`.
ld.sys = {}

---`true` on a machine with a battery of its own, `false` on one without; the battery of a mouse or a keyboard does not count.
---@return boolean
function ld.sys.has_battery() end

---The first card, and every card as a list: `for _, card in ipairs(ld.sys.gpu)`.
---@class ld.sys.gpu
---@field vendor string A short name (`nvidia`, `amd`, `intel`), or the PCI identifier when the vendor is not a known one.
---@field name string The model as `lspci` reports it, empty when `lspci` is not installed.
---@field driver string The kernel driver bound to the card.
---@field [integer] ld.Card Every card, in order.
ld.sys.gpu = {}

---The host.
---@class ld.sys.host
---@field name string The hostname.
---@field os string The operating system, as Rust names it: `linux`.
---@field arch string The architecture: `x86_64`, `aarch64`.
ld.sys.host = {}
