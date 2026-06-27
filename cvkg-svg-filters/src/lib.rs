#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::field_reassign_with_default)]
//! # CVKG SVG Filters
//!
//! WGPU-based SVG filter primitive evaluation.
//! Parses `usvg::filter::Filter` into a directed acyclic graph of filter primitives,
//! then evaluates each primitive as a WGPU render/compute pass.

pub mod benchmark;
pub mod diagnostics;
pub mod engine;
pub mod graph;
pub mod heatmap;
pub mod pipeline;
pub mod pool;
pub mod types;
pub mod validators;

pub use benchmark::*;
pub use diagnostics::*;
pub use engine::*;
pub use graph::*;
pub use heatmap::*;
pub use pool::*;
pub use types::*;
pub use validators::*;
