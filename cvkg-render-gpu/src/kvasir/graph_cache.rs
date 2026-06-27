//! Render graph execution plan caching.
//!
//! Stores compiled execution plans keyed by the active render pipeline settings
//! to prevent redundant topological graph rebuilding on the hot frame loop.

use crate::kvasir::graph::{KvasirGraph, NodeKey};

/// Holds the configuration signature, render graph, and compiled pass sequence.
pub struct CachedGraphPlan {
    /// Whether backdrop glass blur pass is enabled.
    pub has_glass: bool,
    /// Whether bloom post-processing pass is enabled.
    pub has_bloom: bool,
    /// Whether accessibility color-blind simulation is enabled.
    pub has_accessibility: bool,
    /// Whether volumetric raymarching pass is enabled.
    pub has_volumetric: bool,
    /// Number of active offscreen compositor nodes.
    pub active_offscreens_count: usize,
    /// Content hash for offscreen effects (effect name, blend mode, args).
    /// Changes when effect content changes even if count stays the same.
    pub offscreen_content_hash: u64,
    /// Number of active portal blur boundary regions.
    pub portal_regions_count: usize,
    /// Content hash for portal regions (rect coordinates).
    /// Changes when portal positions/sizes change even if count stays the same.
    pub portal_content_hash: u64,
    /// Frame buffer width in physical pixels.
    pub width: u32,
    /// Frame buffer height in physical pixels.
    pub height: u32,
    /// Bits representation of scale factor float to allow exact comparison.
    pub scale_bits: u32,
    /// Content hash for material graph compilation results.
    /// Changes when a material's WGSL output changes (e.g., a Custom
    /// material node is modified) so the cached plan is invalidated
    /// rather than reused with stale shader bindings.
    /// P1-9 fix: previously the cache key did not include material
    /// compilation, so a material change would silently produce stale
    /// shader bindings on the next frame.
    pub material_compilation_hash: u64,
    /// The cached render graph DAG structure.
    pub graph: KvasirGraph,
    /// The compiled execution order of graph node keys.
    pub plan: Vec<NodeKey>,
}

impl CachedGraphPlan {
    /// Check if the cached graph configuration matches the incoming parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn matches(
        &self,
        has_glass: bool,
        has_bloom: bool,
        has_accessibility: bool,
        has_volumetric: bool,
        active_offscreens_count: usize,
        offscreen_content_hash: u64,
        portal_regions_count: usize,
        portal_content_hash: u64,
        width: u32,
        height: u32,
        scale_bits: u32,
        material_compilation_hash: u64,
    ) -> bool {
        self.has_glass == has_glass
            && self.has_bloom == has_bloom
            && self.has_accessibility == has_accessibility
            && self.has_volumetric == has_volumetric
            && self.active_offscreens_count == active_offscreens_count
            && self.offscreen_content_hash == offscreen_content_hash
            && self.portal_regions_count == portal_regions_count
            && self.portal_content_hash == portal_content_hash
            && self.width == width
            && self.height == height
            && self.scale_bits == scale_bits
            && self.material_compilation_hash == material_compilation_hash
    }
}

#[cfg(test)]
mod p1_9_tests {
    use super::*;

    fn make_plan(material_hash: u64) -> CachedGraphPlan {
        // We can't easily construct a KvasirGraph here, but matches() only
        // looks at the simple fields. Provide a default KvasirGraph and
        // empty plan -- matches() never reads graph/plan fields.
        CachedGraphPlan {
            has_glass: true,
            has_bloom: false,
            has_accessibility: false,
            has_volumetric: false,
            active_offscreens_count: 0,
            offscreen_content_hash: 0,
            portal_regions_count: 0,
            portal_content_hash: 0,
            width: 1280,
            height: 720,
            scale_bits: 1.0f32.to_bits(),
            material_compilation_hash: material_hash,
            graph: crate::kvasir::graph::KvasirGraph::new(),
            plan: Vec::new(),
        }
    }

    #[test]
    fn matches_returns_true_when_material_hash_matches() {
        let plan = make_plan(42);
        assert!(plan.matches(
            true,
            false,
            false,
            false,
            0,
            0,
            0,
            0,
            1280,
            720,
            1.0f32.to_bits(),
            42,
        ));
    }

    #[test]
    fn matches_returns_false_when_material_hash_changes() {
        // P1-9 regression: a material change must invalidate the cache,
        // even if all other fields are identical.
        let plan = make_plan(42);
        assert!(!plan.matches(
            true,
            false,
            false,
            false,
            0,
            0,
            0,
            0,
            1280,
            720,
            1.0f32.to_bits(),
            43, // different material hash
        ));
    }
}
