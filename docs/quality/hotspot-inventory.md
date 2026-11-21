# Panic / Unsafe Hotspot Inventory (P0-1 / P0-2)

Follow-up (2026-11-21): a fresh grep for `TODO|FIXME|XXX` in non-test
`cvkg-*/src` and `demos/*/src` found exactly one remaining hit —
`cvkg-gallery/src/state.rs:201` (`persisted_sidebar_width = 240.0`), which is
demo-only and intentionally left behind a dev boundary. All production TODOs
are closed.

Audit date: 2026-11-21. Scope: `cvkg-gallery/src/`, `cvkg-scene/src/`,
`cvkg-render-gpu/src/renderer/` (draw, init, mod, frame, context_helpers, geometry,
pipelines, svg, capability), `cvkg-render-native/src/`, `cvkg-anim/src/`,
`cvkg-svg-filters/src/pipeline.rs`, `cvkg-physics/src/`.

Method: ripgrep over the exact tokens `unwrap()`, `expect(`, `panic!`,
`unreachable!`, `todo!`, `.lock().unwrap()`, `unsafe` (plus `TODO`/`unimplemented!`
as secondary signals), then **every** library hit was re-read at the cited line to
confirm the snippet and its enclosing scope. Excluded: `#[cfg(test)]` modules,
`mod tests` blocks, all files under `tests/`, and doc-comment prose that merely
mentions the word "unsafe" (e.g. the `//! 6. COMMENT ALL ... unsafe block` banner
at `cvkg-scene/src/lib.rs:14`, `cvkg-render-native/src/lib.rs:14`,
`cvkg-anim/src/lib.rs:14`, `cvkg-physics/src/lib.rs:14`).

---

## 1. Summary

### Counts by crate / area (library + runtime paths only)

| Area | unwrap / expect | panic! | unreachable! | todo / TODO | `.lock().unwrap()` | unsafe (block/fn/impl) |
|---|---:|---:|---:|---:|---:|---:|
| `cvkg-gallery/src/` — **demo only** | 0 | 0 | 36 | 1 (demo TODO) | **31** | 0 |
| `cvkg-scene/src/` | 2 (`unwrap`) | 0 | 0 | 0 | 0 | 0 |
| `cvkg-render-gpu/src/renderer/` | **58** (21 `expect`, 37 `unwrap`) | 0 | 0 | 0 | 0 (safe `match lock.lock()` at mod.rs:735) | 3 |
| `cvkg-render-native/src/` | 5 (3 `expect`, 2 `unwrap`) | 0 | 0 | 0 | 0 (uses `unwrap_or_else` recovery) | **8** |
| `cvkg-anim/src/` | 2 (`unwrap`) | 0 | 0 | 0 | 0 | 0 |
| `cvkg-svg-filters/src/pipeline.rs` | 0 | 0 | 0 | 2 | 0 | 0 |
| `cvkg-physics/src/` | 0 | 0 | 0 | 0 | 0 | 1 |
| **TOTAL** | **67** | **0** | **36 (demo)** | **3 (2 prod + 1 demo)** | **31 (demo)** | **12** |

Core-library totals (excluding all demo code): **67 unwrap/expect, 0 panic!,
2 TODO, 12 unsafe sites**. No `panic!` macro occurs in any scanned library path —
every `panic!` hit is inside `#[cfg(test)]` code.

### Excluded test-only hits (documented for completeness)

- `cvkg-scene/src/lib.rs:615+` (`mod tests`): 16 `unwrap`/`expect` hits
  (lines 673, 674, 726, 747, 773, 822, 828, 834, 843-845, 848, 917, 940-942).
- `cvkg-render-gpu/src/renderer/tests.rs` (whole file is `#[cfg(test)]` via
  `mod.rs:20-21`): 15 lines incl. `panic!` at 27, 54, 69, 278 and 4
  `.lock().unwrap()` at 19, 26, 45, 53 (these are deliberate poison tests).
- `cvkg-render-native/src/tests.rs` (whole file `#[cfg(test)]` via `lib.rs:54-55`):
  ~41 lines incl. `panic!` at 160, 168, 188, 200, 207, 672, 692, 713, 732.
- `cvkg-anim`: 4 hits in test modules (skeletal.rs:518/529/541; growth.rs:1169;
  test modules start at skeletal.rs:506, growth.rs:914).
- `cvkg-physics`: 26 hits in test modules (scene_bridge.rs:179;
  heightmap.rs:213/214/242/265; xpbd.rs:798/799/862/863/864/893/916/923;
  narrowphase.rs:1141/1165/1247/1279; world.rs:1079/1124/1125/1130/1146/
  1160/1175/1258/1259). `world.rs:346` is a doc example.
- All files under `*/tests/` (snapshot/integration suites) ignored per scope.

### Key observations

- **GPU startup is the single largest panic cluster.** 21 `expect` calls, most in
  `forge`/`from_external`/`register_window`, abort the whole process on missing
  adapter/device/surface — even though each constructor already returns `Self`,
  not `Result`. `capability.rs:84-87` explicitly acknowledges this: *"request_device
  FAILS OUTRIGHT ... this is what currently risks a hard panic on weaker hardware
  via the unconditional .expect() at init.rs."*
- **22 of the 37 GPU `unwrap`s are one fragile short-circuit pattern**:
  `self.draw_calls.is_empty() || last_call.unwrap().field != ...`. They are
  currently safe only because of the first disjunct; any reordering panics.
- **The raw-pointer fast path in `cvkg-render-native` is the highest-UB-risk area**
  (`renderer.rs:96,115`, fed by `main_loop.rs:467`). Invariants are documented but
  not mechanically enforced.
- **The svg-filters component-transfer TODO (P0-5) is a silent correctness bug**,
  not a panic: R-channel function is applied to all four channels.

---

## 2. TOP 20 highest-risk sites

| # | file:line | snippet | category | why it can panic / break | severity | concrete fallible-API recommendation |
|---:|---|---|---|---|---|---|
| 1 | `cvkg-render-gpu/src/renderer/init.rs:127` | `let adapter = adapter.expect("Failed to find a suitable GPU for Surtr");` | expect | All adapter requests (high-power, low-power, software fallback) returned `None`; `forge()` aborts the process on startup on unsupported/headless hosts. | High | Add `async fn try_forge(...) -> Result<Self, RendererInitError>` containing the body; keep `forge(...) -> Self` as a thin wrapper that logs and calls it. Return a typed error instead of `expect`. |
| 2 | `cvkg-render-gpu/src/renderer/init.rs:178` | `.expect("Failed to create Surtr device even after capability detection")` | expect | `request_device` fails when `required_features`/tier `required_limits` exceed adapter limits (exact risk called out at `capability.rs:84-87`). Aborts on weaker hardware. | High | Same `try_forge` `Result` path; map the wgpu error into `RendererInitError::Device`, log tier + error (already done at 174-177) and propagate. |
| 3 | `cvkg-render-gpu/src/renderer/mod.rs:1439` | `let adapter = adapter.expect("Failed to find a suitable GPU for Surtr");` | expect | Headless forge: same total adapter failure as #1, kills headless/server rendering instead of returning an error. | High | Mirror #1 with `try_forge_headless(...) -> Result<Self, _>`; retain `forge_headless() -> Self` wrapper for API compatibility. |
| 4 | `cvkg-render-gpu/src/renderer/mod.rs:1472` | `.expect("Failed to create Surtr device even after capability detection")` | expect | Headless device creation failure; the `map_err` at 1468-1471 only logs, then panics anyway. | High | Propagate the already-produced error via `?` in the `try_forge_headless` Result path. |
| 5 | `cvkg-render-gpu/src/renderer/mod.rs:1608` | `.expect("No compatible adapter found")` | expect | `from_external(...)` (public, `-> Self`) panics if no adapter matches the externally supplied surface — e.g. surface from a different backend. | High | Add `try_from_external(...) -> Result<Self, RendererInitError>`; `from_external` delegates and documents the panic as a deliberate compatibility shim. |
| 6 | `init.rs:40`; `draw.rs:180`; `frame.rs:163` | `.create_surface(window.clone()).expect("Failed to create surface")` (×3) | expect | Surface creation fails on raw-window-handle incompatibility / destroyed window; kills startup or window registration. Three duplicated sites drift independently. | High | Route through one helper `fn create_surface_checked(...) -> Result<Surface, SurfaceError>`; `try_forge` propagates, `register_window` logs + returns a `bool`/error status rather than panicking. |
| 7 | `cvkg-render-native/src/renderer.rs:157` | `.expect("failed to create winit event loop: platform initialization failed")` | expect | Winit event-loop build fails (unsupported platform / display server); entire native app aborts with no user-facing fallback. | High | Return `Result<Self, NativeInitError>` from an internal constructor; public `run` logs a clear error and returns instead of unwinding across FFI/event-loop setup. |
| 8 | `cvkg-render-native/src/renderer.rs:180` | `.expect("winit event loop terminated with error")` | expect | Event-loop `run_app` error after the app is already torn down; panics during shutdown/panic-unwind. | High | Replace with `if let Err(e) = ... { tracing::error!(...); }` and return normally — no API change. |
| 9 | `cvkg-render-native/src/window.rs:340` | `.expect("failed to create native window")` | expect | Window creation fails (bad attrs, headless display, platform shadow extension); aborts. | High | Change the internal `create_window` helper to return `Result<Window, WindowError>`; caller can surface a dialog/error rather than panic. |
| 10 | `context_helpers.rs:254,257,260,263` and `593,596,599,602` | `.get_texture_view(blur_tex_a).expect("resize: blur_tex_a view must exist after allocation")` (×8) | expect | If allocation/registry registration is ever skipped (OOM, capability tier change, resize race), the bind-group build panics. These are `pub(crate)` helpers — safe to make fallible internally. | High | Change `create_surface_context`/`create_headless_context` to return `Result<SurfaceContext, ContextError>` and use `?`; callers already inside fallible-ish init paths. Alternatively pass a validated dummy view fallback + warn. |
| 11 | `cvkg-render-native/src/renderer.rs:96` | `GpuRef::Ptr(unsafe { &mut *raw })` | unsafe / raw ptr | Aliasing `&mut GpuRenderer` from a thread-local raw pointer. UB if `GpuFramePtrGuard` outlives the `MutexGuard` or a reentrant render call occurs. | High | Keep the fast path but add a runtime `debug_assert!` that the guard is live (thread-local generation counter), or drop the pointer fast path in debug builds and always use `lock_or_recover`. Add a targeted reentrancy test. |
| 12 | `cvkg-render-native/src/renderer.rs:115` | `GpuRefShared::Ptr(unsafe { &*raw })` | unsafe / raw ptr | Same aliasing hazard via `&GpuRenderer`; additionally creates a shared reference while a mutable one may be live higher in the stack (stacked-borrows violation). | High | Same as #11; consider splitting the thread-local into separate `PtrMut`/`PtrShared` slots so shared/mutable use cannot overlap. |
| 13 | `cvkg-render-native/src/main_loop.rs:467` | `let _guard = unsafe { GpuFramePtrGuard::set(raw) };` | unsafe / raw ptr | Feeds #11/#12; `raw` is derived from `&mut *gpu` where `gpu` is a `MutexGuard`. Correct today, but if the guard is moved out of the lexical scope the invariant silently breaks. | Medium | Have `GpuFramePtrGuard::set` accept a lifetime-bound `&mut GpuRenderer` and store `NonNull` with an invariant marker; keep the existing `// SAFETY:` comment at 466 and extend it to reference the lock lifetime explicitly. |
| 14 | `draw.rs:960,963,980,983` | `let ctx = self.surfaces.get(&window_id).unwrap();` / `self.headless_context.as_ref().unwrap()` | unwrap | Frame-submit path: mismatched `current_window` vs registered surfaces (window destroyed mid-frame) panics while building the frame graph. | Medium | `let Some(ctx) = ... else { tracing::error!(...); return; };` — `render_frame` already returns `()`, so no API change; skip frame instead of aborting. |
| 15 | `draw.rs:1128` | `let cached = self.cached_graph_plan.as_ref().unwrap();` | unwrap | The `if` block at 1120-1126 populates the cache; if that branch is ever skipped/returns early, the frame panics. | Medium | `let Some(cached) = self.cached_graph_plan.as_ref() else { tracing::warn!(...); return; };` (or recompute the plan inline). |
| 16 | `draw.rs:910,921,932` | `wgpu::BufferSize::new(v_bytes.len() as u64).unwrap()` (×3) | unwrap | `BufferSize::new(0)` is `None`; guarded today by `!is_empty()`, but a zero-sized element type or refactor of the guard makes staging upload panic. | Medium | `let Some(size) = BufferSize::new(len) else { tracing::warn!("skip empty staging upload"); continue; };` — skip the write instead of unwrapping. |
| 17 | `draw.rs:457-461,678-684,1687-1691`; `geometry.rs:214-218,380-386` | `self.draw_calls.is_empty() \|\| last_call.unwrap().scissor_rect != scissor \|\| ...` (22 unwraps in 5 expressions) | unwrap | Only the leading `is_empty()` disjunct keeps these safe. Reordering, caching `last_call` across a mutation, or a future `||` branch that runs first causes a panic on every draw call. | Medium | Replace each chain with `last_call.is_none_or(\|c\| c.scissor_rect != scissor \|\| c.panel_id != current \|\| ...)` (bind current values to locals first to avoid borrow conflicts). Mechanically removes 22 unwrap sites with no API change. |
| 18 | `cvkg-scene/src/lib.rs:174,181` | `self.nodes.get(&root_id).unwrap().local_rect` / `self.nodes.get_mut(&id).unwrap()` | unwrap | `nodes` and `root` are **public fields** (`lib.rs:111-112`); external code can delete/rewrite them, leaving `root` dangling. `update_transforms()` then panics for all descendants. | Medium | `let Some(root_rect) = self.nodes.get(&root_id).map(\|n\| n.local_rect) else { debug_assert!(false, "dangling scene root"); return; };` and likewise in `update_node_transform`. Both fns return `()`, so no API change; scene has no `tracing` dep, so use `debug_assert!` + early return. |
| 19 | `cvkg-anim/src/lib.rs:605`; `growth.rs:234` | `child.solver.as_mut().unwrap()` / `rules.last().unwrap()` | unwrap | A child registered without a solver panics mid-tick; an empty rule slice (`rules.last()`) panics in weighted selection. Both reachable via public builder state. | Medium / Low | Use `let Some(solver) = ... else { tracing::warn!("child missing solver"); continue; };` and `rules.last().ok_or(AnimError::NoRules)?` (or `unwrap_or(&DEFAULT_RULE)`); change `select_rule` to return `Option<&Rule>` internally. |
| 20 | `cvkg-svg-filters/src/pipeline.rs:566,569` | `// TODO: Pack func_r, func_g, func_b, func_a into param0..param3` / `log::debug!("... using R channel function for all channels (full per-channel support TODO)")` | todo | P0-5: documented per-channel component-transfer filters silently render incorrectly (G/B/A ignored). No panic, but a correctness contract violation. | Medium | Either implement per-channel packing into `param0..param3` + shader dispatch, or gate with an explicit `Err(FilterError::Unsupported("per-component transfer"))` when `func_g/func_b/func_a != func_r`, with tests for supported and unsupported cases. |

### Other library sites not in the TOP 20 (all Low severity unless noted)

| file:line | snippet | category | recommendation |
|---|---|---|---|
| `cvkg-render-native/src/renderer.rs:455` | `*self.translation_stack.last().unwrap()` | unwrap | Guarded by `!is_empty()` at 451; use `.copied().unwrap_or(glam::Vec2::ZERO)` for defence. |
| `cvkg-render-native/src/main_loop.rs:159` | `.get(&winit_id).unwrap().window.clone()` | unwrap | Log + `return` like the adjacent branch at 148-153. |
| `init.rs:920,1220,1223,1245,1246,1258,1333` | `NonZeroUsize::new(...).unwrap()` (×7) | unwrap | All inputs are hardcoded tier constants (>0), but use `.unwrap_or(NonZeroUsize::MIN)` + `debug_assert!` to remove the unwrap mechanically. |
| `cvkg-render-gpu/src/renderer/mod.rs:475,477` | `unsafe impl Send/Sync for GpuRenderer` (wasm32) | unsafe | Add explicit SAFETY rationale (see §3). |
| `cvkg-render-native/src/audio.rs:17,18` | `unsafe impl Send/Sync for RodioAudioEngine` | unsafe | Comment exists; verify main-thread-only claim (see §3). |
| `cvkg-render-native/src/renderer.rs:666,677` | `unsafe { libc::setpriority(...) }` | unsafe | Add SAFETY comments (see §3). |
| `cvkg-physics/src/xpbd.rs:585` | `Some(unsafe { ... ptr.add(a) ... })` | unsafe | Already validated + commented (see §3). |
| `init.rs:347` | `Some(unsafe { device.create_pipeline_cache(...) })` | unsafe | Already commented (see §3). |
| `cvkg-gallery/src/state.rs:201` | `// TODO: get from sidebar render` | todo (demo) | Demo-only; leave behind dev boundary. |

---

## 3. Complete `unsafe` site inventory

Twelve library/runtime unsafe sites. "SAFETY comment" means an adjacent
`// SAFETY:` / `/// # Safety` note stating the concrete invariant (not merely the
`//! COMMENT ALL` doc banner).

| # | file:line | site | SAFETY / invariant comment? | assessment |
|---:|---|---|---|---|
| 1 | `cvkg-render-gpu/src/renderer/mod.rs:475` | `#[cfg(target_arch = "wasm32")] unsafe impl Send for GpuRenderer {}` | **NO** | No justification for why a wgpu renderer is thread-safe on wasm; add rationale (single-threaded wasm event loop / `!Send` fields never shared). |
| 2 | `cvkg-render-gpu/src/renderer/mod.rs:477` | `#[cfg(target_arch = "wasm32")] unsafe impl Sync for GpuRenderer {}` | **NO** | Same as #1; the two impls together are the broadest unsafety in the GPU crate. |
| 3 | `cvkg-render-gpu/src/renderer/init.rs:347` | `Some(unsafe { device.create_pipeline_cache(&wgpu::PipelineCacheDescriptor { ... }) })` | **YES** — lines 344-346 explain the wgpu marker/`Vec<u8>` lifetime | Adequate. Note the integrity-check failure path at 335-343 already degrades to an empty cache. |
| 4 | `cvkg-render-native/src/renderer.rs:51` | `pub(crate) unsafe fn set(ptr: *mut GpuRenderer) -> Self` | **YES** — `/// # Safety` at 49-50: pointer must outlive guard (typically a `MutexGuard`) | Adequate doc; pair with a runtime debug assertion if feasible. |
| 5 | `cvkg-render-native/src/renderer.rs:96` | `GpuRef::Ptr(unsafe { &mut *raw })` | **YES** — line 95: pointer valid, mutex guard live above on call stack | Highest UB risk; see TOP 20 #11. |
| 6 | `cvkg-render-native/src/renderer.rs:115` | `GpuRefShared::Ptr(unsafe { &*raw })` | **YES** — lines 113-114 | High UB risk; see TOP 20 #12. |
| 7 | `cvkg-render-native/src/renderer.rs:666` | `unsafe { let ret = libc::setpriority(libc::PRIO_PROCESS, 0, -10); ... }` (`#[cfg(target_os = "linux")]`, GodMode) | **NO** | Add `// SAFETY: setpriority is a pure syscall on the current process (id 0); errno checked below; no memory safety impact.` |
| 8 | `cvkg-render-native/src/renderer.rs:677` | `unsafe { let _ = libc::setpriority(libc::PRIO_PROCESS, 0, 0); }` | **NO** | Same one-line SAFETY comment as #7; note the ignored errno is intentional. |
| 9 | `cvkg-render-native/src/main_loop.rs:467` | `let _guard = unsafe { GpuFramePtrGuard::set(raw) };` | **YES** — line 466 plus RAII note at 471 | Good; strengthen by naming the `MutexGuard` whose lifetime is relied upon. |
| 10 | `cvkg-render-native/src/audio.rs:17` | `unsafe impl Send for RodioAudioEngine {}` | **PARTIAL** — lines 14-16 explain "only used from the main thread" | Prose invariant but not a formal SAFETY note, and the claim is not enforced. Prefer `// SAFETY:` + a debug assertion of thread id, or wrap the sink so `Send` is structurally true. |
| 11 | `cvkg-render-native/src/audio.rs:18` | `unsafe impl Sync for RodioAudioEngine {}` | **PARTIAL** — same shared comment at 14-16 | Same as #10; `Sync` is the stronger claim and deserves explicit justification. |
| 12 | `cvkg-physics/src/xpbd.rs:585` | `Some(unsafe { let ptr = particles.as_mut_ptr(); [&mut *ptr.add(a), &mut *ptr.add(b), &mut *ptr.add(c)] })` | **YES** — lines 579-584 (bounds/disjointness checks at 572-578) | Adequate; prefer `get_disjoint_mut` when MSRV permits. |

**Unsafe comment gaps — CLOSED 2026-11-21 (P0-2):** all six sites now carry
formal `// SAFETY (P0-2):` rationale comments. Two hardening changes were made
alongside the comments:

- `renderer/mod.rs`: the wasm `unsafe impl Send/Sync for GpuRenderer` are gated
  on `not(target_feature = "atomics")`, so enabling shared-memory threading
  turns the unsafe claim into a compile error rather than UB.
- `render-native/src/renderer.rs`: `GpuFramePtrGuard::set` now takes the source
  mutex and asserts in debug builds that it is currently locked, catching the
  common "pointer published after guard release" misuse. A full safe refactor
  would require threading `&mut GpuRenderer` through the `Renderer` trait,
  which is out of scope for this batch.
- `render-native/src/audio.rs`: a `created_on: ThreadId` field plus a warning in
  `play_buffer` detects violations of the thread-confinement invariant that the
  `Send`/`Sync` impls rely on (no panic introduced).

Non-sites (text only, ignore): `cvkg-scene/src/lib.rs:14`,
`cvkg-scene/src/test_renderer.rs:14`, `cvkg-render-native/src/lib.rs:14`,
`cvkg-anim/src/lib.rs:14`, `cvkg-physics/src/lib.rs:14`, `capability.rs:87`,
`init.rs:344`, `renderer/tests.rs:298`.

---

## 4. Recommended first fixes (safest / lowest blast radius, no public API break)

All three are in the preferred crates (`cvkg-render-gpu/src/renderer`, `cvkg-scene`),
are private/`pub(crate)` or return `()`, and require no new dependencies.

1. **Replace the batching `last_call.unwrap()` family — 22 sites, 5 expressions**
   (`cvkg-render-gpu/src/renderer/draw.rs:457-461`, `:678-684`, `:1687-1691`;
   `cvkg-render-gpu/src/renderer/geometry.rs:214-218`, `:380-386`).
   Proposed change: introduce the already-populated `last_call: Option<&DrawCall>`
   and write each predicate as
   `let needs_new_call = last_call.is_none_or(|c| c.scissor_rect != scissor || c.panel_id != panel || c.material != material /* ... */);`
   (bind `self.current_panel_id`, `self.current_texture_id`, resolved material to
   locals before the closure to avoid borrow conflicts). Purely internal, semantics
   identical, removes the entire unwrap cluster in one PR. Add a regression test
   that renders one draw call into an empty renderer.

2. **Make `cvkg-scene::SceneGraph::update_transforms` tolerate a dangling public root**
   (`cvkg-scene/src/lib.rs:174` and `:181`).
   Proposed change: at 174,
   `let Some(root_rect) = self.nodes.get(&root_id).map(|n| n.local_rect) else { debug_assert!(false, "scene root missing from nodes"); self.rebuild_spatial_hash(); return; };`
   and at 181,
   `let Some(node) = self.nodes.get_mut(&id) else { debug_assert!(false, "scene node missing in update_node_transform"); return; };`.
   Both functions return `()`; `nodes`/`root` are public so external mutation makes
   this reachable. No new crate dependency (`cvkg-scene` has no `tracing` dep — use
   `debug_assert!` + early return). Add a unit test that sets `scene.root = Some(missing_id)`
   and asserts `update_transforms()` does not panic.

3. **Remove the `NonZeroUsize::new(...).unwrap()` cluster in GPU init**
   (`cvkg-render-gpu/src/renderer/init.rs:920`, `:1220`, `:1223`, `:1245`,
   `:1246`, `:1258`, `:1333`).
   Proposed change: a tiny private helper
   `fn nonzero_or_default(n: usize) -> NonZeroUsize { if let Some(v) = NonZeroUsize::new(n) { v } else { tracing::warn!("[GPU] non-zero capacity input was 0; using minimum"); NonZeroUsize::MIN } }`
   and call `.unwrap_or(NonZeroUsize::MIN)` at each site with a `debug_assert!(n > 0, "...")`
   where the inputs are tier constants. No API change, no behaviour change for
   valid configs, and it makes the renderer robust to a capability tier ever
   reporting a zero capacity. (Sibling candidate, same PR: `draw.rs:910/921/932`
   `BufferSize::new(...).unwrap()` → `if let Some(size) = ... else { warn; skip }`.)

Follow-up (also zero API impact): add the six missing SAFETY comments listed at
the end of §3.

---

## 5. Demo-only hotspots — gate, do not harden

**Every one of the 67 `cvkg-gallery` hits is demo/gallery-only code** (the crate is
a showcase binary: `cvkg-gallery/src/main.rs:7-33` installs a crash-dump panic hook
and calls `NativeRenderer::run(GalleryApp::new(), None)`). None are in a core library
path. Do **not** spend P0-1 hardening effort on them:

- **36 × `unreachable!()`** — `canvas.rs:29,168,362,...,1234`; `app.rs:94`;
  `docs_panel.rs:27`; `spotlight.rs:130`; `sidebar.rs:69`; `props_panel.rs:69`.
  These are all `fn body(self) -> Self::Body { unreachable!() }` where
  `type Body = cvkg_core::Never` and `pub enum Never {}` (`cvkg-core/src/modifiers.rs:911`)
  is uninhabited. They are **provably unreachable** and idiomatic for views that
  override `render` directly — safe to ignore / `#[allow(unreachable_code)]`, not
  a hardening target.
- **31 × `.lock().unwrap()`** over `Arc<Mutex<GalleryState>>` —
  `canvas.rs:21,34,38,171,218,245,271,300`; `app.rs:28,39,47,55,63,104,130,215`;
  `spotlight.rs:20,65,81,97,109,116,135,150,166`; `sidebar.rs:27`;
  `props_panel.rs:20,25,35,62`; `docs_panel.rs:20`. Single-threaded demo state;
  poisoning would require an unrelated panic in the same demo. Leave as-is, or
  gate the whole crate behind a `dev-demo` feature boundary so CI lint rules for
  core crates (`P0-1` acceptance criteria) can simply exclude `cvkg-gallery`.
- **1 × demo TODO** — `cvkg-gallery/src/state.rs:201` (`persisted_sidebar_width`
  hardcoded to `240.0`). Demo-only; keep behind the dev boundary rather than
  promoting to a library API.

Recommended boundary treatment: add `#![cfg_attr(not(feature = "dev-demo"), ...)]`
at the gallery crate root (or simply document `cvkg-gallery` as a demo crate in the
core-crate lint allowlist), so the "no new panicking paths in core crates" rule is
enforceable without touching demo shortcuts.
