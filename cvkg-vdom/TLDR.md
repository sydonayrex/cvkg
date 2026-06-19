# cvkg-vdom TLDR.md

## Purpose
Own the Virtual DOM layer: VDOM node types, accessibility tree construction, event propagation, and the bridge between the View system and the renderer.

## Ownership
- `src/lib.rs` — VDOM node types, accessibility tree, event handling, AccessKit bridge
- ARIA role mapping (AriaRole enum → AccessKit roles)

## Local Contracts
- All ARIA roles must be mapped to AccessKit equivalents.
- `query_accessibility_tree()` must return the real VDOM tree, not mock data.
- Event propagation must follow bubbling/capture semantics.
- NodeId is a type alias for cvkg_core::KvasirId — never define a separate identity struct. External accesskit::NodeId is a distinct type for the accessibility tree bridge only.

## Verification
- Run `cargo test -p cvkg-vdom`
- Run `cargo check --workspace`
