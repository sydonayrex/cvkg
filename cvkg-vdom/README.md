# cvkg-vdom

![CVKG Hero HUD](../docs/images/cvkg_hero.png)

`cvkg-vdom` provides a stateless Virtual DOM implementation for CVKG, enabling efficient UI reconciliation and platform-independent event handling.

## Boundaries and Responsibilities

This crate manages the logical representation of the UI tree. It does NOT handle GPU rendering or physics-based animations directly. Its responsibilities include:
- Capturing the `View` hierarchy into a serializable `VNode` tree.
- Computing differences between tree states to produce `VDomPatch` sets.
- Managing the accessibility tree (ShieldWall) via AccessKit.
- Routing events (pointer, keyboard) through the hierarchy with bubbling support.

## Public API Overview

### Core Types
- `VNode`: A serializable node representing a component instance, its properties, layout, and children.
- `VDom`: The root container for the Virtual DOM state and node mappings.
- `NodeId`: A unique 64-bit identifier for every node in the tree.
- `VDomPatch`: An enum representing discrete mutations (Create, Update, Remove, Move, Replace).

### Key Systems
- `VNodeRenderer`: A specialized `Renderer` implementation that builds a `VDom` from a `View` tree.
- `EventHandlerMap`: A thread-safe registry of closures for handling UI events.

## Usage Example

```rust
use cvkg_vdom::{VDom, VNodeRenderer};
use cvkg_core::{View, Rect};

// Build a VDOM from a view
let view = MyView::new();
let rect = Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 };
let vdom = VDom::build(&view, rect);

// Access the root node
if let Some(root_id) = vdom.root {
    let root_node = vdom.nodes.get(&root_id).unwrap();
    println!("Root component type: {}", root_node.component_type);
}
```

## Known Limitations
- VDOM updates are currently full-rebuilds by default; incremental diffing is optimized for high-frequency updates in the Surtr pipeline.
- Event handlers must be `Send + Sync` to support multi-threaded event processing.