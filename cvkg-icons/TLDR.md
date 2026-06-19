# cvkg-icons TLDR.md

## Purpose
Own the icon library: icon definitions, icon rendering, and the icon theme system used across all CVKG components.

## Ownership
- `src/lib.rs` — Icon definitions, icon rendering primitives
- Icon theme tokens and sizing scale

## Local Contracts
- Icons must render crisply at all standard sizes (12, 14, 16, 18, 20, 24px).
- Icon colors must respect the current theme (theme::text(), theme::text_muted(), etc.).
- Icon definitions must be pure data — no side effects.

## Verification
- Run `cargo test -p cvkg-icons`
- Run `cargo check --workspace`
