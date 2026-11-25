# Changelog

## v0.4.0 - 2026-11-22

Minor release. Consumers must pin every CVKG crate to `v0.4.0`; mixed
`0.3.x` and `0.4.x` path pins are rejected by `scripts/check_version_pins.py`.

### Added

- Typed entity handles, action/focus dispatch primitives, and the full
  accessibility companion tree.
- Bounded parser limits, exact visual fixtures, semantic `Style`, RTL
  direction helpers, and a docking layout model with JSON persistence.
- Component registry CLI with `list`, `view`, `diff`, `add`, and `update`.
- Generated gallery docs, element inspector, form derive macro, and a
  generated Button variant matrix.
- Render hot patching behind the `hot-patching` feature.
- Platform abstraction crates (`cvkg-platform-core`, `-native`, `-web`) and
  the `cvkg-hooks`, `cvkg-blocks`, and `cvkg-charts` crates.
- MCP discovery server in `cvkg-cli mcp --stdio`.
- Geometry-aware DOM accessibility in the adele-web demo, including caret
  position announcements.

### Changed

- Consolidated semantic tokens to `body=16`, `spacing.m=16`, and
  `radius.m=6` across theme presets.
- DockingWorkspace and VirtualList now have production consumers.
- The Niflheim golden image was regenerated after the token consolidation.

### Verification

- `python3 scripts/check_version_pins.py`: all pins equal `0.4.0`.
- `cargo build --workspace --locked`: success.
- `cargo check --workspace --all-targets`: 0 errors.
- `cargo test --workspace --tests --lib --no-fail-fast`: 0 failures after
  golden regeneration.
- `cargo check --workspace --target wasm32-unknown-unknown`: 0 errors.
- DOM smoke page: `pass: true`, 0 failures.
