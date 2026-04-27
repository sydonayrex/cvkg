# cvkg-core

**cvkg-core** contains the fundamental traits and types that define the CVKG framework. It is the "glue" that allows the VDOM, renderers, and components to interoperate.

## Core Concepts

### `View` Trait
The primary building block of the UI. Every component implements `View`.
*   `body()`: Returns the view's composition.
*   `render()`: Low-level hook for direct interaction with the `Renderer`.
*   `intrinsic_size()`: Negotiates preferred dimensions based on content and layout constraints.

### `Renderer` Trait
An abstraction over the target platform (GPU, Web, Native Primitive).
*   `fill_rect`, `stroke_rect`, `draw_text`: Basic drawing primitives.
*   `push_vnode`: VDOM integration.
*   `register_handler`: Event system integration.
*   `get_telemetry`: Real-time performance metric harvesting.
*   `set_aria_role`, `set_aria_label`: Accessibility integration.

### State & Bindings
*   `State<T>`: Reactive state container with atomic version tracking.
*   `Binding<T>`: Read/Write handle for state sharing with minimal re-render overhead.

### `ViewExt` and Modifiers
A fluent API for applying transformations to views.
*   `.padding()`, `.background()`, `.opacity()`
*   `.on_click()`, `.on_pointer_enter()`, `.on_pointer_leave()`
*   `.on_appear()`, `.on_disappear()`

### Security & Sandboxing
*   `Capability`: Granular permission system for plugins (Network, File, Agent).
*   `SecurityPolicy`: Enforcement layer for capability-based access control.
*   `SandboxLimits`: Resource metering and isolation (CPU/Memory/Events).
*   `PluginManifest`: Secure metadata for untrusted component loading.

### Events
Defines the `Event` enum used across the framework:
*   `PointerDown`, `PointerUp`, `PointerMove`, `PointerClick`
*   `KeyDown`, `KeyUp`
*   `Ime` (Input Method Editor)
*   `PointerEnter`, `PointerLeave` (Synthetic)
