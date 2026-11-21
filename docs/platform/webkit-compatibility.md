# WebKit / Safari Compatibility — Current State and Gaps

**Status:** Addresses fix-plan task **P1-4** (WebKit/Safari compatibility matrix & fallback UX). Adjacent gaps from **P1-3** (web interaction baseline) and **P1-5** (input/UX parity) are flagged but not resolved here. **Date:** 2026-11-21.

**Evidence basis:** static review of the files cited below. **No browser, device, or WebKit interoperability testing was performed.** Nothing in this document should be read as a tested compatibility claim. Items marked *Planned*, *Gap*, or *Not yet implemented* do not work today.

---

## 1. Terminology: `cvkg-webkit-server` is not a WebKit engine

The name `cvkg-webkit-server` is misleading for this topic. It is an **axum HTTP/WebSocket/WASM development host**, not a WebKit engine, webview, or browser integration layer:

- `cvkg-webkit-server/README.md:5` — "axum-based HTTP and WebSocket server … hot-module-reload … static file serving, WASM execution through Wasmtime, and a Prometheus metrics endpoint."
- `cvkg-webkit-server/README.md:9-13` — owns HTTP/routing/WS; explicitly states "Client-side rendering is not in scope".
- `cvkg-webkit-server/README.md:124` — `wgpu` was removed in 0.2.15; only the `backend-wasm` feature remains (`:132`).
- `cvkg-webkit-server/src/router.rs:265-267`, `:499` — serves `/pkg`, `/assets`, `/static`; handshake clients identify as `"webkit-runtime"` / `"webkit-hmr"`.

There is **no** WebKit/WKWebView integration, no Safari-specific code, and no browser-capability negotiation in this repository. Audit §4.5 rates this naming confusion as a Medium finding.

## 2. What CVKG actually detects at runtime today

Three distinct mechanisms exist. Only one is runtime, and none of them is browser-level WebKit capability detection.

### 2.1 Compile-time shader variant selection — **NOT runtime detection**

`cvkg-render-gpu/src/lib.rs:124-156` selects different WGSL includes via `#[cfg(target_arch = "wasm32")]`:

| Constant | `wasm32` | native |
|---|---|---|
| `WGSL_COMMON` | `shaders/common_wasm.wgsl` | `shaders/common.wgsl` |
| `WGSL_MATERIAL_OPAQUE` | `shaders/material_opaque_wasm.wgsl` | `shaders/material_opaque.wgsl` |
| `WGSL_BLOOM` | `shaders/bloom_wasm.wgsl` | `shaders/bloom.wgsl` |

The comment at `lib.rs:124-130` explains the reason: "on wasm32/WebGL2, texture binding arrays are not supported" and the bind group layout therefore uses `count: None` on WASM. Other shaders (`shapes`, `material_glass`, `bifrost`, `color_blind`, `tonemap`, `particles`) are shared.

**Important caveats:**
- This is decided **at compile time by target architecture**, not at runtime by querying the browser or GPU. A `wasm32` binary built for WebGPU gets the same reduced variants as one running on WebGL2.
- The comment conflates `wasm32` with WebGL2; `wasm32-unknown-unknown` can also target WebGPU. No code distinguishes the two.
- Feature flags in `cvkg-render-gpu/Cargo.toml:130-132` are `wgpu 29` with `["spirv", "webgl", "fragile-send-sync-non-atomic-wasm"]`; `naga 29` has `["wgsl-in", "spv-out"]` (`:111-116`). Declared features are not evidence of an in-repo SPIR-V pipeline (audit §4.3).

### 2.2 Runtime **GPU adapter** capability detection (exists, but is not browser detection)

- `cvkg-render-gpu/src/renderer/capability.rs:29-120` — `GpuCapabilityTier::detect(adapter)` reads `adapter.get_info()` and `adapter.limits()`, derives a "reduced" vs "full" tier from device type and `max_storage_buffer_binding_size`, and intersects `required_limits` with adapter limits to avoid `request_device` panics.
- `cvkg-render-gpu/src/renderer/init.rs:90-125` — tries HighPerformance, then LowPower, then a software fallback adapter (`force_fallback_adapter: true`).
- `cvkg-render-gpu/src/renderer/init.rs:141-142` — probes `TIMESTAMP_QUERY` and `PIPELINE_CACHE` features.
- `cvkg-render-gpu/src/renderer/init.rs:143-155` — on native requests `SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING | TEXTURE_BINDING_ARRAY`; on `wasm32` requests `wgpu::Features::empty()` with the comment "Fallbacks for WebGL".
- `cvkg-render-gpu/src/subsystems/gpu_capabilities.rs:47` — vendor detection from `Adapter::get_info().name`.

This detects **GPU hardware/driver limits**, not whether Safari exposes `navigator.gpu`, whether WebGL2 is available, or which WebKit version is running.

### 2.3 Browser-level capability detection — **Gap / Not yet implemented**

Repository-wide search found **no** use of `navigator.gpu`, `getContext("webgl2")`, user-agent/Safari detection, `matchMedia`, or any browser feature probe in Rust or JS/HTML sources. The only user-facing failure handling in the web demo is a `try/catch` around `init()` in `demos/adele-web/index.html:52-65` that changes a status `<div>` text to "Initialization Failed" in red. That is an error message, not capability detection or a fallback renderer.

`demos/adele-web/src/lib.rs:20-25` (`main()`) only installs a panic hook and logger.

## 3. Compatibility matrix — **PROPOSED, UNVERIFIED**

There is no tested matrix in-tree. The table below is a **template for the P1-4 acceptance criteria**, not a support statement. Every cell is untested.

| Safari / WebKit | WebGPU | WASM (`wasm32-unknown-unknown`) | WebGL2 fallback | CVKG status |
|---|---|---|---|---|
| Safari 26+ (current) | Untested | Untested | Untested | **Untested** |
| Safari 18 | Untested (WebGPU availability varies by release/OS) | Untested | Untested | **Untested** |
| Safari 17 | Untested | Untested | Untested | **Untested** |
| Safari 16 and older | Untested / likely unavailable | Untested | Untested | **Untested** |
| iOS Safari (any) | Untested | Untested | Untested | **Untested** |

A `RenderTier` enum exists in `cvkg-core/src/render_tier.rs:4-11` (`Tier1GPU` WebGPU/Vulkan/Metal/DX12, `Tier2GPU` WebGL2/OpenGL 3.3, `Tier3Fallback` Canvas 2D/GDI+). It is a **type definition**; no code in the reviewed files selects a tier from browser capabilities. Do not treat it as implemented tiering.

### iOS Safari considerations (all risks, not verified behaviour)

- No touch/pen/gesture normalization is demonstrated in the web demo; audit §4.5 rates this Medium.
- `demos/adele-web/index.html:19-27` hardcodes a 1024×768 container with `overflow: hidden` on `body` — no responsive/mobile layout.
- No IME/composition bridge, selection, or clipboard integration on web (audit §4.5, §4.6).
- No DOM accessibility mirror (see `docs/ux/accessibility-baseline.md`).

## 4. Graceful fallback strategy — **DESIGN PROPOSAL, NOT IMPLEMENTED**

Proposed sequence (none of these steps exists in code today):

1. **Probe** at startup in the web shell: `navigator.gpu` → `canvas.getContext('webgl2')` → neither.
2. **Report** the result to the Rust app through an explicit `wasm_bindgen` capability object, so shader/feature choices are runtime data rather than `cfg`.
3. **Select** a renderer path: WebGPU, the existing WebGL2-oriented `*_wasm.wgsl` variants, or a documented "unsupported" state. (The existing `*_wasm` shaders currently cover only WebGL2-style binding-array limitations; they are not a full fallback renderer.)
4. **Surface** a user-visible, localized fallback/error UI with a retry path — replacing the single red status string at `index.html:58-62`.
5. **Announce** the failure through an accessibility live region once the web ARIA bridge from P1-3 exists.

Acceptance for P1-4 requires runtime detection **and** user-facing fallback behaviour; both are currently missing.

## 5. Known gaps

| Gap | Evidence / status |
|---|---|
| No browser/WebKit feature detection | Search of `.rs`/`.html`: no `navigator.gpu`, `getContext`, user-agent, or `matchMedia`. |
| No graceful fallback UI beyond an error string | `demos/adele-web/index.html:52-65`. |
| Shader variants are compile-time only | `cvkg-render-gpu/src/lib.rs:124-156`. |
| Web path is canvas-only; no shipped DOM ARIA/labels | `demos/adele-web/index.html:47-51`; `demos/adele-web/src/lib.rs:60+` (all `renderer.*` draw calls). Audit §4.5. An **in-progress, uncommitted P1-3 spike** exists (`demos/adele-web/src/a11y_bridge.rs:1-18`, wired via `src/lib.rs:15,26-51`), but it is self-described proof-of-concept, covers two example nodes, and is **not called from `index.html:52-65`** — so the shipped demo still has no DOM a11y. |
| No IME, selection, clipboard, or touch parity on web | Audit §4.5, §4.6. `arboard` is explicitly non-wasm (`cvkg-components/Cargo.toml:120-121`). |
| Web build guide is stale | `docs/howto/build-for-web.md:18-19` points to `demos/niflheim-web`, which is not a workspace member (`Cargo.toml:3-43`); `:42` claims "WebGL2 or WebGPU"; `:48` has a "WebPU" typo and suggests browser flags. |
| No verified Safari/iOS test report anywhere | Audit §2 explicitly lists no browser/device interoperability testing. |

## 6. Verification steps

Run (proves compilation only, **not** WebKit compatibility):

```bash
cargo check -p cvkg-render-gpu --target wasm32-unknown-unknown
cargo test -p cvkg-render-gpu
wasm-pack build --target web --out-dir pkg demos/adele-web   # path may need adjusting
```

Manual checks needed to close P1-4 (all currently expected to **expose gaps**):

1. Serve `demos/adele-web` locally (`docs/howto/build-for-web.md:22-29`) and open in Safari 26, Safari 18, and iOS Safari. Record console errors.
2. In devtools, disable WebGPU / WebGL2 separately and reload — confirm whether any fallback UI appears (today: only the red status text).
3. Inspect the DOM tree with VoiceOver or Safari's accessibility inspector — confirm whether any canvas content is exposed (today: expected **none** from normal startup; the P1-3 spike is not invoked by `index.html`).
4. Record results into the matrix in §3 with browser/OS versions and screenshots; only then change any cell from "Untested".
