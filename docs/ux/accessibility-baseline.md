# Accessibility Baseline — Desktop and Web Canvas

**Status:** Addresses fix-plan tasks **P2-2** (accessibility conformance hardening) and documents the current state relevant to **P1-3** (web interaction baseline). **Date:** 2026-11-21.

**Evidence basis:** static review of cited files. No screen-reader, keyboard-only, or browser assistive-technology session was run for this document. Items marked *Partial*, *Missing*, *Gap*, or *Not yet implemented* must not be presented as working features.

---

## 1. What exists today

### 1.1 `cvkg-accessibility` crate — data-only foundation

`cvkg-accessibility/src/lib.rs:25-42` provides `AccessibilityTree`, `FocusManager`, `ScreenReaderBridge`, `Announcement`, and `SemanticRole`. Its README is explicit about scope:

- `cvkg-accessibility/README.md:9-19` — owns tree model, focus management, and screen-reader bridge abstraction; **does not own** ARIA parsing (that is `cvkg-vdom`), platform AT backends, or rendering/layout ("this crate is data-only").
- `cvkg-accessibility/README.md:96-101` — the tree does not listen for platform events; a consumer must drive updates; no default platform bridge is provided; tab order is not tracked against DOM mutations; `SemanticRole` is not exhaustive.
- `cvkg-accessibility/src/tree.rs:38-83` — `SemanticRole` enum (Button, Checkbox, ComboBox, Dialog, Grid, Heading, Image, Link, List, Menu, Tab, Table, TextInput, etc.); `:91-103` `is_focusable()`.
- `cvkg-accessibility/src/focus.rs:70-200` — `FocusManager` owns an ordered `tab_order`, wraps at boundaries, clears stale focus on rebuild. **Caveat:** `rebuild_from_tree` (`focus.rs:172-200`) sorts by visual position (y, then x, then ID); its own doc comment at `:176-178` states a production integration should use document/DOM order instead.
- `cvkg-accessibility/src/bridge.rs:36-83` — `AnnouncementPriority` (Polite/Assertive), `Announcement`, `ScreenReaderBridge` trait. No concrete platform implementation ships; `NullScreenReaderBridge` is the no-op fallback (`lib.rs:40`).

**Documentation drift:** the usage example at `cvkg-accessibility/README.md:61-84` uses `tree.add_node(...)` and `move_focus(dir, &tree)` and an `Announcement { text: ... }` field; the actual APIs are `insert(AccessNode::new(...))`, `move_focus(dir)` (`focus.rs:120`), and `Announcement { message: ... }` (`bridge.rs:51-56`). Do not copy that example.

### 1.2 Native AccessKit bridge

- `cvkg-render-native/src/window.rs:366-377` — creates `accesskit_winit::Adapter::with_event_loop_proxy`; logs AT-SPI backend availability.
- `cvkg-render-native/src/main_loop.rs:403-408`, `:1323-1331` — builds `accesskit::TreeUpdate` from VDOM nodes and calls `adapter.update_if_active`.
- `cvkg-render-native/src/main_loop.rs:1393-1416` — `ActionHandler`, `ActivationHandler`, `DeactivationHandler` implementations.
- `cvkg-render-native/src/contracts.rs:170-229` — explicit AccessKit/CVKG role mapping registry.
- `cvkg-render-gpu/src/lib.rs:163-169` — re-exports AccessKit types (`Node`, `Role`, `TreeUpdate`, `ShieldWallAdapter`) so callers need not depend on AccessKit directly.

This provides a native platform-facing tree. It does **not** prove that every component emits complete roles/states/actions (audit §4.4, Medium).

### 1.3 ARIA primitives and icon helpers

- `cvkg-core/src/aria.rs:9`, `:61-78` — `AriaRole` enum (README of that crate claims 52 WCAG 2.1 variants at `cvkg-core/README.md:206`) and `AriaProperties` (role, label, description, value, pressed, checked, expanded, disabled, hidden, level, shortcut, focused, live, atomic).
- `cvkg-core/src/view.rs:328-333` — `View::aria_properties()` default returns `None`; a view is "not explicitly accessible" unless overridden.
- `cvkg-icons/src/lib.rs:186-272` — `Icon` carries `aria_label` / `aria_hidden`; `with_aria_label`, `decorative`; `aria_properties()` returns `Presentation` for decorative and `Img` for labelled icons, otherwise `None` (`:266-272`, documented at `cvkg-icons/README.md:135`).

### 1.4 Reduced motion and contrast

- `cvkg-core/src/reduced_motion.rs:1-8` — `is_reduced_motion()` delegates to `AccessibilityPreferences::detect_from_system()`; `effective_duration()` returns `0.0` when active.
- `cvkg-core/src/a11y_prefs.rs:38-168` — native detection via macOS `defaults`, Linux `GTK_A11Y`/gsettings, Windows `reg_query`.
- `cvkg-components/src/hlin_accessibility.rs:63-84` — additional component-level detection (GTK theme env vars, `gsettings`, `MACOS_REDUCED_MOTION`).
- Many component docs claim reduced-motion awareness (e.g. `cvkg-components/src/interactive/button.rs:15`, `data_grid.rs:24`, `toast.rs:103`). These are **documentation claims in component files**, not a verified conformance sweep.
- `cvkg-render-gpu/src/color_blindness.rs`, re-exported as `ColorBlindMode` (`cvkg-render-gpu/src/lib.rs:158-161`), plus OKLCH clamping in `cvkg-themes` (audit §4.2) provide colour-related building blocks. No contrast-ratio audit or report was found.

## 2. Checklist (native = desktop AccessKit path; web = canvas path)

| Item | Status | Evidence / note |
|---|---|---|
| Semantic roles/properties/actions | **Partial** (native), **Missing** (web) | Roles exist in `tree.rs:45-83`, `aria.rs:9`; `View::aria_properties()` defaults to `None` (`view.rs:331`). No evidence of complete role/state/action coverage per component; no web exposure. Audit §4.4, §4.5. |
| Focus order | **Partial** (native), **Missing** (web) | `FocusManager` cycles tab order (`focus.rs:120-149`); tab order derived from visual position, not DOM order (`focus.rs:176-178`). No web focus model. |
| Announcements / live regions | **Partial** | `ScreenReaderBridge` + `AnnouncementPriority` (`bridge.rs:36-83`) are a trait and data type only; no default platform backend (`README.md:97`). Native AccessKit tree updates exist (`main_loop.rs:403-408`); web live regions do not. |
| Reduced motion | **Partial** (native), **Missing** (web) | Native detection in `a11y_prefs.rs` / `hlin_accessibility.rs`. No `matchMedia('(prefers-reduced-motion: reduce)')` or equivalent in web shell (`demos/adele-web/index.html`, `src/lib.rs`). |
| Contrast | **Missing (as a guarantee)** | Theming/clamping and `ColorBlindMode` exist; no WCAG contrast-ratio audit or enforcement found. Build blocks only. |
| Keyboard navigation / shortcuts | **Partial (improved)** | `on_key_event` / `key_shortcuts` hooks exist (`view.rs:337-344`); `FocusManager` provides cycling. The web demo now has a focusable canvas with normalized key/modifier state and live-region announcements (`demos/adele-web/src/web_input.rs`), verified in Chromium. No full keyboard-only flow test yet. |
| IME / text composition | **Missing** | No composition bridge found in web or native core; audit §4.5/§4.6 rates this High. |
| Clipboard / selection | **Partial (native dependency only)** | `cvkg-components/Cargo.toml:120-121` puts `arboard` behind `not(target_arch = "wasm32")`; `rfd` at `:118`. No unified clipboard API or web implementation evidenced. |

## 3. Web DOM ARIA mirroring — **GAP, being prototyped (P1-3)**

The web demo mounts a single `<canvas>` (`demos/adele-web/index.html:47-51`) and renders all content through `renderer.draw_text_raw`, `fill_rect`, `bifrost`, and component `.render()` calls (`demos/adele-web/src/lib.rs:63-578`). There are no DOM nodes for headings, buttons, inputs, or live regions, so browser assistive technology has nothing to expose.

Audit §4.5: "no evidence of DOM text input overlays, IME/composition handling, clipboard integration, or ARIA mirror for web"; §4.4: "No evidence of complete semantic roles/states/keyboard patterns for all components or web DOM ARIA overlay."

Planned approach (not fully implemented): mirror the `AccessibilityTree` into hidden, focusable DOM elements positioned over the canvas, map `SemanticRole` → ARIA roles, forward focus and actions through the existing `ScreenReaderBridge` abstraction, and add an `aria-live` region for announcements.

**P1-3 prototype (committed 2026-11-21):** `demos/adele-web/src/a11y_bridge.rs:1-18` self-describes as "proof of concept only, not production code" and explicitly does **not** attempt a complete accessibility tree, layout-based hit testing, or a virtualised caret. It is wired into `demos/adele-web/src/lib.rs` via an `init_a11y_bridge_spike()` entry point that mirrors two example nodes (a heading and a textbox) and logs forwarded text-proxy events. It is now called from `#[wasm_bindgen(start)] main()` during normal startup. Verified in Chromium 158: the DOM contains the `data-cvkg-a11y-spike` container, a `role=heading` node, a `role=textbox` node, an `aria-live=polite` status region, and a focusable textarea proxy; dispatching an `input` event forwarded `TextChanged("hello")` to the Rust side. This is still a proof of concept — two mirrored nodes, no layout-based hit testing, no virtual caret and no screen-reader session — but the shipped demo is no longer DOM-invisible.

**Hardware verification performed 2026-11-21 (Chromium 158 / Linux / RTX 4070):** the P1-4 fallback notice was compiled into a minimal wasm harness and run in a real browser. `show_unsupported_notice()` inserted a **visible** `<div role="alert">` with the documented message, confirmed by DOM query and screenshot. This verifies the notice element only — it does **not** verify the P1-3 text/IME bridge, screen-reader behaviour, or any WebKit engine. See `docs/platform/webkit-compatibility.md` §7.

## 4. Verification steps

Automated (proves compilation/tests, not conformance):

```bash
cargo test -p cvkg-accessibility
cargo test -p cvkg-render-native
cargo test -p cvkg-icons
cargo test -p cvkg-components
```

Manual checks for the P2-2 checklist (today these are expected to surface gaps):

1. On desktop, build a small app using `AccessibilityTree` + `FocusManager`, then inspect the AccessKit tree with a native inspector (macOS Accessibility Inspector, Windows Accessibility Insights, Linux Accerciser/AT-SPI). Verify roles, labels, focus, and actions.
2. Tab/Shift+Tab through a representative flow with keyboard only; check wrap behaviour and whether focus is visually indicated.
3. Toggle OS reduced-motion preference and confirm animation durations collapse to zero via `effective_duration()`.
4. On Safari/WebKit, open `demos/adele-web` with VoiceOver running. Expected today: **canvas content is not exposed** — record this as the baseline gap, not as a pass.
5. Once the P1-3 prototype lands, re-run step 4 and add a DOM ARIA snapshot comparison.
