# Embedded templates

Plan for an ERB-style embedded-Lua template format, in two forms: a call
available inside a template directory, and a standalone template file that
needs no directory at all.

Nothing here is implemented. It supersedes the ROADMAP item asking for a
`{{ var }}` mini-syntax for `ld.alt.render`, since `<%= var %>` covers
substitution and adds control flow on top.

## The two forms

| Form | Shape | Produces | Has |
| --- | --- | --- | --- |
| Directory | `.zshrc.luadot/` with `luadot.lua` and `zshrc.tmpl.zsh` | any number of files | variants, several outputs, `dest`/`link`/`conflict`, `require` of its own `lua/` |
| File | `.zshrc.luadot` as a plain file | exactly one file, at the mirrored path | the `ld` interface and nothing else |

Both are resolved by `luadot tmpl alt`. `apply` and `status` walk past both, exactly
as they walk past a template directory today.

The two are mutually exclusive by construction: a path is either a file or a
directory, never both, so `.zshrc.luadot` unambiguously names one of them.

## Format

```zsh
export EDITOR=<%= ld.class.get("editor") or "nvim" %>
<% for _, dir in ipairs({ "~/bin", "~/.local/bin" }) do -%>
path+=(<%= dir %>)
<% end -%>
```

The tags are EJS's, and they keep EJS's meanings:

| Tag | EJS | Here |
| --- | --- | --- |
| `<%` | scriptlet, control flow, no output | Lua statements, emits nothing |
| `<%_` | strips all whitespace before it | the same |
| `<%=` | outputs the value, HTML escaped | outputs `tostring(expr)` |
| `<%-` | outputs the value unescaped | outputs `tostring(expr)` |
| `<%#` | comment, no execution, no output | the same |
| `<%%` | a literal `<%` | the same |
| `%>` | plain ending tag | the same |
| `-%>` | trims the following newline | the same |
| `_%>` | removes all whitespace after it | the same |

The one place the mapping cannot be exact is escaping, because there is none:
a dotfile is not HTML, and turning `&&` into `&amp;&amp;` inside a `.zshrc`
would be a silent corruption. So `<%=` and `<%-` emit the same raw text. Both
spellings are accepted rather than one being an error, because that is what a
reader coming from EJS expects to be able to type, and in a template that never
escapes they are the same tag.

`%%>` writes a literal `%>` inside a tag. It is not in EJS's documented list,
but the grammar below carries it and it is the only way out of a `%>` built by
concatenation.

### Trimming

Straight from EJS, which means the two slurping tags are greedy:

- `<%_` removes every whitespace character before the tag, newlines included.
- `-%>` removes the newline immediately after the tag.
- `_%>` removes every whitespace character after the tag, newlines included.

In HTML that is harmless. In a `.zshrc` it is not, and the README should say so
rather than let it be discovered:

```zsh
export A=1
  <%_ for _, dir in ipairs(paths) do -%>
path+=(<%= dir %>)
  <%_ end -%>
```

`<%_` eats the newline after `export A=1` along with the indentation, so the
first line is welded onto the next and zsh reads one command named
`A=1path+=(~/bin)`. The safe form of the same block is the one the first
example uses, an unindented `<%` with `-%>`:

```zsh
export A=1
<% for _, dir in ipairs(paths) do -%>
path+=(<%= dir %>)
<% end -%>
```

`<%_` and `_%>` stay in because they are part of the standard and nothing uses
them unless a template asks for them. `<%` with `-%>` is the everyday form.

### Where the spellings come from

The tag set is not a free choice: it is what `tree-sitter-embedded-template`
parses, which is what buys the editor support described at the end of this
document. Its rules, verbatim:

```js
directive:         seq(choice('<%', '<%_', '<%|', '<%~'), optional($.code),
                       choice('%>', '-%>', '_%>')),
output_directive:  seq(choice('<%=', '<%==', '<%|=', '<%|==', '<%-'), optional($.code),
                       choice('%>', '-%>', '=%>')),
comment_directive: seq('<%#', optional($.code), '%>'),
content:           repeat1(choice(/[^<]+|</, '<%%')),
code:              repeat1(choice(/[^%=_-]+|[%=_-]/, '%%>')),
```

That grammar serves several dialects at once, and the ones we adopt are exactly
EJS's. The rest stay unclaimed: `<%~` is Eta's raw-output block, `<%|`, `<%|=`
and `<%|==` are capturing blocks, `<%==` and `=%>` come from another dialect
again, and `<%graphql` is not ours to define. None of them mean anything here,
and a template using one is a syntax error rather than a silent misreading.

Two constraints the grammar imposes on the EJS set:

- `<%_` opens a statement tag only. There is no `<%_=`, and that is also the
  only place it is wanted: `<%_ end -%>`.
- A comment closes on `%>` alone; `<%# ... -%>` does not parse. The comment
  itself emits nothing, but the newline after it survives, so one sitting on a
  line of its own leaves a blank line behind. `<% --[[ ... ]] -%>` is the form
  that takes its line with it.

## Naming

The standalone form reuses `TEMPLATE_SUFFIX` (`.luadot`), already the suffix of
a template directory: a `.luadot` **directory** is a template with a resolver, a
`.luadot` **file** is a template rendered directly. `template_target` and
`template_dir` work unchanged for both.

The cost is syntax highlighting: an editor sees `.luadot`, not `.zsh`. Both
yadm (`.gitconfig##template.esh`) and chezmoi (`dot_gitconfig.tmpl`) pay the
same cost, and Editor support below buys it back for a few lines of editor
configuration.

Rejected: a second suffix (`.lt`, `.esh`). It would add a concept, constants
and resolution rules to a form whose point is that there is less to learn.

Inside a template directory the problem does not exist: the rendered file is
named by hand in `luadot.lua`, so `zshrc.tmpl.zsh` keeps its real extension.

## The compiler

Both forms share it: text in, Lua chunk out, run by mlua.

Literals never reach the generated source. They go into a `Vec<String>` on the
Rust side and are referenced by index, so escaping never comes up. A dotfile
holding `]==]`, quotes or invalid Lua is just bytes in a table.

The example above compiles to:

```lua
__ld_emit(1);__ld_write(ld.class.get("editor") or "nvim");__ld_emit(2);
 for _, dir in ipairs({ "~/bin", "~/.local/bin" }) do 
__ld_emit(3);__ld_write(dir);__ld_emit(4);
 end 
```

with `literals = ["export EDITOR=", "\n", "path+=(", ")\n"]`.

Four details carry the design:

1. Every emitted call ends in `;`. Without it, `__ld_emit(1)` followed by a
   statement opening with `(` is a Lua "ambiguous syntax" error.
2. Generated lines are padded to match source lines. Newlines inside literals
   do not exist in the generated source, so before each segment the compiler
   appends newlines until the generated line number equals the source line
   number. With `set_name(path)`, a runtime error in a template reports the
   real line of the real file. Without the padding, every message would point
   at a line the author never wrote.
3. The buffer lives on the Rust side, an `Arc<Mutex<String>>` captured by the
   two closures (`Rc` is out, since the `send` feature of mlua is on). Compared
   with accumulating into a Lua table and ending on
   `return table.concat(__o)`, a `return` written by the user inside `<% %>`
   merely ends the template early with what was rendered so far, instead of
   breaking the return contract.
4. `__ld_write` refuses `nil`. An undefined name in a template is `nil`, and
   `tostring(nil)` would put the word `nil` into the generated file and say
   nothing about it, so a typo in `<%= edtior %>` would quietly produce
   `export EDITOR=nil`. It errors instead, naming the line and the fact that
   the value was `nil`. Everything else goes through `tostring`, `false`
   included.

### Variables

A free name in a template, `editor` in `<%= editor %>`, resolves against the
environment the chunk runs in: the `vars` table of the call first, then the
globals, through `__index`. `ld.alt.render` already builds that environment,
and it is reused as it stands.

The two forms differ here, and it is the main difference between them:

```lua
-- .zshrc.luadot/luadot.lua
ld.alt.out({ content = ld.alt.expand("zshrc.tmpl.zsh", { editor = "nvim" }) })
```

gives `<%= editor %>` a value. A standalone `.zshrc.luadot` has no `luadot.lua`
to pass one, so a bare `<%= editor %>` there is `nil` and errors by rule 4. It
has to name what it wants:

```zsh
export EDITOR=<%= ld.class.get("editor") or "nvim" %>
```

`ld` is a global, so it is reachable from both. Vars are what a template
directory buys, on top of several outputs and `require`.

### Scanner

`%>` inside a Lua string (`local s = "100%>"`) defeats a naive search. `esh`
has that bug. The scanner skips short strings, long brackets and comments while
looking for the closing delimiter. That is about forty lines of code, and what
it prevents is a silent mis-parse.

A `<%#` tag is the exception: its body is prose, not Lua, so it is scanned for
the closing delimiter alone and produces no segment at all. No literal, no line
of generated code, only the newline padding that keeps the following lines
aligned.

`%%>` inside a tag is the grammar's own escape for a literal `%>`, so the
scanner consumes it as two characters of code rather than as a close. It is the
explicit way out of the one case string-skipping cannot cover: a `%>` built by
concatenation.

## Layout

| Path | Holds | State |
| --- | --- | --- |
| `src/lua/embed/mod.rs` | submodules and the public `compile` | new |
| `src/lua/embed/constants.rs` | `OPEN`, `CLOSE`, the closure names | new |
| `src/lua/embed/scan.rs` | text into segments | new |
| `src/lua/embed/compile.rs` | segments into `Chunk { source, literals }` | new |
| `src/lua/embed/run.rs` | installs the closures, loads the chunk, returns the string | new |
| `src/lua/ld/alt/expand.rs` | `ld.alt.expand` | new |
| `src/lua/ld/alt/constants.rs` | `EXPAND` | edited |
| `src/lua/ld/alt/table.rs` | the fourth entry of the array | edited |
| `src/lua/template/file.rs` | `load_template_file`, the standalone form | new |
| `src/lua/template/mod.rs` | its export | edited |
| `src/lua/mod.rs` | `mod embed;` and the new export | edited |
| `src/lua/ld/surface/types.rs` | `Surface::Standalone` | edited |
| `src/files/walk.rs` | `Entry::Standalone` | edited |
| `src/cli/commands/alt_cmd.rs` | resolving the new variant | edited |
| `src/cli/commands/apply_cmd.rs` | skipping the new variant | edited |
| `src/cli/commands/status_cmd.rs` | skipping the new variant | edited |

`src/lua/embed/` is not part of the `ld` API. It is a text-to-Lua compiler, so
it sits beside `runtime` and `script`, and `ld/alt/expand.rs` is the one file
holding the one `ld` function, as the design system requires.

## The directory form: `ld.alt.expand`

```lua
ld.alt.out({ content = ld.alt.expand("zshrc.tmpl.zsh", { editor = "nvim" }) })
```

Same signature and same resolution as `ld.alt.render`: a name relative to the
template directory, absolute or `..` anywhere else, both through
`alt::file::resolve`. Same environment as `render`, the `vars` table with
`__index = globals`, so `ld` stays reachable. It returns a string, so `Output`,
`Content::Text` and everything downstream of `alt` are untouched.

A separate function, not extension detection inside `render`: `render` runs a
Lua script that returns a string, `expand` runs a text file that emits one. The
two contracts differ enough to keep the names apart.

## The standalone form

`load_template_file(command, home, repo, path, classes) -> Result<Output>`,
mirroring `load_template`. It compiles the file, runs the chunk with
`Surface::Standalone` installed, and returns a single
`Output::new(dest, Content::Text(rendered), None, None)`.

`dest` is `template_target(path)` through `utils::system_path`, the same
mirroring the directory form uses. `link` and `conflict` are `None`, so
`place()` in `alt_cmd` falls back to the configured policy exactly as it
already does for generated content, and no code path is added there.

What the standalone form does not get, and why the tradeoff holds:

| Missing | Reason |
| --- | --- |
| several outputs | there is no `ld.alt.out` to call |
| `dest`, `link`, `conflict` | no table to carry them; `ld.rules` in `config.lua` still applies |
| `require` of its own `lua/` directory | there is no directory; the configuration's `lua/` is still requirable |
| `ld.alt.*` | inert, warned |

`Surface::Standalone` makes `ld.alt.*` inert for free: every one of those
functions already calls `surface::inert(lua, name, Surface::Template)`, which
is true for any surface that is not `Template`. No call site changes. Its
`label()` is the file's own name and its `cost()` is `TEMPLATE_COST`, since it
pays the Lua runtime on every `alt` just like a template directory.

`ld.path.dir` stays `nil`, since there is no template directory. `ld.path.repo`
covers the case of reaching a shared file.

## Walking

```rust
pub enum Entry {
    File(PathBuf),
    Template(PathBuf),
    Standalone(PathBuf),
}
```

- `target()`: the `Standalone` arm joins the `Template` arm, and both strip the
  suffix through `template_target`.
- `collect_into`: the non-directory branch checks `is_template` before pushing
  `Entry::File`. Without it a standalone template would be copied to the home
  directory verbatim, tags and all.
- `collect_entries`: the single-file root case gets the same check.
- `collect_files`: `Standalone` pushes the file itself, so `rm` and `add` reach
  it by name.

The three commands matching on `Entry` (`alt`, `apply`, `status`) match
exhaustively, so adding the variant produces a compile error at each of the
three places that must decide about it. `restore_cmd` and `rm_cmd` go through
`collect_files` and are unaffected.

In `alt_cmd`, `template_root` filters `template_dir(&managed)` on `is_dir`
today; it becomes `exists`, so `luadot tmpl alt ~/.zshrc` reaches a standalone
template as well as a directory.

## Errors

Every message keeps the command prefix and names the file:

- an unterminated tag reports the line where it was opened;
- a compile error and a runtime error both report the template's own line,
  through the padding described above;
- a file that is not valid UTF-8 fails on read, naming the path;
- an output tag holding `nil` reports the line and the expression's text,
  rather than writing `nil` into the file;
- inside `ld.alt.expand`, failures are prefixed with `` `ld.alt.expand` ``,
  matching the wording `ld.alt.render` already uses.

## Tests

Unit tests at the bottom of each new file, as the design system requires:

- `scan.rs`: every tag type, `<%-` landing on the same segment as `<%=`, a
  `<%#` producing nothing, each of `<%_`, `-%>` and `_%>` slurping exactly what
  EJS says including across newlines, `<%%` and `%%>`, `%>` inside a Lua
  string, inside a Lua comment and inside a long bracket, an unterminated tag,
  and an unclaimed spelling (`<%~`, `<%|`, `<%==`) reported as an error.
- `compile.rs`: the generated shape, the literal table, and line alignment
  asserted directly (a template whose error must report line 7 reports line 7).
- `run.rs`: a chunk emitting nothing, a `return` mid-template, a runtime error,
  `<%= nil %>` and an undefined name both erroring, `<%= false %>` writing
  `false`.
- `expand.rs`: vars in scope, a var shadowing a global, `ld` reachable through
  `__index` when vars are given, a missing file, an absolute path.
- `template/file.rs`: the mirrored destination, `ld.sys`/`ld.class` reachable,
  `ld.alt.*` inert, and a bare name erroring since nothing can define one.
- `walk.rs`: a standalone template is its own entry, is not an `Entry::File`,
  and `collect_files` returns it.
- `alt_cmd.rs`: a standalone template lands on the mirrored path, follows the
  configured conflict policy, is backed up when replaced, and is reported by
  `--dry-run`.

Integration, in `tests/cli.rs` through `assert_cmd`: a temporary `HOME` with a
repository holding both forms; `alt` writes both, `apply` writes neither, and
`status` reports neither.

## Benchmarks

`benches/lua.rs` gains an `expand` case beside `render`, over a template large
enough for the compile cost to show, so the two forms can be compared. The
standalone form gets its own case, since it skips `luadot.lua` entirely and
should be measurably cheaper.

## Documentation

- README, Templates section: the two forms, the tag table naming EJS as where
  it comes from, the warning that `<%_` and `_%>` slurp across newlines and
  that `<%` with `-%>` is the everyday form, the `.luadot` file next to the
  `.luadot` directory in the tree diagram, `ld.alt.expand` in the call table,
  the surface table row for a standalone template, and the editor configuration
  from the next section.
- ROADMAP: drop the `{{ var }}` mini-syntax item, which this replaces.

## Editor support

None of this is part of the work below. It is here because the first layer is
what pays for the naming tradeoff, and because the third one is only safe to
write once the format has stopped moving.

### Highlighting, no server

`tree-sitter-embedded-template`, the grammar Neovim already ships for `eruby`
and `ejs`, parses this format as it stands: `directive`, `output_directive`,
`comment_directive`, `content`, `code`, with `<%`, `<%=`, `<%#`, `%>` and
`-%>`. Nothing to write:

```lua
vim.filetype.add({ pattern = { [".*%.luadot"] = "luadot" } })
vim.treesitter.language.register("embedded_template", "luadot")
```

plus an `injections.scm` sending `(code)` to `lua` and `(content)` to the
language of the file being generated.

That last one is the fiddly part: the outer language changes per file, and
nothing in the name says which. The answer costs nothing now that `<%#` exists,
a directive on the first line, ignored by the renderer:

```
<%# luadot: zsh %>
```

### Completion, still no server

A `---@meta` file declaring `ld.sys`, `ld.class.get`, `ld.path` and the rest,
published as a `lua_ls` library through `.luarc.json`, gives completion and
hover inside `luadot.lua` and the `lua/` modules without any server of ours.
`mlua-extras` and `tealr` generate that kind of definition from the mlua
registration; writing the file by hand is also viable and adds no dependency.

### `luadot lsp`

Worth it only for what nothing else can do:

- hover showing this machine's value, `ld.sys.host.name` over `thinkpad`;
- completion of the classes the repository's `config.lua` declares;
- diagnostics for an unterminated tag and for Lua errors inside one, since the
  position mapping is already built by the compiler above;
- go-to-definition on `ld.alt.file("laptop.zsh")`;
- a render preview, which is not even LSP, just `luadot render <path>`.

The hard part is that an editor attaches a server to the whole buffer, not to
an injected region, so `lua_ls` on a `.zshrc.luadot` reads zsh and objects to
all of it. Either our server owns the Lua diagnostics alone, or it extracts the
Lua regions into a virtual document, forwards to `lua_ls` and maps positions
back, which is most of what `svelte-language-server` and Volar do.

Crate, when the time comes: `lsp-server`, the rust-analyzer one. It is
synchronous, minimal, and it keeps tokio out of a binary that has no async in
it. `tower-lsp` had no release since August 2023; its live fork is
`tower-lsp-server`.

A language server is written against a settled syntax, so the order matters.
The first two layers are worth having as soon as the format exists, the third
one only after it has been used enough to stop changing.

## Order of work

Each step is a commit, in this order, and each one compiles and passes on its
own.

1. `chore(lua): add the embedded template compiler`: `src/lua/embed/`, scanner
   through `run`, no caller yet.
2. `feat(lua): render embedded templates`: `ld.alt.expand` and the standalone
   form, wired into `alt`, `Entry::Standalone`, the three command matches.
3. `test(cli): drive both template forms end to end`.
4. `chore(bench): measure the embedded templates`.
5. `docs: document embedded templates`, README and ROADMAP.

Step 2 is the only `feat:`. The compiler alone changes nothing a user can see,
and tests, benchmarks and docs are not features.

## Open questions

- ~~Whether `ld.alt.expand` should be callable from inside an embedded
  template, making partials possible.~~ Settled: it is, in the directory form.
  Nothing had to be added. The chunk runs with `ld` reachable through
  `__index`, the template stays in the app data, and every `run` builds its own
  environment, so `__ld_emit`/`__ld_write` are per call rather than global and
  a nested expansion cannot write into its caller's buffer. The standalone form
  keeps it inert, since a partial needs a directory to resolve against.
- Whether a line-bounded trim is worth having beside the greedy pair. EJS's
  `<%_`/`_%>` cross newlines, which is what a `.zshrc` cannot afford, and the
  everyday recipe avoids them entirely. The day someone wants an indented block
  to disappear cleanly there is no spelling for it. The grammar leaves `<%|`
  and `<%~` unclaimed on the opening side, so one could be added without losing
  the highlighting; the closing side offers only `%>`, `-%>` and `_%>`, so it
  would have to stay asymmetric.
- Whether a template that uses an unclaimed spelling should error or pass it
  through as literal text. Erroring is the plan, on the grounds that `<%~` in a
  file means the author expected Eta's behavior and should be told they will
  not get it.
- Whether a standalone template should be able to opt out of the mirrored
  destination somehow. Any answer reintroduces the resolver the form exists to
  avoid, so the honest answer is probably "use a directory".
- Whether `status` should report what either form produces. That gap already
  exists for template directories and is tracked in the ROADMAP; the standalone
  form widens it without changing its nature.
