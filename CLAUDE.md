# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

luadot is a small Rust CLI (edition 2024); the toolchain is pinned with proto (`.prototools`).

## Commands

- `cargo build` — build
- `cargo run -- <command> [args]` — run (e.g. `cargo run -- clone <url>`)
- `cargo test` — test (single: `cargo test <name>`)
- `cargo clippy` — lint

## Architecture

- `main.rs` calls `cli::run()` and maps the `Result` to an `ExitCode`.
- `cli` is a command registry: `get_commands()` maps a name to a `Command`
  (`Run(handler)` or a nestable `Group`), and `run()` splits argv and
  dispatches recursively. A handler is `fn(&[String]) -> anyhow::Result<()>`;
  add a command by registering it in `get_commands()`.
- `git` clones via `gix`, `state` persists `State` as JSON under
  `utils::data_dir()` (`$XDG_DATA_HOME` or `~/.local/share`, then `luadot`),
  and `lua` wraps an `mlua` runtime.

## Language

- The entire project must be written in English. Everything — code,
  comments, identifiers, documentation, commit messages, and this file —
  must be in English. No other language is allowed anywhere.

## Commits

- Messages follow Conventional Commits, enforced by `.githooks/commit-msg`
  (installed automatically by `build.rs`).

## Changes

- After making any change, check for errors. If there are any, change the
  code again to fix them.

## main.rs

- `main.rs` must contain only the `main` function. All other logic
  belongs in the appropriate module.

## This file

- Keep this file short and simple. It must stay easy to read and review,
  so avoid adding complex or lengthy content.
