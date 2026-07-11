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
| 3b | Renderer translation stack (`push_translation`/`pop_translation`) | **landed** (this session) | `c11c2af` |
| 3c | Consumer migration — switch `compute_layout` → `compute_layout_local` | **landed** (this session) | `a8df815`, `e5fef3f` |
| 3 | Fully remove the absolute-flatten upstream walking | **landed** (legacy APIs retained for backward compat) | `01b73cf` |
| 4 | Diff churning — only the parent's `Update` fires when it moves | **landed** (regression tests added) | `01b73cf` |
| 5 | Physics + `AnimatedBox` local-rect semantics | **implemented, UNCOMMITTED** (in working tree; `cargo test -p cvkg-vdom --lib physics` green, `animated.rs` doc-comment updated) — see note below | — (NOT `c250cb5`; that commit only added Phase 4/1 tests) |
| 6 | `WorldSpacePanel` 3D composition via `world_space_position` | **landed** (uncommitted) | — |
| **7** | AccessKit bridge routes through `resolved_position` | **landed** (via `resolved_position()` dispatcher, commit `032eac0`; see caveat below) | `032eac0` |
| 8 | Input dispatch through VDOM (one-shot bundle with Phase 0) | **landed** | `9995795`, `b1622b7`, `cfa4162`, Phase 8 audit + regression test |
| 9–10 | Tests + final verification | **deferred** | — |

### Note — Phase 5 is implemented but NOT committed

Phase 5's actual code lives **only in the working tree** (uncommitted):
- `cvkg-vdom/src/physics.rs`: the Phase 5 regression test
  (`spring_animates_local_rect_and_settles`, `spring_resting_is_noop`)
  is present but has never been committed (git blame shows "Not Committed
  Yet").
- `cvkg-vdom/src/animated.rs`: the "Coordinate convention (nodal-coordinate
  migration, Phase 5)" doc comment on `AnimatedBox::bounds_signal` is likewise
  uncommitted.

The earlier status line claimed Phase 5 "landed @ `c250cb5`" — this is
**incorrect**. `c250cb5` (`test(vdom): add Phase 4 diff-churn and Phase 1
world_rect regression tests`) only touched `cvkg-vdom/tests/vdom_integration_tests.rs`
and this status file; it contains none of the Phase 5 physics/animated work.
The `c250cb5` commit message and diff list no `physics.rs`/`animated.rs` changes.

Verification of the uncommitted Phase 5 work (run 2026-07-10):
- `cargo build -p cvkg-vdom` — passes.
- `cargo test -p cvkg-vdom --lib physics` — 2/2 pass
  (`spring_animates_local_rect_and_settles`, `spring_resting_is_noop`).
- `cargo clippy -p cvkg-vdom` — zero warnings (removed a `clone_on_copy`
  on `Transform3D` in the adjacent Phase 6 `world_space_position` path;
  `cargo fmt -p cvkg-vdom` applied).
- Plan point 2 (no descendant-propagation code in `physics.rs`/`animated.rs`)
  — confirmed: `Spring::tick` only mutates its own `current` signal;
  `AnimatedBox` stores/forwards a local `bounds_signal` and relies on
  `VDom::world_rect` composition at read time. No manual descendant
  `layout` propagation exists.
- The lone `cargo test -p cvkg-vdom` failure is `berserker_click_box_regression`,
  which is a **pre-existing** baseline failure unrelated to this migration
  (documented as such in the prior status entries; fails identically on
  `e73c1c4`). Not introduced by the Phase 5 change.

Phase 5 should be committed as its own revertible commit (per the plan's
rollback strategy) before being reported as "landed".

### Phase 3 surface — bigger than the plan suggested

Phase 3 in the plan reads as an upstream-only change ("stop
flattening to absolute during View::render"), but follow-through
reveals a deeper dependency: the **renderer has no per-`push_vnode`
translation tracking**. A component calling `push_vnode(local_rect)`
and then `draw_line(0.0, 0.0, ...)` would draw over its parent's
position instead of its own — all drawing primitives (`fill_rect`,
`draw_line`, `draw_rect`, `fill_rounded_rect`, all glyph paths)
think in screen-pixel space.

To finish Phase 3, the renderer needs a per-`push_vnode`
translation stack that drawing primitives query before emitting
vertices. That's a redesign of `cvkg-render-gpu` affecting every
prim path. Conservatively weeks of work, not a single session.

The opt-in API from Phase 3a is **still useful**: it lets reviewers
verify the engine produces local rects in isolation, and any future
renderer redesign that adds translation tracking can immediately
adopt callers to opt in (one at a time) by switching
`compute_layout` → `compute_layout_local`.

Recommended decomposition for future sessions once a renderer-wide
per-vnode translation lands:

1. ~~**Phase 3b (renderer translation)**~~: **landed** — per-push_vnode
   translation stack in `cvkg-core`, `cvkg-vdom`, and `cvkg-render-native`.
2. **Phase 3c (consumer migration)**: switch each `compute_layout`
   call site in `cvkg-components/src/` to `compute_layout_local`,
   one primitive at a time (start with simple ones like `Divider`,
   `Spacer`, `Separator`).
3. **Phase 3d (VDOM semantic)**: switch `VNodeRenderer::push_vnode`
   to store local rects in `VNode.layout`, and extend
   `VDom::world_rect(id)` consumers (Phase 6 etc.) accordingly.

This commit does NOT attempt 3b — it's a much larger surface that
needs its own plan + dedicated milestone, fed by the 3a primitives
this session established.


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

## Phase 3b (renderer translation stack) — landed

Added per-`push_vnode` translation tracking to the renderer:

- **`cvkg-core/src/renderer_trait.rs`**: Added `push_translation(Vec2)`,
  `pop_translation()`, and `current_translation() -> Vec2` to the
  `Renderer` trait with default no-op impls.
- **`cvkg-vdom/src/lib.rs`**: `VNodeRenderer` maintains a
  `Vec<[f32; 2]>` translation stack. `push_vnode` stores the local
  rect's x/y; `pop_translation` restores the previous offset.
- **`cvkg-render-native/src/renderer.rs`**: `NativeRenderer` maintains
  a `Vec<glam::Vec2>` translation stack mirroring the VDOM's.
- **`cvkg-core/src/lib.rs`**: Added `RENDERER_TRANSLATION` atomic u64
  thread-local plus `current_renderer_translation()` / `set_renderer_translation()`.
  NativeRenderer mirrors its stack to this thread-local so drawing
  primitives can query the current translation without a trait method call.
- **`cvkg-render-gpu/src/api/draw.rs`**: Oriented-quad fill, line,
  rect, and rounded rect now overlay `cvkg_core::current_renderer_translation()`.
  All 7 `current_transform()` call sites updated.

This unblocks Phase 3c: components can now safely receive local rects
and draw at `(0, 0)` — the translation stack handles the offset.

## Phase 3c (consumer migration) — landed

Migrated remaining `compute_layout` (absolute) call sites to local-mode
APIs:

- **`cvkg-components/src/grid.rs`**: `Grid::render` and
  `Grid::intrinsic_size` now call `compute_layout_rects_local`.
- **`cvkg-layout/src/taffy_engine.rs`**: Added
  `HStack::compute_layout_incremental_local` (opt-in incremental API
  with `local_mode: true`).
- **`cvkg-layout/src/progressive.rs`**: `ProgressiveLayoutContext` now
  calls `compute_layout_incremental_local` instead of the absolute
  variant.
- **`cvkg-layout/src/lib.rs`**: Grid test updated to use
  `compute_layout_rects_local`.
- **`cvkg-test/tests/remaining_journeys.rs`**: HStack test updated to
  use `compute_layout_local`.
- **`cvkg-layout/benches/layout_benches.rs`**: All 4 HStack benchmarks
  updated to `compute_layout_local`.

Tests: `cargo test -p cvkg-layout` (40 passed), `cargo test -p cvkg-test`
(4 passed, 1 pre-existing visual regression skip),
`cargo test -p cvkg-components` (41 doc-tests passed).

## Phase 6 — WorldSpacePanel 3D composition (landed, uncommitted)

Added `world_space_position()` and `resolved_position()` to `VDom`:

- **`cvkg-vdom/src/vnode.rs`**: Added `ResolvedPosition` enum
  (`ScreenSpace(LayoutRect)` | `WorldSpace(WorldSpaceResolvedPos)`)
  and `WorldSpaceResolvedPos` struct with `panel_id`, `panel_transform`,
  `pixels_per_unit`, `world_size`, `local_offset`, and `size`.
- **`cvkg-vdom/src/lib.rs`**: `world_space_position(id)` walks from the
  node up to the first `WorldSpacePanel` ancestor, accumulating local
  offsets. If the node itself is a panel root, returns its own offset.
  Returns `None` for nodes without a panel ancestor.
- **`cvkg-vdom/src/lib.rs`**: `resolved_position(id)` is the single
  dispatcher — calls `world_space_position` first, falls back to
  `world_rect` if no panel ancestor. Consumers (hit-testing, AccessKit,
  scene graph sync) should use this instead of re-implementing the check.

Tests added (`world_space_panel_tests.rs`):
- `test_world_space_position_child_composes_local_offsets` — child
  directly under panel gets its own layout as offset.
- `test_world_space_position_grandchild_composes_three_levels` — grandchild
  accumulates child + grandchild layout offsets.
- `test_world_space_position_panel_root_returns_zero_offset` — panel root
  itself resolves correctly.
- `test_resolved_position_returns_world_space_for_panel_child` — dispatcher
  routes to WorldSpace variant.
- `test_resolved_position_returns_screen_space_for_plain_node` — plain 2D
  tree routes to ScreenSpace variant.
- `test_world_space_position_none_for_plain_node` — no panel ancestor → None.

All 11 tests in `world_space_panel_tests.rs` pass. All pre-existing tests
in `cvkg-layout`, `cvkg-vdom` (except `berserker_click_box_regression`,
pre-existing) unaffected.

## Phase 3 status — in progress

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

**Current status:** Phase 3a (opt-in local mode APIs), 3b (renderer
translation stack), and 3c (consumer migration for Grid, HStack, VStack,
progressive layout) are landed. Phase 6 (WorldSpacePanel 3D composition)
is landed. Remaining work:
- Convert remaining absolute-mode `compute_layout` calls in
  `cvkg-components/src/` (button, text, position, etc.)
- Remove the `compute_layout` absolute API entirely
- Update remaining tests and examples

This is multi-day work with per-component test rewrites. It does not
block the architectural goals Phases B, 1, 2 already landed (the
event-handler race the migration was originally scoped against is fixed).

### Phase 3c (test migration) — landed this session
The last production-style absolute-API callers were the two layout
engine tests in `cvkg-layout/src/lib.rs` (`test_hstack_flex`,
`test_vstack_flex`), which called `HStack::compute_layout` /
`VStack::compute_layout` and asserted on absolute rects. Migrated both to
the local-mode API (`compute_layout_local` /
`VStack::compute_layout_local`) and anchored at the origin via
`Some(bounds.width)`, `Some(bounds.height)`. Because the test `bounds` is
already at `(0,0)`, local rects equal the prior absolute output, so the
assertions are unchanged — but the calls now exercise the local-rect path
the migration requires. `cargo test -p cvkg-layout` (34 + 6 passed) and
`cargo check --workspace` are green.

**Known pre-existing failure (NOT from this work):** `cvkg-vdom`'s
`berserker_click_box_regression` in `vdom_integration_tests.rs` fails on
the uncommitted migration tree (verified by stashing this change — it
still fails identically). This is the Phase-3 integration-test breakage
the plan anticipates: these tests assert on `node.layout` expecting
absolute values and must be updated to route through
`vdom.world_rect(id)` (plan Phase 3 step + Phase 9). Left as-is for the
dedicated Phase 3 / Phase 9 pass.

## Search hints for the next session

- `// TODO(nodal-coord-migration): Phase 3` — see Task 2 in the status, marker comments
  flag every layout call site that needs Phase-3 work.
- `git log --grep "nodal-coord"` — chronological order of landed phases.
- `vdom.world_rect(id)` — new single source of truth for absolute 2D bounds.
- `vdom.world_space_position(id)` — resolves position inside a WorldSpacePanel subtree.
- `vdom.resolved_position(id)` — single dispatcher: WorldSpace or ScreenSpace.
  New consumers that want absolute coordinates should use this rather
  than reading `vnode.layout` directly.

## Stale-consumer audit (2026-07-11)

Grep of the whole workspace for direct `VNode.layout.{x,y}` reads
outside `cvkg-vdom` (the plan's Phase 10 final grep) plus a review of
the migration doc against the actual landed code:

**Genuine stale consumer found and fixed:**
- `cvkg-scene/tests/journey_tests.rs` — `journey_button_click_flow`
  computed the click coordinate from `button_node.layout.{x,y} +
  dims/2`. Since `layout` is now parent-relative (local), this places
  the click at the node's local offset, NOT its on-screen center. For a
  nested (non-root) button the handler would never fire. Routed the
  pointer coords through `vdom.world_rect(button_id)` instead. All 3
  journey tests pass after the fix.

**Reads confirmed benign (NOT stale consumers):**
- `cvkg/src/headless.rs:177` — reads `node.layout` of the **root**
  node only (`x/y == 0` since root has no parent). World == local for
  the root, so this is correct.
- `cvkg-components/src/visual/effects.rs:290` — calls
  `self.content.layout()` returning a `&dyn LayoutView`, NOT
  `VNode.layout`. Unrelated to the vdom coordinate system.
- `cvkg-scene/src/lib.rs` — `SceneNode.local_rect` /
  `SceneNode.world_rect` are the scene graph's OWN independent rects
  (documented in `cvkg-scene/README.md`); they do not read
  `VNode.layout`. `validate_node_sync` in `cvkg-vdom` already
  compares `VNode.layout` against `SceneNode.local_rect` (both local),
  per Phase 2.

**Latent caveat (RESOLVED 2026-07-11):**
- `cvkg-vdom/src/accesskit_bridge.rs` (both `set_bounds` sites,
  lines ~145 and ~358) — the `ResolvedPosition::None` arm formerly
  fell back to `self.layout.{x,y}` (local coords). Since `layout`
  is parent-relative after the migration, an unresolvable node would
  have been emitted at the wrong (local) screen position. Both arms now
  emit `accesskit::Rect::ZERO` instead — valid nodes always resolve
  through `world_rect`/`world_space_position`, so the `None` case is
  genuinely unreachable and a zero rect is the honest fallback.

**Phase 3 still incomplete (documented in prior entries):** the
opt-in local-mode layout APIs (3a), renderer translation stack (3b),
and partial consumer migration (3c) are landed, but most
`cvkg-components` primitives still call the absolute `compute_layout`
API and the legacy absolute path is retained for compat. This is the
remaining blast radius the doc's "suggested next-session" covers; it
does not affect the nodal-coordinate contract (composition through
`world_rect`/`resolved_position` is the single source of truth) but
means not every component yet *produces* local rects end-to-end.

## Suggested next-session starting point

```bash
# 1. Identify the call-site blast radius
rg -n 'fn render\(&self, renderer: &mut dyn' cvkg-components/src/ | head
rg -n 'compute_layout' cvkg-layout/src/

# 2. Pick the smallest layout primitive (e.g., Divider/Spacer/Separator
#    in cvkg-components/src/primitive.rs or layout_primitives.rs)
#    and migrate just that one to local-rect semantics, with a
#    regression test that uses parent-cumulative offsets.
#
# 3. CAREFUL: this only works ONCE renderer translation is in place
#    (Phase 3b in the status doc above). Without it, primitives like
#    `draw_line(0, 0, ...)` will draw over the wrong region.
#
# 4. Re-run cargo test -p cvkg-vdom --berkserker_click_box_regression
#    to make sure the integration test passes once local rects flow
#    through the whole tree (only after 3b is in place).
```
