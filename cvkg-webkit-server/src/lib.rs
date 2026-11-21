#![cfg(not(target_arch = "wasm32"))]
#![allow(unused_imports, deprecated)]

//! # CVKG Webkit Server Library
//!
//! Provides WASM execution, WASI preview1 mapping, and server-side utilities.

pub mod router;
pub mod wasm_server;
