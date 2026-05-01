# Flow Plan: Node-Based Graph Components for CVKG

## Executive Summary

This document outlines the implementation plan for integrating xyflow-inspired node-based UI concepts into CVKG (Cyber Viking GUI). The goal is to create a `cvkg-flow` crate that enables workflow builders, visual programming interfaces, and graph editors within the CVKG ecosystem.

---

## Why xyflow Concepts Should Be Integrated

### CVKG's Current State
- **Has**: Retained scene graph (VNode/SceneGraph) for rendering optimization
- **Has**: Layout system (HStack, VStack, Grid) for traditional UI
- **Has**: Component library with cards, lists, tables, navigation
- **Missing**: Node-based editing, graph manipulation, flow visualization

### xyflow's Value Proposition
1. **Node-based Editing**: Interactive canvas with draggable nodes
2. **Graph Connectivity**: Edges connecting node ports with smart routing
3. **Workflow Builders**: Perfect for no-code platforms and automation
4. **Visual Programming**: Ideal for AI agent interfaces and graph manipulation
5. **Ready-to-use Patterns**: Proven patterns from React Flow ecosystem

### Strategic Fit for CVKG
- Enables new UI paradigms within the Cyber Viking aesthetic
- Complements existing components as a specialized visualization tool
- Aligns with agentic/AI interface needs for graph manipulation
- Fills gap in current component library

---

## Core Concepts to Implement

### 1. FlowCanvas (Viewport)
```rust
/// Infinite canvas viewport with pan/zoom
pub struct FlowCanvas {
    /// Current viewport offset
    pub offset: (f32, f32),
    /// Current zoom scale
    pub scale: f32,
    /// Min/max zoom bounds
    pub min_scale: f32,
    pub max_scale: f32,
}
```

### 2. FlowNode
```rust
/// A node in the flow graph
pub struct FlowNode {
    pub id: NodeId,
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub node_type: NodeType,
    pub data: NodeData,
    pub selected: bool,
}
```

### 3. FlowPort
```rust
/// Connection port on a node
pub struct FlowPort {
    pub id: PortId,
    pub node_id: NodeId,
    pub position: PortPosition,
    pub direction: PortDirection,
    pub connections: Vec<EdgeId>,
}
```

### 4. FlowEdge
```rust
/// Connection between two ports
pub struct FlowEdge {
    pub id: EdgeId,
    pub source: PortId,
    pub target: PortId,
    pub path: EdgePath,
    pub selected: bool,
}
```

---

## Implementation Phases

### Phase 1: Core Data Structures and Types (Week 1-2)
- `cvkg-flow` crate creation in workspace
- Core types: NodeId, PortId, EdgeId, NodeType enum
- FlowCanvas, FlowNode, FlowPort, FlowEdge structs
- Basic serialization support (serde)

### Phase 2: Rendering and Visual Styles (Week 2-3)
- Node rendering with rounded rectangles and shadows
- Port visualization
- Edge rendering with path interpolation
- Selection overlays and hover states
- Glass morphism styling (Cyber Viking aesthetic)

### Phase 3: Interaction and Events (Week 3-4)
- Mouse/touch event handling
- Drag-and-drop for nodes
- Selection box (marquee) selection
- Edge creation on port drag
- Delete/duplicate node operations

### Phase 4: Advanced Features (Week 4-5)
- Mini-map overview panel
- Grid snapping and alignment guides
- Undo/redo stack
- Copy/paste functionality
- Keyboard shortcuts

### Phase 5: Integration and Polish (Week 5-6)
- Example applications (workflow editor, graph visualizer)
- Documentation and examples
- Performance optimization for large graphs
- TypeScript bindings for web usage

---

## Rust Implementation Details

### Core Module Structure
```rust
// cvkg-flow/src/lib.rs
pub mod canvas;
pub mod node;
pub mod port;
pub mod edge;
pub mod event;
pub mod layout;

pub use canvas::FlowCanvas;
pub use node::FlowNode;
pub use port::FlowPort;
pub use edge::FlowEdge;
```

### Type Definitions
```rust
// cvkg-flow/src/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Default,
    Input,
    Output,
    Group,
    Annotation,
}
```

---

## File Structure
```
cvkg-flow/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs          # Module exports
    ├── types.rs        # Core type definitions
    ├── graph.rs        # FlowGraph - collection of nodes/edges
    ├── canvas.rs       # FlowCanvas viewport
    ├── node.rs         # FlowNode definition
    ├── port.rs         # FlowPort definition
    ├── edge.rs         # FlowEdge definition
    ├── event.rs        # FlowEvent definitions
    ├── layout.rs       # Auto-layout algorithms
    ├── interaction.rs  # Drag, select, connect handlers
    └── style.rs        # Visual styling constants
```

---

## Dependencies
```toml
[dependencies]
cvkg-core = { path = "../cvkg-core" }
cvkg-scene = { path = "../cvkg-scene" }
cvkg-themes = { path = "../cvkg-themes" }
serde = { version = "1.0", features = ["derive"] }
slotmap = "1.0"
```

---

## Success Criteria
1. Basic Functionality: Create, move, delete nodes and edges
2. Performance: Handle 1000+ nodes at 60 FPS
3. Integration: Seamless use within existing CVKG applications
4. Examples: Working workflow editor demo
5. Documentation: API docs and usage examples
