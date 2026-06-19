# cvkg-cli TLDR.md

## Purpose
Own the command-line interface for CVKG: CLI argument parsing, project scaffolding, development server, and build tooling.

## Ownership
- `src/lib.rs` or `src/main.rs` — CLI entry point, argument parsing
- Subcommands: build, dev, test, scaffold

## Local Contracts
- CLI must work cross-platform (macOS, Linux, Windows).
- Help text and error messages must be clear and actionable.
- Project scaffolding must generate valid, compilable CVKG projects.

## Verification
- Run `cargo build -p cvkg-cli`
- Run `cargo test -p cvkg-cli` if tests exist
