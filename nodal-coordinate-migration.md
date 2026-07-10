# Nodal coordinate system migration — execution plan

Audience: an AI coding agent with write access to the `cvkg-vdom` and
`cvkg-render-gpu` crates (and read access to `cvkg-core`, `cvkg-scene`,
`cvkg-components`, `cvkg-render-native` for call-site updates). Follow the
phases in order. Each phase ends with a compile/test checkpoint — do not
start the next phase on a red build.

## Why, in one paragraph

`VNode.layout` (`cvkg-vdom/src/vnode.rs`) currently stores absolute world
coordinates per node. `cvkg-scene`'s `SceneNode.world_rect` stores a second,
independent copy of the same absolute position. `validate_node_sync`
(`cvkg-vdom/src/lib.rs`) exists only to detect when those two copies
disagree ("Spatial drift detected"). Separately, `cvkg-render-gpu` runs a
**second, independent event-handler registry** (`GpuRenderer::event_handlers`,
keyed by `event_type: &str`, populated via `push_vnode`/`register_handler` in
`src/api/mod.rs`) that has no hit-testing, no `NodeId`, and no bubbling —
and `get_handlers()` is never called anywhere in that crate. This plan
replaces per-node absolute storage with parent-relative offsets composed on
read, and deletes the dead GPU-side event system in favor of the one
`cvkg-vdom` already owns correctly.

## Non-negotiable invariants

1. `cvkg-vdom` remains the single owner of: node identity (`NodeId`),
   event-handler storage, hit-testing, and event dispatch/bubbling. No other
   crate stores its own copy of any of these.
2. Every node's **world-space** position must always be derivable by
   composing exactly one chain: its own local offset plus every ancestor's
   local offset plus every ancestor's `WorldSpacePanel` transform (if any),
   walked via `VDom.parents`. No code path may read or write a node's world
   position from anywhere else.
3. `AccessKit` and hit-testing consumers still see absolute/world bounds —
   they must not be pushed the burden of composing transforms themselves.
   Composition happens once, in `cvkg-vdom`, behind a single accessor.
4. No behavior change in a single commit that isn't covered by an existing
   or new test. Each phase must leave `cargo test -p cvkg-vdom` and
   `cargo check --workspace` green before moving on.

---

## Phase 0 — Delete the competing GPU-side event system first

Do this before touching coordinates. It's independent, low-risk, and removes
a trap that would otherwise get re-wired against the *old* absolute-coordinate
API and have to be redone.

In `cvkg-render-gpu`:

1. Remove these fields from `GpuRenderer` (`src/renderer/mod.rs`):
   - `event_handlers: HashMap<String, Vec<Arc<dyn Fn(Event) + Send + Sync>>>`
   - `vnode_stack: Vec<(Rect, &'static str)>`
2. Remove their initializers in `src/renderer/init.rs` (`vnode_stack: Vec::new()`,
   `event_handlers: HashMap::new()`).
3. Remove the clear calls in `src/renderer/draw.rs:72-73` and
   `src/renderer/frame.rs:30-31`.
4. Remove `push_vnode`, `pop_vnode`, and `register_handler` from the
   `Renderer` trait impl in `src/api/mod.rs` (~line 1478-1494), and remove
   `clear_event_handlers` / `get_handlers` from the inherent `impl GpuRenderer`
   block (~line 1568-1584).
5. Grep the whole workspace for `get_handlers(`, `clear_event_handlers(`,
   `push_vnode(`, `register_handler(` outside `cvkg-vdom` to confirm nothing
   else called into the GPU-side registry. (Expect zero hits — `get_handlers`
   has no call sites today, which is the point.) If any caller relied on
   `GpuRenderer::get_handlers`, redirect it to `VDom::dispatch_event` /
   `VDom::event_handlers` instead — do not preserve a parallel path.
6. Confirm `GpuRenderer`'s `Renderer` trait impl still compiles once these
   methods are gone — `register_handler` must remain a required trait method
   satisfied by `VNodeRenderer` in `cvkg-vdom` (it already is; see
   `cvkg-vdom/src/lib.rs:1518-1533`), not by `GpuRenderer`. If the `Renderer`
   trait in `cvkg-core` requires every implementor to provide
   `push_vnode`/`pop_vnode`/`register_handler`, either:
   - make those trait methods default no-ops (preferred — `GpuRenderer` draws
     pixels, it should not be asked to track node identity or handlers), or
   - split `Renderer` into a pure-drawing trait and a separate
     `EventRegistrar` trait that only `VNodeRenderer` implements, and update
     the `render(&self, renderer: &mut dyn Renderer, rect: Rect)` call sites
     (`cvkg-vdom/src/animated.rs`, and wherever `View::render` is invoked) to
     take `&mut dyn Renderer` for drawing and route handler registration
     through the `VNodeRenderer` pass only.
7. Confirm `README.md`'s own boundary claim in `cvkg-render-gpu` — "input
   routing is the caller's responsibility" / "does not own ... DOM logic" —
   is now actually true of the code, not just the doc.

**Checkpoint**: `cargo check -p cvkg-render-gpu --tests` and
`cargo check --workspace` both pass. `cargo test -p cvkg-vdom` unaffected
(this phase touches zero files in `cvkg-vdom`).

---

## Phase 1 — Introduce the relative coordinate type in `cvkg-vdom`

In `cvkg-vdom/src/vnode.rs`:

1. Keep `LayoutRect { x, y, width, height }` as the *local* representation —
   rename nothing in its shape, but change its meaning: `x, y` become the
   node's offset from its parent's content origin (as already produced by
   the upstream layout engine before it gets flattened to absolute — check
   `cvkg-layout`/wherever `View::render(rect: Rect)` absolute rects are
   currently assembled, and stop doing that flattening there instead of in
   this crate. `width`/`height` are unchanged — they are already
   parent-independent).
2. Add a derived, non-serialized accessor rather than a stored field:
   ```rust
   impl VNode {
       /// Local offset + size, relative to the parent's content origin.
       /// This is the only field that is ever mutated directly.
       pub layout: LayoutRect, // now local, not world
   }
   ```
3. Do **not** add a `world_layout` field to `VNode`. Storing a second copy
   of derived data is exactly the bug this migration removes. World rects
   are computed, never stored, on the `VDom`.

In `cvkg-vdom/src/lib.rs`, add the composition primitive on `VDom`:

```rust
/// Compute a node's world-space rect by composing local offsets up the
/// parent chain. This is the single source of truth for "where is this
/// node on screen" — no other type stores this.
pub fn world_rect(&self, id: NodeId) -> Option<LayoutRect> {
    let mut node = self.nodes.get(&id)?;
    let mut x = node.layout.x;
    let mut y = node.layout.y;
    let (width, height) = (node.layout.width, node.layout.height);
    let mut current = id;
    while let Some(&parent_id) = self.parents.get(&current) {
        let parent = self.nodes.get(&parent_id)?;
        // If the parent owns a WorldSpacePanel, this subtree's local
        // coordinates are already relative to the panel's own space —
        // stop composing 2D offsets and hand off to the panel's 3D
        // transform (see Phase 5). Flag this rather than silently
        // producing a wrong 2D answer.
        if parent.world_space.is_some() {
            break;
        }
        x += parent.layout.x;
        y += parent.layout.y;
        current = parent_id;
        node = parent;
    }
    Some(LayoutRect { x, y, width, height })
}
```

Cache invalidation: do not memoize `world_rect` yet (Phase 9 covers
optional caching once correctness is proven). A correct O(depth) walk per
call is the baseline; only optimize after tests are green.

**Checkpoint**: crate compiles with `layout` now meaning "local", even
though every consumer still assumes "absolute" (they'll be broken — that's
expected and fixed in the following phases, tracked one call site at a
time). Do not run tests yet; several will legitimately fail until Phase 6.

---

## Phase 2 — Fix every direct reader of `node.layout` as if it were absolute

Grep `cvkg-vdom/src` for `.layout.x`, `.layout.y`, `.layout.width`,
`.layout.height`, and `node.layout` generally. For each call site, decide:
does it need the node's **local** rect (rare — mostly layout-engine internals
like `expand_batch_rect`/`begin_decorative` in `lib.rs`, which operate on
rects *already in the coordinate space the caller is drawing in* and are
fine as-is) or the node's **world** rect (hit-testing, AccessKit bounds,
scene-graph sync)? Route the latter through `self.world_rect(id)`.

Known call sites to fix:

- `src/lib.rs` `hit_test_recursive` (~line 423-470): currently calls
  `Self::sdf_distance(node.sdf_shape.as_ref(), &node.layout, x, y)` assuming
  `node.layout` is absolute. Change `hit_test_recursive` to thread an
  accumulated `(offset_x, offset_y)` down the recursion instead of calling
  `world_rect` fresh at every node (recomputing the full ancestor chain per
  node during a tree walk you're already doing top-down is wasted work —
  the recursive walk is the natural place to accumulate). Pass the parent's
  cumulative offset in, add the current node's local `x, y`, use that for
  the SDF test, and pass the updated cumulative offset to children.
- `src/lib.rs` `validate_node_sync` (~line 202-233): replace the direct
  `vnode.layout.x/y` comparison with `self.world_rect(id)` vs.
  `snode.world_rect`. This function's job doesn't change — it's still a
  useful cross-check against `cvkg-scene` — but it must compare like-for-like
  (composed world rect vs. the scene graph's world rect), not raw local
  offset vs. world rect (which would now *always* fail below the root).
- `src/lib.rs` decorative-batch code (`begin_decorative`,
  `expand_batch_rect`, `push_decorative_cmd`, ~lines 946-1010): these consume
  a `cvkg_core::Rect` passed in from the render pass, which is already
  produced by the caller's own coordinate accumulation during `View::render`
  walks. Confirm with Phase 3 whether that accumulation is still
  absolute-flattening at render time (it should stop being so — see Phase 3)
  before deciding whether these need `world_rect` too, or whether they
  should also switch to storing/receiving local rects.
- `src/vnode.rs` — no reader changes needed here beyond the doc comment
  correction from "computed layout bounds" to "local layout bounds, relative
  to parent".

**Checkpoint**: `cargo check -p cvkg-vdom` passes. Do not fix tests yet.

---

## Phase 3 — Stop flattening to absolute during `View::render`

Find where `render(&self, renderer: &mut dyn Renderer, rect: Rect)` currently
computes an absolute `rect` by adding a running parent offset as it walks the
`View` tree (this lives upstream of `cvkg-vdom`, likely in `cvkg-layout` or
wherever `VNodeRenderer::evaluate` drives the walk — locate it via
`VNodeRenderer::evaluate` in `cvkg-vdom/src/lib.rs` and trace where `layout:`
gets assigned onto each `VNode` during evaluation, e.g. around lines 1066,
1101, 1148, 1184+).

Change that assignment to write the node's **local** rect (offset from its
immediate parent's content origin) instead of the flattened absolute rect.
The layout engine itself (Taffy or equivalent) already computes local rects
per node before whatever step currently flattens them — find that flattening
step and delete it; do not reimplement flattening elsewhere.

`AnimatedBox::render` (`src/animated.rs`) forwards `rect` straight to
`self.content.render(renderer, rect)` — confirm this still receives a local
rect appropriate to `content`'s parent frame (itself), not a world rect.

**Checkpoint**: `cargo test -p cvkg-vdom` — expect `world_space_panel_tests.rs`
and `vdom_integration_tests.rs` to start failing here if they assert on
`node.layout` values directly rather than through a public accessor. Fix
tests to call `vdom.world_rect(id)` wherever they previously read
`node.layout` expecting an absolute value; leave assertions on `node.layout`
alone wherever the test's intent was genuinely about local offset (e.g.
"this node is 10px right of its parent").

---

## Phase 4 — Diffing (`src/diff.rs`)

`diff_node` (~line 304-395) currently does `old_node.layout != new_node.layout`
and emits a full `Update { layout: Some(new_node.layout), .. }` patch
whenever they differ. This is unchanged in shape — `layout` is now local, so
a parent moving no longer changes any child's `layout`, which means:

1. `layout_changed` will now correctly be `false` for children of a moved
   parent (only the parent's own `Update` patch fires). This is the payoff
   of the migration — confirm it with a new test (Phase 9) rather than
   assuming it.
2. No changes needed to `VDomPatch::Update`'s shape or to `Move` — both
   already operate per-node and per-local-rect, which is now the correct
   semantics for free.
3. Audit `apply_patches` in `lib.rs` for any place that assumes applying an
   `Update` to a parent must also touch descendants' stored `layout` — there
   should be none (grep confirms `Update` only ever indexes `self.nodes.get_mut(&id)`
   for the single patched id), but confirm after Phase 3 lands.

**Checkpoint**: `cargo test -p cvkg-vdom -- diff` green.

---

## Phase 5 — Physics and animation (`src/physics.rs`, `src/animated.rs`)

`Spring::tick` interpolates a `Rect` (`target` → `current`) and writes the
result via `self.current.set_with_flags(next_bounds, DirtyFlags::LAYOUT)`.

1. Confirm what `Spring` is attached to. If a `Spring` animates a single
   node's **local** rect (offset from its own parent), no change is needed —
   composition at read time means children automatically track the moving
   parent with zero extra work, which is the main win from this migration.
2. If any call site was using `Spring` to animate a node's rect and then
   manually propagating that position into descendants' `layout` fields
   (a workaround for the old absolute-storage model), delete that
   propagation code — it's now not just unnecessary but actively wrong
   (double-composition).
3. `AnimatedBox`'s `bounds_signal: Signal<Rect>` — confirm it's documented
   and used as a **local** rect override relative to `AnimatedBox`'s own
   parent, matching the new convention. Update the doc comment in
   `src/animated.rs` if it currently implies absolute/world bounds.

**Checkpoint**: `cargo test -p cvkg-vdom -- physics` and any reactivity/panel
integration tests green.

---

## Phase 6 — `WorldSpacePanel` composition (`src/vnode.rs`, `src/lib.rs`)

`WorldSpacePanel` already carries a `Transform3D` and `pixels_per_unit` —
this is the one place composition is genuinely 3D, not 2D-offset addition.

1. In `world_rect` (Phase 1), the current code intentionally `break`s the 2D
   walk when it hits an ancestor with `world_space: Some(_)`. Add the
   complementary function for that case:
   ```rust
   /// Resolve a node's position when it is inside a WorldSpacePanel subtree:
   /// local 2D offset within the panel's own offscreen-texture space,
   /// combined with the panel's Transform3D and pixels_per_unit.
   pub fn world_space_position(&self, id: NodeId) -> Option<WorldSpaceResolvedPos> { ... }
   ```
   Exact return shape depends on what `cvkg-scene`/`cvkg-render-native`
   need to composite the offscreen texture as a quad — check their current
   consumers of `WorldSpacePanel` before inventing a new type.
2. Any consumer that needs "where is this node, period" (hit-testing,
   AccessKit) must call a single dispatcher that checks whether the node's
   ancestor chain passes through a `WorldSpacePanel` and routes to either
   `world_rect` (pure 2D) or `world_space_position` (3D-projected) — do not
   make every consumer re-implement that branch.

**Checkpoint**: `cargo test -p cvkg-vdom -- world_space_panel` green.

---

## Phase 7 — AccessKit bridge (`src/accesskit_bridge.rs`)

Lines ~129-133 and ~328-332 build `accesskit::Rect` directly from
`self.layout.x/y` and `node.layout.x/y` — both assumed absolute today.
Replace both with `vdom.world_rect(id)` (or the `WorldSpacePanel`-aware
dispatcher from Phase 6 if the node may be panel-nested). AccessKit consumers
outside this crate must keep seeing absolute screen bounds — this is the
one place that composition is mandatory, not optional.

**Checkpoint**: `cargo test -p cvkg-vdom -- accesskit` green.

---

## Phase 8 — Wire GPU input events into `cvkg-vdom`'s dispatch, not around it

This is the other half of "no competing systems," now that Phase 0 deleted
the dead registry. `cvkg-render-gpu`'s own README already says input routing
is the caller's responsibility — make the caller actually call `cvkg-vdom`.

1. Find where `cvkg-render-gpu` (or its caller, likely `cvkg-render-native`)
   receives raw `winit` input events (cursor moved, mouse button, touch).
   Do not have `GpuRenderer` translate these into `cvkg_core::Event` and
   handle them itself — it should hand the raw window-space coordinates to
   whoever holds the live `VDom`.
2. At that call site, invoke `VDom::hit_test(x, y, pointer_precision)` to
   resolve the target `NodeId`, then `VDom::dispatch_event(event)` /
   `dispatch_event_to_target` (`cvkg-vdom/src/lib.rs:490, 619`) to run
   bubbling and invoke the registered handlers from `VDom.event_handlers`.
   This is the exact mechanism `register_handler` in `VNodeRenderer` already
   populates during evaluation (`lib.rs:1518-1533`) — there is nothing left
   to build here, only to connect.
3. Coordinate space check: `hit_test` takes `x, y` and compares against
   `node.layout` via the SDF distance helper. After Phase 2, confirm
   `hit_test_recursive` is composing world coordinates internally (it now
   must, since `node.layout` is local) so that the `x, y` passed in from the
   window/cursor position — which is a world/screen coordinate — is compared
   correctly at every depth. This is why Phase 2's fix to
   `hit_test_recursive` has to land before this phase is wired up.
4. Delete any input-handling code path in `cvkg-render-gpu` or
   `cvkg-render-native` that was resolving "what did the user click" by
   walking GPU-side draw state, z-order buffers, or anything other than
   `VDom::hit_test`. If such a path exists, it is the second half of the
   competing system and must be removed, not kept as a fallback.

**Checkpoint**: an end-to-end input test — synthesize a click at a known
screen coordinate inside a nested, panel-free subtree, assert the correct
handler fires exactly once via `VDom::dispatch_event`, with no code path in
`cvkg-render-gpu` independently matching or firing handlers.

---

## Phase 9 — Tests to add (beyond fixing existing ones)

Add to `cvkg-vdom/tests/`:

1. `nodal_coordinates_tests.rs`:
   - A 3-level tree (panel → row → label). Move the top node's local
     `layout.x/y` via an `Update` patch. Assert the label's `world_rect`
     changes by the same delta and the label's own stored `layout` is
     byte-identical before and after (proves composition, not storage).
   - Diff two trees where only the root moved. Assert `diff()` returns
     exactly one `Update` patch (the root's), not one per descendant —
     this is the concrete regression test for the diff-churn problem this
     migration fixes.
   - A `Spring` animating a parent's local rect across several `tick()`
     calls. Assert a child's `world_rect` tracks the parent every tick
     without any patch being applied to the child.
   - `validate_node_sync` still correctly reports drift when `cvkg-scene`'s
     `world_rect` is deliberately desynced, and correctly reports no drift
     otherwise (guards against a no-op comparison bug from Phase 2).
2. `gpu_event_wiring_tests.rs` (or extend
   `cvkg-render-gpu/tests/integrated_ui_scenarios.rs`): the end-to-end click
   test from Phase 8's checkpoint, plus a negative test asserting
   `GpuRenderer` has no `event_handlers` field / `get_handlers` method
   (compile-fail or reflection-style check, whichever the test harness
   supports) so the deleted system can't silently be reintroduced.

---

## Phase 10 — Final verification

Run, in order, and require all green before calling this done:

```
cargo check --workspace
cargo test -p cvkg-vdom
cargo test -p cvkg-render-gpu
cargo check -p cvkg-render-gpu --tests
```

Then, per each crate's own `TLDR.md` verification section:

- `cvkg-vdom`: `cargo test -p cvkg-vdom`, `cargo check --workspace`.
- `cvkg-render-gpu`: `cargo check -p cvkg-render-gpu --tests`,
  `cargo test -p cvkg-render-gpu`, plus the pixel-comparison render tests and
  a native demo run if available, since Phase 0's trait surgery touches the
  `Renderer` trait every draw call goes through.

Finally, grep the full workspace (not just these two crates) for
`world_rect`, `.layout.x`, `.layout.y` outside `cvkg-vdom` to catch any
other consumer (e.g. `cvkg-components`, `cvkg-render-native`) that was
reading `VNode.layout` assuming absolute coordinates and hasn't been updated
in this pass — this plan only covers the two crates provided, and the
`README.md` dependency graphs show at least `cvkg-components` and
`cvkg-render-native` also depend on `cvkg-vdom`.

## Rollback strategy

Land Phase 0 as its own commit (pure deletion, independently revertible).
Land Phases 1-7 as one commit or stack per phase — each phase's checkpoint
is a valid revert point since `layout`'s meaning only fully changes once
Phase 3 lands; Phases 1-2 can be reverted together cleanly if Phase 3
surfaces a call site this plan didn't anticipate. Land Phase 8 only after
Phases 1-7 are merged and stable, since it's the phase that changes runtime
behavior for real users (input dispatch), not just internal representation.
