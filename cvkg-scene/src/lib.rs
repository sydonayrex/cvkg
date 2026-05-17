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

//! High-performance retained scene graph for CVKG.
//! This crate implements hierarchical AABB culling, automatic layering,
//! and dirty-rect tracking for the Surtr GPU pipeline.

pub mod test_renderer;

use cvkg_core::Rect;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a node in the retained scene graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

/// A node in the retained scene graph.
/// Section 3.2: "Retained tree of rendered nodes for efficient differential updates."
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VNode {
    pub id: NodeId,
    pub component_type: String,
    pub children: Vec<NodeId>,

    /// Local bounds relative to parent
    pub local_rect: Rect,
    /// Absolute world-space bounds (computed during layout pass)
    pub world_rect: Rect,

    /// Whether this node or its children have changed since the last frame
    pub is_dirty: bool,

    /// Layer identifier for GPU batching (0 = default UI, 100 = Glass, etc.)
    pub layer_id: u32,

    /// Z-index for depth sorting
    pub z_index: f32,
}

impl VNode {
    pub fn new(id: NodeId, component_type: impl Into<String>, local_rect: Rect) -> Self {
        Self {
            id,
            component_type: component_type.into(),
            children: Vec::new(),
            local_rect,
            world_rect: local_rect,
            is_dirty: true,
            layer_id: 0,
            z_index: 0.0,
        }
    }
}

/// The Retained Scene Graph.
/// Manages the tree structure and performs high-performance queries (culling, batching).
pub struct SceneGraph {
    pub nodes: HashMap<NodeId, VNode>,
    pub root: Option<NodeId>,

    /// Accumulated dirty regions for the current frame
    dirty_regions: Vec<Rect>,

    /// Next available unique ID
    next_id: u64,
}

impl Default for SceneGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneGraph {
    /// Create a new empty scene graph.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root: None,
            dirty_regions: Vec::new(),
            next_id: 1,
        }
    }

    /// Generate a new unique NodeId.
    pub fn next_id(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Add a node to the graph and mark its region as dirty.
    pub fn add_node(&mut self, mut node: VNode, parent: Option<NodeId>) {
        let id = node.id;
        node.is_dirty = true;
        self.dirty_regions.push(node.world_rect);

        if let Some(parent_id) = parent {
            if let Some(p) = self.nodes.get_mut(&parent_id) {
                p.children.push(id);
                p.is_dirty = true;
            }
        } else if self.root.is_none() {
            self.root = Some(id);
        }

        self.nodes.insert(id, node);
    }

    /// Update the world-space bounds of all nodes (Recursive).
    /// This should be called after any local changes to ensure culling is accurate.
    pub fn update_transforms(&mut self) {
        if let Some(root_id) = self.root {
            let root_rect = self.nodes.get(&root_id).unwrap().local_rect;
            self.update_node_transform(root_id, root_rect);
        }
    }

    fn update_node_transform(&mut self, id: NodeId, parent_world_rect: Rect) {
        let node = self.nodes.get_mut(&id).unwrap();

        // Compute new world rect based on parent offset
        let old_world_rect = node.world_rect;
        node.world_rect = Rect {
            x: parent_world_rect.x + node.local_rect.x,
            y: parent_world_rect.y + node.local_rect.y,
            width: node.local_rect.width,
            height: node.local_rect.height,
        };

        if node.world_rect != old_world_rect {
            node.is_dirty = true;
            self.dirty_regions.push(old_world_rect);
            self.dirty_regions.push(node.world_rect);
        }

        let children = node.children.clone();
        let world_rect = node.world_rect;

        for child_id in children {
            self.update_node_transform(child_id, world_rect);
        }
    }

    /// Perform Hierarchical AABB Culling.
    /// Returns a list of NodeIds that are visible within the provided viewport.
    pub fn cull(&self, viewport: Rect) -> Vec<NodeId> {
        let mut visible = Vec::new();
        if let Some(root_id) = self.root {
            self.cull_node(root_id, viewport, &mut visible);
        }
        visible
    }

    fn cull_node(&self, id: NodeId, viewport: Rect, visible: &mut Vec<NodeId>) {
        if let Some(node) = self.nodes.get(&id) {
            // Check if node's world bounds intersect the viewport
            if self.intersects(node.world_rect, viewport) {
                visible.push(id);

                // Recurse to children
                for child_id in &node.children {
                    self.cull_node(*child_id, viewport, visible);
                }
            }
        }
    }

    fn intersects(&self, a: Rect, b: Rect) -> bool {
        a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
    }

    /// Perform Automatic Layering (Batching).
    /// Groups visible nodes into discrete layers for optimized GPU rendering.
    pub fn batch(&self, visible_nodes: &[NodeId]) -> HashMap<u32, Vec<NodeId>> {
        let mut layers = HashMap::new();
        for id in visible_nodes {
            if let Some(node) = self.nodes.get(id) {
                layers
                    .entry(node.layer_id)
                    .or_insert_with(Vec::new)
                    .push(*id);
            }
        }
        layers
    }

    /// Binary Serialization using bincode for sub-millisecond sync.
    pub fn serialize_binary(&self) -> Result<Vec<u8>, bincode::Error> {
        // We only serialize the nodes and root to keep the payload minimal
        let data = (&self.nodes, &self.root);
        bincode::serialize(&data)
    }

    /// Deserialize a scene graph from binary data.
    pub fn deserialize_binary(data: &[u8]) -> Result<Self, bincode::Error> {
        let (nodes, root): (HashMap<NodeId, VNode>, Option<NodeId>) = bincode::deserialize(data)?;
        Ok(Self {
            nodes,
            root,
            dirty_regions: Vec::new(),
            next_id: 0, // Should be re-calculated or preserved if needed
        })
    }

    /// Get the dirty regions for the current frame.
    pub fn dirty_regions(&self) -> &[Rect] {
        &self.dirty_regions
    }

    /// Clear dirty flags and regions after a successful render.
    pub fn clear_dirty(&mut self) {
        for node in self.nodes.values_mut() {
            node.is_dirty = false;
        }
        self.dirty_regions.clear();
    }

    /// Apply a retained scene graph patch.
    /// Section 3.2: "Retained tree of rendered nodes for efficient differential updates."
    pub fn apply_patch(&mut self, patch: Patch) {
        match patch {
            Patch::Create(node) => {
                self.add_node(node, None); // Root case or handled by parent update
            }
            Patch::Remove(id) => {
                if let Some(node) = self.nodes.remove(&id) {
                    self.dirty_regions.push(node.world_rect);
                    // Remove from parent's children
                    for p in self.nodes.values_mut() {
                        p.children.retain(|&c| c != id);
                    }
                }
            }
            Patch::Update { id, changes } => {
                for change in changes {
                    self.apply_change(id, change);
                }
            }
            Patch::Move {
                id,
                new_parent,
                new_index,
            } => {
                // Remove from old parent
                for p in self.nodes.values_mut() {
                    p.children.retain(|&c| c != id);
                }
                // Add to new parent
                if let Some(p) = self.nodes.get_mut(&new_parent) {
                    p.children.insert(new_index.min(p.children.len()), id);
                    p.is_dirty = true;
                }
            }
        }
    }

    fn apply_change(&mut self, id: NodeId, change: Change) {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.is_dirty = true;
            match change {
                Change::ComponentType(t) => node.component_type = t,
                Change::Children(c) => node.children = c,
                Change::LocalRect(r) => {
                    self.dirty_regions.push(node.world_rect);
                    node.local_rect = r;
                }
                Change::LayerId(l) => node.layer_id = l,
                Change::ZIndex(z) => node.z_index = z,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_graph_culling() {
        let mut scene = SceneGraph::new();
        let id1 = scene.next_id();
        let node1 = VNode::new(
            id1,
            "Rect",
            Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
        );
        scene.add_node(node1, None);

        let id2 = scene.next_id();
        let mut node2 = VNode::new(
            id2,
            "Rect",
            Rect {
                x: 150.0,
                y: 0.0,
                width: 50.0,
                height: 50.0,
            },
        );
        node2.layer_id = 1;
        scene.add_node(node2, Some(id1));

        scene.update_transforms();

        // Culling with viewport that only sees node 1
        let visible = scene.cull(Rect {
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
        });
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0], id1);

        // Culling with viewport that sees both
        let visible = scene.cull(Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 100.0,
        });
        assert_eq!(visible.len(), 2);

        // Batching
        let batches = scene.batch(&visible);
        assert_eq!(batches.len(), 2);
        assert_eq!(batches.get(&0).unwrap().len(), 1);
        assert_eq!(batches.get(&1).unwrap().len(), 1);
    }

    #[test]
    fn test_scene_graph_dirty_tracking() {
        let mut scene = SceneGraph::new();
        let id = scene.next_id();
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        scene.add_node(VNode::new(id, "Rect", rect), None);

        assert_eq!(scene.dirty_regions().len(), 1);
        assert_eq!(scene.dirty_regions()[0], rect);

        scene.clear_dirty();
        assert_eq!(scene.dirty_regions().len(), 0);
    }
}

/// A patch operation to apply to the retained scene graph.
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

/// A change to a VNode's properties.
pub enum Change {
    ComponentType(String),
    Children(Vec<NodeId>),
    LocalRect(Rect),
    LayerId(u32),
    ZIndex(f32),
}
