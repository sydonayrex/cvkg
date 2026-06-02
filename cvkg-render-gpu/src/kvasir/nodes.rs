//! Kvasir render pass nodes for the Surtr renderer.
//!
//! Each node identifies a render pass by `PassId`. During execution,
//! `SurtrRenderer::execute_node()` dispatches to the correct encoding method.
//! This avoids importing renderer-internal types into the kvasir module.

use super::ExecutionContext;
use super::KvasirError;
use super::KvasirNode;
use super::ResourceRegistry;

/// Identifies which render pass a node represents.
/// The SurtrRenderer dispatches `execute_node(node.id(), ...)` to the correct encoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassId {
    /// Clear scene+depth, draw atmosphere, draw opaque geometry
    Geometry,
    /// Identity copy scene → blur texture
    BackdropCopy,
    /// Gaussian blur on backdrop texture (4 H+V iterations)
    BackdropBlur,
    /// Draw glass panels with backdrop blur sampling
    Glass,
    /// Draw UI overlay
    UI,
    /// Luminance-gated extract → bloom texture
    BloomExtract,
    /// Gaussian blur on bloom texture (2 H+V iterations)
    BloomBlur,
    /// Additive composite scene+bloom → swapchain + ACES tonemap
    Composite,
    /// Color blindness transform (final post-process)
    Accessibility,
    /// Present swapchain
    Present,
}

/// A render graph node.
pub struct PassNode {
    pub id: PassId,
    pub enabled: bool,
}

impl PassNode {
    pub const fn new(id: PassId) -> Self {
        Self { id, enabled: true }
    }

    pub const fn disabled(id: PassId) -> Self {
        Self { id, enabled: false }
    }
}

impl KvasirNode for PassNode {
    fn label(&self) -> &'static str {
        match self.id {
            PassId::Geometry => "geometry",
            PassId::BackdropCopy => "backdrop_copy",
            PassId::BackdropBlur => "backdrop_blur",
            PassId::Glass => "glass",
            PassId::UI => "ui",
            PassId::BloomExtract => "bloom_extract",
            PassId::BloomBlur => "bloom_blur",
            PassId::Composite => "composite",
            PassId::Accessibility => "accessibility",
            PassId::Present => "present",
        }
    }

    fn inputs(&self) -> &[super::ResourceId] {
        &[]
    }

    fn outputs(&self) -> &[super::ResourceId] {
        &[]
    }

    fn execute(
        &self,
        _ctx: &mut ExecutionContext<'_>,
        _registry: &mut ResourceRegistry,
    ) -> Result<(), KvasirError> {
        // The actual GPU encoding is performed by SurtrRenderer::execute_pass()
        // which is called from the graph execution loop with full &mut self access.
        // This placeholder exists to satisfy the trait; the real work happens
        // in the renderer's dispatch method.
        log::trace!("[Kvasir] {} (enabled={})", self.label(), self.enabled);
        Ok(())
    }
}

impl PassId {
    /// Returns `true` if this pass writes to the scene texture.
    pub const fn writes_scene(self) -> bool {
        matches!(
            self,
            PassId::Geometry | PassId::Glass | PassId::UI | PassId::Composite
        )
    }

    /// Returns `true` if this pass reads from the scene texture.
    pub const fn reads_scene(self) -> bool {
        matches!(
            self,
            PassId::BackdropCopy | PassId::BloomExtract | PassId::Composite | PassId::Present
        )
    }
}

// Re-export for use in graph construction
pub use PassId::*;

/// Helper: create the standard 10-pass frame graph.
/// Caller must supply the SurtrRenderer to execute it.
pub fn build_frame_graph(
    has_glass: bool,
    has_bloom: bool,
    accessibility_enabled: bool,
) -> Vec<PassNode> {
    let mut nodes = Vec::with_capacity(10);
    nodes.push(PassNode::new(Geometry));
    if has_glass {
        nodes.push(PassNode::new(BackdropCopy));
        nodes.push(PassNode::new(BackdropBlur));
        nodes.push(PassNode::new(Glass));
    }
    nodes.push(PassNode::new(UI));
    if has_bloom {
        nodes.push(PassNode::new(BloomExtract));
        nodes.push(PassNode::new(BloomBlur));
    }
    nodes.push(PassNode::new(Composite));
    nodes.push(if accessibility_enabled {
        PassNode::new(Accessibility)
    } else {
        PassNode::disabled(Accessibility)
    });
    nodes.push(PassNode::new(Present));
    nodes
}
