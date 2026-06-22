#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::field_reassign_with_default)]
//! # CVKG SVG Filters
//!
//! WGPU-based SVG filter primitive evaluation.
//! Parses `usvg::filter::Filter` into a directed acyclic graph of filter primitives,
//! then evaluates each primitive as a WGPU render/compute pass.

pub mod types;
pub mod graph;
pub mod pool;
pub mod validators;
pub mod diagnostics;
pub mod heatmap;
pub mod benchmark;
pub mod engine;
pub mod pipeline;

pub use types::*;
pub use graph::*;
pub use pool::*;
pub use validators::*;
pub use diagnostics::*;
pub use heatmap::*;
pub use benchmark::*;
pub use engine::*;
