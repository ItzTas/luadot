# Templates

A path whose name ends in `.luadot` is a template, in one of two forms. A
**directory** holds a `luadot.lua` deciding what ends up on the system, next
to the files that decision picks from or renders. A plain **file** is an
embedded template rendered directly to the mirrored path. `luadot tmpl alt`
resolves both; `apply` walks past them.

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
repository keeps mirroring the system. Inside a directory, a `dest` of your
own overrides it.

## tmpl new

`luadot tmpl new` creates either form, empty, next to the file the path names
(the `.luadot` suffix is added when missing), and adds it to the repository
the way `add` does:

```
luadot tmpl new ~/.zshrc               -- ~/.zshrc.luadot/luadot.lua
luadot tmpl new .config/nvim/init.lua  -- ~/.config/nvim/init.lua.luadot/luadot.lua
luadot tmpl new -f ~/.zprofile         -- ~/.zprofile.luadot, a standalone template
```

The template stays where it was written and the repository links it, so
`~/.zshrc.luadot/luadot.lua` and `~/dotfiles/home/.zshrc.luadot/luadot.lua`
are the same file. Files you put in a directory template afterwards are yours
to `add`. A relative path resolves against the current directory and has to
land inside your home directory. Both forms resolve to an empty file until you
write them, and neither replaces anything already there.

## The resolver

`luadot.lua` declares what the template produces, by calling `ld.alt.out` or
by returning the same table:

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

The rest of the [ld interface](ld.md) is there too: `ld.sys`, `ld.class.get`,
`ld.cmd`, `ld.git`, `ld.argv`, `ld.print`, `ld.path` (whose `dir` is the
template's own directory).

A declared file carries what it needs and nothing else:

| Key | Values | Effect |
| --- | --- | --- |
| `content` | a string or an `ld.alt.file` | What lands on the system: a string is written, a file is linked. Required. |
| `dest` | a path | Where it lands; `~/` and a relative path both start at your home directory. Defaults to the mirrored path. |
| `link` | `"hard"`, `"symbolic"`, `"copy"` | How an `ld.alt.file` is placed. Defaults to the configured mode. A destination under `/` is always a copy. |
| `conflict` | `"overwrite"`, `"skip"`, `"error"` | Answer when the destination already holds something else. Defaults to the configured policy. |
| `mode` | three or four octal digits, as a string | The permissions of the generated file, `"600"` for one holding a secret. Defaults to what your umask gives. Only for generated content: an `ld.alt.file` keeps its own mode. |
| `on_change` | a command line | Runs through `sh -c` after the file is created or replaced, and only then. Wins over an `on_change` rule matching the same path. |

Returning a string or an `ld.alt.file` is shorthand for a table carrying only
`content`:

```lua
return ld.alt.file("laptop.zsh")
```

A rendered file is a normal Lua script that returns a string, with the
variables of the call in scope:

```lua
-- init.tmpl.lua
return string.format("vim.g.mapleader = %q\n", leader)
```

`ld.alt.file`, `ld.alt.render` and `ld.alt.expand` take a relative path
starting at the template directory, and an absolute path (or one climbing out
with `..`) anywhere else, so templates can share a file kept elsewhere in the
repository:

```lua
ld.alt.out({ content = ld.alt.file(ld.path.repo .. "/shared/aliases.zsh") })
```

The template's own `lua/` directory is requirable, and so is
`~/.config/luadot/lua/`. A name defined in both is taken from the template.

## Building a file out of fragments

`ld.alt.glob` lists what the template holds, `ld.alt.read` hands back the
bytes:

```lua
-- home/.zshrc.luadot/luadot.lua
local parts = {}
for _, name in ipairs(ld.alt.glob("conf.d/*.zsh")) do
  parts[#parts + 1] = ld.alt.read(name)
end

return table.concat(parts, "\n")
```

`*` stays inside one path segment, `**` crosses them, directories are never
listed, and the names come back sorted (`10-env.zsh` before `20-path.zsh`), so
the result never depends on filesystem order. `ld.alt.exists` answers for a
single name:

```lua
local name = ld.alt.exists("laptop.zsh") and "laptop.zsh" or "default.zsh"
return ld.alt.file(name)
```

A generated file that is JSON is built by `ld.alt.json`. It takes a table of
names or a list, never both in the same table, and its keys come out sorted,
so the file changes only when the data does:

```lua
return ld.alt.json({
  editor = ld.class.get("editor") or "nvim",
  gpu = ld.sys.gpu.vendor,
})
```

## Secrets and reloads

A generated file inherits your umask, wrong for a secret; `mode` fixes it. A
daemon reading a file has to be told it changed; `on_change` does:

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

`mode` is compared as well as written: right content with wrong permissions is
reported as differing and put back at `600`. `on_change` runs only when the
file was actually created or replaced, and `--dry-run` prints the command
instead of running it. The same command belongs in an `ld.rules` `on_change`
when it is about a path rather than one template; both forms end up in the
same list, deduplicated, run once at the end of the run, and the template's
own wins over a rule matching the same destination.

## Embedded templates

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
always raw: a dotfile is not HTML, so nothing is escaped. Beware the slurping
pair: `<%_` and `_%>` eat newlines too, welding the previous or the next line
onto the tag's own, so an unindented `<%` closed by `-%>` is the everyday
recipe. A comment closes on `%>` alone, so a `<%# ... %>` on a line of its own
leaves a blank line behind; `<% --[[ ... ]] -%>` takes its line with it.
Errors report the template's own line, an output tag holding `nil` is an error
rather than the word `nil` in the file, and a `%>` inside a Lua string,
comment or long bracket does not close the tag.

A template reaches the whole interface, `ld.alt.expand` included, so a partial
is an ordinary call; each one keeps its own variables and emits into its own
buffer:

```zsh
<%= ld.alt.expand("header.tmpl.zsh", { title = "zsh" }) -%>
export EDITOR=<%= editor %>
```

## The standalone form

A `.luadot` **file** is an embedded template needing no directory: `tmpl alt`
renders it and places the result at the mirrored path, following the
configured `link` and `conflict` rules. The `ld` interface is all there; what
is missing is what needs the directory:

| Missing | Reason |
| --- | --- |
| several outputs | there is no `ld.alt.out` to call |
| `dest`, `link`, `conflict` | no table to carry them; `ld.rules` in `config.lua` still applies |
| `require` of its own `lua/` directory | there is no directory; `~/.config/luadot/lua/` is still requirable |
| `ld.alt.*` | inert, warned; `ld.alt.json` is the exception, it needs no directory |

`ld.path.dir` is `nil`, since there is no template directory; `ld.path.repo`
still reaches a file shared elsewhere in the repository.

## The other commands

A template is one thing, not the files it holds:

| Command | On a template |
| --- | --- |
| `apply` | Walks past it; `tmpl alt` is what resolves it. |
| `status -t`, `diff -t` | Resolves it and reports the files it produces; without the flag both say how many templates they left out. |
| `add` | Refuses a file a template already produces, and leaves it out when it walks a directory. |
| `rm` | Takes the whole template out and leaves what it produced on the system. |
| `edit` | Opens the `luadot.lua` of a directory template, the file itself of a standalone one. |

`status --templates` and `diff --templates` resolve the templates the way
`tmpl alt --dry-run` does: `luadot.lua` runs, `ld.cmd` and the rest run with
it, nothing is written. That is why they take a flag: a template asking a
password store for a token has no business doing it on every `luadot status`.
`diff --templates` puts the generated file under `generated/` rather than
`repository/`, since what a template produces is not in the repository.

`rm`, `edit`, `status` and `diff` reach a template through the path it
produces (`luadot rm ~/.zshrc` takes out `home/.zshrc.luadot/`); the
template's own path works as well. `rm` backs up every file the template holds
before removing it and leaves the system alone: a generated file stays where
it is, and a symlink pointing into the template becomes a file of its own.

## Editor support

An editor sees `.luadot`, not `.zsh`, so highlighting needs a nudge. Neovim
already ships the grammar this format parses as,
`tree-sitter-embedded-template`, the `eruby`/`ejs` one:

```lua
vim.filetype.add({ pattern = { [".*%.luadot"] = "luadot" } })
vim.treesitter.language.register("embedded_template", "luadot")
```

plus an `injections.scm` sending `(code)` to `lua` and `(content)` to the
language of the file being generated. A `<%# luadot: zsh %>` comment on the
first line names that language; the renderer ignores it.
