---
name: cvkg-implementation-patterns
description: "CVKG-specific implementation patterns learned from real sessions"
---

# CVKG Implementation Patterns

## File Organization

**NEVER create files named `modern_missing.rs`, `misc.rs`, `extras.rs`, `new_components.rs`, or any other "hodge-podge" name.** Every component file must be named after the domain/function it covers:

| Domain | File Name |
|--------|-----------|
| Dialogs/Overlays | `dialog.rs` |
| Layout primitives | `layout_primitives.rs` |
| Navigation | `navigation.rs` |
| Form controls | `form_controls.rs` |
| Display/Typography | `display.rs` |
| Text animation | `text_anim.rs` |
| Agent/AI chat | `agent_chat.rs` |
| Material 3 | `m3_components.rs` |
| Advanced layout | `layout_components.rs` |
| Multimedia | `multimedia.rs` |
| UX patterns | `patterns.rs` |

If a file grows beyond ~1200 lines, split into sub-modules.

## Component Pattern

Every CVKG component follows this pattern:

```rust
#[derive(Clone)]
pub struct ComponentName {
    pub field: String,
    // ... all pub fields
}

impl ComponentName {
    pub fn new() -> Self { /* sensible defaults */ }
    
    // Builder methods return Self
    pub fn field(mut self, value: impl Into<String>) -> Self {
        self.field = value.into();
        self  // NEVER forget this
    }
}

impl View for ComponentName {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }
    
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ComponentName");
        // Actual drawing logic using theme::* helpers
        renderer.pop_vnode();
    }
    
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size { width: proposal.width.unwrap_or(DEFAULT_W), height: DEFAULT_H }
    }
}
```

## Key Rules

1. Theme exclusively through `theme::*` helpers - no hardcoded colors
2. Builder methods MUST return `Self` (E0308 if forgotten)
3. All fields `pub` for user configuration
4. Wrap render bodies in `push_vnode`/`pop_vnode` for hit testing
5. Use `register_handler` for interactive components
6. `intrinsic_size()` returns reasonable defaults

## Related Specialized Skills

- Use `cvkg-employment` first for broad CVKG work spanning rendering, components, design, app architecture, WebAssembly, demos, verification, or Git delivery.
- Use `wgsl-wgpu-shader-pipeline` for shader, bind group, and pipeline debugging beyond CVKG-specific conventions.
- Use `rendering-architecture-audit` for GPU architecture audits and capability mapping.

## Audit Philosophy

When auditing existing code:
- READ every file to verify render bodies have actual drawing logic
- Do NOT delete code just because it looks unfamiliar -- investigate first
- "Placeholder" fields (like `pub placeholder: String`) are legitimate UI features, not code stubs
- Empty-looking render bodies may have early returns (`if !self.open { return; }`) that are correct
- Check `lib.rs` exports to verify components are properly wired up
- Use `cargo clippy --fix --lib -p <crate> --allow-dirty` to auto-fix issues across all crates

## Verification Protocol (FULL)

After implementing or auditing components:
1. `cargo check --workspace` - zero errors
2. `cargo clippy --workspace` - zero errors (run `cargo clippy --fix --lib -p <crate> --allow-dirty` first)
3. `cargo fmt --check` - zero formatting issues (run `cargo fmt` first)
4. `cargo test --workspace` - zero failures
5. Read every file to verify render bodies have actual drawing logic
6. Check each module is registered in `lib.rs` with `pub mod` and `pub use`

## Common Pitfalls

- `if-same_then_else`: Clippy catches identical branches -- eliminate dead branches
- Approximate TAU: Use `std::f32::consts::TAU` instead of `6.28`
- Unused fields in existing code: These are pre-existing and should NOT be removed unless explicitly tasked
- Count components by parsing `pub struct` definitions across all files, don't rely on filenames alone
