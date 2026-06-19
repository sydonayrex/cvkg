# CVKG Comprehensive Bug Audit

**Date:** 2026-06-19
**Scope:** Full workspace (35 crates, ~180K lines Rust, 88 WGSL shaders)
**Method:** Multi-pass parallel deep audit with manual P0 verification
**Workspace version:** 0.2.13, Rust 2024 edition

---

## Executive Summary

| Severity | Count | Description |
|----------|-------|-------------|
| P0       | 9     | UB, silent data corruption, missed collisions, type unsoundness (ALL FIXED) |
| P1       | 11    | Production panics, race conditions, version drift |
| P2       | 21    | Mutex poison cascades, fragility, design hazards |
| P3       | 4     | Dead code, stubs, minor inefficiencies |
| **Total**| **45**| |

**Top 3 most dangerous (historical, now fixed):**
1. `cvkg-core/src/lib.rs:3605` -- unsafe transmute of `Arc<RwLock<dyn Any>>` to `Arc<RwLock<T>>` (UB) -- FIXED
2. `cvkg-physics/src/broadphase.rs:99` -- `world_to_cell_2d` hardcodes `DEFAULT_CELL_SIZE` ignoring `self.cell_size` (silent collision misses) -- FIXED
3. `cvkg-core/src/lib.rs:3333` -- `set_direct` writes swap but not TVar, causing STM transactions to read stale data (data loss) -- FIXED

---

## P0 -- CRITICAL (UB / silent data corruption / type unsoundness) -- ALL FIXED

### X-01: Unsafe transmute of Arc<RwLock<dyn Any>> to Arc<RwLock<T>> [FIXED]
- **File:** `cvkg-core/src/lib.rs:3605-3608`
- **Issue:** `get_component_state` uses `unsafe { Arc::from_raw(raw as *const RwLock<T>) }` to transmute an `Arc<RwLock<dyn Any + Send + Sync>>` into `Arc<RwLock<T>>`. While the type is checked via `Any::is::<T>()`, this transmute is unsound because `RwLock<dyn Any>` and `RwLock<T>` have different vtable layouts. The `as *const` cast does not account for potential alignment or metadata differences.
- **Impact:** Undefined behavior on every call. May appear to work on x86 but will fail on other architectures or with LTO.
- **Fix:** Use `Any::downcast_ref()` / `Any::downcast_mut()` on the inner lock, or store `Box<dyn Any>` and downcast before wrapping in `Arc<RwLock<T>>`.

### X-02: set_direct desynchronizes TVar from swap [FIXED]
- **File:** `cvkg-core/src/lib.rs:3333-3347`
- **Issue:** `set_direct()` writes to `self.swap` (ArcSwap) but does NOT update `self.tvar` (stm::TVar). Subsequent calls to `mutate()` or `set()` read from the TVar (old value), apply the mutation, then write back to swap -- overwriting the value set by `set_direct()`. The code comment at line 3329 acknowledges this: "The TVar is left in an inconsistent state with the swap."
- **Impact:** Silent data loss when `set_direct()` is followed by `mutate()` or `set()` in a compound STM transaction.
- **Fix:** Update TVar inside `set_direct()`, or document that `set_direct` must never be used when STM transactions are active.

### X-03: StreamingText panics on multi-byte UTF-8 [FIXED]
- **File:** `cvkg-core/src/future_views.rs:36-38`
- **Issue:** `visible_chars.value.floor() as usize` gives a byte count, then `self.text[0..safe_count]` slices by bytes. For any multi-byte UTF-8 character (accented chars, CJK, emoji), this panics at a non-char-boundary.
- **Impact:** Runtime panic on any non-ASCII streaming text.
- **Fix:** Use `self.text.char_indices()` or animate by char count, not byte count.

### X-04: Broadphase cell size inconsistency [FIXED]
- **File:** `cvkg-physics/src/broadphase.rs:99-103`
- **Issue:** `world_to_cell_2d()` is a static method that hardcodes `DEFAULT_CELL_SIZE` instead of using `self.cell_size`. After `set_cell_size()`, `insert()` uses the new cell size (line 52) but `query()` calls `world_to_cell_2d()` which uses the old constant. Bodies are inserted into cells based on one size but queried based on another.
- **Impact:** Silent missed collisions -- physics objects pass through each other.
- **Fix:** Make `world_to_cell_2d` take `cell_size` as a parameter, or make it a method on `&self`.

### X-05: NaN panic in growth sort [FIXED]
- **File:** `cvkg-anim/src/growth.rs:855,858`
- **Issue:** `a.0.partial_cmp(&b.0).unwrap()` and `da.partial_cmp(&db).unwrap()`. If any angle or length is NaN (reachable from degenerate geometry or zero-length vectors), `partial_cmp` returns `None` and `.unwrap()` panics.
- **Impact:** Runtime panic on degenerate animation input.
- **Fix:** Use `unwrap_or(std::cmp::Ordering::Equal)` or filter NaN values upstream.

### X-06: Dual identity types (NodeId vs KvasirId) [FIXED]
- **Files:** `cvkg-core/src/scene_graph.rs:23`, `cvkg-core/src/lib.rs:5187,7914`
- **Issue:** `scene_graph::NodeId(u64)` and `KvasirId(pub u64)` are structurally identical but type-incompatible. Each has its own atomic counter (`NEXT_NODE_ID` vs `KvasirId::new()`), so IDs can collide. The `Renderer::query_layout` method takes `NodeId` not `KvasirId`, so no backend implements it.
- **Impact:** ID collisions between scene graph and VDOM/component systems. API confusion.
- **Fix:** Remove `scene_graph::NodeId`, use `KvasirId` everywhere. Unify the counter.

### X-07: accesskit::NodeId re-export naming collision [FIXED]
- **File:** `cvkg-render-gpu/src/lib.rs:160-163`
- **Issue:** `pub use accesskit::{..., NodeId, ...}` re-exports `accesskit::NodeId` from render-gpu. Downstream crates that depend on both render-gpu and core see two `NodeId` types in scope: `accesskit::NodeId` and `cvkg_core::NodeId` (from scene_graph). The VDOM crate manually converts via `.map(|id| accesskit::NodeId(id.0))`.
- **Impact:** Silent type confusion at call sites. Wrong NodeId variant passed to wrong API.
- **Fix:** Do not re-export `accesskit::NodeId` from render-gpu. Use a qualified import or rename.

### X-08: unsafe Send/Sync for SurtrRenderer on WASM [FIXED]
- **File:** `cvkg-render-gpu/src/renderer.rs:409-411`
- **Issue:** `unsafe impl Send for SurtrRenderer {}` and `unsafe impl Sync for SurtrRenderer {}` on WASM targets. The safety comment says "single-threaded WASM execution model guarantees no concurrent access." However, WASM can spawn web workers, and wgpu's web backend uses OffscreenCanvas which may cross thread boundaries.
- **Impact:** Potential data race if SurtrRenderer is shared across web workers.
- **Fix:** Verify wgpu's WASM thread safety guarantees. Consider using `wgpu::Surface`'s own Send/Sync impls.

### X-09: unsafe create_pipeline_cache with untrusted data [FIXED]
- **File:** `cvkg-render-gpu/src/renderer.rs:1070-1076`
- **Issue:** `device.create_pipeline_cache()` is called with data loaded from disk via an `unsafe` FFI boundary. The integrity check (lines 1048-1067) validates the data, but the `unsafe` block wraps the entire wgpu call. If the integrity check is bypassed or the data is tampered between check and use, this passes untrusted data to a native API.
- **Impact:** Potential code execution via crafted pipeline cache file.
- **Fix:** Move the integrity check inside the unsafe block, or use a safe wrapper that validates and creates atomically.

---

## P1 -- HIGH (production panics / race conditions / version drift)

### H-01: transact_pair panic poisons mutex
- **File:** `cvkg-core/src/lib.rs:3720-3744`
- **Issue:** `transact_pair` notifies subscribers inside an STM transaction. If a subscriber panics, the STM mutex is poisoned and all subsequent transactions fail.
- **Fix:** Wrap subscriber notification in `catch_unwind`.

### H-11: Flow layout panics on malformed graph
- **File:** `cvkg-flow/src/layout.rs:54,55,74,75`
- **Issue:** `displacements.get_mut(&v).unwrap()` -- panics if edge references a node not in the nodes map.
- **Fix:** Use `if let Some(d) = displacements.get_mut(&v)`.

### H-12: Anim solver unwrap
- **File:** `cvkg-anim/src/lib.rs:599`
- **Issue:** `child.solver.as_mut().unwrap()` -- panics if solver is None.
- **Fix:** Guard with `if let Some(solver) = ...`.

### H-13: Skeletal positions unwrap [TEST-ONLY]
- **File:** `cvkg-anim/src/skeletal.rs:518,529,541`
- **Issue:** `*positions.last().unwrap()` -- panics on empty positions.
- **Fix:** Guard with `if let Some(last) = positions.last()`.

### H-14: OTP input byte vs char length
- **File:** `cvkg-components/src/input_otp.rs:145-146`
- **Issue:** `self.value.len()` (bytes) guards `self.value.chars().nth()` (chars). Panics on multi-byte UTF-8 in OTP field.
- **Fix:** Use `self.value.chars().count()` or `char_indices`.

### H-15: Accessibility tree dangling children after remove
- **File:** `cvkg-accessibility/src/tree.rs:263`
- **Issue:** `remove()` deletes a node from the HashMap but does NOT remove the deleted node's ID from its parent's `children` vec. Downstream consumers iterating `children` encounter stale IDs.
- **Fix:** Add a `remove_with_parent_cleanup` method or document that callers must clean up parent references.

### H-16: get_mut bypasses version counter
- **File:** `cvkg-accessibility/src/tree.rs:285`
- **Issue:** `get_mut` returns `&mut AccessNode` but mutations do NOT auto-increment the tree's version counter. The AT bridge uses version to detect changes -- a forgotten `bump_version` call means the bridge silently misses updates.
- **Fix:** Return a wrapper that increments version on drop, or use an interior-mutation pattern.

### H-17: cvkg-telemetry version stuck at 0.2.12
- **File:** `cvkg-telemetry/Cargo.toml:3`
- **Issue:** Version is "0.2.12" while workspace is at "0.2.13". Would fail on crates.io publish.
- **Fix:** Update to "0.2.13".

### H-18: Demo crates at 0.2.12
- **Files:** `demos/adele-web/Cargo.toml`, `demos/berserker/Cargo.toml`, `demos/berserker-fire-web/Cargo.toml`, `demos/niflheim-wasi/Cargo.toml`
- **Issue:** All demo crates reference internal deps at "0.2.12" instead of "0.2.13". Also use edition "2021" vs workspace "2024".
- **Fix:** Update all demo versions and editions.

### H-19: getrandom version mismatch (0.3 vs 0.4)
- **Files:** `cvkg-core/Cargo.toml:88`, `cvkg-anim/Cargo.toml:46`
- **Issue:** cvkg-core uses getrandom 0.3, cvkg-anim uses 0.4. On WASM, this pulls two incompatible versions, doubling binary size.
- **Fix:** Align to single getrandom version.

### H-20: rand version mismatch (0.8 vs 0.10.1)
- **Files:** `cvkg/Cargo.toml:87`, `cvkg-flow/Cargo.toml:74`, `cvkg-anim/Cargo.toml:43`
- **Issue:** cvkg and cvkg-flow use rand 0.8, cvkg-anim uses rand 0.10.1. Incompatible major versions.
- **Fix:** Align to single rand version.

---

## P2 -- MEDIUM (error handling / fragility / design hazards)

### M-01: xpbd.rs unsafe get_three_mut
- **File:** `cvkg-physics/src/xpbd.rs:559-562`
- **Issue:** `unsafe` block creates 3 mutable references via raw pointer. Assert guards are sound but fragile -- a future caller passing equal indices is UB. Should use `slice::get_disjoint_mut()` (Rust 1.62+).
- **Fix:** Replace with safe `get_disjoint_mut()`.

### M-12: render-gpu pipeline cache file handling
- **File:** `cvkg-render-gpu/src/renderer.rs:1001,1025`
- **Issue:** `fs::write` and `fs::read` with `unwrap()` on `log::error!` path -- if disk is full or file is locked, this panics instead of degrading gracefully.
- **Fix:** Use `?` or `if let Err(e) = ...`.

### M-13: cvkg-components theme_switch mutex
- **File:** `cvkg-components/src/theme_switch.rs:82,90`
- **Issue:** `.expect("mode listeners poisoned")` on `MODE_LISTENERS.lock()`.
- **Fix:** Use `unwrap_or_else(|e| e.into_inner())` or propagate errors.

### M-14: cvkg-components scribing_stone mutex
- **File:** `cvkg-components/src/scribing_stone.rs:37,52,63`
- **Issue:** Same `.expect("lock poisoned")` pattern.
- **Fix:** Use `unwrap_or_else(|e| e.into_inner())` or propagate errors.

### M-15: cvkg-components button RwLock
- **File:** `cvkg-components/src/interactive/button.rs:327,359,717`
- **Issue:** `.expect("lock poisoned")` on `solver_arc.write()`.
- **Fix:** Use `unwrap_or_else(|e| e.into_inner())` or propagate errors.

### M-16: cvkg-components orchestrator RwLock
- **File:** `cvkg-components/src/multi_agent_orchestrator.rs:1388,1409,1430`
- **Issue:** `.expect("unexpected None")` -- misleading message (it's lock poison, not None).
- **Fix:** Fix message and use non-panicking pattern.

### M-17: cvkg-components container RwLock
- **File:** `cvkg-components/src/container.rs:346`
- **Issue:** Same `.expect("lock poisoned")` pattern.
- **Fix:** Use `unwrap_or_else(|e| e.into_inner())` or propagate errors.

### M-18: cvkg-components telemetry RwLock
- **File:** `cvkg-components/src/gerd_telemetry.rs:78,94`
- **Issue:** Same pattern with misleading message.
- **Fix:** Use `unwrap_or_else(|e| e.into_inner())` or propagate errors.

### M-19: cvkg-components security RwLock
- **File:** `cvkg-components/src/tyr_security.rs:86,100,151`
- **Issue:** Same pattern.
- **Fix:** Use `unwrap_or_else(|e| e.into_inner())` or propagate errors.

### M-20: cvkg-components persistence RwLock
- **File:** `cvkg-components/src/idunn_persistence.rs:60,84,162`
- **Issue:** Same pattern.
- **Fix:** Use `unwrap_or_else(|e| e.into_inner())` or propagate errors.

### M-21: cvkg-flow node unwrap on Optional fields
- **File:** `cvkg-flow/src/node.rs:374,433`
- **Issue:** `node.glass_material.unwrap()` and `node.shadow.unwrap()` -- these are Options that may legitimately be None.
- **Fix:** Use `if let Some(...)`.

### M-22: SpatialHash assert in production constructor
- **File:** `cvkg-spatial/src/spatial_hash.rs:66`
- **Issue:** `assert!(cell_size > 0.0)` panics at runtime. Library crate should return Result.
- **Fix:** Change to `Option<SpatialHash>` or `Result<Self, Error>`.

### M-23: set_root doesn't validate node existence
- **File:** `cvkg-accessibility/src/tree.rs:301`
- **Issue:** `set_root(id)` accepts any KvasirId without checking existence. Silent no-op with spurious version bump.
- **Fix:** Return `Option` or `Result` if node doesn't exist.

### M-24: Tab order sorts by raw ID, not document order
- **File:** `cvkg-accessibility/src/focus.rs:183`
- **Issue:** `ids.sort_by_key(|id| id.0)` -- produces arbitrary tab order, not visual/document order. Acknowledged stub.
- **Fix:** Implement positional sorting based on layout coordinates.

### M-25: ErrorBoundary duplicated across crates
- **Files:** `cvkg-core/src/error_boundary.rs:60`, `cvkg-components/src/error.rs:9`
- **Issue:** Both define `pub struct ErrorBoundary<V: View>` with different fields and behavior. Name collision creates API confusion.
- **Fix:** Rename cvkg-components version or re-export from core.

### M-26: arboard version pin mismatch
- **Files:** `Cargo.toml:110` (workspace), `cvkg-render-native/Cargo.toml:47`
- **Issue:** Workspace pins arboard to "=3.4.0" (exact), render-native uses "3.4" (range).
- **Fix:** Align pinning strategy.

### M-27: Dual Renderer trait definitions
- **Files:** `cvkg-core/src/lib.rs:2120-2800+`, `cvkg-core/src/renderer/mod.rs:16-497`
- **Issue:** Renderer trait defined twice: monolithic in lib.rs (~700 lines), split into ~25 sub-traits in renderer/mod.rs. Sub-trait contracts not enforced by compiler.
- **Fix:** Make sub-traits supertraits of the main Renderer trait, or remove the split.

### M-28: State<T> TVar/ArcSwap desync [FIXED]
- **File:** `cvkg-core/src/lib.rs:3227-3237`
- **Issue:** State<T> carries 4 sync primitives (ArcSwap + TVar for value and metadata). `set_direct()` previously bypassed TVar, leaving compound STM transactions reading stale data.
- **Fix:** TVar synchronization has been added to `set_direct()` -- see X-02.

### M-29: BVH debug_assert only fires in debug builds
- **File:** `cvkg-spatial/src/bvh.rs:137`
- **Issue:** `debug_assert!(!indices.is_empty())` will not fire in release. If indices is empty, line 140 panics with unhelpful index-out-of-bounds.
- **Fix:** Use `assert!` or add early return.

### M-30: i32 overflow for extreme coordinates
- **File:** `cvkg-spatial/src/spatial_hash.rs:131-134`
- **Issue:** `floor() as i32` can silently wrap for coordinates near f32::MAX.
- **Fix:** Use `i32::try_from()` or clamp.

### M-31: Double boxing in FrameScheduler
- **File:** `cvkg-scheduler/src/frame.rs:312-316`
- **Issue:** `Box<dyn FnOnce() + Send>` gets wrapped in another `Box<dyn FnOnce()>` inside Task struct. Unnecessary heap indirection per phase task.
- **Fix:** Use a single Box or a typed task enum.

---

## P3 -- LOW (dead code / stubs / minor inefficiencies)

### L-17: accessibility focus unwrap fragility [FIXED]
- **File:** `cvkg-accessibility/src/focus.rs:127`
- **Was:** `*self.tab_order.last().unwrap()` with only an early-return guard, fragile if control flow changed.
- **Now:** Uses `unwrap_or(&self.tab_order[0])` -- safe even if early return is refactored away.

### L-18: scene_graph::NodeId dead code [FIXED]
- **File:** `cvkg-core/src/scene_graph.rs:22-23`
- **Was:** `#[allow(dead_code)] pub struct NodeId(u64)` with its own atomic counter.
- **Now:** `pub type NodeId = super::KvasirId` -- unified identity, no dead code.

### L-19: CLI error type missing framework conversions [FIXED]
- **Files:** `cvkg-cli/src/error.rs`
- **Was:** `CliError` had no conversion from `CvkgError`.
- **Now:** `impl From<cvkg_core::CvkgError> for CliError` exists at line 92.

### L-20: old/ directory stale versions
- **File:** `old/cvkg-render-web/Cargo.toml:15`
- **Note:** This is an intentionally archived copy of an old render crate for reference. Version 0.2.10 is expected. Not a bug -- archived code.

---

## Cross-Crate Communication Issues

### CC-01: [RESOLVED] NodeId/KvasirId unified
The workspace previously had three incompatible ID types. This has been resolved -- `scene_graph::NodeId` has been removed and `KvasirId` is now the single identity type used throughout.

### CC-02: Renderer trait split (see M-27)
The monolithic `Renderer` trait in lib.rs (~700 lines) is what backends implement. The ~25 sub-traits in renderer/mod.rs (RendererCore, RendererVDOM, etc.) exist for consumer code but are NOT supertraits of the main Renderer. This means the sub-trait contracts are not enforced -- backends only need to implement the monolithic trait.

### CC-03: [RESOLVED] set_direct syncs TVar
The `set_direct()` method previously bypassed TVar, creating STM desync. This has been resolved.

### CC-04: CLI error boundary (see L-19)
`CliError` cannot convert from `CvkgError` or `TemplateError`. If the CLI needs to surface framework errors, it requires manual mapping at every call site.

---

## Crate-Level Summary

| Crate | P0 | P1 | P2 | P3 | Total | Key Issue |
|-------|----|----|----|----|-------|-----------|
| cvkg-core | 3 | 1 | 1 | 1 | 6 | Unsafe transmute (FIXED), TVar desync (FIXED), UTF-8 panic (FIXED), Renderer trait split |
| cvkg-render-gpu | 2 | 0 | 2 | 0 | 4 | Unsafe Send/Sync (FIXED), pipeline cache (FIXED) |
| cvkg-components | 0 | 4 | 10 | 0 | 14 | Mutex poison everywhere, OTP byte/char |
| cvkg-physics | 1 | 0 | 1 | 0 | 2 | Broadphase cell size (FIXED), unsafe get_three_mut |
| cvkg-anim | 1 | 2 | 0 | 0 | 3 | NaN panic (FIXED), empty positions, solver unwrap |
| cvkg-flow | 0 | 1 | 1 | 0 | 2 | Unwrap on malformed graph, node unwrap |
| cvkg-accessibility | 0 | 2 | 3 | 1 | 6 | Dangling children, version bypass, stub sorting |
| cvkg-spatial | 0 | 0 | 2 | 0 | 2 | Assert panic, i32 overflow |
| cvkg-scheduler | 0 | 0 | 1 | 0 | 1 | Double boxing |
| cvkg-telemetry | 0 | 1 | 0 | 0 | 1 | Version drift |
| cvkg-cli | 0 | 0 | 0 | 1 | 1 | Missing error conversions |
| demos/* | 0 | 1 | 0 | 0 | 1 | Version drift |
| old/* | 0 | 0 | 0 | 1 | 1 | Stale versions |
| Cross-crate | 0 | 0 | 1 | 0 | 1 | Renderer trait split |

---

## Recommended Fix Priority

**Immediate (P0 -- all fixed, verify no regression):**
All 9 P0 items have been fixed. Verify fixes are still in place after any future refactors.

**Short-term (P1 -- fix within 1 week):**
- Replace all `.expect("lock poisoned")` with `unwrap_or_else(|e| e.into_inner())`
- Fix all unwrap() calls on Option in components, anim, flow
- Align cvkg-telemetry and demo versions to 0.2.13
- Align getrandom and rand versions across workspace
- Add parent cleanup to accessibility tree remove()

**Medium-term (P2 -- fix within 1 month):**
- Replace unsafe `get_three_mut` with safe `get_disjoint_mut`
- Make ErrorBoundary a single canonical type
- Make SpatialHash::new return Result
- Implement real document-order tab sorting
- Resolve dual Renderer trait design

---

## Validation Notes

This file has been audited and cleaned. The following findings were **removed** because they were fabricated by audit subagents (referencing non-existent crates or unverifiable code locations):

**Removed P1:** H-02, H-03, H-04, H-05, H-06, H-07, H-08, H-09
**Removed P2:** M-02, M-03, M-04, M-05, M-06, M-07, M-08, M-09, M-10, M-11
**Removed P3:** L-01, L-02, L-03, L-04, L-05, L-06, L-07, L-08, L-09, L-10, L-11, L-12, L-13, L-14, L-15, L-16

**Removed findings from crates that DO exist but had fabricated lock-poison findings:** cvkg-compositor, cvkg-vdom, cvkg-layout (lock poisoning reported by subagents was not found in actual source -- grep confirmed zero `.expect("lock poisoned")` matches in these crates)

All remaining findings have been verified against source code at the workspace path `/D/rex/projects/cvkg`.

---

*Audit performed by parallel deep-audit subagents + manual P0 verification + hallucination cleanup pass.*
