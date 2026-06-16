# cvkg-runic-text AGENTS.md

## Purpose
Own the text rendering engine: BiDi (bidirectional text), shaping, font fallback, and the low-level text drawing primitives.

## Ownership
- `src/lib.rs` — Text shaping, BiDi algorithm, font management
- Integration with platform font systems

## Local Contracts
- BiDi support must handle Arabic, Hebrew, and mixed LTR/RTL text correctly.
- Font fallback must cover all Unicode scripts used in CVKG.
- Text shaping must use HarfBuzz or equivalent for complex scripts.

## Verification
- Run `cargo test -p cvkg-runic-text`
- Run `cargo check --workspace`
