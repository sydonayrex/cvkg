# cvkg-flow

**cvkg-flow** provides flow graph visualization and interaction components for CVKG applications.

## 🚀 Quick Start

```rust
use cvkg_flow::{FlowGraph, FlowNode, FlowEdge};
use cvkg_components::{VStack, Text};

// Create a flow graph
let mut graph = FlowGraph::new();

// Add nodes
let node1 = graph.add_node(