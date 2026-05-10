# cvkg-vdom

![Reactive Agent VDOM Graph](../docs/images/vdom_agent_graph.png)

**cvkg-vdom** provides a stateless Virtual DOM implementation for CVKG with differential updates, event handling, and accessibility tree generation.

## What This Crate Does

- Provides `VNode` for representing UI nodes in memory
- Implements diffing via `VDiff` for efficient updates
- Handles event dispatching and lifecycle callbacks
- Generates accessibility trees via AccessKit integration

## What This Crate Does NOT Do

- Does not perform actual rendering (see cvkg-render-gpu)
- Does not manage application state
- Does not provide layout calculations

## Public API Overview

### VNode

```rust
/// Unique identifier for a node in the Virtual DOM tree
pub struct NodeId(pub u64);

/// Computed layout bounds of a component
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Accessibility ARIA properties
pub struct AriaProps {
    pub label: Option<String>,
    pub description: Option<String>,
    pub value: Option<String>,
    pub disabled: bool,
    pub hidden: bool,
}

/// A node in the Virtual DOM tree
pub struct VNode {
    pub id: NodeId,
    pub key: Option<String>,
    pub component_type: String,
    pub props: HashMap<String, serde_json::Value>,
    pub state: Option<HashMap<String, serde_json::Value>>,
    pub layout: LayoutRect,
    pub children: Vec<NodeId>,
    pub aria_role: String,
    pub aria_props: AriaProps,
    pub portal_target: Option<NodeId>,
}
```

### VDomPatch

```rust
/// A discrete mutation to the Virtual DOM tree
pub enum VDomPatch {
    /// Create and append a new node
    Create(VNode),
    /// Update properties of an existing node
    Update {
        id: NodeId,
        props: Option<HashMap<String, serde_json::Value>>,
        layout: Option<LayoutRect>,
        aria_props: Option<AriaProps>,
        aria_role: Option<String>,
        children: Option<Vec<NodeId>>,
    },
    /// Delete a node
    Delete(NodeId),
    /// Move a node within its parent's children
    Move { id: NodeId, to_index: usize },
}
```

### VBox

```rust
/// Virtual DOM container managing the tree structure
pub struct VBox {
    nodes: HashMap<NodeId, VNode>,
    root: Option<NodeId>,
}

impl VBox {
    /// Create a new empty Virtual DOM
    pub fn new() -> Self;
    
    /// Create a new node
    pub fn create(&mut self, node: VNode) -> NodeId;
    
    /// Apply patches to update the tree
    pub fn patch(&mut self, patches: Vec<VDomPatch>);
    
    /// Get a node by ID
    pub fn get(&self, id: NodeId) -> Option<&VNode>;
    
    /// Delete a node and its children
    pub fn delete(&mut self, id: NodeId);
}
```

### Events

```rust
/// Event types dispatched from the DOM
pub enum Event {
    PointerDown { x: f32, y: f32 },
    PointerUp { x: f32, y: f32 },
    PointerMove { x: f32, y: f32 },
    PointerClick { x: f32, y: f32 },
    KeyDown { key: String },
    KeyUp { key: String },
    Ime { text: String },
}
```

## Usage Example

```rust
use cvkg_vdom::{VBox, VNode, NodeId, LayoutRect};
use std::collections::HashMap;

let mut vdom = VBox::new();

let root = VNode {
    id: NodeId(1),
    key: None,
    component_type: "VStack".to_string(),
    props: HashMap::new(),
    state: None,
    layout: LayoutRect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 },
    children: vec![NodeId(2)],
    aria_role: "group".to_string(),
    aria_props: Default::default(),
    portal_target: None,
};

let child = VNode {
    id: NodeId(2),
    key: None,
    component_type: "Text".to_string(),
    props: { let mut m = HashMap::new(); m.insert("text".to_string(), "Hello".into()); m },
    state: None,
    layout: LayoutRect { x: 10.0, y: 10.0, width: 100.0, height: 20.0 },
    children: vec![],
    aria_role: "text".to_string(),
    aria_props: Default::default(),
    portal_target: None,
};

vdom.create(root);
vdom.create(child);
```

## Known Limitations

- VNode comparison uses ID equality; components must manage keys for list diffing
- No built-in batching; multiple patches require manual accumulation
- Accessibility tree updates require explicit `to_accesskit_node()` calls