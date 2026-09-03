# Commands

| Command | Effect |
| --- | --- |
| `luadot init [dir]` | Creates an empty dotfiles repository and makes it the managed one. |
| `luadot clone <url> [dir]` | Clones a dotfiles repository and makes it the managed one. |
| `luadot add [path]...` | Starts managing a file or directory, mirroring it into the repository; with no path, whatever a `track = "auto"` rule covers. |
| `luadot take [path]...` | Stores a managed file or directory as the system holds it, and links it again; with no path, everything the repository holds. |
| `luadot rm [-y] [-n] <path>...` | Stops managing a file or directory, leaving the system copy in place. |
| `luadot mv [-n] <path>... <dest>` | Moves a managed file or directory, in the repository and on the system. |
| `luadot status [-t] [path]` | Lists the managed files whose system copy is not in sync, `-t` the files the templates produce too. |
| `luadot diff [-t] [path]` | Shows what the repository holds and the system does not, `-t` what the templates produce too. |
| `luadot apply [-n] [path]` | Puts the repository's files back on the system. |
| `luadot relink [-n] [path]` | Links the repository's files again, leaving the ones the system changed. |
| `luadot tmpl alt [-n] [path]` | Runs the templates and puts the files they produce on the system. |
| `luadot tmpl new [-f] <path>` | Creates an empty template next to the file that path names, and starts managing it. |
| `luadot restore [-l] [-y] [-n] [backup]` | Puts back the files an earlier `apply` or `tmpl alt` replaced. |
| `luadot edit <path>` | Opens the repository's copy of a file, or the script of the template producing it, in `$VISUAL`/`$EDITOR`. |
| `luadot rekey [-n] [path]` | Re-encrypts the repository's secrets for the recipients set now. |
| `luadot exec <source\|file.lua> [args]...` | Runs Lua with `ld` installed, from a string or a `.lua` file. |
| `luadot config [show\|path\|repo\|edit]` | Shows the resolved configuration, prints its path or the path of the repository, or opens it. |
| `luadot class [list\|set\|unset\|get]` | Lists the declared classes and answers them for this machine. |
| `luadot bootstrap` | Runs the repository's `bootstrap.lua`. |
| `luadot setup [-l] [name]...` | Runs the setup scripts the names ask for, `-l` prints the names instead. |
| `luadot task [name] [args]...`, `luadot task --names` | Runs a task the configuration registers, `luadot <name> [args]...` too. With no name, lists every task with what it does; `--names` prints the names alone. |
| `luadot cd` | Starts a shell in the repository. |
| `luadot sync [-m MSG] [--no-push]` | Stages what changed in the repository, commits it and pushes it. |
| `luadot git <args>...` | Runs git inside the repository. |
| `luadot push [args]...` | Shorthand for `luadot git push`. |
| `luadot doc [call]`, `luadot doc -l` | Describes a call of the `ld` interface; with no call, and with `-l`, names every one. |
| `luadot meta [install [dir]]` | Prints the editor definitions of `ld`; `install` writes them into the data directory and a `.luarc.json` loading them into the configuration directory, or into one directory. |
| `luadot completions <shell>` | Prints a completion script for that shell. |
| `luadot man` | Prints the manual page, the one the packages install. |

`luadot --help` explains any command in place (`luadot rm --help`);
`luadot --version` prints the version.

`config.lua` can run a function before and after any command in the table but
`doc`, `meta`, `completions`, `man` and `task`; see
[customizing a command](ld.md#customizing-a-command).

`luadot doc` answers for the `ld` interface instead: `luadot doc opt.link`
writes what that call takes and does, `luadot doc opt` every call under the
namespace, `luadot doc ld` all of them. On its own it names every call, one per
line, which is what `-l` prints for a script. The `ld.` prefix is optional and a
piece of a name is enough, so `luadot doc backup` finds the four calls carrying
the word. The text is the one on the pages here, built into the binary.

`luadot init` writes a `config.lua` of commented examples when the
configuration directory has none, and `luadot config edit` writes the same one
before opening it. Neither touches a file already there.

`luadot init` and `luadot clone` give lua-language-server the same interface,
for completion and hover text while editing `config.lua` and the scripts;
`luadot meta install` does it for a directory of your own. See
[editor support](ld.md#editor-support).

The bash, zsh and fish completions hand `luadot git` and `luadot push` over to
git's own completion, pointed at the managed repository: `luadot git checkout
<Tab>` answers with the branches of that repository, not of the directory you
are in. `luadot setup <Tab>` answers with the setups the repository declares,
`luadot task <Tab>` with the tasks the configuration registers, `luadot doc
<Tab>` with the calls of the interface.

`man luadot` opens the same reference after any of the packages. The page is
built from the commands themselves; `luadot man` prints it:

```
luadot man >~/.local/share/man/man1/luadot.1
```

## Where the repository lives

`clone` puts the repository in `~/.local/share/luadot/repo` (or
`$XDG_DATA_HOME/luadot/repo`) and remembers the path. A directory of your own
is resolved against the current directory, the way `git clone` does it:

```
luadot clone git@github.com:me/dotfiles.git ~/dotfiles
```

`init` creates an empty git repository in the same default place or in the
directory you name, and remembers it too. The directory has to be empty or
not exist yet. Nothing is committed: `add` stages what it wrote.

```
luadot init ~/dotfiles
luadot add ~/.zshrc
luadot git commit -m "first"
```

`ld.opt.repo_dir` sets the repository from the configuration instead, for a
repository luadot did not clone. It wins over what `clone` remembered, and the
path is read on every command; luadot never moves the directory for you.

```lua
ld.opt.repo_dir("~/dotfiles")
```

## add

`add` takes the paths you name. With none, it takes what the `track = "auto"`
rules cover and the repository does not hold yet:

```lua
ld.rules({
  { match = ".config/nvim/**", track = "auto" },
  { match = ".config/nvim/spell/**", track = "manual" },
})
```

```
$ luadot add
added      .config/nvim/init.lua
added      .config/nvim/lua/plugins.lua
added 2 file(s)
```

luadot looks under the literal part of each pattern, `~/.config/nvim` for the
rule above, so `track = "auto"` needs a `match` opening on a name: a pattern
starting with `*` or a `regex` is refused, since neither says where to look.
A file the repository already holds, one a template produces and one the
repository's ignore rules exclude are all left where they are. Everything the
rules say about a file, from `link` to `encrypt`, holds for the ones taken this
way.

## status

`status` reads like `git status`: one section per state, each naming the
command that resolves it. Files already in sync are left out.

```
$ luadot status
On repository /home/u/dotfiles
12 managed file(s), 2 template(s) not resolved

Files not on the system:
  (use "luadot apply <path>..." to write them)
        missing:     .bashrc

Files not linked:
  (use "luadot relink" to link them again)
        unlinked:    .vimrc

Files that differ:
  (use "luadot diff <path>..." to see what changed)
  (use "luadot apply" to keep the repository's copy, "luadot take" to keep the system's)
        differs:     .zshrc
```

- `missing`: the file is in the repository but not on the system.
- `unlinked`: the contents match, but the system copy is not the link the
  configuration asks for.
- `differs`: the system copy holds something else, or other permission bits
  than the `mode` rule asks for.
- `unreadable`: a secret luadot could not decrypt; `apply` stops with the
  backend's own error.

With nothing left to apply, the sections go away and the line under the header
says so. Every line is replaceable through `ld.on.status`; see
[customizing a command](ld.md#customizing-a-command).

A last line counts the files a `track = "auto"` rule covers and the repository
does not hold yet:

```
1 file(s) an `auto` rule covers, not managed yet; `luadot add` takes them
```

## diff

`diff` shows the content behind a `differs`. The repository is the left side,
the system the right side: what the diff adds is what `apply` would overwrite,
what it removes is what `take` would bring in. A path narrows the report to
that file or everything below that directory. A file the system does not have
shows as a deleted file; one reported `unlinked` holds the same content and
has nothing to show.

```
$ luadot diff
diff --git a/.vimrc b/.vimrc
index 3f8a2b1..7c4d9e0 100644
--- a/.vimrc
+++ b/.vimrc
@@ -1,2 +1,2 @@
 set number
-set ruler
+set paste
luadot: 1 of 12 managed file(s) differ
```

The diff itself is `git diff`, run over a private repository holding the two
sides, so the output is the diff git always prints: the same `a/` and `b/`
paths, your pager and your `diff.*` settings. Binary files are
reported as differing instead of printed. A file whose content matches but
whose mode drifted gets a line of its own:

```
mode       .ssh/config 0644 -> 0600
```

`ld.on.diff` replaces any of it, the compare program included; see
[customizing a command](ld.md#customizing-a-command).

The files templates produce are left out of both reports; the summary says how
many templates were held back. `-t` (or `--templates`) resolves them and
reports what they produce, without writing it. `diff --templates` shows the
generated side under `generated/` rather than `repository/`. The files a
template is made of are managed like any other, so they show up in both
reports without the flag. [templates.md](templates.md) says more.

## relink

`relink` places the repository's files the way `apply` does, but stops at the
ones the system changed. A file whose contents match and whose link broke is
linked again, and a file the system does not have is written. A file that
drifted is left where it is and counted as skipped, so an edit you made on the
system survives the run.

```
$ luadot status
Files not linked:
  (use "luadot relink" to link them again)
        unlinked:    .vimrc
        unlinked:    .zshrc

Files that differ:
  (use "luadot diff <path>..." to see what changed)
  (use "luadot apply" to keep the repository's copy, "luadot take" to keep the system's)
        differs:     .gitconfig

$ luadot relink
replaced   .vimrc
replaced   .zshrc
skipped    .gitconfig
luadot: relinked 12 path(s) (0 created, 2 replaced, 9 unchanged, 1 skipped)
```

`apply` writes the repository's copy over those three files; `relink` writes it
over the first two. A file whose contents match but whose mode drifted counts
as differing, so `relink` leaves that one to `apply` too.

A path narrows the run, and `-n` reports what it would do without writing
anything. What a real run replaces is backed up first.

## take

`take` is `apply` in the other direction: the system copy goes into the
repository and the file is linked again.

A hard link makes both copies the same file, so editing either one edits both.
An editor that writes a new file over the old one instead of writing into it
breaks that link, and neovim, VS Code and `sed -i` all do. From there the two
copies drift, and `apply` would write the system's edit away.

```
$ luadot status
Files that differ:
  (use "luadot diff <path>..." to see what changed)
  (use "luadot apply" to keep the repository's copy, "luadot take" to keep the system's)
        differs:     .zshrc

$ luadot take ~/.zshrc
replaced   .zshrc
luadot: took 1 file(s) (0 added, 1 replaced)
```

A path names a file or a directory, the way `add` does. A directory takes the
files the repository already holds and leaves the rest out, so a new file under
a managed directory still needs `add`. A file the repository does not hold is
refused, and so is one a template produces.

With no path, `take` covers every file the repository holds, whole directories
included, and leaves out the ones the system has no copy of:

```
$ luadot take
replaced   .vimrc
replaced   .zshrc
luadot: took 2 file(s) (0 added, 2 replaced)
luadot: backed up 2 file(s) in ~/.local/share/luadot/backups/1786677956412
```

The repository entries it writes over are saved first, under their own path.
`take <path>` takes no backup.

The rules are read again for what it stores: a file with an `encrypt` rule is
re-encrypted for the recipients set now, one tracked in LFS stays there, and a
`symbolic` rule stores the content and points the system copy back at it. The
repository's copy is replaced through a temporary file beside it, so a run that
fails leaves the copy that was there. Permission bits stay where `apply` puts
them.

## rm

`rm` is the inverse of `add`: it removes the file from the repository and
leaves the system with a plain, unmanaged copy. A symlink into the repository
is replaced by the content it pointed at; a hard link or a plain file is left
alone. Directories that become empty are pruned from the repository.

Removing more than one file asks first, listing what is about to go. `-y` (or
`--yes`) answers upfront; without a terminal, `rm` refuses.

## mv

`mv` renames a managed path on both sides at once: the repository's copy moves,
and the system copy follows it. A symlink into the repository is pointed at the
file where it landed. A hard link, a copy, or a system file that diverged is
moved to the new path as it is, and a file the system does not have is only
moved in the repository. Directories that become empty are pruned.

The last path is where everything goes. One path renames, several need a
directory of the repository to land in:

```
luadot mv ~/.vimrc ~/.config/vim/vimrc
luadot mv ~/.vimrc ~/.gvimrc ~/.config/vim
```

The paths are the ones you use, never the ones the repository stores: a secret
kept as `.netrc.age` is named `~/.netrc`, a template kept as `.zshrc.luadot` is
named `~/.zshrc`. Both keep the form they are in, so a secret lands encrypted
and a template moves whole. The rules are not read again for the destination;
`rm` and `add` are how a file changes the way it is stored.

A destination either side already holds is refused, and so is a directory moved
into itself.

## sync

`add`, `rm` and `mv` stage what they changed. `sync` commits what is staged and
pushes it:

```
$ luadot sync
[main 9f1c2ab] sync from thinkpad
 2 files changed, 31 insertions(+)
```

`-m` writes the message instead of the default `sync from <host>`; `--no-push`
stops after the commit. A run with nothing to commit says so and pushes
whatever is still ahead of the remote. A branch tracking nothing yet is pushed
with `--set-upstream origin HEAD`, so the first `sync` after `init` only needs
a remote:

```
luadot git remote add origin git@github.com:me/dotfiles.git
luadot sync
```

`ld.opt.autocommit` makes `add`, `rm` and `mv` commit on their own, and
`ld.opt.autopush` pushes that commit (it implies the commit, so it stands
alone). Both are rule keys too, so they can be set for one part of the
repository:

```lua
ld.rules({
  { match = ".config/nvim/**", autopush = true },
  { match = ".ssh/**", autocommit = false },
})
```

The rules answer per file, and a run commits as soon as one file it touched
asks for it. `autocommit = false` holds both back; `autopush = false` keeps
the commit and skips the push. A repository with no commit yet is not pushed.
A rule decides whether a file *starts* a commit, not what the commit carries:
git commits the whole index, so a file staged earlier goes in with the next
commit.

## Dry runs

`-n` (or `--dry-run`) makes `apply`, `relink`, `tmpl alt`, `rm` and `mv` report
what they would do and touch nothing: nothing is written and no backup is
taken. Every path is listed, the unchanged ones included, and under each file
`apply` or `tmpl alt` would create, replace or skip comes the diff between what
sits on the system and what the run would put there:

```
$ luadot apply --dry-run
create     .config/mako/config
  @@ -0,0 +1,2 @@
  + font=monospace
  + background-color=#111111
replace    .zshrc
  @@ -1,4 +1,4 @@
  - export EDITOR=vim
  + export EDITOR=nvim
    export PAGER=less
    alias ll="ls -l"
    setopt autocd
unchanged  .gitconfig
luadot: would apply 3 file(s) (1 created, 1 replaced, 1 unchanged, 0 skipped)
```

Three lines of context surround each change. A destination that already holds
the right bytes says `no content change`, so the `replace` on it is about the
mode or the link; one that is not text says `binary content`; a directory
placed whole (`whole = true`) has no diff.

A real run lists the files it changed, and the summary counts the rest:

```
$ luadot apply
created    .config/nvim/init.lua
replaced   .zshrc
luadot: applied 12 file(s) (1 created, 1 replaced, 10 unchanged, 0 skipped)
```

`-u` (or `--unchanged`) puts the files that were already in sync back in that
list:

```
$ luadot apply --unchanged
created    .config/nvim/init.lua
replaced   .zshrc
unchanged  .gitconfig
luadot: applied 3 file(s) (1 created, 1 replaced, 1 unchanged, 0 skipped)
```

Every file a real run replaces is copied aside first; see
[backups.md](backups.md).
