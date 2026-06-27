//! # CVKG Compositor
//!
//! Retained-mode layer orchestration engine for the CVKG UI framework.
//!
//! The compositor sits between `cvkg-vdom` and `cvkg-render-gpu`, providing:
//! - **Material Routing**: Organizes draw calls into GPU pass buckets (scene, glass, overlay).
//! - **Damage Tracking**: Tracks which layers changed to avoid re-recording static content.
//! - **Layer Orchestration**: Maintains a retained `LayerTree` with Z-sorting and hierarchy.
//!
//! ## Architecture
//!
//! ```text
//! VDom → LayerTreeBuilder → CompositorEngine → GpuRenderer
//!                                    │
//!                          ┌─────────┼─────────┐
//!                          ▼         ▼         ▼
//!                     scene_cmds  glass_cmds  overlay_cmds
//!                          │         │         │
//!                          ▼         ▼         ▼
//!                     ┌─────────────────────────────┐
//!                     │  Backdrop Capture Pipeline  │
//!                     │  (Scene→Blur→Composite→UI)  │
//!                     └─────────────────────────────┘
//! ```

pub mod engine;
pub mod layer;
pub mod template;

// Re-export primary types for convenience.
pub use engine::{CommandBuckets, CompositorEngine, DamageInfo, RoutedDrawCommand};
pub use layer::{DrawCommand, Layer, LayerId, LayerTree, Material};
pub use template::{RenderTemplate, TemplateError};

/// Current version of the cvkg-compositor crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
