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
pub use engine::{CommandBuckets, CompositorEngine, DamageInfo, RenderCommand, RoutedDrawCommand};
pub use layer::{DrawCommand, Layer, LayerId, LayerTree, Material};
pub use template::{RenderTemplate, TemplateError};

/// Current version of the cvkg-compositor crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod smoke_tests {
    use super::*;

    #[test]
    fn compositor_engine_constructs() {
        let engine = CompositorEngine::new();
        let _ = engine; // just verify it constructs without panicking
    }

    #[test]
    fn command_buckets_default_is_empty() {
        let buckets = CommandBuckets::default();
        assert!(buckets.is_empty());
        assert_eq!(buckets.total_count(), 0);
    }

    #[test]
    fn needs_reflatten_static_after_frame() {
        // Regression C1: needs_reflatten must be false after a full
        // flatten+end_frame for UNCHANGED static content. The old logic compared
        // the tree generation (which advances every end_frame) against the last
        // flatten generation, so it returned true every frame — forcing a full
        // re-flatten of static UI.
        let mut engine = CompositorEngine::new();

        let layer = Layer {
            id: LayerId(1),
            ..Default::default()
        };
        engine.create_layer(layer);
        engine.set_roots(vec![LayerId(1)]);

        // First flatten flags work; after end_frame nothing is dirty.
        assert!(engine.needs_reflatten());
        let _buckets = engine.flatten_and_route();
        engine.end_frame();

        // Static content: no more re-flatten until something changes.
        assert!(
            !engine.needs_reflatten(),
            "static content should not require re-flatten after a frame"
        );

        // A real content change (mark_dirty) must re-arm it.
        engine.mark_dirty(LayerId(1));
        assert!(engine.needs_reflatten());
        let _buckets = engine.flatten_and_route();
        engine.end_frame();
        assert!(!engine.needs_reflatten());
    }
}
