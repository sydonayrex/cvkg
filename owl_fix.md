# OWL Fix Plan — CVKG Codebase Remediation

**Date**: 2026-06-21
**Status**: Draft
**Author**: OWL (merged from OWL, DeepSeek, Google audits)
**Branch**: owl-fix/all-audit-findings

> **For Hermes:** Use `subagent-driven-development` skill to implement this plan task-by-task. Each phase should be delegated as a separate subagent with full context from this document.

---

## Overview

This plan remediates 67 distinct issues found across the CVKG GPU UI framework codebase (~35,000+ lines, 30+ crates) by three independent audits. The fixes are organized into 7 phases from critical crash-risk patches through decomposition and renaming.

**Goal:** Eliminate all production crash risks, security vulnerabilities, logic bugs, dead code, and themed naming across the CVKG workspace while maintaining build stability and test passage throughout.

**Architecture:** Phased approach following Build → Prove → Remove discipline. Each phase is independently verifiable. No phase breaks the build for the next.

**Tech Stack:** Rust 2024, wgpu 29, Taffy, Wasmtime, winit, AccessKit, lyon, rustybuzz, swash

---

## Motivation

### Current State

Three independent audits (OWL, DeepSeek, Google Flash) were run against the CVKG codebase using structured file-by-file static analysis. The audits found:

```
Total issues: 67
  HIGH (production crash):     3
  MEDIUM (logic/security):    28
  LOW (edge cases/quality):   36
  Security findings:           8
  Monolithic files:           14
  Themed identifiers:        100+
  TODO/FIXME sites:           30+
```

Example of a HIGH severity issue (backdrop_region.rs:50):
```rust
// CURRENT: Panics if scene texture is not registered
let scene_tex = ctx.registry
    .get_texture(crate::kvasir::nodes::RES_SCENE)
    .expect("scene texture must exist");  // ← PANIC in production
```

Example of a security issue (wasm_server.rs:83):
```rust
// CURRENT: Grants WASM guest full read/write access to entire CWD
wasi_builder
    .preopened_dir(&safe_root, ".", 
        wasmtime_wasi::DirPerms::all(),      // ← Full read
        wasmtime_wasi::FilePerms::all());     // ← Full write
```

Example of a logic bug (lib.rs:8930 — DependencyGraph):
```rust
// CURRENT: Pushes duplicate entries into reverse map
pub fn register(&mut self, component_id: u64, state_key: u64) {
    self.deps.entry(state_key).or_default().insert(component_id);
    self.reverse.entry(component_id).or_default().push(state_key);
    // ← No dedup: calling register twice pushes state_key twice
    // ← unregister only removes one entry, leaving stale reference
}
```

This creates problems:

1. **Production crashes**: 3 HIGH severity `.expect()`/`.unwrap()` calls in render passes panic when GPU resources are missing, crashing the entire rendering pipeline.
2. **Security vulnerabilities**: WASI configuration grants untrusted WASM modules full filesystem read/write access and has no fuel metering for DoS protection.
3. **Silent data corruption**: DependencyGraph, LRU caches, and SceneGraph ID allocation have logic bugs that corrupt state silently.
4. **GPU driver risk**: Unverified binary pipeline cache data is passed directly to `create_pipeline_cache`, risking GPU driver memory corruption.
5. **14 monolithic files** (9556, 6636, 4277, 4037, 4036, 4021, 2811, 2341, 1456, 1309, 1231, 1226, 1172, 890 lines) make the codebase unmaintainable and prevent parallel work.

### Desired State

After this plan is fully implemented:

```rust
// Render passes handle missing resources gracefully
let scene_tex = match ctx.registry.get_texture(RES_SCENE) {
    Some(v) => v,
    None => { log::error!("Missing scene texture"); return; }
};

// WASI grants minimal read-only access with fuel metering
wasi_builder
    .preopened_dir(&safe_root, ".", 
        wasmtime_wasi::DirPerms::READ_ONLY,
        wasmtime_wasi::FilePerms::READ_ONLY);
config.consume_fuel(true);

// DependencyGraph maintains consistent bidirectional mapping
pub fn register(&mut self, component_id: u64, state_key: u64) {
    let is_new = self.deps.entry(state_key).or_default().insert(component_id);
    if is_new { self.reverse.entry(component_id).or_default().push(state_key); }
}

// Pipeline cache is verified before GPU driver loading
if verify_cache_checksum(&cache_data, expected) {
    Some(unsafe { device.create_pipeline_cache(&cache_data) })
} else { None }
```

---

## Research Findings

### Audit Methodology Comparison

| Dimension | OWL Audit | DeepSeek Audit | Google Flash Audit |
|-----------|-----------|----------------|-------------------|
| Files reviewed | ~60 key files | 32 (render-gpu) + 20 (core) | ~40 files across 15 crates |
| Approach | Broad workspace | Deep GPU layer | Deep cross-crate |
| Unique findings | 18 bugs | 10 render-gpu bugs | 25 cross-crate bugs |
| False positive rate | 0% | ~6% (1 hallucinated type) | 0% |
| Security focus | Medium | Low | High |

**Key finding:** The three audits are highly complementary. OWL found broad structural issues (monoliths, dead code, naming). DeepSeek found deep GPU pipeline issues. Google found cross-crate logic bugs and security issues. Only ~5 issues overlap between any two audits.

**Implication:** All three audits should be merged for complete coverage. This plan merges all findings.

### Monolithic File Analysis

| File | Lines | Responsibilities | Risk |
|------|-------|-----------------|------|
| `cvkg-core/src/lib.rs` | 9556 | 37+ (View, Renderer, State, Layout, Event, etc.) | Critical — blocks all parallel work |
| `cvkg-render-gpu/src/renderer.rs` | 6636 | 7+ (init, frame, pipeline, cache, capture) | Critical — GPU pipeline bottleneck |
| `cvkg-render-native/src/lib.rs` | 4277 | 5+ (window, event, audio, clipboard, loop) | High — platform integration |
| `cvkg-svg-filters/src/lib.rs` | 4021 | 3+ (parsing, pipeline, textures) | High — filter system |
| `cvkg-runic-text/src/lib.rs` | 4037 | 6+ (shaping, BiDi, layout, cache, MSDF, subpixel) | High — text engine |
| `cvkg-layout/src/lib.rs` | 2811 | 9+ (Taffy, animation, primitives, grid, focus) | High — layout engine |
| `cvkg-vdom/src/lib.rs` | 2341 | 8+ (VNode, patch, diff, events, hit-test) | Medium — VDOM layer |
| `cvkg-themes/src/lib.rs` | 1309 | 6+ (color, material, contrast, theme, builder) | Medium — theming |
| `cvkg-anim/src/physics.rs` | 1456 | 4+ (rigid, soft, cloth, fluid) | Medium — physics |
| `cvkg-anim/src/growth.rs` | 1231 | 2+ (L-system, Voronoi) | Low — procedural |
| `cvkg-physics/src/world.rs` | 1172 | 3+ (config, registry, step) | Medium — physics world |
| `cvkg-render-gpu/src/material.rs` | 1226 | 3+ (types, compiler, builtins) | Medium — materials |
| `cvkg-scene/src/lib.rs` | 834 | 2+ (VNode/Patch, SceneGraph) | Low — scene graph |
| `cvkg-cli/src/main.rs` | 890 | 4+ (commands, build, devtools, scaffold) | Low — CLI |

**Key finding:** The 5 largest files (9556, 6636, 4277, 4037, 4021 lines) account for ~28,000 lines of code that cannot be worked on in parallel by multiple agents.

**Implication:** Decomposition (Phase 5) is the highest-impact structural change, enabling all future parallel work.

---

## Design Decisions

| Decision | Class | Choice | Rationale |
|----------|-------|--------|-----------|
| Phased approach | 2 coherence | 7 phases, build-stable between each | Enables rollback, parallel verification |
| HIGH first | 1 evidence | Fix crash risks before logic bugs | Production stability is highest priority |
| Build → Prove → Remove | 2 coherence | Never delete old code before new is verified | Prevents regression windows |
| Decomposition before renaming | 2 coherence | Split files first, then rename symbols | Renaming across file boundaries is harder |
| Themed naming last | 3 taste | Rename after all logic fixes | Avoids merge conflicts with logic changes |
| Mutex poison recovery | 1 evidence | `unwrap_or_else(\|p\| p.into_inner())` | Standard Rust pattern, recovers from poisoned locks |
| WASI read-only | 1 evidence | `DirPerms::READ_ONLY` | Principle of least privilege for untrusted WASM |
| Fuel metering | 1 evidence | 10B fuel units | Prevents infinite loop DoS in WASM guests |
| SHA-256 full comparison | 1 evidence | Compare all 32 bytes | 64-bit truncation is vulnerable to targeted collision |
| Pipeline cache verification | 1 evidence | Verify before `create_pipeline_cache` | Prevents GPU driver memory corruption from tampered cache |

### Decisions Log

- **Keep `cvkg-scene/src/quadtree.rs` until Phase 4**: It's a duplicate of `cvkg-spatial`'s quadtree but may have local modifications. Verify before deleting.
  Revisit when: Phase 4 orphan file cleanup.
- **Keep `cvkg-svg-filters/src/lib.rs.bak` until Phase 4**: Backup file should be verified against current before deletion.
  Revisit when: Phase 4 orphan file cleanup.
- **Keep `TEST_ENGINE` in cvkg-runic-text until Phase 4**: It's `#[allow(dead_code)]` but may be used by integration tests.
  Revisit when: Phase 4 dead code removal.

---

## Architecture

### Before (Monolithic)

```
cvkg-core/src/lib.rs (9556 lines)
├── View trait
├── Renderer trait
├── State<T>
├── Layout engine
├── Event system
├── Focus management
├── Accessibility
├── Notifications
├── File dialogs
├── Undo/redo
├── Window management
├── ... 25 more responsibilities

cvkg-render-gpu/src/renderer.rs (6636 lines)
├── Device init
├── Frame submission
├── Pipeline cache
├── Shader cache
├── Frame capture
├── Swapchain management
└── Error formatting
```

### After (Decomposed)

```
cvkg-core/src/
├── lib.rs (module declarations only)
├── view_trait.rs
├── view_modifier.rs
├── renderer_trait.rs
├── layout/ (mod.rs, cache.rs, view.rs)
├── event.rs
├── focus.rs
├── aria.rs
├── state.rs
├── system_state.rs
├── environment.rs
├── theme.rs
├── geometry.rs
├── telemetry.rs
├── asset.rs
├── suspense.rs
├── notification.rs
├── file_dialog.rs
├── document.rs
├── menu.rs
├── l10n.rs
├── clipboard.rs
├── text_input.rs
├── dirty.rs
├── virtual_list.rs
├── dependency.rs
├── identity.rs
├── tokens.rs
├── accessibility.rs
├── system_theme.rs
├── batch.rs
├── sdf_shadow.rs
├── parallax.rs
└── material.rs

cvkg-render-gpu/src/
├── renderer.rs (SurtrRenderer struct + high-level methods)
└── renderer/
    ├── init.rs
    ├── frame.rs
    ├── pipelines.rs
    ├── cache.rs
    └── capture.rs
```

### Fix Flow (Build → Prove → Remove)

```
PHASE 1-3: Fix bugs in place
┌─────────────────────────────────────────┐
│ Fix HIGH/MED/LOW bugs in current files  │
│ → cargo check → cargo test → commit     │
└─────────────────────────────────────────┘
              │
              ▼
PHASE 4: Clean up dead code & orphans
┌─────────────────────────────────────────┐
│ Remove dead_code, orphans, placeholders │
│ → cargo check → cargo test → commit     │
└─────────────────────────────────────────┘
              │
              ▼
PHASE 5: Decompose monoliths
┌─────────────────────────────────────────┐
│ For each monolith:                      │
│   1. Create new submodules             │
│   2. Move code to submodules           │
│   3. Update imports in lib.rs          │
│   4. cargo check → cargo test → commit │
└─────────────────────────────────────────┘
              │
              ▼
PHASE 6: Rename themed identifiers
┌─────────────────────────────────────────┐
│ Project-wide find-and-replace          │
│ → cargo check → cargo test → commit     │
└─────────────────────────────────────────┘
              │
              ▼
PHASE 7: Final verification
┌─────────────────────────────────────────┐
│ cargo check --workspace                │
│ cargo test --workspace                 │
│ cargo clippy --workspace -- -D warnings│
│ cargo audit --workspace                │
└─────────────────────────────────────────┘
```

---

## Implementation Plan

### Phase 0: Triage & Preparation

**Objective:** Establish baseline and verify all findings still apply.

**Files:**
- N/A (verification only)

- [ ] **0.1** Create feature branch
  ```bash
  git checkout -b owl-fix/all-audit-findings
  ```

- [ ] **0.2** Establish baseline
  ```bash
  cargo check --workspace 2>&1 | tee /tmp/baseline_errors.txt
  cargo test --workspace 2>&1 | tee /tmp/baseline_tests.txt
  ```

- [ ] **0.3** Verify findings still apply
  For each bug in this plan, verify the code at the specified line still exhibits the issue. Mark any that have already been fixed.

- [ ] **0.4** Commit baseline
  ```bash
  git add .
  git commit -m "chore: establish baseline before owl-fix"
  ```

---

### Phase 1: HIGH Severity Fixes

**Objective:** Eliminate all production crash risks.

**Skills:** `debugging`, `error-handling`, `code-reviewer`, `clean-code-guard`, `rust-patterns`, `rendering-architecture-audit`, `cvkg-render-debug`

- [ ] **1.1** Fix `.expect()` panics in cvkg-render-gpu passes

  **Files:**
  - Modify: `cvkg-render-gpu/src/passes/backdrop_region.rs:50,54`
  - Modify: `cvkg-render-gpu/src/passes/accessibility.rs:58`
  - Modify: `cvkg-render-gpu/src/passes/pyramid.rs:20`

  **Step 1: Replace `.expect()` in backdrop_region.rs line 50**
  ```rust
  // BEFORE:
  let scene_tex = ctx.registry
      .get_texture(crate::kvasir::nodes::RES_SCENE)
      .expect("scene texture must exist");
  
  // AFTER:
  let scene_tex = match ctx.registry.get_texture(crate::kvasir::nodes::RES_SCENE) {
      Some(v) => v,
      None => {
          log::error!("[BackdropRegion] Missing scene texture");
          return;
      }
  };
  ```

  **Step 2: Replace `.expect()` in backdrop_region.rs line 54**
  ```rust
  // BEFORE:
  let blur_tex = ctx.registry
      .get_texture(self.output_id)
      .expect("blur target texture must exist");
  
  // AFTER:
  let blur_tex = match ctx.registry.get_texture(self.output_id) {
      Some(v) => v,
      None => {
          log::error!("[BackdropRegion] Missing blur target texture");
          return;
      }
  };
  ```

  **Step 3: Replace `.unwrap()` in accessibility.rs line 58**
  ```rust
  // BEFORE:
  let scene_view = ctx.registry
      .get_texture_view(crate::kvasir::nodes::RES_SCENE)
      .unwrap();
  
  // AFTER:
  let scene_view = match ctx.registry.get_texture_view(crate::kvasir::nodes::RES_SCENE) {
      Some(v) => v,
      None => {
          log::error!("[Accessibility] Missing scene texture view");
          return;
      }
  };
  ```

  **Step 4: Replace `.unwrap()` in pyramid.rs pass line 20**
  ```rust
  // BEFORE:
  for mip in 0..pyramid.levels as usize {
      views.push(self.registry.get_texture_view(pyramid.mips[mip]).unwrap());
  }
  
  // AFTER:
  for mip in 0..pyramid.levels as usize {
      match self.registry.get_texture_view(pyramid.mips[mip]) {
          Some(v) => views.push(v),
          None => {
              log::error!("[Pyramid] Missing mip {} view, skipping", mip);
              continue;
          }
      };
  }
  ```

  **Step 5: Build and test**
  ```bash
  cargo check -p cvkg-render-gpu
  cargo test -p cvkg-render-gpu
  ```

  **Step 6: Commit**
  ```bash
  git add cvkg-render-gpu/src/passes/backdrop_region.rs \
          cvkg-render-gpu/src/passes/accessibility.rs \
          cvkg-render-gpu/src/passes/pyramid.rs
  git commit -m "fix: graceful error handling in render pass resource lookups"
  ```

- [ ] **1.2** Fix dangling thread-local pointer on panic

  **Files:**
  - Modify: `cvkg-render-native/src/lib.rs:~1248`

  **Step 1: Create RAII guard struct** (add near the top of the file, after imports)
  ```rust
  /// RAII guard that clears GPU_FRAME_PTR on drop, even during panic.
  struct GpuFrameGuard;
  impl GpuFrameGuard {
      fn new(raw: *mut cvkg_render_gpu::SurtrRenderer) -> Self {
          GPU_FRAME_PTR.with(|ptr| ptr.set(raw));
          Self
      }
  }
  impl Drop for GpuFrameGuard {
      fn drop(&mut self) {
          GPU_FRAME_PTR.with(|ptr| ptr.set(std::ptr::null_mut()));
      }
  }
  ```

  **Step 2: Wrap the render block**
  ```rust
  // BEFORE:
  {
      GPU_FRAME_PTR.with(|ptr| ptr.set(gpu as *mut _));
      self.view.render(&mut renderer, content_rect);
      GPU_FRAME_PTR.with(|ptr| ptr.set(std::ptr::null_mut()));
  }
  
  // AFTER:
  {
      let _guard = GpuFrameGuard::new(gpu as *mut _);
      self.view.render(&mut renderer, content_rect);
      // _guard drops here, even if render() panics
  }
  ```

  **Step 3: Build and test**
  ```bash
  cargo check -p cvkg-render-native
  cargo test -p cvkg-render-native
  ```

  **Step 4: Commit**
  ```bash
  git add cvkg-render-native/src/lib.rs
  git commit -m "fix: RAII guard for GPU_FRAME_PTR thread-local"
  ```

- [ ] **1.3** Fix compositor cyclic reference stack overflow

  **Files:**
  - Modify: `cvkg-compositor/src/engine.rs:248`

  **Step 1: Add visited set parameter to flatten_layer**
  ```rust
  // BEFORE:
  fn flatten_layer(
      layer_tree: &mut LayerTree,
      layer_id: LayerId,
      buffer: &mut Vec<RenderCommand>,
      z_counter: &mut u32,
      has_active_shaders: &mut bool,
  ) {
      // ... processes children recursively
      for child_id in children.iter().rev() {
          Self::flatten_layer(layer_tree, *child_id, buffer, z_counter, has_active_shaders);
      }
  }
  
  // AFTER:
  fn flatten_layer(
      layer_tree: &mut LayerTree,
      layer_id: LayerId,
      buffer: &mut Vec<RenderCommand>,
      z_counter: &mut u32,
      has_active_shaders: &mut bool,
      visited: &mut HashSet<LayerId>,
  ) {
      if !visited.insert(layer_id) {
          log::error!("Compositor: cyclic layer reference detected at {:?}", layer_id);
          return;
      }
      // ... rest of function unchanged
      for child_id in children.iter().rev() {
          Self::flatten_layer(layer_tree, *child_id, buffer, z_counter, has_active_shaders, visited);
      }
      visited.remove(&layer_id);
  }
  ```

  **Step 2: Update the call site in flatten_and_route**
  ```rust
  // Add at the start of flatten_and_route:
  let mut visited = HashSet::new();
  Self::flatten_tree(&mut self.layer_tree, &roots, &mut self.flatten_buffer, &mut self.z_counter, &mut self.has_active_shaders, &mut visited);
  ```

  **Step 3: Build and test**
  ```bash
  cargo check -p cvkg-compositor
  cargo test -p cvkg-compositor
  ```

  **Step 4: Commit**
  ```bash
  git add cvkg-compositor/src/engine.rs
  git commit -m "fix: cycle detection in compositor layer flattening"
  ```

---

### Phase 2: MEDIUM Severity Fixes

**Objective:** Fix all logic bugs, security concerns, and data corruption risks.

**Skills:** `debugging`, `code-reviewer`, `clean-code-guard`, `rust-development`, `error-handling`, `wgsl-wgpu-shader-pipeline`, `rust-patterns`, `strong-tests`, `test-driven-development`

- [ ] **2.1** Fix SHA256 truncation in shader cache

  **Files:**
  - Modify: `cvkg-render-gpu/src/renderer.rs:517-521,6107-6111`

  **Step 1: Replace 8-byte comparison with full 32-byte comparison**
  ```rust
  // BEFORE (line 517-521):
  let actual_hex = format!(
      "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
      actual[0], actual[1], actual[2], actual[3],
      actual[4], actual[5], actual[6], actual[7]
  );
  
  // AFTER: Compare all 32 bytes
  if actual.as_slice() != expected_hash.as_bytes() {
      return Err(format!("hash mismatch: expected {expected_hash}, got {}", hex::encode(actual)));
  }
  ```

  **Step 2: Update write_cache test helper similarly**
  ```rust
  // BEFORE (line 6107-6111):
  let hash_hex = format!(
      "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
      hash[0], hash[1], hash[2], hash[3],
      hash[4], hash[5], hash[6], hash[7]
  );
  
  // AFTER: Write full 32-byte hex string
  let hash_hex = hex::encode(hash);
  ```

  **Step 3: Build, test, commit**
  ```bash
  cargo check -p cvkg-render-gpu && cargo test -p cvkg-render-gpu
  git add cvkg-render-gpu/src/renderer.rs
  git commit -m "fix: full SHA256 comparison for shader cache integrity"
  ```

- [ ] **2.2** Fix unsafe pipeline cache loading

  **Files:**
  - Modify: `cvkg-render-gpu/src/renderer.rs:~1138`

  **Step 1: Add verification before create_pipeline_cache**
  ```rust
  // BEFORE:
  Some(unsafe { device.create_pipeline_cache(&cache_data) })
  
  // AFTER:
  if verify_cache_checksum(&cache_data, &expected_checksum) {
      Some(unsafe { device.create_pipeline_cache(&cache_data) })
  } else {
      log::warn!("Pipeline cache checksum mismatch, ignoring cache");
      None
  }
  ```

  **Step 2: Build, test, commit**
  ```bash
  cargo check -p cvkg-render-gpu && cargo test -p cvkg-render-gpu
  git add cvkg-render-gpu/src/renderer.rs
  git commit -m "fix: verify pipeline cache checksum before GPU driver loading"
  ```

- [ ] **2.3** Fix mutex poison in cvkg-core global state

  **Files:**
  - Modify: `cvkg-core/src/lib.rs:3565,3602` and all other `.lock().unwrap()` sites

  **Step 1: Replace STATE_WRITE_MUTEX**
  ```rust
  // BEFORE:
  let _lock = STATE_WRITE_MUTEX.lock().unwrap();
  // AFTER:
  let _lock = STATE_WRITE_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
  ```

  **Step 2: Replace BATCH_QUEUE mutex**
  ```rust
  // BEFORE:
  let mut queue = BATCH_QUEUE.get_or_init(|| ...).lock().unwrap();
  // AFTER:
  let mut queue = BATCH_QUEUE.get_or_init(|| ...).lock().unwrap_or_else(|p| p.into_inner());
  ```

  **Step 3: Replace ENVIRONMENT mutexes**
  ```rust
  // BEFORE:
  let env_lock = ENVIRONMENT.get_or_init(|| ...).lock().unwrap();
  // AFTER:
  let env_lock = ENVIRONMENT.get_or_init(|| ...).lock().unwrap_or_else(|p| p.into_inner());
  ```

  **Step 4: Replace State::subscribe mutex**
  ```rust
  // BEFORE:
  self.subscribers.lock().unwrap().push(Box::new(callback));
  // AFTER:
  self.subscribers.lock().unwrap_or_else(|p| p.into_inner()).push(Box::new(callback));
  ```

  **Step 5: Build, test, commit**
  ```bash
  cargo check -p cvkg-core && cargo test -p cvkg-core
  git add cvkg-core/src/lib.rs
  git commit -m "fix: mutex poison recovery for all global state in cvkg-core"
  ```

- [ ] **2.4** Fix DependencyGraph.register deduplication

  **Files:**
  - Modify: `cvkg-core/src/lib.rs:8930-8937`

  **Step 1: Add dedup check**
  ```rust
  // BEFORE:
  pub fn register(&mut self, component_id: u64, state_key: u64) {
      self.deps.entry(state_key).or_default().insert(component_id);
      self.reverse.entry(component_id).or_default().push(state_key);
  }
  
  // AFTER:
  pub fn register(&mut self, component_id: u64, state_key: u64) {
      let is_new = self.deps.entry(state_key).or_default().insert(component_id);
      if is_new {
          self.reverse.entry(component_id).or_default().push(state_key);
      }
  }
  ```

  **Step 2: Build, test, commit**
  ```bash
  cargo check -p cvkg-core && cargo test -p cvkg-core
  git add cvkg-core/src/lib.rs
  git commit -m "fix: deduplicate DependencyGraph.register reverse map entries"
  ```

- [ ] **2.5** Fix VDomPatch serialization round-trip

  **Files:**
  - Modify: `cvkg-vdom/src/lib.rs:394-451`

  **Step 1: Fix Serialize impl to skip handlers**
  ```rust
  // In the Serialize impl for VDomPatch::Update:
  // Replace the handlers serialization with a skip:
  state.serialize_field("handlers", &None::<()>)?;
  // Or simply remove the handlers field from serialization entirely
  ```

  **Step 2: Fix Deserialize impl to start with empty handlers**
  ```rust
  // In the Deserialize impl for VDomPatchInternal::Update:
  // handlers field defaults to None on deserialization
  handlers: None,
  ```

  **Step 3: Build, test, commit**
  ```bash
  cargo check -p cvkg-vdom && cargo test -p cvkg-vdom
  git add cvkg-vdom/src/lib.rs
  git commit -m "fix: VDomPatch serialization round-trip for handlers"
  ```

- [ ] **2.6** Fix WASM session panic recovery

  **Files:**
  - Modify: `cvkg-webkit-server/src/wasm_server.rs:112-124`

  **Step 1: Wrap execute_tick in catch_unwind**
  ```rust
  // BEFORE:
  pub fn tick(&self) -> anyhow::Result<()> {
      let mut session = {
          let mut guard = self.session.lock().unwrap();
          guard.take()
      }.ok_or_else(|| anyhow::anyhow!("No active WASM session"))?;
      let result = self.execute_tick(&mut session);
      let mut guard = self.session.lock().unwrap();
      *guard = Some(session);
      result
  }
  
  // AFTER:
  pub fn tick(&self) -> anyhow::Result<()> {
      let mut session = {
          let mut guard = self.session.lock().unwrap_or_else(|p| p.into_inner());
          guard.take()
      }.ok_or_else(|| anyhow::anyhow!("No active WASM session"))?;
      let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
          self.execute_tick(&mut session)
      }));
      let mut guard = self.session.lock().unwrap_or_else(|p| p.into_inner());
      *guard = Some(session);
      match result {
          Ok(r) => r,
          Err(_) => Err(anyhow::anyhow!("WASM tick panicked")),
      }
  }
  ```

  **Step 2: Build, test, commit**
  ```bash
  cargo check -p cvkg-webkit-server && cargo test -p cvkg-webkit-server
  git add cvkg-webkit-server/src/wasm_server.rs
  git commit -m "fix: WASM session panic recovery in tick()"
  ```

- [ ] **2.7** Fix WASI security hardening

  **Files:**
  - Modify: `cvkg-webkit-server/src/wasm_server.rs:32,65-86`

  **Step 1: Enable fuel metering**
  ```rust
  // BEFORE:
  config.consume_fuel(false);
  // AFTER:
  config.consume_fuel(true);
  // Add fuel limit after engine creation:
  // store.set_fuel(10_000_000_000)?; // 10B fuel units
  ```

  **Step 2: Restrict filesystem permissions**
  ```rust
  // BEFORE:
  wasi_builder.preopened_dir(&safe_root, ".",
      wasmtime_wasi::DirPerms::all(),
      wasmtime_wasi::FilePerms::all());
  // AFTER:
  wasi_builder.preopened_dir(&safe_root, ".",
      wasmtime_wasi::DirPerms::READ_ONLY,
      wasmtime_wasi::FilePerms::READ_ONLY);
  ```

  **Step 3: Remove stdin inheritance**
  ```rust
  // BEFORE:
  wasi_builder.inherit_stdout().inherit_stderr().inherit_stdin();
  // AFTER:
  wasi_builder.inherit_stdout().inherit_stderr();
  // .inherit_stdin() removed
  ```

  **Step 4: Build, test, commit**
  ```bash
  cargo check -p cvkg-webkit-server && cargo test -p cvkg-webkit-server
  git add cvkg-webkit-server/src/wasm_server.rs
  git commit -m "fix: WASI security hardening — read-only FS, fuel metering, no stdin"
  ```

- [ ] **2.8** Fix use_state RwLock poisoning

  **Files:**
  - Modify: `cvkg-vdom/src/lib.rs:~2319`

  **Step 1: Replace unwrap with poison recovery**
  ```rust
  // BEFORE:
  pub fn get(&self) -> T {
      self.value.read().unwrap().clone()
  }
  // AFTER:
  pub fn get(&self) -> T {
      self.value.read().unwrap_or_else(|e| e.into_inner()).clone()
  }
  ```

  **Step 2: Build, test, commit**
  ```bash
  cargo check -p cvkg-vdom && cargo test -p cvkg-vdom
  git add cvkg-vdom/src/lib.rs
  git commit -m "fix: RwLock poison recovery in use_state hook"
  ```

- [ ] **2.9** Fix layout cycle guard leak on panic

  **Files:**
  - Modify: `cvkg-layout/src/lib.rs:57,77`

  **Step 1: Create RAII guard struct**
  ```rust
  struct LayoutCycleGuard {
      hash: u64,
  }
  impl Drop for LayoutCycleGuard {
      fn drop(&mut self) {
          ACTIVE_LAYOUT_NODES.with(|nodes| {
              nodes.borrow_mut().remove(&self.hash);
          });
      }
  }
  ```

  **Step 2: Update with_layout_cycle_guard to use RAII**
  ```rust
  fn with_layout_cycle_guard<F, R>(hash: u64, fallback: R, f: F) -> R
  where F: FnOnce() -> R {
      if hash == 0 { return f(); }
      let already_active = ACTIVE_LAYOUT_NODES.with(|nodes| !nodes.borrow_mut().insert(hash));
      if already_active {
          log::warn!("[Layout] Cycle detected for view hash 0x{:X}!", hash);
          return fallback;
      }
      let _guard = LayoutCycleGuard { hash }; // drops even on panic
      f()
  }
  ```

  **Step 3: Build, test, commit**
  ```bash
  cargo check -p cvkg-layout && cargo test -p cvkg-layout
  git add cvkg-layout/src/lib.rs
  git commit -m "fix: RAII guard for layout cycle detection"
  ```

- [ ] **2.10** Fix next_id collision after deserialization

  **Files:**
  - Modify: `cvkg-scene/src/lib.rs:~361`

  **Step 1: Compute max ID from loaded nodes**
  ```rust
  // BEFORE:
  pub fn deserialize_binary(data: &[u8]) -> Result<Self, bincode::Error> {
      let (nodes, root): (HashMap<NodeId, VNode>, Option<NodeId>) = bincode::deserialize(data)?;
      Ok(Self { nodes, root, dirty_regions: Vec::new(), next_id: 0, cell_size: DEFAULT_CELL_SIZE, spatial_grid: HashMap::new() })
  }
  // AFTER:
  pub fn deserialize_binary(data: &[u8]) -> Result<Self, bincode::Error> {
      let (nodes, root): (HashMap<NodeId, VNode>, Option<NodeId>) = bincode::deserialize(data)?;
      let next_id = nodes.keys().map(|k| k.0).max().map(|m| m + 1).unwrap_or(1);
      Ok(Self { nodes, root, dirty_regions: Vec::new(), next_id, cell_size: DEFAULT_CELL_SIZE, spatial_grid: HashMap::new() })
  }
  ```

  **Step 2: Build, test, commit**
  ```bash
  cargo check -p cvkg-scene && cargo test -p cvkg-scene
  git add cvkg-scene/src/lib.rs
  git commit -m "fix: next_id collision after SceneGraph deserialization"
  ```

- [ ] **2.11** Fix startup panic on builtin shader compile

  **Files:**
  - Modify: `cvkg-render-gpu/src/material.rs:1039`

  **Step 1: Replace unwrap with graceful error**
  ```rust
  // BEFORE:
  let compiled = MaterialCompiler::compile(&graph).unwrap();
  // AFTER:
  let compiled = match MaterialCompiler::compile(&graph) {
      Ok(c) => c,
      Err(e) => {
          log::error!("Failed to compile builtin shader '{}': {}", name, e);
          return Err(MaterialError::BuiltinCompileFailed(name.clone()));
      }
  };
  ```

  **Step 2: Build, test, commit**
  ```bash
  cargo check -p cvkg-render-gpu && cargo test -p cvkg-render-gpu
  git add cvkg-render-gpu/src/material.rs
  git commit -m "fix: graceful error handling for builtin shader compilation"
  ```

- [ ] **2.12** Fix NaN in OKLCH color conversion

  **Files:**
  - Modify: `cvkg-themes/src/lib.rs:~35`

  **Step 1: Clamp RGB inputs**
  ```rust
  // BEFORE:
  pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
      let r_lin = to_linear(r);
      // ...
  }
  // AFTER:
  pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
      let r = r.clamp(0.0, 1.0);
      let g = g.clamp(0.0, 1.0);
      let b = b.clamp(0.0, 1.0);
      let r_lin = to_linear(r);
      // ...
  }
  ```

  **Step 2: Build, test, commit**
  ```bash
  cargo check -p cvkg-themes && cargo test -p cvkg-themes
  git add cvkg-themes/src/lib.rs
  git commit -m "fix: clamp RGB inputs to prevent NaN in OKLCH conversion"
  ```

- [ ] **2.13** Fix float sorting panic on NaN

  **Files:**
  - Modify: `cvkg-core/src/lib.rs:~3703`
  - Modify: `cvkg-runic-text/src/lib.rs:381`

  **Step 1: Replace partial_cmp().unwrap() with total_cmp()**
  ```rust
  // BEFORE:
  intersections.sort_by(|a, b| a.partial_cmp(b).unwrap());
  // AFTER:
  intersections.sort_by(|a, b| a.total_cmp(b));
  ```

  **Step 2: Build, test, commit**
  ```bash
  cargo check -p cvkg-core && cargo check -p cvkg-runic-text
  cargo test -p cvkg-core && cargo test -p cvkg-runic-text
  git add cvkg-core/src/lib.rs cvkg-runic-text/src/lib.rs
  git commit -m "fix: use total_cmp for float sorting to handle NaN"
  ```

- [ ] **2.14** Fix component builder panic on missing fields

  **Files:**
  - Modify: `cvkg-macros/src/lib.rs:~191`

  **Step 1: Generate Result-returning build methods**
  ```rust
  // BEFORE (generated code):
  fn build(self) -> MyComponent {
      MyComponent {
          field: self.field.expect("missing required field"),
      }
  }
  // AFTER (generated code):
  fn build(self) -> Result<MyComponent, ComponentBuildError> {
      Ok(MyComponent {
          field: self.field.ok_or(ComponentBuildError::Missing("field"))?,
      })
  }
  ```

  **Step 2: Build, test, commit**
  ```bash
  cargo check -p cvkg-macros && cargo test -p cvkg-macros
  git add cvkg-macros/src/lib.rs
  git commit -m "fix: component builder returns Result instead of panicking"
  ```

- [ ] **2.15** Fix duplicate vdom_id generation

  **Files:**
  - Modify: `cvkg-macros/src/lib.rs:~284`

  **Step 1: Use atomic counter instead of empty hasher**
  ```rust
  // BEFORE:
  let mut hasher = std::collections::hash_map::DefaultHasher::new();
  // (no data hashed)
  let id = hasher.finish();
  // AFTER:
  static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
  let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
  ```

  **Step 2: Build, test, commit**
  ```bash
  cargo check -p cvkg-macros && cargo test -p cvkg-macros
  git add cvkg-macros/src/lib.rs
  git commit -m "fix: unique vdom_id generation in cvkg_model macro"
  ```

---

### Phase 3: LOW Severity Fixes

**Objective:** Fix edge cases, performance issues, and code quality.

**Skills:** `debugging`, `code-reviewer`, `rust-development`, `refactoring`, `documentation`, `rust-testing`, `cvkg-render-debug`, `gpu-rendering-optimization`

- [ ] **3.1** Fix P2-7 scissor rect in glass and UI passes

  **Files:**
  - Modify: `cvkg-render-gpu/src/passes/glass.rs:539`
  - Modify: `cvkg-render-gpu/src/passes/ui.rs:115`

  **Step 1: Replace 1x1 scissor with zero-area**
  ```rust
  // BEFORE:
  p.set_scissor_rect(0, 0, 1, 1);
  // AFTER:
  p.set_scissor_rect(0, 0, 0, 0); // wgpu spec: zero area = no draw
  ```

  **Step 2: Build, test, commit**
  ```bash
  cargo check -p cvkg-render-gpu && cargo test -p cvkg-render-gpu
  git add cvkg-render-gpu/src/passes/glass.rs cvkg-render-gpu/src/passes/ui.rs
  git commit -m "fix: zero-area scissor rect in glass and UI passes"
  ```

- [ ] **3.2** Fix dur="indefinite" SVG parsing

  **Files:**
  - Modify: `cvkg-render-gpu/src/draw.rs:20-28`

  **Step 1: Handle "indefinite" sentinel**
  ```rust
  // BEFORE:
  let dur_str = node.attribute("dur").unwrap_or("1s");
  let duration = if dur_str.ends_with("ms") {
      dur_str.trim_end_matches("ms").parse::<f32>().unwrap_or(1000.0) / 1000.0
  } else {
      dur_str.trim_end_matches('s').parse::<f32>().unwrap_or(1.0)
  };
  // AFTER:
  let dur_str = node.attribute("dur").unwrap_or("1s");
  let duration = if dur_str == "indefinite" {
      f32::INFINITY
  } else if dur_str.ends_with("ms") {
      dur_str.trim_end_matches("ms").parse::<f32>().unwrap_or(1000.0) / 1000.0
  } else {
      dur_str.trim_end_matches('s').parse::<f32>().unwrap_or(1.0)
  };
  ```

  **Step 2: Build, test, commit**
  ```bash
  cargo check -p cvkg-render-gpu && cargo test -p cvkg-render-gpu
  git add cvkg-render-gpu/src/draw.rs
  git commit -m "fix: handle dur=indefinite in SVG animation parsing"
  ```

- [ ] **3.3 through 3.30** follow the same pattern (build → test → commit) for each LOW severity fix. See the full list in the audit_comparison.md for complete details.

---

### Phase 4: Dead Code, Placeholders & Unwired Features

**Objective:** Remove dead code, address TODO/FIXME sites, remove orphans, wire placeholders.

**Skills:** `refactoring`, `clean-code-guard`, `documentation`, `rust-development`, `cvkg`

- [ ] **4.1** Remove dead code from all files with `#[allow(dead_code)]`
- [ ] **4.2** Address TODO/FIXME sites (implement or document)
- [ ] **4.3** Remove orphan files (`cvkg-scene/src/quadtree.rs`, `cvkg-svg-filters/src/lib.rs.bak`)
- [ ] **4.4** Wire placeholder features (SvgFilterNode identity filter, ToneMapNode passthrough)

---

### Phase 5: File Decomposition

**Objective:** Split 14 monolithic files into focused submodules.

**Skills:** `refactoring`, `rust-workspace-audit`, `rust-development`, `code-reviewer`, `specification-writing`

- [ ] **5.1** Decompose `cvkg-core/src/lib.rs` (9556 lines → 35+ submodules)
- [ ] **5.2** Decompose `cvkg-render-gpu/src/renderer.rs` (6636 lines → 5 submodules)
- [ ] **5.3** Decompose remaining 12 monolithic files

---

### Phase 6: Themed Naming Renaming

**Objective:** Rename 100+ themed identifiers to descriptive names.

**Skills:** `refactoring`, `rust-development`, `code-reviewer`, `cvkg`

- [ ] **6.1** Project-wide find-and-replace for all themed identifiers

---

### Phase 7: Verification

**Objective:** Verify all fixes are correct and the workspace is clean.

**Skills:** `rust-testing`, `strong-tests`, `code-reviewer`, `clean-code-guard`, `rust-development`

- [ ] **7.1** Build verification: `cargo check --workspace`
- [ ] **7.2** Test verification: `cargo test --workspace`
- [ ] **7.3** Security audit: `cargo audit --workspace`
- [ ] **7.4** Clippy lint: `cargo clippy --workspace -- -D warnings`

---

## Edge Cases

### Edge Case 1: Render Pass Resource Missing During Initialization

1. GPU resources may not be registered during early initialization frames
2. The `.expect()` calls in backdrop_region, accessibility, and pyramid passes would panic
3. **Expected outcome:** Graceful return with error log, no panic

### Edge Case 2: WASM Module Panic During tick()

1. A WASM guest module panics during `cvkg_update` or `cvkg_render`
2. The session is lost because `take()` removes it before the panic
3. **Expected outcome:** Session is restored via `catch_unwind`, error is returned

### Edge Case 3: Cyclic Layer References in Compositor

1. A bug in application code creates a cycle in the layer tree
2. The DFS traversal enters infinite recursion, causing stack overflow
3. **Expected outcome:** Cycle is detected, error is logged, traversal returns gracefully

### Edge Case 4: NaN in Color Conversion

1. User provides out-of-range RGB values (e.g., negative or > 1.0)
2. `powf(2.4)` on negative values produces NaN
3. **Expected outcome:** RGB values are clamped to [0, 0, 1.0] before conversion

### Edge Case 5: Deserialized SceneGraph ID Collisions

1. Scene is serialized, then deserialized
2. `next_id` resets to 0, new nodes get IDs that collide with existing nodes
3. **Expected outcome:** `next_id` is computed as `max(existing_ids) + 1`

### Open Questions

1. **Should `hamr!` macro be renamed to `ui!`?**
   - Options: (a) Rename to `ui!`, (b) Keep `hamr!` for brand consistency, (c) Add `ui!` as alias
   - **Recommendation:** Rename to `ui!` for clarity, but this requires updating all dependent crates and documentation. Defer to Phase 6.

2. **Should WASI fuel limit be configurable?**
   - Options: (a) Hard-coded 10B units, (b) Config via environment variable, (c) Config via CLI flag
   - **Recommendation:** Start with hard-coded 10B, make configurable in a follow-up PR.

3. **Should the `TEST_ENGINE` in cvkg-runic-text be gated behind `#[cfg(test)]`?**
   - Options: (a) Gate behind `#[cfg(test)]`, (b) Keep as-is for integration test access, (c) Move to a test-utils crate
   - **Recommendation:** Gate behind `#[cfg(test)]` to avoid compiling test infrastructure into production binaries.

---

## Success Criteria

- [ ] All 3 HIGH severity fixes verified (no panics on missing resources)
- [ ] All 28 MEDIUM severity fixes verified (no logic bugs, no security vulnerabilities)
- [ ] All 36 LOW severity fixes verified (edge cases handled, performance improved)
- [ ] All 8 security findings addressed (WASI hardened, cache verified, mutex poison recovered)
- [ ] All 14 monolithic files decomposed (each < 1000 lines)
- [ ] All 100+ themed identifiers renamed
- [ ] All dead code removed
- [ ] All TODO/FIXME sites addressed
- [ ] `cargo check --workspace` passes clean
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo audit --workspace` passes

---

## References

- `owl_audit.md` — Complete audit findings (2,467 lines)
- `deepseek_audit.md` — DeepSeek audit findings (1,339 lines)
- `Google_audit.md` — Google Flash audit findings (1,469 lines)
- `audit_comparison.md` — Cross-audit comparison and verification
- `cvkg-render-gpu/src/passes/backdrop_region.rs` — HIGH severity panic fix
- `cvkg-render-gpu/src/passes/accessibility.rs` — HIGH severity panic fix
- `cvkg-render-gpu/src/passes/pyramid.rs` — HIGH severity panic fix
- `cvkg-render-native/src/lib.rs` — Dangling pointer fix
- `cvkg-compositor/src/engine.rs` — Cyclic reference fix
- `cvkg-core/src/lib.rs` — Mutex poison, DependencyGraph, float sorting fixes
- `cvkg-webkit-server/src/wasm_server.rs` — WASM panic recovery, WASI security
- `cvkg-vdom/src/lib.rs` — RwLock poisoning, VDomPatch serialization
- `cvkg-layout/src/lib.rs` — Layout cycle guard
- `cvkg-scene/src/lib.rs` — next_id collision
- `cvkg-render-gpu/src/renderer.rs` — SHA256, pipeline cache
- `cvkg-render-gpu/src/material.rs` — Builtin shader compile
- `cvkg-themes/src/lib.rs` — NaN in color conversion
- `cvkg-runic-text/src/lib.rs` — Float sorting
- `cvkg-macros/src/lib.rs` — Component builder, vdom_id generation
