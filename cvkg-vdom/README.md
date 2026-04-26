# cvkg-vdom

**cvkg-vdom** implements the stateless Virtual DOM engine and the centralized event dispatcher for CVKG.

## Features

### `VDom`
Maintains a tree of `VNode`s representing the UI state.
*   `build(view, rect)`: Performs a virtual render pass to construct the tree.
*   `diff(old_vdom)`: Calculates the minimal set of `VDomPatch`es to transform one state to another.
*   `dispatch_event(event)`: Routes incoming OS events to the correct UI nodes.

### Event Dispatching & Hit Testing
*   **Recursive Hit Testing**: Finds the deepest node at a specific coordinate.
*   **Hover Tracking**: Automatically synthesizes `PointerEnter` and `PointerLeave` events as the mouse moves across component boundaries.
*   **Focus Management**: Tracks the currently focused node for keyboard and IME input.
*   **Event Bubbling**: Supports standard event propagation from child to parent.

### Accessibility Integration
*   Translates VDOM nodes into `AccessKit` nodes for screen reader compatibility.
*   Synchronizes ARIA roles and labels from the view definition to the OS accessibility tree.
