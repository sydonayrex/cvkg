//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     -- State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     -- Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     -- Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    -- Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     -- Read the target, its surrounding context, and its full call graph
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     -- Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   -- Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//!   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//!   CVKG Extended: Section 2 of the CVKG Design Specification
#![allow(
    clippy::type_complexity,
    clippy::unwrap_or_default,
    dead_code,
    unused_variables,
    unused_imports,
    unused_mut,
    unused_parens
)]

mod kvasir;
mod material;

// Re-export material types for downstream users
pub use material::builtins;
pub use material::{CompiledMaterial, MaterialCompiler, MaterialError, MaterialGraph, MaterialOp};

pub mod accessibility;
pub mod ai;
mod api;
mod draw;
pub(crate) mod passes;
pub mod pyramid;
pub mod renderer;
mod surtr_util;
pub mod types;
pub mod vertex;

pub mod heim;
pub use heim::SundrPacker;

// P1-1 (phase 6): subsystems module. Each subsystem (config,
// geometry, text, svg, particles) is a self-contained module
// that can be tested, reviewed, and modified in isolation.
pub mod subsystems;
pub use subsystems::SurtrConfig;

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
        assert_eq!(anims[0].keyframe_values, vec![0.0, 360.0]);

        assert_eq!(anims[1].target_id, "pulse");
        assert_eq!(anims[1].attribute_name, "opacity");
        assert_eq!(anims[1].duration, 0.5);
        assert_eq!(anims[1].keyframe_values, vec![0.5, 1.0]);

        assert_eq!(anims[2].target_id, "myRect");
        assert_eq!(anims[2].attribute_name, "x");
        assert_eq!(anims[2].duration, 0.5); // 500ms parsed as 0.5
        assert_eq!(anims[2].keyframe_values, vec![10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_shelf_packer_full() {
        let mut packer = SundrPacker::new(10, 10);
        assert_eq!(packer.pack(11, 5), None);
        assert_eq!(packer.pack(5, 11), None);
    }
}

// P1-12 fix: on wasm32/WebGL2, texture binding arrays are not supported.
// The bind group layout uses count: None (single texture) on WASM, so the
// WGSL must declare t_diffuse as a single texture, not a binding_array.
// We swap the three affected WGSL files (common, material_opaque, bloom)
// to WASM-specific variants on wasm32 targets. All other shader files
// (shapes, material_glass, bifrost, color_blind, tonemap, particles) are
// the same on both targets.
#[cfg(target_arch = "wasm32")]
pub(crate) const WGSL_COMMON: &str = include_str!("shaders/common_wasm.wgsl");
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const WGSL_COMMON: &str = include_str!("shaders/common.wgsl");

pub(crate) const WGSL_SHAPES: &str = include_str!("shaders/shapes.wgsl");

#[cfg(target_arch = "wasm32")]
pub(crate) const WGSL_MATERIAL_OPAQUE: &str = include_str!("shaders/material_opaque_wasm.wgsl");
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const WGSL_MATERIAL_OPAQUE: &str = include_str!("shaders/material_opaque.wgsl");

pub(crate) const WGSL_MATERIAL_GLASS: &str = include_str!("shaders/material_glass.wgsl");
pub(crate) const WGSL_BIFROST: &str = include_str!("shaders/bifrost.wgsl");

#[cfg(target_arch = "wasm32")]
pub(crate) const WGSL_BLOOM: &str = include_str!("shaders/bloom_wasm.wgsl");
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const WGSL_BLOOM: &str = include_str!("shaders/bloom.wgsl");

pub(crate) const WGSL_COLOR_BLIND: &str = include_str!("shaders/color_blind.wgsl");
pub(crate) const WGSL_TONEMAP: &str = include_str!("shaders/tonemap.wgsl");
pub(crate) const WGSL_PARTICLES: &str = include_str!("shaders/particles.wgsl");

pub mod color_blindness;

// Re-export ColorBlindMode for downstream users
pub use color_blindness::ColorBlindMode;

// ShieldWall -- re-export AccessKit types so callers can build tree updates
// without depending on accesskit directly.
pub use accesskit::{
    ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, Node, NodeId, Role, Tree,
    TreeId, TreeUpdate,
};
pub use accesskit_winit::Adapter as ShieldWallAdapter;

// Re-export ColorTheme and SceneUniforms for cvkg-render-gpu users
pub use cvkg_core::{ColorTheme, SceneUniforms};

pub use renderer::SurtrRenderer;

// P1-35: SVG filter graph integration
pub mod svg_filter_graph;
pub use types::{SvgAnimation, SvgModel};
pub use vertex::{InstanceData, Vertex};
