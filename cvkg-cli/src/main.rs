//! Entry point: native implementation lives in `native_main.rs`.
#[cfg(not(target_arch = "wasm32"))]
mod native_main;
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    native_main::run();
}
#[cfg(target_arch = "wasm32")]
fn main() {}
