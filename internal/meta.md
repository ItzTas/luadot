# Editor definitions

`meta/ld.lua` is a lua-language-server stub of the whole `ld` interface,
committed and embedded in the binary through the `include_str!` in
`src/lua/meta/constants.rs`. `luadot meta` prints it; `luadot meta install`,
`luadot init` and `luadot clone` write it into the data directory and point a
`.luarc.json` at it.

## Where it comes from

The runtime still builds `ld` out of plain tables in `src/lua/ld/`; nothing
there registers through tealr. Beside each group's registration array,
`constants.rs` describes the same calls: `SIGNATURES` carries the name, the
parameters, the returns and the hover text, and the tables a call takes
(`ld.Options`, `ld.Rule`, `ld.PrintOptions`) are described as fields. The
vocabulary is the `Kind`, `Param`, `Signature` and `Field` types in
`src/lua/ld/signature.rs`, small enough to be written in `const` arrays. The
group's `describe.rs` turns that data into tealr's model (`RecordGenerator`,
`EnumGenerator`, `GlobalInstance`), and `src/lua/ld/walker.rs` collects every
group into one `TypeWalker`.

The string enums (`ld.LinkMode`, `ld.Conflict`, `ld.Tone`, `ld.StatusState`)
are built from the arrays the parser reads, so an alias cannot promise a value
the parser rejects, and a new link mode reaches the editor when it reaches
`LINK_MODES`.

Namespaces are registered as global instances with dotted names (`ld`,
`ld.opt`, `ld.sys.gpu`); data tables are plain records. `src/lua/meta/render.rs`
writes the walker as LuaCATS: enums as `---@alias`, records as `---@class`,
instances as `ld.opt = {}` with an `---@overload` when the table is callable,
functions with `---@param` and `---@return`. tealr's own lua-language-server
generator is not used: it drops the call syntax of the namespaces and the hover
text of the functions.

All of it sits behind the `meta` cargo feature, the only thing that links
tealr. The release build never enables it; the shipped binary only carries the
generated file.

## Keeping it current

```
./packaging/meta/update.sh            # regenerate meta/ld.lua
./packaging/meta/update.sh --check    # what the meta job in CI runs
```

`cargo test --features meta` holds the line from four sides: each group's
`SIGNATURES` lists its registration array in the same order, the model matches
the installed `ld` name by name (kinds, `__call`, `__index`), every type a
description names is declared, and the committed file is what the emitter
renders. Adding a call without describing it fails the first; describing it
without regenerating fails the last.

## The HTML reference

`cargo run --features meta --bin luadot-meta -- --json` prints the model as the
JSON [tealr_doc_gen](https://crates.io/crates/tealr_doc_gen) takes, which
renders an HTML reference and a Teal `.d.tl`. Nothing depends on either, so it
stays a manual step.
