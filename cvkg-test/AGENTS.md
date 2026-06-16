# cvkg-test AGENTS.md

## Purpose
Own the testing framework: test utilities, snapshot testing, accessibility testing helpers, and the test harness for CVKG components.

## Ownership
- `src/lib.rs` — Test utilities, component test harness
- Snapshot testing infrastructure
- Accessibility test helpers

## Local Contracts
- Test utilities must not pollute production builds (cfg(test)).
- Snapshot tests must be deterministic across platforms.
- Accessibility tests must verify real ARIA attributes, not mock data.

## Verification
- Run `cargo test -p cvkg-test`
- Run `cargo test -p cvkg-components` (consumes test utilities)
