---
name: design-system
description: luadot module layout and code conventions. Read before adding a file, a module, or a command, and before moving shared logic.
---

# Design system

Keep this file short. One line per rule.

## Layout

| Path | Holds |
|---|---|
| `src/main.rs` | only `main` |
| `src/cli/` | argument dispatch: `run.rs`, `types.rs`, the command map in `mod.rs` |
| `src/cli/commands/` | one `<name>_cmd.rs` per command, nothing else |
| `src/files/` | link and sync primitives |
| `src/git/` | git operations |
| `src/lua/` | configuration loading and the Lua runtime |
| `src/state/` | persisted state: `State` and its store |
| `src/utils/` | shared helpers used across modules |

## Rules

- `src/cli/commands/` is for commands only. Logic shared by two or more commands moves to `src/utils/` (or the module that owns the domain), never to a sibling of the commands.
- Every module is a directory with a `mod.rs` that only declares submodules and re-exports its public surface.
- Constants live in the module's own `constants.rs` (`lua/constants.rs`), never inline in the file that uses them.
- A command exposes one `pub fn <name>_cmd(args: &[String]) -> Result<()>`; register it in `cli::get_commands`.
- Errors use `anyhow` and every message is prefixed with the command name: `bail!("add: ...")`. Helpers shared between commands take the prefix as a parameter.
- Split IO from logic so the logic is testable: the IO wrapper (`load`, `require_repo`) calls a pure function (`load_from`, `resolve`).
- Tests live in a `#[cfg(test)] mod tests` at the bottom of the file they test.
- New capability for the configuration is added to the `ld` API, never by unlocking the runtime: `Lua::new()` stays safe (no `ffi`, no C modules) and the language stays PUC Lua 5.4.
- One `ld` function per file, one directory per group: `ld/root/` is `ld.<function>`, and a named group (`ld/git/`) owns its `NAMESPACE` and becomes `ld.<namespace>.<function>`. Each group has a `table.rs` listing its functions.
- No comments anywhere.
