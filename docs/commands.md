# Commands

| Command | Effect |
| --- | --- |
| `luadot init [dir]` | Creates an empty dotfiles repository and makes it the managed one. |
| `luadot clone <url> [dir]` | Clones a dotfiles repository and makes it the managed one. |
| `luadot add <path>...` | Starts managing a file or directory, mirroring it into the repository. |
| `luadot rm [-y] [-n] <path>...` | Stops managing a file or directory, leaving the system copy in place. |
| `luadot status [-t] [path]` | Lists the managed files whose system copy is not in sync, `-t` the files the templates produce too. |
| `luadot diff [-t] [path]` | Shows what the repository holds and the system does not, `-t` what the templates produce too. |
| `luadot apply [-n] [path]` | Puts the repository's files back on the system. |
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
| `luadot task [-l] <name> [args]...` | Runs a task the configuration registers, `-l` prints the names instead; `luadot <name> [args]...` is the same. |
| `luadot cd` | Starts a shell in the repository. |
| `luadot sync [-m MSG] [--no-push]` | Stages what changed in the repository, commits it and pushes it. |
| `luadot git <args>...` | Runs git inside the repository. |
| `luadot push [args]...` | Shorthand for `luadot git push`. |
| `luadot doc <call>`, `luadot doc -l` | Describes a call of the `ld` interface, `-l` names every one. |
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
namespace, `luadot doc ld` all of them. The `ld.` prefix is optional and a piece
of a name is enough, so `luadot doc backup` finds the four calls carrying the
word. The text is the one on the pages here, built into the binary.

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
built from the commands themselves, so `luadot man` prints it wherever the
binary came from:

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
not exist yet. Nothing is committed for you: `add` stages what it wrote, so
the commit is one command away:

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

## status

`status` reads like `git status`: one section per state, each naming the
command that settles it. Files already in sync are left out.

```
$ luadot status
On repository /home/u/dotfiles
12 managed file(s), 2 template(s) not resolved

Files not on the system:
  (use "luadot apply <path>..." to write them)
        missing:     .bashrc

Files not linked:
  (use "luadot apply <path>..." to link them)
        unlinked:    .vimrc

Files that differ:
  (use "luadot diff <path>..." to see what changed)
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

## diff

`diff` shows the content behind a `differs`. The repository is the left side,
the system the right side: what the diff adds is what `apply` would overwrite,
what it removes is what `add` would bring in. A path narrows the report to
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
paths, your pager, your colors, your `diff.*` settings. Binary files are
reported as differing instead of printed. A file whose content matches but
whose mode drifted gets a line of its own:

```
mode       .ssh/config 0644 -> 0600
```

`ld.on.diff` replaces any of it, the compare program included; see
[customizing a command](ld.md#customizing-a-command).

Templates are left out of both reports; the summary says how many were. `-t`
(or `--templates`) resolves them and reports the files they produce, without
writing them. `diff --templates` shows the generated side under `generated/`
rather than `repository/`. [templates.md](templates.md) says more.

## rm

`rm` is the inverse of `add`: it removes the file from the repository and
leaves the system with a plain, unmanaged copy. A symlink into the repository
is replaced by the content it pointed at; a hard link or a plain file is left
alone. Directories that become empty are pruned from the repository.

Removing more than one file asks first, listing what is about to go. `-y` (or
`--yes`) answers upfront; without a terminal, `rm` refuses rather than
assuming an answer.

## sync

`add` and `rm` stage what they changed, so the repository is always one commit
away. `sync` is that commit and the push behind it:

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

`ld.opt.autocommit` makes `add` and `rm` commit on their own, and
`ld.opt.autopush` pushes that commit (it implies the commit, so it stands
alone). Both are rule keys too, so one part of the repository can travel on
its own:

```lua
ld.rules({
  { match = ".config/nvim/**", autopush = true },
  { match = ".ssh/**", autocommit = false },
})
```

The rules answer per file, and a run commits as soon as one file it touched
asks for it. `autocommit = false` holds both back; `autopush = false` keeps
the commit and leaves the pushing to you. A repository with no commit yet is
left alone rather than pushed. A rule decides whether a file
*starts* a commit, not what the commit carries: git commits the whole index,
so a file staged earlier travels with the next commit something else triggers.

## Dry runs

`-n` (or `--dry-run`) makes `apply`, `tmpl alt` and `rm` report what they
would do and touch nothing: nothing is written and no backup is taken. Only
the files that would change are listed. A real run names every file it went through,
unchanged ones included:

```
$ luadot apply --dry-run
create     .config/nvim/init.lua
replace    .zshrc
luadot: would apply 12 file(s) (1 created, 1 replaced, 10 unchanged, 0 skipped)

$ luadot apply
created    .config/nvim/init.lua
replaced   .zshrc
unchanged  .gitconfig
luadot: applied 3 file(s) (1 created, 1 replaced, 1 unchanged, 0 skipped)
```

Every file a real run replaces is copied aside first; see
[backups.md](backups.md).
