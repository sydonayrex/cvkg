# cvkg-themes TLDR.md

## Purpose
Own the design token system: OKLCH color model, APCA contrast validation, theme tokens (light/dark), and semantic color roles.

## Ownership
- `src/lib.rs` — Theme tokens, color generation, contrast validation
- `themes/src/lib.rs` — Theme variants (dark/light), toggle logic

## Local Contracts
- All colors must be defined as `TokenValue::Adaptive { light, dark }` for light/dark mode support.
- APCA contrast ratios must meet WCAG 2.2 AA minimums (4.5:1 normal text, 3:1 large text).
- Theme::validate_accessibility() must test both dark and light themes.

## Verification
- Run `cargo test -p cvkg-themes`
- Run `cargo check --workspace`
