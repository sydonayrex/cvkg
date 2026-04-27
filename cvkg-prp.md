# CVKG Production Revision Plan
*Updated: v0.1.12 — April 2026*

> Items marked ✅ are **fully implemented** in the current codebase and are retained here
> as acceptance-criteria records. Items without a mark are open work.

---

## Priority 1 — State Management

### 1.1 Lock-free reactive state (`State<T>` / `Binding<T>`) ✅
- `State<T>` uses `Arc<ArcSwap<T>>` for lock-free render-loop reads and `Arc<TVar<T>>`
  (native) for STM-retry write semantics.
- `Binding<T>` shares the same `Arc<ArcSwap<T>>` as its parent `State<T>` — all holders
  always see the latest published value without copying.
- `State::mutate<F>` applies an STM-transacted read-modify-write on native; clone-store on WASM.
- `State::subscribe` / `Binding::set` — public API unchanged from the original `RwLock` surface;
  no callsite migration required.

### 1.2 Global state registry (`KnowledgeState` / `SYSTEM_STATE`) ✅
- `SYSTEM_STATE: OnceLock<Arc<ArcSwap<KnowledgeState>>>` — lock-free snapshot per render frame.
- `load_system_state()` returns an `arc_swap::Guard` safe to hold for one frame.
- `update_system_state(f)` — clone-and-swap; last-writer-wins; safe for single-field mutations.
- `transact_system_state(f)` — wraps `f` in `stm::atomically()` on `KNOWLEDGE_TVAR`; retries on
  conflict; guarantees no lost updates across multi-field mutations (`fragments` + `last_query_results`).
- `transact_pair(state_a, state_b, f)` — coordinates two independent `State<T>` cells in one
  STM transaction; atomic swap / transfer semantics with subscriber notification after commit.

### 1.3 Asset cache migration ✅
- `DefaultAssetManager`, `NativeAssetManager`, `WebAssetManager` all migrated from
  `Arc<RwLock<HashMap>>` to `Arc<ArcSwap<HashMap>>`.
- Reads are lock-free (`load().get(url)`); writes use `rcu()` (clone-insert-swap).
- `WebAssetManager` runs the second `rcu()` inside the WASM async fetch future — no lock is
  ever held across an `.await` point.

### 1.4 Dev-server state ✅
- `cvkg-webkit-server` `AppState.last_vdom_snapshot` migrated from `RwLock<Option<String>>`
  to `ArcSwap<Option<String>>`. HTTP reads are now lock-free; writes call `store()`.

### 1.5 Async state / `Suspense<T>` ✅
- Define a `Suspense<T>` wrapper that holds `AsyncState<T> { Loading, Ready(T), Error(E) }`.
- Wire into the Tokio/WASM-bindgen-futures executor so async state transitions publish via
  `update_system_state` without blocking the render thread.
- Integrate with `State<T>` subscription so components can declaratively react to each variant.
- `AssetState<T>` (already in cvkg-core) is the logical seed for this; `Suspense<T>` should
  unify async patterns across the framework.

### 1.6 Batched state update queue ✅
- Introduce an update queue that accumulates `update_system_state` / `State::set` calls within
  one frame tick.
- Flush the queue post-input, pre-layout to produce a single notification pass per frame.
- Prevents N state changes from triggering N separate layout+render cycles.

---

## Priority 2 — Performance and Scaling

### 2.1 Frame budget enforcement ✅
- Add a `FrameBudget` config (target: 16 ms) tracking per-pass timing:
  input → state-flush → layout → draw → GPU submit.
- Implement a graceful degradation ladder: reduce effect quality, drop LOD shaders, skip
  non-critical animations if the budget is exceeded.
- Expose a telemetry event so the host application can react.
- The `TelemetryData` struct already exists in cvkg-core; wire frame-timing data into it.

### 2.2 Scalability to 100k+ components ✅
- Implement virtualized `VirtualList<T>` and `VirtualTable<T>` components (only render
  components in the visible viewport).
- Add spatial culling in the layout pass so off-screen subtrees skip layout computation.
- `LayoutCache` already exists in cvkg-layout; partial invalidation hooks should be added.
- Benchmark explicitly at 10k / 50k / 100k components and gate PRs against a regression
  threshold.

### 2.3 GPU memory profiling ✅
- Integrate GPU memory tracking into the existing `TelemetryData` with per-resource-type
  breakdowns (textures, vertex buffers, pipelines).
- Add a debug overlay panel surfacing live memory stats alongside frame timing.

---

## Priority 3 — AI-Native Capabilities

### 3.1 Multi-agent conflict resolution ✅
- Define an `AgentTransaction` type wrapping a set of UI mutations with an agent identity and
  priority level.
- On conflict (two agents targeting the same component/state), apply a resolution strategy:
  last-writer-wins by default, configurable per-component to priority-wins or merge.
- Expose a conflict event stream for the host or orchestrating agent.
- `transact_system_state` / `transact_pair` already provide the STM retry substrate; the
  `AgentTransaction` layer sits above this and adds identity + priority policy.

### 3.2 Public agent manipulation API ✅
- Formalize a stable `AgentSurface` trait exposing: component creation, layout modification,
  workflow triggers, and state queries.
- Version this API explicitly so agents compiled against older CVKG versions don't break.
- Add integration tests that exercise the API from a simulated agent (no UI rendering required).

### 3.3 Agent development documentation ✅
- Write a dedicated guide covering: how to observe UI state via `load_system_state()`, how to
  issue commands via `update_system_state` / `transact_system_state`, how to handle async
  responses, and conflict resolution patterns.
- Include worked examples for common agentic patterns: form-filling, navigation, reactive data
  binding, multi-field atomic mutations.

---

## Priority 4 — Rendering System

### 4.1 GPU fallback chain ✅
- WebGPU backend exists in `cvkg-render-web` (behind `webgpu` feature flag).
- Canvas 2D fallback exists as the default web renderer.
- WebGL2 tier implemented via `Tier2GPU` with automated detection.
- Bifrost / Gungnir / Mjolnir shader passes degrade gracefully at each tier
  (glassmorphism → simple transparency, neon → flat colour).
- Automated `forge()` method at initialization selects the highest supported backend dynamically.

### 4.2 Material system documentation ✅
- Documented the shader composition pipeline: pass ordering (Bifrost → Gungnir →
  Mjolnir), authoring new materials, and effect layering.
- Implemented `MaterialRegistry` in `cvkg-core` allowing components to declare material requirements.


### 4.3 vDOM diff implementation ✅
- Implement a real keyed-diff algorithm to produce minimal patch sequences.
- Gate against regression with snapshot tests for common mutation patterns.
---

## Priority 5 — Layout Engine

### 5.1 Virtualized list and table components ✅
- Built `VirtualList<T>` and `VirtualTable<T>` in `cvkg-components`.
- Implemented efficient O(visible) rendering using vertical virtualization.
- Integrated with `LayoutView` and `AnyView` for type-erased item/cell builders.

### 5.2 Layout debugging tools ✅
- Added `query_layout`, `set_debug_layout`, and `get_debug_layout` to `Renderer` trait.
- Implemented debug layout overlay in `WebRenderer` and `SurtrRenderer` (magenta bounds + labels).


---

## Priority 6 — Animation System

### 6.1 Layout-thrashing prevention ✅
- Audited the `SleipnirSolver` / `Motion` pipeline to ensure modifiers remain layout-neutral.
- Verified the transform-only animation path (via `push_transform`) operates entirely in GPU space.
- Implemented a visual "Layout Flash" lint and `LAYOUT THRASH DETECTED` warning for mutation detection.

---

## Priority 7 — Code Quality

### 7.1 Placeholder component implementations ✅
- `TextField` / `TextEditor` in `cvkg-components/src/interactive.rs` are stubs; implement
   real text-input handling including IME composition (Ime event already exists in cvkg-core). ✅
- `AsyncImage` loading state in `cvkg-components/src/image.rs` uses a placeholder; wire it
   to `WebAssetManager` / `NativeAssetManager` via the `AssetKey` environment. ✅
- Error boundary in `cvkg-components/src/error.rs` needs a real fallback render path. ✅

### 7.2 Reduce component boilerplate ✅
- Introduce a `#[cvkg_component]` derive macro (skeleton exists in cvkg-macros) that generates
  the standard `Component` trait impl, builder pattern, and modifier-chain scaffolding.
- Target: a minimal component should be expressible in ~10 lines, not ~40.

### 7.3 Build time optimization ✅
- Evaluate shader pre-compilation and caching at build time (not just runtime pipeline caching).
- Profile the WASM build pipeline and identify the heaviest compilation units for
  parallelization or lazy compilation.

---

## Priority 8 — Security

### 8.1 Plugin sandboxing ✅
- Implemented `Capability` system and `SecurityPolicy` for granular permission enforcement.
- Integrated `SandboxLimits` for resource isolation (CPU/Memory/Events).
- Documented WASM-first isolation strategy in the threat model.

### 8.2 Security documentation ✅
- Authored `Specs/CVKG_Security_Threat_Model.md` covering trust boundaries, threat scenarios (T1-T4), and mitigation strategies.

---

## Execution Order

| Phase | Items | Rationale |
|---|---|---|
| **Now** | 1.5, 1.6, 4.3 | Completes the state system (Suspense, batching) and fixes the vDOM stub — the two most glaring missing pieces |
| **Next sprint** | 2.1, 2.2, 3.1 | Performance + multi-agent are the highest architectural risk before a wider release |
| **Following sprint** | 3.2, 3.3, 4.1, 4.2 | Stabilizes the AI surface and rendering fallback chain |
| **Backlog** | 5.1, 5.2, 6.1, 7.1, 7.2, 7.3 | High value but not blocking launch |
| **Ongoing** | 8.1, 8.2 | Security can be incrementally addressed |

---

## Completed Items (retained as acceptance records)

| Item | Completed | Notes |
|---|---|---|
| 1.1 `State<T>` / `Binding<T>` ArcSwap+STM | v0.1.12 | Lock-free reads, STM mutate, subscribers |
| 1.2 `SYSTEM_STATE` / `KnowledgeState` global | v0.1.12 | `load_system_state`, `update_system_state`, `transact_system_state`, `transact_pair` |
| 1.3 Asset manager cache migration | v0.1.12 | All three backends: Default, Native, Web |
| 1.4 Dev-server `AppState` migration | v0.1.12 | `ArcSwap<Option<String>>` for snapshot field |
