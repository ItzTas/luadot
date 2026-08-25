# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

luadot is a small Rust CLI; the toolchain is pinned with proto (`.prototools`).

## Language

- Write the entire project in English: code, comments, identifiers,
  documentation, commit messages, and this file. No other language is
  allowed anywhere.

## Commits

- Messages follow Conventional Commits, enforced by `.githooks/commit-msg`
  (installed automatically by the build script in `build/`).

## Comments

- Do not write comments. No line comments and no doc comments. The code
  must explain itself through names and structure.

## Documentation

- Before writing or editing any documentation, load the `humanizer`
  skill and apply it to the text.

## Changes

- After any change, check for errors and fix the ones you find.

## main.rs

- `main.rs` must contain only the `main` function. All other logic
  belongs in the appropriate module.

## This file

- Keep this file short and simple, so it stays easy to read and review.
