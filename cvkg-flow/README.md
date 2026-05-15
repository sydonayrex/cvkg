# cvkg-flow

![CVKG Hero HUD](../docs/images/cvkg_hero.png)

`cvkg-flow` is a high-fidelity visual node graph editing engine built on top of CVKG, designed for complex data-flow and logic orchestration.

## Boundaries and Responsibilities

This crate provides the logic and UI components for node-based interfaces. It focuses on:
- **Graph Topology**: Managing the relationship between nodes, ports, and edges.
- **Infinite Canvas**: Providing a zoomable, pannable workspace for large-scale graphs.
- **Interaction Logic**: Handling connection dragging, node selection, and group movements.
- **Type Safety**: Ensuring port connections respect defined data types and flow directions.

## Public API Overview

### Core Types
- `FlowGraph`: The authoritative data structure representing the entire node network.
- `FlowNode`: An individual processing unit within the graph.
- `FlowPort`: An input or output anchor on a node.
- `FlowEdge`: A visual and logical connection between two ports.

### UI Components
- `FlowCanvas`: The primary view component for rendering and interacting with a `FlowGraph`.
- `FlowInteraction`: State manager for drag-and-drop and selection logic.

## Usage Example

```rust
use cvkg_flow::{FlowGraph, FlowNode, FlowCanvas};

fn MyNodeEditor() -> impl View {
    let mut graph = FlowGraph::new();
    let node_a = FlowNode::new("Input");
    let node_b = FlowNode::new("Process");
    
    graph.add_node(node_a);
    graph.add_node(node_b);
    
    FlowCanvas::new(graph)
        .background(Color::TACTICAL_OBSIDIAN)
}
```

## Known Limitations
- The engine is currently optimized for directed acyclic graphs (DAGs); cyclic dependencies require manual resolution logic.
- Performance in extremely large graphs (1000+ nodes) is dependent on the `cvkg-scene` culling efficiency.