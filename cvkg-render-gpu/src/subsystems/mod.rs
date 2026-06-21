//! P1-1 (phase 6): Subsystem modules.
//!
//! Each renderer subsystem is a self-contained module. This
//! replaces the previous design where subsystem state was mixed
//! in with the rest of the renderer state in the giant
//! `types.rs` file.
//!
//! Phase 6 of the P1-1 refactor: move the subsystem structs from
//! `types.rs` into their own files so they can be reviewed,
//! tested, and modified in isolation.

pub mod config;
pub mod geometry_buffers;
pub mod gpu_capabilities;

pub use config::RendererConfig;
pub use gpu_capabilities::{detect_gpu_vendor, GpuCapabilities, GpuVendor};

// Re-export the existing subsystem structs that still live in
// types.rs. These will be moved to their own files in subsequent
// commits.
pub use crate::types::{
    GeometryBuffers, ParticleSubsystem, SvgSubsystem, TextSubsystem,
};
