# cvkg-flow

```mermaid
graph TD
    cvkg-core["cvkg-core"]
    cvkg-vdom["cvkg-vdom"]
    cvkg-scene["cvkg-scene"]
    cvkg-layout["cvkg-layout"]
    cvkg-render-gpu["cvkg-render-gpu"]
    cvkg-render-native["cvkg-render-native"]
    cvkg-compositor["cvkg-compositor"]
    cvkg-themes["cvkg-themes"]
    cvkg-anim["cvkg-anim"]
    cvkg-flow["cvkg-flow"]
    cvkg-runic-text["cvkg-runic-text"]
    cvkg-svg-filters["cvkg-svg-filters"]
    cvkg-svg-serialize["cvkg-svg-serialize"]
    cvkg-components["cvkg-components"]
    cvkg-macros["cvkg-macros"]
    cvkg-cli["cvkg-cli"]
    cvkg-webkit-server["cvkg-webkit-server"]
    cvkg-test["cvkg-test"]
    cvkg-physics["cvkg-physics"]
    cvkg["cvkg (umbrella)"]

    cvkg-vdom --> cvkg-core
    cvkg-vdom --> cvkg-scene
    cvkg-layout --> cvkg-core
    cvkg-layout --> cvkg-anim
    cvkg-scene --> cvkg-core

    cvkg-render-gpu --> cvkg-core
    cvkg-render-gpu --> cvkg-compositor
    cvkg-render-gpu --> cvkg-svg-filters
    cvkg-render-gpu --> cvkg-svg-serialize
    cvkg-render-gpu --> cvkg-runic-text

    cvkg-render-native --> cvkg-core
    cvkg-render-native --> cvkg-render-gpu
    cvkg-render-native --> cvkg-vdom
    cvkg-render-native --> cvkg-themes

    cvkg-compositor --> cvkg-core

    cvkg-themes --> cvkg-core
    cvkg-themes --> cvkg-anim
    cvkg-anim --> cvkg-core
    cvkg-flow --> cvkg-core
    cvkg-flow --> cvkg-scene
    cvkg-flow --> cvkg-themes

    cvkg-runic-text --> cvkg-core
    cvkg-svg-filters --> cvkg-core

    cvkg-components --> cvkg-core
    cvkg-components --> cvkg-vdom
    cvkg-components --> cvkg-layout
    cvkg-components --> cvkg-themes
    cvkg-components --> cvkg-anim
    cvkg-components --> cvkg-runic-text

    cvkg-macros --> cvkg-core
    cvkg-cli --> cvkg-core
    cvkg-cli --> cvkg-physics
    cvkg-cli --> cvkg-anim
    cvkg-cli --> cvkg-macros
    cvkg-webkit-server --> cvkg-cli
    cvkg-physics --> cvkg-core
    cvkg-physics --> cvkg-scene

    cvkg --> cvkg-core
    cvkg --> cvkg-vdom
    cvkg --> cvkg-scene
    cvkg --> cvkg-layout
    cvkg --> cvkg-themes
    cvkg --> cvkg-anim
    cvkg --> cvkg-macros
    cvkg --> cvkg-components
    cvkg --> cvkg-render-gpu
    cvkg --> cvkg-render-native
```

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
