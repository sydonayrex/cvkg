# Nodal coordinate migration — status

Companion to `docs/nodal-coordinate-migration.md`. Tracks which phases
of the migration plan are landed on `main` and which are still open.

**Plan file:** `docs/nodal-coordinate-migration.md` (10 phases total),
plus an Option B that bundles Phase 0 + Phase 8 for early shippability.

## Status snapshot

| Phase | Title | Status | PR / commits |
|-------|-------|--------|--------------|
| **B.0** | Delete competing GPU-side event system (in `cvkg-render-gpu`) | **landed** | `9995795` |
| **B.1** | Drop redundant `fire_renderer_handlers` sites and the function itself | **landed** | `b1622b7` |
| **B.2** | Regression test for VDom-dispatch routing | **landed** | `cfa4162` |
| **1** | Document local rect semantics; add `world_rect(id)` accessor | **landed** | `eec00a4` |
| **2** | Readers route through cumulative offset, not absolute rect | **landed** | `06f0702` |
| 3a | Layout engine opt-in `local_mode` (`compute_layout_local`) | **landed** (this session) | `5d7e0ed` |
| 3 | Fully remove the absolute-flatten upstream walking | **deferred** | — |
| 4 | Diff churning — only the parent's `Update` fires when it moves | **blocked** (depends on Phase 3) | — |
| 5 | Physics + `AnimatedBox` local-rect semantics | **blocked** | — |
| 6 | `WorldSpacePanel` 3D composition via `world_space_position` | **deferred** | — |
| 7 | AccessKit bridge routes through `world_rect` | **deferred** (depends on Phase 3) | — |
| 8 | Input dispatch through VDOM (one-shot bundle with Phase 0) | **landed** as part of B.1 + B.1 | `b1622b7`, `cfa4162` |
| 9–10 | Tests + final verification | **deferred** | — |

### Phase 3a landed — opt-in Layout API

`docs/nodal-coordinate-migration.md` Phase 3 has been started (commit `5d7e0ed`) but
only the layout-engine layer, not the consumer migration. The change adds a
non-breaking opt-in API to `cvkg-layout`:

- `cvkg_layout::VStack::compute_layout_local(spacing, alignment,
  distribution, subviews, cache, width, height)` — returns rects
  anchored at `(0, 0)`, with sizes honored. Width/height are
  `Option<f32>` so callers can pass `None` (uses `f32::MAX`) or hint
  the inner size.
- `cvkg_layout::HStack::compute_layout_local(...)` — same shape,
  Row direction.
- `cvkg_layout::Grid::compute_layout_rects_local(...)` — same shape.
- Existing `compute_layout` and `compute_layout_rects` are
  unchanged. No consumer breaks.

Migration of any single component to local rects is now
self-contained: stop calling `compute_layout` in `View::render`,
start calling `compute_layout_local`, and adjust the downward walk
to use local coords. Two unit tests (`test_local_layout_*` in
`cvkg-layout/src/lib.rs`) cover the engine behavior.

The remaining work for full Phase 3 (migrating every `View::render`
callsite in `cvkg-components/src/` and propagating the
`local_mode = true` flag through) is still multi-day.


**Net result after commit `06f0702`:**
- Architectural foundations for parent-relative coordinates exist; all
  composition goes through one accessor.
- The original bug (`coordinate-routing` GPU flat-map vs NodeId routing)
  is gone; components register handlers on a single NodeId-keyed map and
  events route through `VDom::dispatch_event`.

## What is landed (Phases B, 1, 2 — five commits, all on `main` as of plan-resumption)

### Phase B (event registry collapse)
- `cvkg-render-gpu/src/api/mod.rs`: removed `push_vnode`/`pop_vnode`/`register_handler`
  overrides on the Renderer impl. The trait's default no-op impls now apply,
  forcing registration to flow through `VNodeRenderer`.
- `cvkg-render-gpu/src/renderer/{mod,init,frame,draw}.rs`: removed
  `event_handlers: HashMap<String, …>` and `vnode_stack: Vec<(Rect, &str)>`
  fields, initializers, and per-frame clears. `RenderStateSnapshot.vnode_depth`
  is set to `0` (kept for snapshot-struct compat only).
- `cvkg-render-gpu/src/api/mod.rs`: removed `clear_event_handlers` and
  `get_handlers` inherent methods on `GpuRenderer`.
- `cvkg-render-native/src/main_loop.rs`: removed all 25 redundant
  `fire_renderer_handlers(...)` call sites (each had a nearby
  `vdom.dispatch_event(...)` already), deleted the function itself, and
  removed the unused `gpu_arc` capture in the AccessKit handler.

**Why this first:** the GPU flat map was the original bug reported by
another model (every registered handler fired for every event; each
self-hit-tested coordinates; first-handler-wins race conditions).
Removing it exposes `cvkg-vdom`'s NodeId-keyed map as the single source
of truth. Phases 3+ can now build on this contract.

### Phase 1 (local-rect documentation + accessor)
- `cvkg-vdom/src/vnode.rs`: `VNode.layout` documented as local
  (offsets from parent content origin, NOT absolute screen);
  `LayoutRect` struct doc-commented accordingly.
- `cvkg-vdom/src/lib.rs`: added `pub fn world_rect(id: NodeId)` on
  `VDom`. Walks the parent chain summing local offsets up to root.
  `WorldSpacePanel` ancestors short-circuit with `tracing::warn!` and
  return `None` (the 3D-projected resolution is Phase 6).

### Phase 2 (readers use the cumulative offset)
- `cvkg-vdom/src/lib.rs`: `hit_test_recursive` now takes an
  accumulated `(offset_x, offset_y)` and threads it through children.
  WorldSpacePanel ancestors return `None` (Phase 6 complete fix).
  SDF is tested against `(local_x, local_y) = (x, y) - offset`, which
  matches local-rect semantics from Phase 1.
- `cvkg-vdom/src/lib.rs`: `validate_node_sync` now compares
  `vdom.layout.x/y/w/h` against `scene.local_rect.x/y/w/h` (both
  sides are local). Previously compared against `scene.world_rect`
  (always inconsistent below the root once Phase 3 lands the upstream
  coordinate flattening).

### Regression test added with Phase B.2
- `cvkg-render-native/src/tests.rs::pointer_click_via_vdom_dispatch_fires_registrations`
  pins the new contract: a registered `pointerclick` handler fires
  through `VDom::dispatch_event` once `VDom::event_handlers` is populated.
  Future re-introductions of the flat fan-out will be caught.

### Pre-existing test failure (NOT introduced by this work)
- `cvkg-render-native/src/tests.rs::native_pointer_capture_falls_back_to_rebuilt_target`
  has been failing at git baseline `e73c1c4` (pre-Phase B) due to a setup bug
  in the test: the test constructs a `pressed` VDom with handlers keyed
  at `NodeId(2)` but the rebuilt VDom uses `NodeId(3)` for the same node,
  so `rebuilt.event_handlers = pressed.event_handlers.clone()` keys
  handlers at `2` while `rebuilt.nodes` keys at `3`. The dispatch cannot
  find a handler in that state. **Unrelated** to this migration.

## Why Phase 3 is not landed

Phase 3 in the plan is "stop flattening to absolute during `View::render`".
The flattening happens **inside cvkg-layout's taffy engine** at
`cvkg-layout/src/taffy_engine.rs::compute_layout` — every layout
algorithm in that file passes absolute rects to children:

- `VStack::compute_layout`
- `HStack::compute_layout`
- `ZStack::compute_layout`
- `Flex::compute_layout`
- `Grid::compute_layout`

Subsequently, every `View::render(rect)` callsite that goes through a
layout library (e.g., `cvkg-components/src/container/stacks.rs:97`,
`button.rs`, `text.rs`, `position.rs`, …) receives those absolute
rects and forwards them. Components that pre-flatten manually (e.g.,
a `Position { x: 100, y: 50 }.child(child)` is rendered with
`Rect { x: parent.x + 100, y: parent.y + 50, … }`) would also need
to stop doing so per Phase 3.

Conservatively Phase 3 touches 100+ files across:
- `cvkg-layout/src/` (all engines)
- `cvkg-components/src/` (every layout-aware primitive)
- `cvkg/examples/*`, `cvkg-gallery/`, tests that construct Views

This is multi-day work with per-component test rewrites. It does not
block the architectural goals Phases B, 1, 2 already landed (the
event-handler race the migration was originally scoped against is fixed).

**Decision:** pause Phase 3 here; revisit as a dedicated multi-day PR.

## Search hints for the next session

- `// TODO(nodal-coord-migration): Phase 3` — see Task 2 in the status, marker comments
  flag every layout call site that needs Phase-3 work.
- `git log --grep "nodal-coord"` — chronological order of landed phases.
- `vdom.world_rect(id)` — new single source of truth for absolute bounds.
  New consumers that want absolute coordinates should use this rather
  than reading `vnode.layout` directly.

## Suggested next-session starting point

```bash
# 1. Identify the call-site blast radius
rg -n 'fn render\(&self, renderer: &mut dyn' cvkg-components/src/ | head
rg -n 'compute_layout' cvkg-layout/src/

# 2. Pick the smallest layout primitive (e.g., Position { x, y })
#    and migrate just that one to local-rect semantics, with a
#    regression test that uses parent-cumulative offsets.

# 3. Re-run cargo test -p cvkg-vdom --berkserker_click_box_regression
#    to make sure the integration test now passes once layout engines
#    produce local rects.
```
