# How to Run Tests

## Goal

Run the full test suite, a single crate's tests, or a specific test by name.

## Prerequisites

- Rust toolchain installed
- `cargo build --workspace` succeeds

## Steps

### Full Workspace

```bash
cargo test --workspace
```

### Single Crate

```bash
cargo test -p cvkg-layout
```

### Single Test by Name

```bash
cargo test -p cvkg-layout tests::test_hstack_basic
```

### Visual Regression Tests

`cvkg-test` provides pixel-level comparison. Run it with:

```bash
cargo test -p cvkg-test
```

Tests use `insta` for snapshot comparison. To review diffs:

```bash
cargo insta test -p cvkg-test --review
```

### Property-Based Tests

`cvkg-test` includes property-based tests via `proptest`. These run automatically with `cargo test -p cvkg-test`.

### Benchmarks

```bash
cargo bench -p cvkg-layout
```

## Expected Output

All tests should pass. Warnings in dependencies are acceptable; warnings in your own code should be addressed.

## What Can Go Wrong

- **"can't find crate for `core`**: Add the WASM target: `rustup target add wasm32-unknown-unknown`.
- **Snapshot mismatch**: If a visual change is intentional, update snapshots with `cargo insta accept -p cvkg-test`.
- **Timeout in CI**: The `profile.test` setting limits debug info to prevent OOM. If a test needs more stack, add `#[serial]` and run sequentially.
