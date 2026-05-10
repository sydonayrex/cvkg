# cvkg-flow

**cvkg-flow** provides node-based graph (flow) UI components for CVKG applications.

## What This Crate Does

- Provides `FlowGraph` container for node-based UI
- Provides `FlowNode` component for individual nodes
- Provides `FlowEdge` component for connections between nodes
- Handles node interaction events (select, move, connect)

## What This Crate Does NOT Do

- Does not provide rendering (see cvkg-render-gpu)
- Does not provide layout for non-flow content
- Does not handle graph algorithms (topological sort, etc.)

## Public API Overview

### FlowGraph

```rust
/// Container for a node-based diagram
pub struct FlowGraph {
    nodes: Vec<FlowNode>,
    edges: Vec<FlowEdge>,
}
impl FlowGraph {
    /// Create a new empty flow graph
    pub fn new() -> Self;
    
    /// Add a node to the graph
    pub fn add_node(&mut self, node: FlowNode) -> usize;
    
    /// Add an edge between two nodes
    pub fn add_edge(&mut self, edge: FlowEdge);
    
    /// Get a node by index
    pub fn node(&self, index: usize) -> Option<&FlowNode>;
}
```

### FlowNode

```rust
/// A node in the flow graph
pub struct FlowNode {
    id: String,
    position: [f32; 2],
    size: [f32; 2],
    inputs: Vec<Port>,
    outputs: Vec<Port>,
}
impl FlowNode {
    /// Create a new node with the given ID and position
    pub fn new(id: impl Into<String>, position: [f32; 2]) -> Self;
    
    /// Set the node size
    pub fn size(mut self, size: [f32; 2]) -> Self;
}
```

### FlowEdge

```rust
/// A connection between two nodes
pub struct FlowEdge {
    from_node: String,
    from_port: usize,
    to_node: String,
    to_port: usize,
}```

## Usage Example

```rust
use cvkg_flow::{FlowGraph, FlowNode};

let mut graph = FlowGraph::new();

let node1 = FlowNode::new("input", [100.0, 100.0]);
let node2 = FlowNode::new("output", [300.0, 100.0]);

graph.add_node(node1);
graph.add_node(node2);
```

## Known Limitations

- Node positions are in screen coordinates, not grid units
- No built-in serialization format
- Port matching is not type-safe