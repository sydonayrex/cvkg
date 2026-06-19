# cvkg-macros TLDR.md

## Purpose
Own the declarative macros for CVKG: derive macros, builder macros, and code generation utilities that reduce boilerplate for component authors.

## Ownership
- `src/lib.rs` — All proc-macro definitions
- Derive macros for component traits
- Builder macros for ergonomic API construction

## Local Contracts
- All macros must produce valid, idiomatic Rust code.
- Macro error messages must be clear and point to the correct source location.
- Macros must not introduce hidden allocations or unnecessary clones.

## Verification
- Run `cargo test -p cvkg-macros`
- Run `cargo check --workspace`
