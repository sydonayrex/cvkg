# cvkg-test

**cvkg-test** provides the integration testing harness and visual regression suites for the CVKG workspace.

## Features

- **Headless Integration Tests**: Specialized tests that use the hardened `Surtr` headless renderer to verify pixel data without requiring a windowing system.
- **Component Unit Tests**: Isolated testing of UI components and their reactive state transitions.
- **Completeness Audits**: Automated discovery of `TODO`, `FIXME`, and architectural placeholders to ensure workspace integrity.
- **Visual Regression**: Automated frame capture and comparison against golden images (Project Niflheim).

## Usage

Run all tests in the crate:
```bash
cargo test -p cvkg-test
```

Run headless rendering tests:
```bash
cargo test --test headless_render -p cvkg-test -- --nocapture
```
