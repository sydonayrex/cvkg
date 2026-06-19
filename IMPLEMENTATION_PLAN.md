# Implementation Plan: P0 Layout Performance & VDOM Event Dispatch

**Date:** 2026-06-19
**Scope:** cvkg-vdom, cvkg-layout, cvkg-render-native, cvkg-render-gpu, cvkg-core
**Symptom:** berserker demo runs at 13.7s/frame layout time, click boxes non-functional
**Root Cause:** Full VDOM rebuild + full layout + full text shaping every frame; event handlers lost on rebuild

---

## Problem Diagnosis

### Evidence from berserker logs

```
Frame timings: layout=13734.00ms state=0.04ms draw=1324.60ms submit=23.77ms total=15082.41ms
VDOM Built with 127 nodes -> 129 -> 131 -> 133 -> 135 (fluctuating per frame)
Mouse events dispatched ("Dispatching PointerDown to VDOM") but buttons never respond
Naga typifier spam at startup (one-time shader compilation cost, not runtime)
```

### Root cause chain

1. `NativeRenderer::window_event` calls `VDom::build(&self.view, rect)` every frame (cvkg-render-native/src/lib.rs:962)
2. `VDom::build` creates a fresh `VNodeRenderer`, calls `view.render(&mut renderer, rect)` which traverses the entire component tree (cvkg-vdom/src/lib.rs:565-569)
3. Every component re-executes its full `render()` method: `measure_text`, `draw_text`, `fill_rounded_rect`, `push_vnode`, `register_handler` -- all involving heap allocations and dynamic dispatch
4. `measure_text` shapes text via HarfBuzz per-frame per-element -- the single biggest cost
5. Event handlers registered via `r.register_handler()` on the old VDOM are destroyed when the new VDOM replaces it -- handlers were in the old VDOM's `event_handlers` HashMap
6. Layout engine recalculates positions for all 127+ nodes with no dirty tracking or caching

### Cost breakdown (estimated from 13.7s layout)

| Component | Est. Cost | Why |
|-----------|-----------|-----|
| Text shaping (measure_text) | ~40% | HarfBuzz shaping per frame for 50+ text elements |
| VDOM allocation (HashMap, Vec) | ~20% | 127+ nodes, each with children Vec, styles, etc |
| View trait dispatch (dynamic) | ~15% | Each component's render() is a vtable call + branching |
| Layout calculation (cvkg-layout) | ~15% | Stacks/grids with 127+ nodes, no caching |
| Accessibility tree build | ~10% | to_accesskit_node() for every created/replaced node |

---

## Phase 1: Stop the Per-Frame Churn (biggest win)

**Goal:** VDom::build stops redoing work that is stable across frames. Split "capture" from "diffable state" so unchanged subtrees are reused instead of re-rendered wholesale.

**Files:** cvkg-core/src/lib.rs, cvkg-vdom/src/lib.rs, cvkg-render-native/src/lib.rs
**Effort:** 2-3 days

### 1.1 Add View dirty-flagging

Add methods to the View trait so components signal when their render output actually changes:

```rust
// cvkg-core/src/lib.rs
pub trait View {
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect);
    type Body: View;
    fn body(self) -> Self::Body;

    /// Return true when this view's render output has changed since last build.
    /// Default true for backward compatibility. Override for incremental skip.
    fn changed(&self) -> bool { true }

    /// Stable identity for diff keying. Return None for anonymous views.
    fn view_id(&self) -> Option<u64> { None }
}
```

Key insight: most UI is static between frames. Only animated elements (rage counter, fuse animation, particle count) actually change. The NornirBar, dock, corner buttons, PerfOverlay, and closed menus are all static chrome that should skip re-rendering entirely.

### 1.2 Cache VDOM between frames

In NativeRenderer, keep the previous VDOM and only rebuild when the view signals a change:

```rust
// cvkg-render-native/src/lib.rs (line 962 area)
// Current:
let new_vdom = cvkg_vdom::VDom::build(&self.view, rect);

// New:
let new_vdom = if self.view.changed() {
    cvkg_vdom::VDom::build(&self.view, rect)
} else {
    state.vdom.as_ref().expect("vdom exists on redraw").clone()
};
```

This alone should cut layout time by ~80% for static frames.

### 1.3 Add cache boundaries for static chrome subtrees

For subtrees that never change (NornirBar, PerfOverlay, closed ContextMenu branches), add an explicit cache boundary in the VDOM layer:

```rust
// cvkg-vdom/src/lib.rs
impl VDom {
    /// Build only the dirty subtrees, reusing cached VNodes for clean ones.
    pub fn build_incremental(view: &V, rect: Rect, prev: &VDom) -> Self { ... }
}
```

Invalidation triggers (explicit state changes only):
- Open/close menu
- Counter/rage changes
- Window size changes
- Animation ticks (time-based)

### 1.4 Fix event handler survival across rebuilds

The current design stores event_handlers inside the VDOM. When VDOM is rebuilt, handlers are lost. This is why click boxes are broken.

**Solution:** Move handler registration out of VDOM into a stable HandlerRegistry owned by NativeRenderer:

```rust
// cvkg-vdom/src/lib.rs
struct HandlerRegistry {
    handlers: HashMap<(NodeId, String), Arc<dyn Fn(Event) + Send + Sync>>,
}

// cvkg-render-native/src/lib.rs
struct AppState {
    vdom: Option<VDom>,
    handlers: HandlerRegistry,  // survives VDOM rebuilds
    focus_manager: FocusManager,
}
```

Component handlers register against stable node IDs. When VDOM rebuilds, the same node IDs get the same handlers. The VDom::dispatch_event method queries the HandlerRegistry by (node_id, event_name) instead of looking up handlers from its own HashMap.

---

## Phase 2: Cut the Hottest Text Cost

**Goal:** Text measurement and shaping caches at the renderer boundary so repeated measure_text/draw_text calls reuse shaped runs instead of reshaping identical strings every frame.

**Files:** cvkg-render-gpu/src/renderer.rs, cvkg-runic-text/src/lib.rs
**Effort:** 1-2 days

### 2.1 Add text shaping cache at renderer boundary

```rust
// cvkg-render-gpu/src/renderer.rs
struct TextCache {
    entries: HashMap<(String, f32), ShapedResult>,
}

struct ShapedResult {
    width: f32,
    height: f32,
    glyphs: Vec<GlyphPlacement>,
}
```

Cache lives on the Renderer (one per frame). On cache hit, `measure_text` returns cached dimensions without calling HarfBuzz. Cache is cleared at the start of each frame -- any text not rendered that frame is evicted next frame. This is a simple "render-time cache" with no complex invalidation needed.

### 2.2 Hoist static labels out of frame-local formatting

In berserker, menu titles ("File", "Edit", "View", "Window", "Help"), dock labels, overlay labels, and repeated menu item strings are formatted every frame. These should be pre-shaped once and reused:

```rust
// Pre-shape static labels at init time
let shaped_file = renderer.shape_text("File", 13.0);
let shaped_edit = renderer.shape_text("Edit", 13.0);
// ... etc
```

### 2.3 Cache shaped output, not raster only

Keep quality high by caching the full shaped output (kerning, fallback, ligatures, positioning), not just raster bitmaps. This ensures text quality is identical on cache hits.

**Estimated savings:** ~5.5s per frame (text shaping at 40% of 13.7s, ~90% cache hit rate expected for static UI)

---

## Phase 3: Make Layout Incremental

**Goal:** Size and placement reuse cached results for unchanged proposals and stable child lists. Invalidation propagates only up the ancestor chain.

**Files:** cvkg-layout/src/node.rs, cvkg-layout/src/lib.rs
**Effort:** 2-3 days

### 3.1 Track layout dirtiness per node

```rust
// cvkg-layout/src/node.rs
struct LayoutNode {
    rect: Rect,
    children: Vec<LayoutNode>,
    dirty: bool,           // true when content/size changed
    cached_size: Option<Size>,
    cached_rect: Option<Rect>,
}
```

### 3.2 Incremental layout pass

When `dirty == false`, return cached_size and cached_rect without recursing. Only recurse into dirty subtrees. Dirty flags propagate UP the ancestor chain only (a child change marks its parent dirty, not siblings).

### 3.3 Budgeted layout service

If a subtree exceeds the per-frame budget, reuse previous rects and keep the frame moving instead of blocking for a full recompute. This prevents the 13.7s stall from ever happening again:

```rust
// cvkg-layout/src/lib.rs
pub fn layout_with_budget(root: &mut LayoutNode, budget: Duration) -> LayoutResult {
    let start = Instant::now();
    // ... incremental layout, but if budget exceeded, stop and reuse cached rects
}
```

### 3.4 Maintain fidelity

Only skip layout for unchanged branches. Animated or size-changing nodes still relayout. The budget is a safety net, not the primary path -- incremental layout should handle 99% of frames.

**Estimated savings:** ~2s per frame

---

## Phase 4: Reduce VDOM Size Pressure

**Goal:** Avoid pushing nodes for purely decorative or repeatedly recomputed elements when they do not participate in hit testing or accessibility. Batch or flatten obvious leaf-heavy branches.

**Files:** cvkg-vdom/src/lib.rs, cvkg-render-gpu/src/renderer.rs
**Effort:** 1 day

### 4.1 Skip decorative nodes

Nodes that are purely visual (background fills, decorative lines, glass effect overlays) and don't participate in hit testing or accessibility should be batched into a single draw call instead of creating individual VNodes.

### 4.2 Flatten leaf-heavy branches

In berserker, the NornirBar has 5 menu items each with text + hit area + handler. These can be batched into a single VNode with 5 sub-rects instead of 5 separate VNodes with 5 separate handler registrations.

### 4.3 Keep interactive nodes intact

Pointer routing, focus, and accessibility must not regress. Only flatten/skip nodes that are confirmed non-interactive.

### 4.4 Reuse allocations

Reuse VNode HashMap and Vec allocations across frames. Pre-allocate VDomPatch buffers instead of allocating fresh every frame.

---

## Phase 5: Frame-Budget Enforcement

**Goal:** Hard frame budget through Native and Layout so the app degrades gracefully before dropping into multi-second stalls.

**Files:** cvkg-render-native/src/lib.rs, cvkg-layout/src/lib.rs
**Effort:** 1 day

### 5.1 Thread budget through the pipeline

```rust
// cvkg-render-native/src/lib.rs
const FRAME_BUDGET: Duration = Duration::from_millis(16);  // 60 FPS target

// In RedrawRequested handler:
let frame_start = Instant::now();
// ... VDOM build, layout, draw ...
let elapsed = frame_start.elapsed();
if elapsed > FRAME_BUDGET {
    log::warn!("Frame budget exceeded: {:?} (budget: {:?})", elapsed, FRAME_BUDGET);
}
```

### 5.2 Graceful degradation

Use the budget to decide when to:
- Reuse previous layout rects (skip layout for non-dirty subtrees)
- Skip non-essential overlay updates (PerfOverlay, debug info)
- Defer secondary chrome work to next frame

### 5.3 Explicit telemetry

Make the budget explicit in telemetry so we can prove 120+ FPS on reasonable hardware instead of guessing. Log: frame time, budget remaining, which phases exceeded budget.

---

## Phase 6: Click Box Regression Tests

**Goal:** Add regression tests for menu items, dock buttons, overlay dismissal, and open/close transitions under repeated VDOM rebuilds.

**Files:** cvkg-vdom/tests/, cvkg-test/tests/
**Effort:** 1 day

### 6.1 Test matrix

| Scenario | Action | Expected |
|----------|--------|----------|
| Static frame (no change) | Click corner button I | Counter increments, handler fires |
| Static frame (no change) | Click menu "File" | Dropdown opens |
| Open menu state | Click outside overlay | Menu closes |
| Repeated rebuilds (100x) | Click button each time | All 100 clicks register |
| Counter/rage change | Click button during animation | Handler fires with correct state |
| Window resize | Click button after resize | Hit test maps to correct position |

### 6.2 VDOM rebuild stress test

```rust
#[test]
fn event_handlers_survive_100_rebuilds() {
    let view = TestView::new();
    let mut prev_vdom = None;
    for _ in 0..100 {
        let new_vdom = VDom::build(&view, Rect::new(0.0, 0.0, 1280.0, 720.0));
        if let Some(prev) = prev_vdom {
            let patches = prev.diff(&new_vdom);
            // Verify patches are minimal (no full rebuild)
            assert!(patches.len() < 5, "expected incremental patches, got {}", patches.len());
        }
        prev_vdom = Some(new_vdom);
    }
}
```

---

## Execution Order

| Phase | Impact | Risk | Depends On | Parallel With |
|-------|--------|------|------------|---------------|
| Phase 1: Stop per-frame churn | -80% layout time | Medium (View trait change) | Nothing | Phase 2 |
| Phase 2: Text shaping cache | -5.5s/frame | Low (isolated cache) | Nothing | Phase 1 |
| Phase 3: Incremental layout | -2s/frame | Medium (dirty tracking) | Phase 1 | Phase 4 |
| Phase 4: VDOM size pressure | -0.5s/frame | Low | Phase 1 | Phase 3 |
| Phase 5: Frame budget | Safety net | Low | Phase 1, 3 | Phase 6 |
| Phase 6: Click regression | Functional | Low | Phase 1 | Phase 5 |

**Recommended execution:**
- Week 1: Phases 1 + 2 in parallel (independent, biggest wins)
- Week 2: Phase 3 (depends on Phase 1)
- Week 2: Phases 4 + 5 + 6 in parallel (all depend on Phase 1, independent of each other)

---

## Verification

### Performance targets

| Metric | Current | After Phase 1 | After Phase 2 | After All |
|--------|---------|---------------|---------------|-----------|
| Layout time | 13734ms | ~2700ms | ~150ms | ~80ms |
| FPS | <1 | ~3 | ~30 | ~60-120 |
| Text cache hit | 0% | 0% | ~90% | ~90% |
| VDOM node count | 127-135 (unstable) | Stable | Stable | Stable |
| Frame budget | N/A | N/A | N/A | <16ms |

### Functional tests

1. berserker demo: click corner buttons I/II/III/IV, verify counter increments
2. berserker demo: click menu items, verify dropdown opens/closes
3. berserker demo: verify text renders correctly after cache warmup
4. berserker demo: verify handlers survive 100+ VDOM rebuilds
5. Run `cargo test --workspace` to verify no regressions

### Profiling

- Add `RUST_LOG=trace` to verify cache hit rates in Phase 2
- Use `perf` or `flamegraph` on native renderer to identify remaining hotspots after Phase 1
- Verify VDOM node count stays stable (not fluctuating 127->135)
- Add timing instrumentation for VDOM capture, text shaping, layout, and submit on the berserker path
- Benchmark specific hot subtrees under realistic UI states: no menu, open menu, overlay visible, fire active, high particle count

---

## Files to Modify

| File | Phases | Scope |
|------|--------|-------|
| cvkg-core/src/lib.rs | 1 | Add `View::changed()` and `View::view_id()` |
| cvkg-vdom/src/lib.rs | 1, 4, 6 | Incremental build, HandlerRegistry, allocation reuse, cache boundaries |
| cvkg-vdom/src/handlers.rs | 1 | Move to stable HandlerRegistry |
| cvkg-layout/src/node.rs | 3 | Add dirty flag, cached_size, cached_rect |
| cvkg-layout/src/lib.rs | 3, 5 | Incremental layout pass, budgeted layout |
| cvkg-render-native/src/lib.rs | 1, 5 | Cache VDOM, stable handler registry ownership, frame budget |
| cvkg-render-gpu/src/renderer.rs | 2, 4 | Text shaping cache, command buffer reuse |
| cvkg-runic-text/src/lib.rs | 2 | Expose cache-friendly shaping API |
| cvkg-vdom/tests/ | 6 | Rebuild stress tests, handler survival tests |
| cvkg-test/tests/ | 6 | Click box regression tests |

---

## Skills Per Phase

Load these skills before starting each phase. They contain proven workflows, API recipes, and pitfalls specific to the crates being modified.

### Phase 1: Stop the Per-Frame Churn

| Skill | Why |
|-------|-----|
| `cvkg-employment` | Umbrella for all CVKG framework work. Contains the component implementation contract, View trait rules, renderer trait split rules, and the "never label errors pre-existing" mandate. Critical for the View trait change in cvkg-core. |
| `wgsl-wgpu-shader-pipeline` | Documents the render graph, pass ordering, and fullscreen triangle patterns used by the GPU renderer. Needed to understand how VDOM draw calls map to GPU work. |
| `debugging` | Systematic debugging methodology for tracing the VDOM rebuild chain and identifying why handlers are lost. |

### Phase 2: Cut the Hottest Text Cost

| Skill | Why |
|-------|-----|
| `cvkg-employment` | Contains the text-glyph-debugging reference and the rendering pipeline contract. The text shaping cache must not break the render-mode contracts documented here. |
| `dsp-engineering` | Audio DSP programming guidance. The HarfBuzz shaping pipeline has similar real-time constraints -- this skill covers low-latency caching patterns that apply to text shaping. |

### Phase 3: Make Layout Incremental

| Skill | Why |
|-------|-----|
| `cvkg-employment` | Contains the layout contract rules (resize clamping, zero-size frame handling, draw_order patterns). The incremental layout must preserve all existing layout contracts. |
| `improve-codebase-architecture` | Framework for finding deepening opportunities in a codebase. Useful for identifying where layout nodes can be extended with dirty flags without breaking the existing architecture. |

### Phase 4: Reduce VDOM Size Pressure

| Skill | Why |
|-------|-----|
| `cvkg-employment` | Contains the component implementation rules and the "evaluate dead code before deleting" mandate. When flattening VDOM nodes, must verify each node has no hidden purpose. |
| `refactoring` | Per-change refactoring mechanics: count callers exactly, update all construction sites. Needed when changing VNode structure or removing decorative nodes. |

### Phase 5: Frame-Budget Enforcement

| Skill | Why |
|-------|-----|
| `cvkg-employment` | Contains the frame-budget-pass-skipping reference that classifies passes as cosmetic vs functional. The budget enforcement must never skip glass or accessibility passes. |
| `progress-summary` | Conversational PR-style summaries with visual diagrams. Useful for tracking budget metrics and reporting frame time improvements. |

### Phase 6: Click Box Regression Tests

| Skill | Why |
|-------|-----|
| `cvkg-employment` | Contains the component implementation contract and the "all errors are your mandate" rule. Tests must cover all interactive component patterns. |
| `tdd-workflow` | TDD discipline with red-green-refactor loop. Phase 6 should write failing tests first (RED), then verify the HandlerRegistry fix makes them pass (GREEN). |
| `strong-tests` | Universal test patterns. Contains the test file conventions, factory patterns, and edge case coverage rules needed for VDOM-level tests. |
| `test-patterns` | Write and run tests across languages and frameworks. Useful for the VDOM rebuild stress test which needs deterministic iteration. |
| `test-guard` | Review generated test code against universal test rules. Ensures the regression tests themselves are correct and not false-positives. |
