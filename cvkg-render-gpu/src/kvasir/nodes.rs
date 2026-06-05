//! Kvasir render pass nodes for the Surtr renderer.
//!
//! Each node identifies a render pass by `PassId`. During execution,
//! `SurtrRenderer::execute_node()` dispatches to the correct encoding method.
//! This avoids importing renderer-internal types into the kvasir module.

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

// Re-export for use in graph construction
pub use PassId::*;

/// Helper: create the standard 10-pass execution sequence.
/// Caller must supply the SurtrRenderer to execute it.
pub fn build_pass_sequence(
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
