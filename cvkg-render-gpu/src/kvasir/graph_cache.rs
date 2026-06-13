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
    /// Number of active portal blur boundary regions.
    pub portal_regions_count: usize,
    /// Frame buffer width in physical pixels.
    pub width: u32,
    /// Frame buffer height in physical pixels.
    pub height: u32,
    /// Bits representation of scale factor float to allow exact comparison.
    pub scale_bits: u32,
    /// The cached render graph DAG structure.
    pub graph: KvasirGraph,
    /// The compiled execution order of graph node keys.
    pub plan: Vec<NodeKey>,
}

impl CachedGraphPlan {
    /// Check if the cached graph configuration matches the incoming parameters.
    pub fn matches(
        &self,
        has_glass: bool,
        has_bloom: bool,
        has_accessibility: bool,
        has_volumetric: bool,
        active_offscreens_count: usize,
        portal_regions_count: usize,
        width: u32,
        height: u32,
        scale_bits: u32,
    ) -> bool {
        self.has_glass == has_glass
            && self.has_bloom == has_bloom
            && self.has_accessibility == has_accessibility
            && self.has_volumetric == has_volumetric
            && self.active_offscreens_count == active_offscreens_count
            && self.portal_regions_count == portal_regions_count
            && self.width == width
            && self.height == height
            && self.scale_bits == scale_bits
    }
}
