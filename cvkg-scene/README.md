# cvkg-scene

![CVKG Hero HUD](../docs/images/cvkg_hero.png)

`cvkg-scene` implements a high-performance retained scene graph for CVKG, providing hierarchical culling, automatic layering, and binary serialization for the GPU pipeline.

## Boundaries and Responsibilities

This crate serves as the bridge between the logical UI (VDOM) and the GPU renderer (Surtr). It does NOT handle event logic or layout calculation. It focuses on:
- Maintaining a retained tree of rendered nodes for efficient differential updates.
- Performing hierarchical Axis-Aligned Bounding Box (AABB) culling.
- Grouping visible nodes into layers for GPU batching.
- Tracking dirty regions to minimize redraw areas.

## Public API Overview

### Core Types
- `SceneGraph`: The central manager for retained nodes and spatial queries.
- `VNode`: A node in the scene graph containing world-space bounds and layering metadata.
- `NodeId`: Unique identifier for scene nodes.

### Spatial Operations
- `SceneGraph::update_transforms()`: Recursively computes absolute world-space bounds from local coordinates.
- `SceneGraph::cull(viewport)`: Returns the IDs of all nodes visible within the given rect.
- `SceneGraph::batch(visible_nodes)`: Organizes nodes into layer-specific buckets for rendering.

### Synchronization
- `SceneGraph::serialize_binary()`: High-speed bincode-based serialization for sub-millisecond state transfer.

## Usage Example

```rust
use cvkg_scene::{SceneGraph, VNode};
use cvkg_core::Rect;

let mut scene = SceneGraph::new();

// Add a node
let id = scene.next_id();
let node = VNode::new(id, "Rect", Rect { x: 10.0, y: 10.0, width: 100.0, height: 100.0 });
scene.add_node(node, None);

// Update transforms and cull
scene.update_transforms();
let visible = scene.cull(Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 });

// Batch for rendering
let batches = scene.batch(&visible);
```

## Known Limitations
- Culling is currently based on simple AABB intersections; complex non-convex clipping is handled at the shader level.
- The scene graph assumes a 2D coordinate system with Z-depth layering.