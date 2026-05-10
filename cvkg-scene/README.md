# cvkg-scene

**cvkg-scene** provides a retained scene graph for CVKG with hierarchical culling, automatic layering, and dirty-rect tracking.

## What This Crate Does

- Manages the tree structure of rendered nodes via `SceneGraph`
- Performs hierarchical AABB (Axis-Aligned Bounding Box) culling for visibility
- Provides automatic layering/batching for optimized GPU rendering
- Tracks dirty regions for differential updates

## What This Crate Does NOT Do

- Does not provide layout calculations (see cvkg-layout)
- Does not perform actual rendering (see cvkg-render-gpu)
- Does not handle window management

## Public API Overview

### SceneGraph

```rust
/// Unique identifier for a node in the scene graph
pub struct NodeId(pub u64);

/// A node in the retained scene graph
pub struct VNode {
    pub id: NodeId,
    pub component_type: String,
    pub children: Vec<NodeId>,
    pub local_rect: Rect,
    pub world_rect: Rect,
    pub is_dirty: bool,
    pub layer_id: u32,
    pub z_index: f32,
}

pub struct SceneGraph {
    pub nodes: HashMap<NodeId, VNode>,
    pub root: Option<NodeId>,
}
impl SceneGraph {
    /// Create a new empty scene graph
    pub fn new() -> Self;
    
    /// Generate a new unique NodeId
    pub fn next_id(&mut self) -> NodeId;
    
    /// Add a node to the graph and mark its region as dirty
    pub fn add_node(&mut self, node: VNode, parent: Option<NodeId>);
    
    /// Update world-space bounds after local changes
    pub fn update_transforms(&mut self);
    
    /// Perform hierarchical AABB culling; returns visible node IDs
    pub fn cull(&self, viewport: Rect) -> Vec<NodeId>;
    
    /// Group visible nodes into layers for optimized GPU rendering
    pub fn batch(&self, visible_nodes: &[NodeId]) -> HashMap<u32, Vec<NodeId>>;
}
```

### Node Types

```rust
impl VNode {
    /// Create a new node with the given ID, type, and local bounds
    pub fn new(id: NodeId, component_type: impl Into<String>, local_rect: Rect) -> Self;
}
```

## Usage Example

```rust
use cvkg_scene::{SceneGraph, VNode, NodeId};
use cvkg_core::Rect;

let mut scene = SceneGraph::new();

// Create root node
let root_id = scene.next_id();
let root = VNode::new(root_id, "root", Rect::new(0.0, 0.0, 800.0, 600.0));

scene.add_node(root, None);

// Add child node
let child_id = scene.next_id();
let child = VNode::new(child_id, "button", Rect::new(10.0, 10.0, 100.0, 40.0));

scene.add_node(child, Some(root_id));

// Update transforms
scene.update_transforms();

// Query visible nodes
let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);
let visible = scene.cull(viewport);

// Batch for rendering
let layers = scene.batch(&visible);
for (layer_id, node_ids) in layers {
    // Render each layer
}
```

## Known Limitations

- Culling is axis-aligned; rotated nodes may produce false positives
- Layer IDs are integer-based; overlapping layers may require explicit ordering
- No built-in animation support; external system must update node properties