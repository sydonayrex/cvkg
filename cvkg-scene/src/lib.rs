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
//                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     — Every major pub fn, unsafe block, and non-trivial algorithm in
//                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   — Check every tool call / command for progress every 30 seconds.
//                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//   CVKG Extended: Section 2 of the CVKG Design Specification

//! Scene graph, retained tree, diff/patch engine
//!
//! The scene graph layer maintains a retained mode tree of rendered nodes for efficient differential updates.

pub mod test_renderer;

use cvkg_core::View;

/// Unique identifier for nodes in the scene graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

impl NodeId {
    #[doc(hidden)]
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

/// Properties of a scene graph node
pub struct VNode {
    pub id: NodeId,
    pub component_type: &'static str,
    pub children: Vec<NodeId>,
}

impl VNode {
    #[doc(hidden)]
    pub fn new<T: View>(id: usize, component_type: &'static str) -> Self {
        Self {
            id: NodeId::new(id),
            component_type,
            children: Vec::new(),
        }
    }
}

/// The scene graph itself
pub struct SceneGraph {
    #[allow(dead_code)]
    nodes: Vec<VNode>,
    #[allow(dead_code)]
    root: Option<NodeId>,
}

impl SceneGraph {
    #[doc(hidden)]
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root: None,
        }
    }
}

/// Diff/patch engine for efficient updates
pub struct DiffEngine;

impl DiffEngine {
    #[doc(hidden)]
    pub fn new() -> Self {
        Self
    }

    /// Compute the difference between two scene graphs
    #[doc(hidden)]
    pub fn diff(&self, _old: &SceneGraph, _new: &SceneGraph) -> Vec<Patch> {
        Vec::new()
    }
}

/// A patch operation to apply to the scene graph
pub enum Patch {
    Create(VNode),
    Remove(NodeId),
    Update {
        id: NodeId,
        changes: Vec<Change>,
    },
    Move {
        id: NodeId,
        new_parent: NodeId,
        new_index: usize,
    },
}

/// A change to a VNode's properties
pub enum Change {
    ComponentType(&'static str),
    Children(Vec<NodeId>),
}
