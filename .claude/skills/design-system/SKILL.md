---
name: design-system
description: luadot module layout and code conventions. Read before adding a file, a module, or a command, and before moving shared logic.
---

# Design system

Keep this file short. One line per rule.

## Layout

| Path | Holds |
|---|---|
| `src/main.rs` | only `main`, calling into the library |
| `src/lib.rs` | only the module declarations, one `pub mod` per line |
| `src/cli/` | argument parsing and dispatch: the clap `Cli`/`Cmd` in `types.rs`, the match in `run.rs` |
| `src/cli/commands/` | one `<name>_cmd.rs` per command, nothing else |
| `src/backup/` | backup runs: `Backup` and the backup directory store |
| `src/files/` | link and sync primitives |
| `src/git/` | git operations |
| `src/hook/` | the commands a rule runs after a file changes |
| `src/lua/` | configuration loading and the Lua runtime |
| `src/output/` | everything written to or read from the terminal: printing, previews, prompts |
| `src/state/` | persisted state: `State` and its store |
| `src/utils/` | shared helpers used across modules |
| `benches/` | one criterion bench per area, fixtures in `benches/support/` |
| `tests/` | integration tests driving the built binary through `assert_cmd` |

## Rules

- `src/cli/commands/` is for commands only. Logic shared by two or more commands moves to `src/utils/` (or the module that owns the domain), never to a sibling of the commands.
- Every module is a directory with a `mod.rs` that only declares submodules and re-exports its public surface.
- Constants live in the module's own `constants.rs` (`lua/constants.rs`), never inline in the file that uses them.
- A command exposes one `pub fn <name>_cmd(args: <Name>Args) -> Result<()>`, its clap `Args` struct in the same file (no struct when it takes nothing); add the variant to `Cmd` in `types.rs` and its arm to the match in `run.rs`. Help texts live in `#[command(about = ...)]`/`#[arg(help = ...)]` attributes, never in doc comments.
- Errors use `anyhow` and every message is prefixed with the command name: `bail!("add: ...")`. Helpers shared between commands take the prefix as a parameter.
- Split IO from logic so the logic is testable: the IO wrapper (`load`, `require_repo`) calls a pure function (`load_from`, `resolve`).
- Tests live in a `#[cfg(test)] mod tests` at the bottom of the file they test; tests that exercise the whole binary go in `tests/`, isolated through a temporary `HOME`.
- Do not test what another test already covers, even indirectly: before adding one, look for a test reaching the same code through a wider path — a shared helper tested on its own, an end-to-end run, a sibling case funnelling into the same function.
- One test per behavior, at the layer that owns it: each test must fail for a reason none of the others would.
- Benchmarks reach the code through the library, one `benches/<area>.rs` per area with a `[[bench]]` entry carrying `harness = false`; shared fixtures go in `benches/support/`.
- New capability for the configuration is added to the `ld` API, never by unlocking the runtime: `Lua::new()` stays safe (no `ffi`, no C modules) and the language stays PUC Lua 5.4.
- One `ld` function per file, one directory per group: `ld/root/` is `ld.<function>`, and a named group (`ld/git/`) owns its `NAMESPACE` and becomes `ld.<namespace>.<function>`. Each group has a `table.rs` listing its functions.
- No comments anywhere.
