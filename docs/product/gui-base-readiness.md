# CVKG as a Base GUI Source — Readiness and Adoption Guide

**Status:** Addresses fix-plan task **P2-4** and reports against the **Section 6 exit criteria**. **Date:** 2026-11-21.

**Evidence basis:** `plans/codebase-audit-2026-11-20.md`, `plans/fix-plan-2026-11-20.md`, `README.md`, and static review of cited files. No full build/test/shader-validation or browser/device run was executed for this document. Anything below marked *Not met*, *Gap*, or *Planned* is not delivered.

---

## 1. Current readiness verdict

**Not ready as a production base GUI toolkit.** The source audit's conclusion (`plans/codebase-audit-2026-11-20.md:118-120`) states:

> "CVKG demonstrates substantial technical ambition and foundational capability, but **this audit finds material gaps in correctness hardening, numeric rigor governance, SPIR-V/assembly auditability, WebKit/web interaction support, and design-system/UX completeness** relative to the stated goal of serving as a base GUI source for desktop and web applications. The accompanying fix plan prioritizes establishing verifiable safety and platform-compatibility baselines before expanding surface area."

Audit §4.7 rates turnkey readiness **High** risk: "Not yet turnkey as a base GUI framework: platform parity, stability contracts, and app-critical capabilities (text editing UX, accessibility on web, input/IME/clipboard, packaging/runtime guidance, compatibility guarantees) are incomplete or under-evidenced."

Treat CVKG today as a **prototype/platform foundation for evaluation and experimentation**, not as a drop-in replacement for a mature GUI toolkit.

## 2. Supported platforms (as evidenced, not as guaranteed)

| Target | Evidence | Caveat |
|---|---|---|
| Native desktop (Vulkan / Metal / DX12 via wgpu 29) | `README.md:3`, `:265`; `cvkg-render-gpu/src/renderer/init.rs:90-178` | Requires Vulkan/Metal/DX12-capable hardware; Linux system deps listed at `README.md:266`. |
| CPU/software fallback | `cvkg-render-software` crate (`README.md:289`, `:296`) | Fallback quality/feature parity not quantified in reviewed material. |
| WebAssembly (`wasm32-unknown-unknown`) | `README.md:267`; `docs/howto/build-for-web.md:9-41`; `cvkg-render-gpu/Cargo.toml:179-190` | Compiles and demos exist; **no verified browser compatibility matrix** (`docs/platform/webkit-compatibility.md`). |
| Web demo | `demos/adele-web` (`README.md:319`) | Canvas-only; no DOM accessibility, IME, clipboard, or touch parity (audit §4.5). |
| Headless WASI | `demos/niflheim-wasi` (`Cargo.toml:30`) | Validation target, not a supported GUI platform. |

No packaging/runtime distribution guidance, no API stability contract, and no CI quality gates were found in the reviewed material (audit §4.7; fix-plan P2-5 is still open).

## 3. Known limitations

Summarised from audit §§4.1–4.7; each is an open item, not a solved one.

| Area | Limitation | Status |
|---|---|---|
| Correctness | Pervasive `unwrap()`/`expect()`/`lock().unwrap()` in library/runtime paths; unsafe/raw-pointer guards and `unsafe impl Send/Sync` in rendering/native paths | P0-1 / P0-2 **open** |
| Shader artifacts | No `.spv`/`.spvasm`/GLSL artifacts; WGSL-first with `naga spv-out` declared but no in-repo SPIR-V generation/validation pipeline | P0-3 / P0-4 **open** |
| Numeric rigor | No unified float policy; ad-hoc `is_finite`/clamps/epsilon usage | P1-1 / P1-2 **open** |
| Components | Broad widget surface but uneven parity (states, keyboard, ARIA, theming, RTL/i18n, docs) | P2-1 / P2-3 **open** |
| Web / WebKit | Canvas-centric web path; no shipped DOM a11y mirror, IME/composition, selection/clipboard, or browser feature detection/fallback UX (an uncommitted, partial P1-3 bridge prototype exists in the web demo; see `docs/ux/accessibility-baseline.md` §3) | P1-3 / P1-4 / P1-5 **open** |
| UX services | Unified text editing, clipboard, shortcuts, focus traversal abstractions not evidenced | P1-5 **open** |
| Tooling | No consolidated build/test/shader-validation baseline or compatibility matrix | P2-5 **open** |

See `docs/platform/webkit-compatibility.md` and `docs/ux/accessibility-baseline.md` for the detailed web and accessibility breakdowns.

## 4. Embedding patterns (patterns only — not API stability guarantees)

- **Umbrella facade:** the `cvkg` crate re-exports core, scene, layout, themes, anim, macros, components, `cvkg-render-gpu`, and `cvkg-render-native` (`README.md:283`, dependency edges `:193-201`).
- **Declarative views:** implement `View` (`cvkg-core/src/view.rs`; `body`, `render`, and optional `aria_properties`/`on_key_event`/`key_shortcuts` at `:328-344`), or use the `hamr!` macro (`cvkg-macros`, `README.md:309`).
- **Native shell:** `cvkg-render-native` provides winit windowing/event loop plus the AccessKit bridge (`cvkg-render-native/src/lib.rs:39-42`).
- **Web shell:** `docs/howto/build-for-web.md` describes `wasm-pack build --target web` and serving `pkg/`; note the guide is stale (see §6).
- **Reference apps:** `demos/berserker` (native), `demos/adele-web` (web), `cvkg-gallery`, `demos/niflheim-wasi`.

No semantic-versioning or deprecation policy was found. These are examples of current code structure, not supported integration contracts.

## 5. Text / input / accessibility guarantees — honest status

| Guarantee | Status today |
|---|---|
| GPU rendering across wgpu backends | Partial — WGSL renderer exists; composed-shader validation in CI not in place (P0-3). |
| Text shaping/BiDi | Foundation exists (`cvkg-runic-text`, `README.md:298`); no end-to-end editing UX guarantee. |
| Text input and IME/composition | **None on web; not evidenced end-to-end on native.** Audit §4.6 rates this High. |
| Clipboard / selection | **No unified cross-platform guarantee.** `arboard` is non-wasm only (`cvkg-components/Cargo.toml:120-121`). |
| Keyboard focus / tab order | Partial — `FocusManager` exists (`cvkg-accessibility/src/focus.rs:70-200`) but needs explicit adoption; visual-order rather than document-order sort (`focus.rs:176-178`). |
| Screen-reader semantics | Partial on native via AccessKit bridge; **absent on web canvas** (audit §4.5). |
| Reduced motion | Partial on native (`cvkg-core/src/a11y_prefs.rs`, `reduced_motion.rs`); absent on web. |
| Contrast / WCAG conformance | **None** — no audit or enforcement found. |

Section 6 of the fix plan states these must be met before CVKG "can be credibly positioned as a base GUI source": panic-safe runtime paths, CI-validated shaders, standardized numeric handling, documented desktop/WebKit baselines for input/focus/a11y/text, component parity rubrics, and maintained compatibility docs. **All are currently Not met.**

## 6. Versioning / stability policy — real issues to fix

There is no stability policy yet, and the repository has concrete version drift:

- **Workspace version skew:** workspace package version is **0.3.5** (`Cargo.toml:46`), while workspace path dependencies are pinned to **0.3.4** (`Cargo.toml:54-60`; also `cvkg-render-gpu/Cargo.toml:76-86`, `cvkg-components/Cargo.toml:117-124`). Audit §4.1 rates this Medium: "Dependency skew, stale artifacts, and maintenance/correctness drift risk."
- **Rust version metadata:** corrected in this batch — `README.md` and `docs/onboarding.md` now state Rust **1.95+**, matching `Cargo.toml:48` (`rust-version = "1.95"`).
- **License metadata:** corrected in this batch — `README.md` now states **EPL 2.0**, matching `Cargo.toml:49`, `cvkg-render-gpu/Cargo.toml:24` and `LICENSE`.
- **Generated manifests:** 16 `Cargo.toml` files begin with `# THIS FILE IS AUTOMATICALLY GENERATED BY CARGO` (e.g. `cvkg-render-gpu/Cargo.toml:1-10`, `cvkg-core/Cargo.toml:1`). These are Cargo-normalized publish artifacts; consult `Cargo.toml.orig` where present rather than treating them as hand-maintained source.
- **No API contract:** no deprecation policy, MSRV enforcement, changelog-as-contract, or stability tiers were found. Do not assume cross-version API compatibility.

## 7. Historical artifacts to avoid

- **`old/` directory:** a large historical tree containing past audits, fix plans, design docs, screenshots, `.bak` files (e.g. `old/renderer_old.rs.bak`), publish scripts (`.publish_dry_run.py`), `old/pkg`, `old/static`, `old/src`, and its own `old/AGENTS.md`. Audit §4.1/§4.7 cite it as maintenance/ergonomics risk. **Do not treat anything under `old/` as current guidance or current code.**
- **Generated `Cargo.toml` files:** the 16 normalized manifests noted above. Edits belong in the original manifests; normalized files are regenerated during publishing.
- **Stale docs:** `docs/howto/build-for-web.md:18` instructs `cd demos/niflheim-web`, which is not a workspace member (`Cargo.toml:3-43` has `demos/adele-web` and `demos/niflheim-wasi`); `:42` claims "WebGL2 or WebGPU" rendering and `:48` suggests browser flags. Verify against actual code before relying on it.
- **Aspirational/historical docs:** audit §4.7 notes `docs/imp_parity.md` is "partially historical/aspirational in tone". `plans/` was empty before the 2026-11-20 audit; prefer `plans/codebase-audit-2026-11-20.md` and `plans/fix-plan-2026-11-20.md` as the current baseline.

## 8. Verification steps

```bash
# Baseline build/test (does not prove platform readiness)
cargo check --workspace
cargo test --workspace
cargo check -p cvkg-render-gpu --target wasm32-unknown-unknown

# Evidence hygiene checks referenced above
grep -rl 'AUTOMATICALLY GENERATED BY CARGO' --include=Cargo.toml .   # 16 files
grep -n 'version' Cargo.toml | head                                  # 0.3.5 vs 0.3.4 skew
```

To change the verdict in §1, close the fix-plan Section 6 exit criteria with evidence: CI shader validation logs, panic/unsafe review sign-off, numeric policy + tests, desktop/WebKit input+a11y manual matrix results, component parity scorecards, and reconciled version/license/MSRV metadata.
