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

//! High-performance retained scene graph for CVKG.
//! This crate implements hierarchical AABB culling, automatic layering,
//! and dirty-rect tracking for the Surtr GPU pipeline.

pub mod test_renderer;

// Spatial types are now provided by cvkg-spatial (crosscrate audit Finding #5).
// Re-exported here so existing downstream consumers of cvkg-scene::Quadtree keep working.
pub use cvkg_spatial::{Bvh, BvhNode, Quadtree, SpatialHash};

pub use cvkg_core::KvasirId;
use cvkg_core::Rect;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// Default cell size for the spatial hash grid.
const DEFAULT_CELL_SIZE: f32 = 64.0;

/// Unique identifier for a node in the retained scene graph.
///
/// # Crosscrate identity (crosscrate.md Finding #2)
///
/// This is a type alias for [`cvkg_core::KvasirId`], the platform-wide unique
/// identifier. Every crate (`cvkg-scene`, `cvkg-vdom`, `cvkg-flow`) uses the
/// same `KvasirId` type so that nodes can be referenced across crate
/// boundaries without conversion. Use `KvasirId::new()` to allocate a fresh id,
/// or `KvasirId::from(some_u64)` to wrap an existing literal.
pub type NodeId = KvasirId;

/// A node in the retained scene graph.
/// Section 3.2: "Retained tree of rendered nodes for efficient differential updates."
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VNode {
    pub id: NodeId,
    pub component_type: String,
    pub children: Vec<NodeId>,

    /// Local bounds relative to parent (2D fallback)
    pub local_rect: Rect,
    /// Absolute world-space bounds (computed during layout pass)
    pub world_rect: Rect,

    /// Whether this node or its children have changed since the last frame
    pub is_dirty: bool,

    /// Layer identifier for GPU batching (0 = default UI, 100 = Glass, etc.)
    pub layer_id: u32,

    /// Z-index for depth sorting (2D fallback)
    pub z_index: f32,

    /// Cached grid cells this node currently occupies
    #[serde(skip, default)]
    pub spatial_cells: Vec<(u32, u32)>,

    // ── 3D Transform (used when is_3d is true) ──────────────────────────────
    /// Whether this node uses 3D transforms. When true, the 3D fields below
    /// are authoritative and the 2D fields (local_rect, z_index) are derived.
    pub is_3d: bool,
    /// 3D world-space position. When is_3d is true, local_rect.x/y/z are derived from this.
    pub position_3d: [f32; 3],
    /// 3D rotation as quaternion (x, y, z, w). Default: identity.
    pub rotation_3d: [f32; 4],
    /// 3D scale. Default: (1, 1, 1).
    pub scale_3d: [f32; 3],
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
            spatial_cells: Vec::new(),
            is_3d: false,
            position_3d: [0.0, 0.0, 0.0],
            rotation_3d: [0.0, 0.0, 0.0, 1.0],
            scale_3d: [1.0, 1.0, 1.0],
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

    /// Spatial hash grid for fast AABB queries
    spatial_grid: HashMap<(u32, u32), Vec<NodeId>>,

    /// Cell size for the spatial hash grid
    cell_size: f32,
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
            spatial_grid: HashMap::new(),
            cell_size: DEFAULT_CELL_SIZE,
        }
    }

    /// Generate a new unique NodeId.
    pub fn next_id(&mut self) -> NodeId {
        let id = KvasirId(self.next_id);
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
    /// Also rebuilds the spatial hash index for fast AABB queries.
    pub fn update_transforms(&mut self) {
        if let Some(root_id) = self.root {
            let root_rect = self.nodes.get(&root_id).unwrap().local_rect;
            self.update_node_transform(root_id, root_rect);
        }
        self.rebuild_spatial_hash();
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
            // Early exit: if node is fully outside viewport, skip entire subtree
            if self.is_fully_outside(node.world_rect, viewport) {
                return;
            }

            // Fast path: if node is fully inside, add it and all descendants
            if self.is_fully_inside(node.world_rect, viewport) {
                self.add_all_descendants(id, visible);
                return;
            }

            // Partial overlap: add this node and check children individually
            visible.push(id);
            for child_id in &node.children {
                self.cull_node(*child_id, viewport, visible);
            }
        }
    }

    /// Check if rect a is fully outside rect b (no overlap at all).
    fn is_fully_outside(&self, a: Rect, b: Rect) -> bool {
        a.x + a.width <= b.x
            || a.x >= b.x + b.width
            || a.y + a.height <= b.y
            || a.y >= b.y + b.height
    }

    /// Check if rect a is fully inside rect b.
    fn is_fully_inside(&self, a: Rect, b: Rect) -> bool {
        a.x >= b.x
            && a.y >= b.y
            && a.x + a.width <= b.x + b.width
            && a.y + a.height <= b.y + b.height
    }

    /// Add a node and all its descendants to the visible list (fast path for fully-inside nodes).
    fn add_all_descendants(&self, id: NodeId, visible: &mut Vec<NodeId>) {
        if let Some(node) = self.nodes.get(&id) {
            visible.push(id);
            for child_id in &node.children {
                self.add_all_descendants(*child_id, visible);
            }
        }
    }

    /// Query the spatial hash for nodes that might overlap the given rect.
    /// Returns candidates that need further AABB testing.
    pub fn query_region(&self, rect: Rect) -> Vec<NodeId> {
        let mut candidates = Vec::new();
        // P1-16 fix: compute cell coordinates as signed i32 first to
        // handle negative rect.x/rect.y (common in scrolled/panned
        // canvases, negative camera offsets, etc.). The previous code
        // used `as u32` on a possibly-negative f32, which saturates to
        // 0 and collapses all negative-coordinate content into bucket
        // (0,0) -- defeating the spatial index for panned scenes.
        let min_cell_x = (rect.x / self.cell_size).floor() as i32;
        let min_cell_y = (rect.y / self.cell_size).floor() as i32;
        let max_cell_x = ((rect.x + rect.width) / self.cell_size).floor() as i32;
        let max_cell_y = ((rect.y + rect.height) / self.cell_size).floor() as i32;

        for cx in min_cell_x..=max_cell_x {
            for cy in min_cell_y..=max_cell_y {
                if let Some(key) = encode_cell_key(cx, cy)
                    && let Some(cell) = self.spatial_grid.get(&key)
                {
                    for id in cell {
                        if let Some(node) = self.nodes.get(id)
                            && self.intersects(node.world_rect, rect)
                        {
                            candidates.push(*id);
                        }
                    }
                }
            }
        }
        candidates
    }

    /// Rebuild the spatial hash index from all node world rects.
    fn rebuild_spatial_hash(&mut self) {
        self.spatial_grid.clear();
        for (id, node) in &self.nodes {
            // P1-16 fix: signed i32 cell coords (see query_region).
            let min_cell_x = (node.world_rect.x / self.cell_size).floor() as i32;
            let min_cell_y = (node.world_rect.y / self.cell_size).floor() as i32;
            let max_cell_x =
                ((node.world_rect.x + node.world_rect.width) / self.cell_size).floor() as i32;
            let max_cell_y =
                ((node.world_rect.y + node.world_rect.height) / self.cell_size).floor() as i32;

            for cx in min_cell_x..=max_cell_x {
                for cy in min_cell_y..=max_cell_y {
                    if let Some(key) = encode_cell_key(cx, cy) {
                        self.spatial_grid.entry(key).or_default().push(*id);
                    }
                }
            }
        }
    }

    fn intersects(&self, a: Rect, b: Rect) -> bool {
        a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
    }

    /// Perform Automatic Layering (Batching).
    /// Groups visible nodes into discrete layers for optimized GPU rendering.
    /// Returns a BTreeMap so layers are always iterated in order.
    /// Nodes within each layer are sorted by z_index.
    pub fn batch(&self, visible_nodes: &[NodeId]) -> BTreeMap<u32, Vec<NodeId>> {
        let mut layers: BTreeMap<u32, Vec<NodeId>> = BTreeMap::new();
        for id in visible_nodes {
            if let Some(node) = self.nodes.get(id) {
                layers.entry(node.layer_id).or_default().push(*id);
            }
        }
        // Sort nodes within each layer by z_index for correct draw order
        //
        // P1-18 fix: the previous sort key was `(n.z_index * 1000.0) as i64`,
        // which truncates z_indices differing by less than 0.001 to the same
        // integer. `sort_by_key` is stable, so ties preserved insertion order
        // rather than z_index order -- correct in practice but fragile.
        // `total_cmp` gives exact IEEE-754 total ordering for floats, which
        // handles NaN consistently and avoids the truncation hazard.
        for nodes in layers.values_mut() {
            nodes.sort_by(|a, b| {
                let za = self.nodes.get(a).map(|n| n.z_index).unwrap_or(0.0);
                let zb = self.nodes.get(b).map(|n| n.z_index).unwrap_or(0.0);
                za.total_cmp(&zb)
            });
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
            next_id: 0,
            cell_size: DEFAULT_CELL_SIZE,
            spatial_grid: HashMap::new(),
        })
    }

    /// Get the dirty regions for the current frame.
    pub fn dirty_regions(&self) -> &[Rect] {
        &self.dirty_regions
    }

    /// Clear dirty flags and regions after a successful render.
    pub fn clear_dirty(&mut self) {
        // Merge overlapping dirty regions before clearing
        self.merge_dirty_regions();
        for node in self.nodes.values_mut() {
            node.is_dirty = false;
        }
        self.dirty_regions.clear();
    }

    /// Merge overlapping dirty rects to reduce the number of regions.
    /// Uses quadtree-based spatial intersection index.
    fn merge_dirty_regions(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;
            let len = self.dirty_regions.len();
            if len <= 1 {
                break;
            }

            let mut min_x = f32::MAX;
            let mut min_y = f32::MAX;
            let mut max_x = f32::MIN;
            let mut max_y = f32::MIN;
            for r in &self.dirty_regions {
                min_x = min_x.min(r.x);
                min_y = min_y.min(r.y);
                max_x = max_x.max(r.x + r.width);
                max_y = max_y.max(r.y + r.height);
            }

            let bounds = Rect {
                x: min_x,
                y: min_y,
                width: max_x - min_x,
                height: max_y - min_y,
            };
            let mut qt = Quadtree::new(bounds);

            for r in &self.dirty_regions {
                qt.insert(*r);
            }

            'outer: for i in 0..len {
                let mut candidates = Vec::new();
                qt.retrieve(self.dirty_regions[i], &mut candidates);

                for candidate in candidates {
                    // Skip self comparison or identical rects (since they will merge trivially but we need to remove one)
                    // If they are exactly identical, we handle it too.
                    if let Some(j) = self.dirty_regions.iter().position(|r| *r == candidate)
                        && i != j
                        && let Some(union) =
                            rect_union(self.dirty_regions[i], self.dirty_regions[j])
                    {
                        self.dirty_regions[i] = union;
                        self.dirty_regions.remove(j);
                        changed = true;
                        break 'outer;
                    }
                }
            }
        }
    }

    /// Apply multiple patches in sequence.
    pub fn apply_patches(&mut self, patches: &[Patch]) {
        for patch in patches {
            self.apply_patch(patch.clone());
        }
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

/// A patch operation to apply to the retained scene graph.
#[derive(Clone)]
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
#[derive(Clone)]
pub enum Change {
    ComponentType(String),
    Children(Vec<NodeId>),
    LocalRect(Rect),
    LayerId(u32),
    ZIndex(f32),
}

/// P1-16: Encode signed (i32, i32) cell coordinates as a (u32, u32) key
/// for the spatial grid HashMap. The offset is 1 << 30 (~1 billion)
/// which is large enough to handle reasonable panned scenes (up to
/// ~32M cells in either direction at typical cell sizes of 64 pixels).
///
/// Returns None if the cell coordinates are out of range (the offset
/// cannot represent them). In practice the spatial grid should never
/// encounter cells outside this range, but the Option return makes the
/// overflow case explicit and safe (the cell is silently dropped rather
/// than panicking on arithmetic overflow).
fn encode_cell_key(cx: i32, cy: i32) -> Option<(u32, u32)> {
    const OFFSET: i64 = 1i64 << 30; // ~1.07 billion
    let x = (cx as i64) + OFFSET;
    let y = (cy as i64) + OFFSET;
    if x < 0 || x > u32::MAX as i64 || y < 0 || y > u32::MAX as i64 {
        return None;
    }
    Some((x as u32, y as u32))
}

/// Compute the union (bounding box) of two rects.
/// Returns None if the rects don't overlap or touch.
fn rect_union(a: Rect, b: Rect) -> Option<Rect> {
    if !rects_overlap(a, b) {
        return None;
    }
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    let x2 = (a.x + a.width).max(b.x + b.width);
    let y2 = (a.y + a.height).max(b.y + b.height);
    Some(Rect {
        x,
        y,
        width: x2 - x,
        height: y2 - y,
    })
}

/// Check if two rects overlap (including edge-touching).
fn rects_overlap(a: Rect, b: Rect) -> bool {
    a.x <= b.x + b.width && a.x + a.width >= b.x && a.y <= b.y + b.height && a.y + a.height >= b.y
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

    // P1-18 regression: z_index values differing by less than 0.001 must
    // still sort in correct order, not be truncated to the same int key.
    #[test]
    fn test_batch_z_index_preserves_sub_milli_ordering() {
        let mut scene = SceneGraph::new();
        let a = scene.next_id();
        let b = scene.next_id();
        let c = scene.next_id();
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        };

        // z values differ by 0.0001, well below the old (z * 1000.0) as i64
        // truncation threshold of 0.001.
        let mut na = VNode::new(a, "R", rect);
        na.z_index = 0.5;
        scene.add_node(na, None);

        let mut nb = VNode::new(b, "R", rect);
        nb.z_index = 0.5001; // between a and c
        scene.add_node(nb, None);

        let mut nc = VNode::new(c, "R", rect);
        nc.z_index = 0.5002;
        scene.add_node(nc, None);

        let layers = scene.batch(&[a, b, c]);
        let layer0 = layers.get(&0).expect("layer 0 should exist");
        assert_eq!(layer0.len(), 3, "all 3 nodes should be in layer 0");
        // Must be sorted in ascending z_index order: a (0.5), b (0.5001), c (0.5002).
        assert_eq!(
            layer0,
            &vec![a, b, c],
            "z_index sub-milli ordering must be preserved: got {:?}",
            layer0
        );

        // Also test the reverse order: insert in descending z, expect ascending after sort.
        let mut scene2 = SceneGraph::new();
        let x = scene2.next_id();
        let y = scene2.next_id();
        let mut nx = VNode::new(x, "R", rect);
        nx.z_index = 1.0;
        scene2.add_node(nx, None);
        let mut ny = VNode::new(y, "R", rect);
        ny.z_index = -1.0;
        scene2.add_node(ny, None);
        let layers2 = scene2.batch(&[x, y]);
        let l0 = layers2.get(&0).unwrap();
        assert_eq!(l0, &vec![y, x], "negative z must sort before positive z");
    }

    // P1-16 regression: spatial hash must handle negative world rect
    // coordinates (e.g., panned canvases) without collapsing all
    // negative-coordinate content into bucket (0, 0). The previous
    // implementation used `as u32` on a negative f32 which saturated
    // to 0, defeating the spatial index.
    #[test]
    fn test_query_region_negative_basic() {
        // Test with a single node at negative coord.
        let mut scene = SceneGraph::new();
        let id = scene.next_id();
        let mut root = VNode::new(id, "R", Rect { x: 0.0, y: 0.0, width: 100.0, height: 100.0 });
        scene.add_node(root, None);
        scene.update_transforms();
        scene.nodes.get_mut(&id).unwrap().world_rect =
            Rect { x: -200.0, y: 0.0, width: 50.0, height: 50.0 };
        scene.rebuild_spatial_hash();
        // At cell_size=64, -200/64 = -3.125, floor = -4
        // Cell key for (-4, 0) should be (OFFSET-4, OFFSET+0)
        // Query at (-190, 0, 20, 20): cell x range is -3 to -2 (positive direction)
        // Hmm, the node at cell -4 with width 50 spans cells -4 to -3
        let result = scene.query_region(Rect { x: -190.0, y: 0.0, width: 20.0, height: 20.0 });
        assert!(result.contains(&id), "negative coord test failed, got {:?}", result);
    }

    #[test]
    fn test_query_region_handles_negative_coordinates() {
        let mut scene = SceneGraph::new();
        let id_a = scene.next_id();
        let id_b = scene.next_id();
        let id_c = scene.next_id();
        let rect_small = Rect { x: 0.0, y: 0.0, width: 10.0, height: 10.0 };

        // Three nodes at distinct world positions: A at (-5000, 0),
        // B at (5000, 0), C at (0, 0). We add them as siblings under
        // a single root, then set their world_rects directly after
        // update_transforms (which otherwise recomputes world_rect from
        // local_rect + parent).
        let mut root = VNode::new(id_c, "C_root", rect_small);
        scene.add_node(root, None);
        scene.add_node(VNode::new(id_a, "A", rect_small), Some(id_c));
        scene.add_node(VNode::new(id_b, "B", rect_small), Some(id_c));
        scene.update_transforms();

        // Now manually set the world_rects to where we want them.
        scene.nodes.get_mut(&id_a).unwrap().world_rect =
            Rect { x: -5000.0, y: 0.0, width: 10.0, height: 10.0 };
        scene.nodes.get_mut(&id_b).unwrap().world_rect =
            Rect { x: 5000.0, y: 0.0, width: 10.0, height: 10.0 };
        scene.nodes.get_mut(&id_c).unwrap().world_rect =
            Rect { x: 0.0, y: 0.0, width: 10.0, height: 10.0 };
        scene.rebuild_spatial_hash();

        // Verify the world_rects are set as expected.
        assert_eq!(scene.nodes.get(&id_a).unwrap().world_rect.x, -5000.0);
        assert_eq!(scene.nodes.get(&id_b).unwrap().world_rect.x, 5000.0);
        assert_eq!(scene.nodes.get(&id_c).unwrap().world_rect.x, 0.0);

        // Debug: dump state (remove for final)
        // eprintln!("A world_rect: {:?}", scene.nodes.get(&id_a).unwrap().world_rect);
        // ...

        // Query near A (overlapping, not just touching):
        // A spans x = [-5000, -4990]. Query must overlap, so use x = [-4995, -4975].
        let query_a = Rect { x: -4995.0, y: 0.0, width: 20.0, height: 20.0 };
        let result_a = scene.query_region(query_a);
        assert!(
            result_a.contains(&id_a),
            "query near A must include A, got {:?}",
            result_a
        );
        assert!(
            !result_a.contains(&id_b),
            "query near A must NOT include B (different cell), got {:?}",
            result_a
        );
        assert!(
            !result_a.contains(&id_c),
            "query near A must NOT include C (different cell), got {:?}",
            result_a
        );

        // Query near C: must return C only.
        let query_c = Rect { x: -5.0, y: -5.0, width: 20.0, height: 20.0 };
        let result_c = scene.query_region(query_c);
        assert!(result_c.contains(&id_c), "query near C must include C");
        assert!(!result_c.contains(&id_a), "query near C must NOT include A");
        assert!(!result_c.contains(&id_b), "query near C must NOT include B");

        // Query spanning A and C: must include both.
        let query_span = Rect { x: -4995.0, y: 0.0, width: 5000.0, height: 20.0 };
        let result_span = scene.query_region(query_span);
        assert!(result_span.contains(&id_a), "span must include A");
        assert!(result_span.contains(&id_c), "span must include C");
    }

    #[test]
    fn test_query_region_basic_works() {
        // Sanity check: spatial hash works for positive coords.
        let mut scene = SceneGraph::new();
        let id = scene.next_id();
        let mut root = VNode::new(id, "R", Rect { x: 0.0, y: 0.0, width: 100.0, height: 100.0 });
        scene.add_node(root, None);
        scene.update_transforms();
        scene.nodes.get_mut(&id).unwrap().world_rect =
            Rect { x: 100.0, y: 100.0, width: 50.0, height: 50.0 };
        scene.rebuild_spatial_hash();
        let result = scene.query_region(Rect { x: 110.0, y: 110.0, width: 10.0, height: 10.0 });
        assert!(result.contains(&id), "positive coord test failed, got {:?}", result);
    }

    #[test]
    fn test_encode_cell_key_handles_negative() {
        // Negative cell coords should produce distinct, non-zero keys.
        let k_neg = encode_cell_key(-5, -10).unwrap();
        let k_pos = encode_cell_key(5, 10).unwrap();
        let k_zero = encode_cell_key(0, 0).unwrap();
        assert_ne!(k_neg, k_pos, "negative and positive cells must not collide");
        assert_ne!(k_neg, k_zero, "negative cells must not collide with origin");
        // All three keys must be valid u32 pairs.
        let _ = (k_neg.0, k_neg.1);
        let _ = (k_pos.0, k_pos.1);
    }
}
