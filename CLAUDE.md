# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

luadot is a small Rust CLI; the toolchain is pinned with proto (`.prototools`).

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
