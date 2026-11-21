# demos/adele-web

Web design system explorer built on CVKG with WebGPU/WASM.

## Purpose

Intended to demonstrate CVKG component rendering in a browser via WebGPU (design system explorer / matrix comparison layout).

**Current state (verified 2026-11-21 in Chromium 158 on an RTX 4070):** the crate compiles to wasm, `index.html` loads the module, and `#[wasm_bindgen(start)] main()` installs the panic hook/logger, runs the P1-4 capability probe, then paints a frame with the real **WebGPU** renderer (`GpuRenderer::forge_headless` → `ImageData` blit), falling back to the software renderer if WebGPU fails. The shipped page produced zero console errors and a full stylized frame (panels, accent bars, bloom/glow). This is a single static frame — no glyphs, input, hit-testing or frame loop yet. The WebGPU-specific bugs found and fixed during verification are listed in `docs/platform/webkit-compatibility.md` §7. Do not read this as a complete CVKG web GUI.

## Boundaries

- Web-only target (WASM via `wasm-pack`).
- Does NOT run natively -- requires a browser with WebGL2 or WebGPU support.

## Usage

```bash
cd demos/adele-web
wasm-pack build --target web
# Serve the pkg/ directory with any static server
```

## Dependencies

- `cvkg-core` -- View trait, color, rect
- `cvkg-render-gpu` -- GPU rendering in browser
- `cvkg-vdom` -- Virtual DOM diffing
- `cvkg-components` -- Widget library
- `cvkg-themes` -- Design tokens
- `wasm-bindgen`, `wasm-bindgen-futures` -- WASM glue
- `web-sys` -- Browser API bindings

---

## Web input layer (P1-5)

`src/web_input.rs` installs keyboard/pointer listeners on the canvas at startup:

- the canvas is focusable (`tabindex=0`) and has an accessible name;
- `keydown` keys are normalized through `cvkg_core::normalize_key_name` and
  modifiers are captured into `KeyModifiers` — `KeyA` → `a`, `ArrowDown` stays
  `ArrowDown` so component handlers keep matching;
- normalized key/modifier/shortcut state is exposed via
  `input_state_json_js()` for browser tests;
- focus and key activity are announced through the P1-3 live region.

Verified in Chromium 158: dispatching `focus`, `keydown` and `pointerdown`
events updates the snapshot, and `Key ArrowDown` appears in the live region.
This is an input-normalization layer only — no hit-testing tree, caret rendering
or declarative shortcut bindings on the web path yet.

## Accessibility / text bridge spike (P1-3)

**Status: proof of concept only. Do NOT treat this as production accessibility
support, and do NOT cite it as WCAG compliance.** It exists to de-risk the
question "can a thin DOM bridge make a canvas-only UI legible to assistive
technology and IME on WebKit?" before any real product work is committed.

`adele-web` renders its entire UI into one `<canvas>`. That is good for pixels
but invisible to the accessibility tree, and it gives IME composition, keyboard
focus and the clipboard no DOM surface to attach to. This spike bridges a small
subset of those interactions onto real, offscreen DOM nodes.

### What works (implemented and compiling)

- `src/a11y_bridge.rs` defines `A11yBridge` with the requested API:
  `new(canvas_id)`, `upsert_node`, `remove_node`, `set_focus`, `set_node_label`,
  `announce`, `attach_text_proxy`, `focus_text_proxy`, `clear_text_proxy`,
  `read_clipboard`, `write_clipboard`, plus the `AccessEvent` enum.
- An offscreen `role="status"` / `aria-live="polite"` / `aria-atomic="true"`
  region receives `announce()` messages.
- An offscreen but **focusable** `<textarea>` acts as the IME/composition
  proxy. It listens for `compositionstart`, `compositionupdate`,
  `compositionend`, `input` and `keydown` (Enter/Escape/Tab) and forwards
  `AccessEvent`s through a callback. The elements use the clip-based offscreen
  technique, so they are not `display:none` and remain in the accessibility
  tree.
- A `Vec`-backed set of mirrored canvas nodes with `role`, `aria-label` and
  `tabindex="0"`. `upsert_node` accepts an optional parent id that nests the
  DOM element under another mirrored element — a gesture at a tree, not a real
  one.
- Clipboard read/write through `navigator.clipboard`, with failures returned
  as descriptive `JsValue` errors.
- A non-wasm fallback with the same API that returns a clear `Unsupported`
  error, so host `cargo check` still works.
- No `unsafe` code, and no `panic!`/`unwrap()`/`expect()` in the bridge's
  runtime paths.
- `src/lib.rs` keeps existing demo behavior intact. It only adds `mod
  a11y_bridge;` and a wasm-only `init_a11y_bridge_spike(canvas_id)` export that
  creates the bridge, mirrors two example nodes, logs `AccessEvent`s through
  the existing `log` facade and keeps the bridge alive in a `thread_local`.

### What is NOT implemented (be honest)

- **No real accessibility tree.** The mirror is a flat `Vec` with optional
  one-level DOM nesting. There is no geometry, hit testing, z-order, clipped
  bounds, or mapping from canvas draw calls to nodes. Only the two example
  nodes are mirrored by the init hook.
- **No caret positioning.** The IME proxy is a fixed offscreen textarea, not a
  caret-anchored input. Candidate windows and composition underlines will not
  appear at the canvas text position.
- **No focus traversal.** `set_focus` calls DOM `.focus()` on a mirrored node;
  there is no tab-order algorithm, roving tabindex, or canvas-side focus ring.
  Tab is reported as an `AccessEvent`, not handled.
- **No ARIA state/property coverage** beyond `role`, `aria-label` and the live
  region: no `aria-pressed`, `aria-expanded`, `aria-checked`, `aria-valuenow`,
  `aria-describedby`, no live-region deduplication.
- **No selection/copy-paste UX.** There is no `select` event forwarding, no
  `execCommand("paste")` fallback for non-secure contexts, and no custom
  clipboard formats. Clipboard access requires a secure context (HTTPS or
  localhost) and a user gesture; denial is returned as an error.
- **Naive event semantics.** `input` and `compositionend` can both emit
  `TextChanged`; the consumer must de-duplicate or reset the proxy.
- **`remove_node` does not reparent descendants.** Children keep their old DOM
  location; callers must remove children first.
- **No automated tests, no VoiceOver run, no conformance claim.** The manual
  matrix below is still outstanding. The spike is now invoked automatically at startup, but it remains a proof of concept, not a complete accessibility layer.
- Pointer/touch/wheel normalization and fallback UX are explicitly out of
  scope (they belong to P1-4/P1-5 in the plan).

### Wiring / usage

The minimal wiring is already in `src/lib.rs` (see above). From JavaScript:

```js
import init, { init_a11y_bridge_spike } from './pkg/adele_web_demo.js';
await init();
// Use the id of the canvas element rendered by the demo.
init_a11y_bridge_spike('cvkg-canvas');
```

From Rust, the equivalent direct use is:

```rust
use adele_web_demo::a11y_bridge::A11yBridge; // re-export path shown for illustration
let mut bridge = A11yBridge::new("cvkg-canvas")?;
bridge.attach_text_proxy(|event| log::info!("{event:?}"))?;
bridge.upsert_node(1, None, "heading", "ADELE design systems explorer")?;
bridge.announce("Bridge ready")?;
```

The module is public (`pub mod a11y_bridge;`) so the API can also be driven
directly from Rust; `init_a11y_bridge_spike` is just a ready-made wasm entry
point for manual testing.

### Manual testing on Safari / WebKit

1. Build and serve locally: `cd demos/adele-web && wasm-pack build --target
   web`, then serve `pkg/` over `http://localhost` (a secure context is
   required for the clipboard calls). See the build-status caveat below — the
   full wasm build is currently blocked upstream.
2. Open the page in Safari, and/or drive WebKit via Playwright
   (`browserType.webkit.launch()`).
3. The bridge is now initialised automatically from Rust startup; the `init_a11y_bridge_spike('cvkg-canvas')` entry point remains available if you want to call it manually with the
   real canvas id. Check the DOM for a container carrying
   `data-cvkg-a11y-spike="1"` containing the live region and textarea.
4. Turn on VoiceOver (Cmd+F5 on macOS) and use the rotor / Tab to reach the
   mirrored nodes; the live-region announcement should be spoken. This has
   **not** been run by the spike author — you must check it yourself.
5. Click/focus the hidden textarea and type with a Japanese or Chinese IME;
   confirm `compositionstart` / `compositionupdate` / `compositionend` and the
   resulting `TextChanged` appear in the console (via the `log` facade).
6. Test clipboard read/write from a user-gesture handler (click) on HTTPS or
   localhost; verify a denial produces a logged error, not a crash.

### Build / verification status (honest)

- `rustup target list --installed` **does** include `wasm32-unknown-unknown` on
  this machine.
- `cargo check -p adele-web-demo` (host) **passed** when this spike was first
  added. A later rerun failed in an unrelated crate,
  `cvkg-render-gpu/src/material.rs` (`E0061` at
  `build_builtins_wgsl(&mut |...| ...)`) while other agents were actively
  editing that file. That failure is not caused by this spike and was left
  untouched per file-ownership rules.
- `cargo check --target wasm32-unknown-unknown -p adele-web-demo` **does not
  complete**: it fails in `cvkg-render-gpu` because the `PassNode: Send + Sync`
  bound conflicts with wgpu's Web-backend types (`ContextWgpuCore` is not
  `Sync`). This is a pre-existing problem outside this spike's scope.
- To verify the bridge itself, `a11y_bridge.rs` plus the `lib.rs` integration
  snippet were compiled standalone against the repo's **locked** dependency
  versions (`web-sys 0.3.103`, `wasm-bindgen 0.2.126`,
  `wasm-bindgen-futures 0.4.76`) for **both** the host and
  `wasm32-unknown-unknown` targets — both checks pass cleanly. This does
  **not** prove the whole demo builds for wasm.
- `Cargo.toml` was changed only to add the web-sys features the bridge needs
  (`Document`, `Element`, `HtmlElement`, `Node`, `HtmlTextAreaElement`,
  `EventTarget`, `Event`, `KeyboardEvent`, `CompositionEvent`, `Navigator`,
  `Clipboard`). No new crates were added.

### Follow-up work (not in this spike)

1. Fix the upstream `cvkg-render-gpu` wasm `Send + Sync` failure so the whole
   demo can actually be built and manually tested in a browser.
2. Run the Safari/WebKit + VoiceOver manual matrix and record results.
3. Replace the flat `Vec` with a real accessibility tree fed by render/scene
   data, including geometry for caret and hit testing.
4. Add automated coverage for `AccessEvent` semantics and error paths.
5. Define ARIA state mapping, focus traversal and clipboard fallbacks as part
   of P1-4/P1-5, and only then consider production use.

---

## Runtime capability probe / fallback UX (P1-4)

`src/capability_probe.rs` adds a small, honest browser-level capability check
that the web demo previously lacked (the WGSL variants are compile-time `cfg`,
not runtime detection). `main()` in `src/lib.rs` calls the probe at startup,
logs the result through the existing `log` facade, and — only when neither API
is found — inserts a visible `<div role="alert">` into the DOM with the message
"This demo needs WebGPU or WebGL2; your browser/WebKit configuration is
unsupported." The notice is idempotent (`id="cvkg-unsupported-notice"`) and is
never `display:none`, so it stays perceivable. The canvas and the P1-3 a11y
bridge are untouched.

### What is detected

- **`navigator.gpu` presence**, read dynamically via `js_sys::Reflect` (no
  unstable `web-sys` GPU binding). This is API-surface presence only.
- **WebGL2 context availability**, probed on a throwaway `<canvas>` so the
  demo's real canvas context is never claimed by detection.
- **`is_webkit_like`**, a `navigator.userAgent` heuristic. It is explicitly
  advisory and is **not** a compatibility guarantee; Blink-based browsers may
  also match.

Results are plain Rust data (`BrowserCapabilities`, `RecommendedTier`) with no
JS types in the public API. A non-wasm stub returns conservative "nothing
available" values so host `cargo check` works.

### What is NOT detected / limitations

- **No device/renderer initialisation check.** A `true` result means "the API
  object exists", not that CVKG can create a device or render correctly.
- **No WebKit/Safari version or feature matrix.** The user-agent check is a
  heuristic, not version detection. No browser interoperability testing was
  performed while adding this.
- **No working software renderer.** The fallback is a user-facing message only;
  it does not switch the demo to a different backend or repair the canvas.
- **No automatic retry** or localisation of the message.

WebKit/Safari runtime verification (Safari 26/18/iOS, disabling WebGPU and
WebGL2 separately, VoiceOver inspection) remains **manual and outstanding** —
see `docs/platform/webkit-compatibility.md`. Record results in that document's
matrix; do not treat any cell as tested until someone actually runs it.

### Build / verification status (honest)

- `cargo check -p adele-web-demo` (host) passes. New warnings about unused
  dependencies (`cvkg-render-gpu`, `wasm-bindgen-futures`) are pre-existing.
- The full `cargo check --target wasm32-unknown-unknown -p adele-web-demo`
  still fails in `cvkg-render-gpu` on a pre-existing `Send + Sync` wgpu bound
  (owned by another agent). This is unrelated to this module.
- The probe module was compiled standalone against the repo's locked versions
  (`wasm-bindgen 0.2.126`, `js-sys 0.3.103`, `web-sys 0.3.103`) for **both**
  host and `wasm32-unknown-unknown`; both checks pass. This does not prove the
  whole demo builds for wasm, nor that the probe behaves correctly in a real
  browser.
- `Cargo.toml` changed only to add the `js-sys` dependency (needed for
  `Reflect`). No new `web-sys` features were required — `Window`, `Document`,
  `Navigator`, `HtmlCanvasElement`, `Element` and `Node` were already enabled.
