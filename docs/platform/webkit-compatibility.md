# WebKit / Safari Compatibility — Current State and Gaps

**Status:** addresses fix-plan task **P1-4** (WebKit/Safari compatibility matrix & fallback UX). Documents the runtime capability probe and fallback implemented for that task, plus what remains unverified. **Date:** 2026-11-21.

**Evidence basis:** static review of cited files plus **one real-browser run** on Chromium 158 / Linux / NVIDIA RTX 4070 (see §7). **No Safari, WebKitGTK, or iOS device run was performed.** Nothing in this document should be read as a tested WebKit compatibility claim. Items marked *Gap*, *Untested*, or *Not yet implemented* do not work today.

---

## 1. Terminology: `cvkg-webkit-server` is not a WebKit engine

The name `cvkg-webkit-server` is misleading for this topic. It is an **axum HTTP/WebSocket/WASM development host**, not a WebKit engine, webview, or browser integration layer (`cvkg-webkit-server/README.md`, router/`main`). WebKit/Safari compatibility concerns the browser that loads the WASM demo, not this crate.

## 2. What CVKG detects at runtime today

### 2.1 Compile-time shader variant selection — **NOT runtime detection**

`cvkg-render-gpu/src/lib.rs:124-156` selects `common_wasm.wgsl`, `material_opaque_wasm.wgsl` and `bloom_wasm.wgsl` with `#[cfg(target_arch = "wasm32")]`. These variants use a single `t_diffuse` texture because WebGL2 cannot declare 32-element binding arrays. The choice is made at compile time for the whole wasm target; it does not observe the actual browser capabilities.

### 2.2 Runtime GPU adapter capability detection (exists, but is not browser detection)

- `cvkg-render-gpu/src/renderer/init.rs:141-142` — probes `TIMESTAMP_QUERY` and `PIPELINE_CACHE` features.
- `cvkg-render-gpu/src/renderer/init.rs:143-155` — native requests array-indexing features; wasm requests `wgpu::Features::empty()`.
- `cvkg-render-gpu/src/subsystems/gpu_capabilities.rs:47` — vendor detection from `Adapter::get_info().name`.

This detects **GPU hardware/driver limits**, not whether a browser exposes WebGPU or which WebKit version is running.

### 2.3 Browser-level capability detection — **IMPLEMENTED (P1-4)**

`demos/adele-web/src/capability_probe.rs` now probes, on wasm32:

- `navigator.gpu` presence via `js_sys::Reflect` (no unstable web-sys GPU bindings);
- WebGL2 context creation on a **throwaway** canvas, so the demo's own canvas is untouched;
- a user-agent heuristic for WebKit/Safari-like agents (`is_webkit_like`).

Results are a plain-data `BrowserCapabilities { webgpu, webgl2, is_webkit_like }` with `is_supported()`, `recommended_tier()` and `fallback_reason()` helpers. A non-wasm stub keeps host `cargo check` working. `demos/adele-web/src/lib.rs` calls the probe at startup and logs the result.

**Important limitation:** `navigator.gpu` presence is API presence only. The probe does **not** call `requestAdapter()`, so `webgpu: true` does not prove a usable GPU adapter. This is documented in the module and in the demo README.

## 3. Compatibility matrix

| Browser / engine | WebGPU | WebGL2 | WASM | CVKG runtime detection | CVKG rendering verified |
|---|---|---|---|---|---|
| Chromium 158 (Linux, RTX 4070) | API present; adapter created in probe harness | Yes | Yes | **Verified** (§7) | **No render path** — wasm demo builds and loads, but `main()` starts no frame loop (blank canvas) |
| Safari 26+ (macOS) | Untested | Untested | Untested | Code compiles for wasm; untested in Safari | **Untested** |
| Safari 18 / 17 | Untested | Untested | Untested | Untested | **Untested** |
| Safari 16 and older | Untested / likely unavailable | Untested | Untested | Untested | **Untested** |
| iOS Safari | Untested | Untested | Untested | Untested | **Untested** |
| WebKitGTK | Untested | Untested | Untested | Untested | **Untested** |

`cvkg-core/src/render_tier.rs:4-11` defines `Tier1GPU`/`Tier2GPU`/`Tier3Fallback`; the capability probe returns a `RecommendedTier` but **no renderer path is selected from it yet**. Do not treat tier selection as implemented.

### iOS Safari considerations (risks, not verified behaviour)

- No touch/pen/gesture normalization demonstrated in the web demo (audit §4.5).
- `demos/adele-web/index.html` hardcodes a 1024×768 container (`overflow: hidden`) — no responsive/mobile layout.
- IME/composition, selection and clipboard are prototyped (`demos/adele-web/src/a11y_bridge.rs`) but not wired through the shipped `index.html` startup.
- No geometry-aware DOM accessibility tree; the bridge mirrors a small fixed set of nodes.

## 4. Graceful fallback UX — **message only (P1-4), not a fallback renderer**

Implemented in `capability_probe::show_unsupported_notice()`:

- creates an idempotent `<div role="alert">` with the text "This demo needs WebGPU or WebGL2; your browser/WebKit configuration is unsupported.";
- the element is visually perceivable (no `display:none`/`visibility:hidden`);
- returns `Result<(), String>` instead of panicking.

Wiring is additive in `demos/adele-web/src/lib.rs`; the existing renderer path is unchanged.

**Not implemented:** renderer tier selection, WebGL2 fallback rendering using the `*_wasm.wgsl` shaders, a retry path, localization, or an accessibility-announced failure from the real demo startup (the spike is not invoked by `index.html`).

## 5. Known gaps

| Gap | Status / evidence |
|---|---|
| WebGPU adapter-level verification | `capability_probe` checks API presence only; no `requestAdapter()` in the shipped probe. |
| Full-demo wasm build | Previously blocked by `Send + Sync` errors in `cvkg-render-gpu`/`render-3d`/`export-raster`/`inputs`; being worked separately. |
| DOM ARIA mirror in shipped demo | `demos/adele-web/src/a11y_bridge.rs` is a P1-3 spike, not called from `index.html`. |
| IME, selection, clipboard, touch parity | Prototype in `a11y_bridge.rs`; not production-wired (`arboard` is non-wasm). |
| Runtime shader tier selection | `RecommendedTier` exists; no consumer selects a shader/renderer path. |
| Stale web build guide | `docs/howto/build-for-web.md` references `demos/niflheim-web`, which is not a workspace member. |
| Safari/iOS test report | None. Only the Chromium run in §7 exists. |

## 6. Verification steps

Compilation (proves wasm codegen only):

```bash
cargo check --workspace --target wasm32-unknown-unknown
cargo test -p cvkg-render-gpu
```

Manual, still required to close P1-4 for WebKit:

1. Serve `demos/adele-web` locally and open in Safari 26, Safari 18 and iOS Safari; record console errors.
2. Disable WebGPU and WebGL2 separately and reload; confirm the `role="alert"` notice appears when both are absent.
3. Inspect the accessibility tree with VoiceOver/Safari inspector; confirm which canvas UI is exposed.
4. Record browser/OS versions, screenshots and results in the §3 matrix; only then change a cell from "Untested".

## 7. Verification performed 2026-11-21 (real hardware, Chromium only)

Environment: Linux Wayland, NVIDIA GeForce RTX 4070 (driver 620.73.04), Chromium 158.0.8127.36 headless via CDP, `wasm32-unknown-unknown`, `wasm-pack`.

Method: the shipped `capability_probe.rs` was compiled into a minimal wasm harness (source included verbatim, module doc comments converted to line comments because `include!` rejects inner docs) and driven in Chromium with `Runtime.evaluate` and `Page.captureScreenshot`.

| Case | Flags | Probe result | Fallback notice |
|---|---|---|---|
| APIs enabled | `--enable-unsafe-webgpu` | `webgpu=true`, `webgl2=true`, `webkit_like=true`, `supported=true`, `tier=WebGpu` | Not expected; `show_unsupported_notice()` returned `ok` when called explicitly |
| APIs disabled | `--disable-webgl --disable-3d-apis`, no WebGPU flag | `webgpu=true` (API still exposed), `webgl2=false`, `webkit_like=true`, `supported=true` | Explicit call inserted a **visible** `role="alert"` node with the documented text |

**Full demo load (same environment):** `wasm-pack build --target web --release demos/adele-web` succeeds. The original `main()` only installed hooks and ran the capability probe, leaving the canvas blank.

**Attempted WebGPU frame path and the bugs it exposed:** wiring `GpuRenderer::forge_headless` to a 2D-canvas `ImageData` blit and running it in Chromium surfaced WebGPU-only problems that naga validation and native tests did not catch:

1. `is_finite_f32` used `3.4028235e38`, which rounds past `f32::MAX`; Dawn's WGSL parser rejects it. **Fixed** (now `3.4e38`).
2. `forge_headless` requested `TEXTURE_BINDING_ARRAY`/non-uniform-indexing features; WebGPU has no `BINDING_INDEXING` extension. **Fixed** by gating them off on wasm.
3. Renderer bind groups passed `TextureViewArray`; WebGPU requires a single `TextureView`. **Fixed** via the shared `texture_binding()` helper.
4. Generated material sampling used `textureSample` inside a non-uniform `material_id` switch. **Fixed** by emitting `textureSampleLevel(..., 0.0)` and switching the textured branches in `material_opaque.wgsl`.
5. `dpdx`/`dpdy`/`fwidth`, `textureSampleCompare` and `textureSample` under non-uniform material branches; a `std::thread::spawn` panic in `cvkg-runic-text::TextEngine::new`; two `Instant::now()`/`temp_dir()` panics on wasm; depth textures paired with filtering samplers. **All fixed** — see below.

**Fixes for item 5:** derivatives are computed once in uniform scope and passed to generated material functions as `fw`; sampling under non-uniform branches uses `textureSampleLevel`/`textureSampleCompareLevel`; the background font thread, pipeline-cache filesystem access and all renderer timestamps are cfg-gated or backed by `js_sys::Date` on wasm; depth samplers are declared `NonFiltering`.

**Shipped web render path:** `demos/adele-web/src/web_render.rs` now prefers `GpuRenderer::forge_headless` on the real WebGPU adapter and blits the readback through `ImageData`, falling back to `cvkg-render-software` if the GPU path errors. Verified in Chromium with the shipped `index.html`: **zero console errors**, `[web-render] WebGPU frame blitted to canvas`, non-transparent pixels, and a full stylized frame with bloom/glow (screenshot `/tmp/opencode/adele-gpu6.png`, session-local). The software fallback path was also verified separately (`/tmp/opencode/adele-software2.png`). Neither path provides glyphs, input or hit-testing yet.

Conclusion: the DOM fallback element and the plain-data probe work in a real browser on real hardware. The second case also confirms the documented limitation that API presence (`navigator.gpu`) can outrank actual usability — WebGL2 was disabled but the probe still reported `supported=true` because it does not call `requestAdapter()`.

Screenshots: `/tmp/opencode/probe.png`, `/tmp/opencode/probe-nogpu.png` (session-local, not committed).

**This is not WebKit/Safari verification.** No Apple browser or WebKit engine was available; every WebKit cell in §3 remains *Untested*.
