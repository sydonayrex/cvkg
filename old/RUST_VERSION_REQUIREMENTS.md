# Rust Version Requirements for CVKG

## Stable Rust Support

CVKG v0.1.16 compiles and runs on **stable Rust 1.75+** for all core functionality:

- ✅ `cargo build` - Full compilation
- ✅ `cargo test` - All unit and integration tests (49+ tests pass)
- ✅ `cargo run --example niflheim_demo` - Examples work

## Nightly Rust Requirement

The following features require **nightly Rust** due to use of unstable features:

### `view_component` Macro
```rust
#![feature(impl_trait_in_assoc_type)]
use cvkg_macros::view_component;

#[view_component]
fn MyView(title: String) {
    Text::new(title)
}
```

**Affected crates:**
- `cvkg-macros/tests/macro_tests.rs`
- `cvkg-test/tests/component_integration.rs`

**Resolution:** This feature (`impl_trait_in_assoc_type`) is tracked in [Rust Issue #63063](https://github.com/rust-lang/rust/issues/63063) and is expected to stabilize.

### Stable Rust Workaround

Replace `view_component` macro with explicit implementation:

```rust
// Before (nightly):
#[view_component]
fn MyView(title: String) {
    Text::new(title)
}

// After (stable):
pub struct MyView { pub title: String }

impl View for MyView {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }
}

pub fn MyView(title: String) -> MyView {
    MyView { title }
}
```

## CI Configuration

We provide two test profiles:

### `cargo test` (default)
Runs tests that work on stable Rust. Some test files are excluded.

### `cargo test --features nightly`
Runs all tests including nightly-only test files.

## Checking Your Rust Version

```bash
# Check stable Rust
rustc --version
# rustc 1.75.0 (stable)

# For nightly tests
rustup install nightly
cargo +nightly test --features nightly
```