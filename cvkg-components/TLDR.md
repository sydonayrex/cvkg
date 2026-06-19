# cvkg-components TLDR.md

## Purpose
Own all UI components: interactive (Button, Input, Toggle, etc.), navigation (Menubar, List, etc.), display (Toast, HoverCard, etc.), and composite widgets.

## Ownership
- `src/lib.rs` — module declarations and public re-exports
- `src/interactive/` — Button, Input, Checkbox, Toggle, RadioGroup, Slider, etc.
- `src/navigation.rs` — Drawer, Menubar, NavigationMenu, List, Section, DisclosureGroup
- `src/breadcrumb.rs`, `src/toggle_group.rs`, `src/hover_card.rs`
- `src/theme_switch.rs` — dark/light/system mode toggle widget
- `src/text_editor.rs` — text editing with undo/redo and keyboard nav

## Local Contracts
- Every interactive component must have keyboard event handlers via `renderer.register_handler("keydown", ...)`.
- Use `theme::*` accessors for ALL colors — never hardcode `[f32; 4]` arrays.
- `render()` takes `&self` — use `Cell<T>` or `use_state()` for mutable render-time state.
- Do NOT add `on_event()` method — it is not part of the View trait.
- ARIA roles must be set via `renderer.set_aria_role()` on every interactive component.

## Work Guidance
- Follow the existing pattern: `renderer.push_vnode()`, render content, `renderer.register_handler()`, `renderer.pop_vnode()`.
- Focus rings: use `crate::draw_focus_ring(renderer, rect, theme::focus_ring())` on all focusable widgets.
- Touch targets: minimum 44×44px. Use `rect.width.max(44.0)` / `rect.height.max(44.0)`.
- Test keyboard handlers in unit tests where possible.

## Verification
- Run `cargo test -p cvkg-components --lib` (110 tests)
- Run `cargo test -p cvkg-core error_boundary` (8 tests)
- Run `cargo check --workspace` to verify no downstream breakage
