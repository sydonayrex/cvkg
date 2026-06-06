//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     — State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     — Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     — Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    — Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     — Read the target, its surrounding context, and its full call graph
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     — Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   — Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//!   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//!   CVKG Extended: Section 2 of the CVKG Design Specification
#![allow(clippy::type_complexity, clippy::unwrap_or_default)]

mod kvasir;
mod material;

// Re-export material types for downstream users
pub use material::{MaterialGraph, MaterialCompiler, CompiledMaterial, MaterialOp, MaterialError};
pub use material::builtins;

pub mod types;
pub mod vertex;
pub mod renderer;
mod surtr_util;
mod draw;
mod api;

pub mod heim;
pub use heim::SundrPacker;

#[cfg(test)]
mod tests {
    use super::*;

    use super::heim::SundrPacker;

    #[test]
    fn test_shelf_packer_basic() {
        let mut packer = SundrPacker::new(100, 100);
        assert_eq!(packer.pack(10, 10), Some((0, 0)));
        assert_eq!(packer.pack(20, 15), Some((10, 0)));
    }

    #[test]
    fn test_shelf_packer_wrap() {
        let mut packer = SundrPacker::new(100, 100);
        packer.pack(60, 10);
        assert_eq!(packer.pack(50, 20), Some((0, 10)));
    }

    #[test]
    fn test_parse_svg_animations() {
        let svg = r##"
            <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
                <g id="spinner">
                    <animateTransform attributeName="transform" type="rotate" from="0" to="360" dur="2s" />
                </g>
                <circle id="pulse">
                    <animate attributeName="opacity" from="0.5" to="1.0" dur="0.5s" />
                </circle>
                <!-- Edge cases: xlink:href, ms suffix, values list -->
                <rect>
                    <animate xlink:href="#myRect" attributeName="x" values="10; 20; 30" dur="500ms" />
                </rect>
            </svg>
        "##;
        let anims = draw::parse_svg_animations(svg.as_bytes());
        assert_eq!(anims.len(), 3);

        assert_eq!(anims[0].target_id, "spinner");
        assert_eq!(anims[0].attribute_name, "transform");
        assert_eq!(anims[0].duration, 2.0);
        assert_eq!(anims[0].from_val, 0.0);
        assert_eq!(anims[0].to_val, 360.0);

        assert_eq!(anims[1].target_id, "pulse");
        assert_eq!(anims[1].attribute_name, "opacity");
        assert_eq!(anims[1].duration, 0.5);
        assert_eq!(anims[1].from_val, 0.5);
        assert_eq!(anims[1].to_val, 1.0);

        assert_eq!(anims[2].target_id, "myRect");
        assert_eq!(anims[2].attribute_name, "x");
        assert_eq!(anims[2].duration, 0.5); // 500ms parsed as 0.5
        assert_eq!(anims[2].from_val, 10.0);
        assert_eq!(anims[2].to_val, 30.0);
    }

    #[test]
    fn test_shelf_packer_full() {
        let mut packer = SundrPacker::new(10, 10);
        assert_eq!(packer.pack(11, 5), None);
        assert_eq!(packer.pack(5, 11), None);
    }
}

pub(crate) const WGSL_SRC: &str = concat!(
    include_str!("shaders/common.wgsl"),
    include_str!("shaders/shapes.wgsl"),
    include_str!("shaders/bifrost.wgsl"),
    include_str!("shaders/bloom.wgsl"),
    include_str!("shaders/color_blind.wgsl"),
    include_str!(concat!(env!("OUT_DIR"), "/materials_generated.wgsl"))
);

/// Specialized shader source for opaque/2D materials (modes 0-20 excluding 7,13-15,18,21).
pub(crate) const WGSL_OPAQUE: &str = concat!(
    include_str!("shaders/common.wgsl"),
    include_str!("shaders/material_opaque.wgsl"),
    include_str!("shaders/bifrost.wgsl"),
    include_str!("shaders/bloom.wgsl"),
    include_str!("shaders/color_blind.wgsl"),
    include_str!(concat!(env!("OUT_DIR"), "/materials_generated.wgsl"))
);

/// Specialized shader source for glass material (mode 7 only).
pub(crate) const WGSL_GLASS: &str = concat!(
    include_str!("shaders/common.wgsl"),
    include_str!("shaders/material_glass.wgsl"),
    include_str!("shaders/bifrost.wgsl"),
    include_str!("shaders/bloom.wgsl"),
    include_str!("shaders/color_blind.wgsl"),
    include_str!(concat!(env!("OUT_DIR"), "/materials_generated.wgsl"))
);


pub mod color_blindness;

// Re-export ColorBlindMode for downstream users
pub use color_blindness::ColorBlindMode;

// ShieldWall — re-export AccessKit types so callers can build tree updates
// without depending on accesskit directly.
pub use accesskit::{
    ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, Node, NodeId, Role, Tree,
    TreeId, TreeUpdate,
};
pub use accesskit_winit::Adapter as ShieldWallAdapter;

// Re-export ColorTheme and SceneUniforms for cvkg-render-gpu users
pub use cvkg_core::{ColorTheme, SceneUniforms};

pub use renderer::SurtrRenderer;
pub use types::{SvgModel, SvgAnimation};
pub use vertex::{Vertex, InstanceData};
