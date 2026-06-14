---
name: cvkg-employment
description: "Class-level CVKG employment: rendering, components, frontend design, app architecture, WebAssembly, demos, verification, and Git workflow."
---

# CVKG Employment

Use this umbrella skill whenever a task asks how to employ CVKG as a framework: programmatic rendering, component implementation, frontend or app design, browser WASM deployment, demos, verification, or Git delivery.

This is the class-level CVKG skill. Prefer this umbrella for broad CVKG work. Use focused support references when a task only needs a command recipe or domain checklist. Avoid creating a long flat list of narrow one-session CVKG skills; add class-level sections or references here instead.

## When to Apply

- Designing or building a CVKG native or web application.
- Implementing or auditing reusable components.
- Writing direct `View::render` primitives, GPU effects, render graph nodes, or shader-backed visuals.
- Choosing layout, visual polish, accessibility, motion, or theme-token strategy.
- Packaging browser WebAssembly output or running WASI validation.
- Running demos, checks, tests, or committing/pushing CVKG work.
- Updating the skill library for CVKG-related work.

## Core Contracts

- Read actual code and docs before changing or auditing. Do not rely on stale summaries.
- Prefer completing or wiring features over deleting them. Stubs, TODOs, placeholders, and disabled features are blockers unless the user explicitly accepts them.
- For components, use the CVKG pattern: `Clone`, public fields, constructor, builder methods returning `Self`, `View` implementation, real `render()` content, `push_vnode`/`pop_vnode` where identity matters, and sensible `intrinsic_size`.
- For reusable components, theme through `cvkg-themes` tokens, not hard-coded colors.
- For rendering, trace the full path: `View` or app entry -> VDOM/compositor if used -> `cvkg-render-gpu` -> WGSL/pipeline/bind groups -> final presentation.
- For GPU work, keep Rust structs, WGSL structs, and bytemuck layouts byte-compatible. 16-byte alignment is not optional.
- Do not disable render features to silence errors. Fix the mismatch or implement the missing path.
- For web apps, keep browser-specific code behind target cfgs or web crates. Native windowing belongs in `cvkg-render-native`; browser canvas/WebGPU/WebGL belongs in WASM/web code.
- A task is not complete until relevant checks/tests pass and the claimed native or web target has been exercised.
- For Git delivery, inspect status first, commit the project changes, and push explicitly to main with `git push origin HEAD:main -v`.

## Domain Routing

| Task type | Use this section/reference |
|-----------|----------------------------|
| Broad CVKG app or feature work | This umbrella skill |
| Component implementation or audit | Component Implementation section |
| Direct rendering, shaders, GPU effects | Program Rendering section |
| Visual system, component polish, accessibility, motion | Frontend Design section |
| Native/web app architecture, state, navigation, telemetry | App Design section |
| Browser WASM build, canvas init, WASI validation | WebAssembly section |
| Demo run commands | `references/demo-runbook.md` |
| Commit and push pattern | `references/git-delivery.md` |
| CVKG skill library shape and mappings | `references/skill-map.md` |

## Program Rendering

Use this path for low-level drawing and GPU work.

1. Define whether the work is a primitive `View`, a compositional component, or a render graph pass.
2. For primitive views, implement `View::render(&self, renderer: &mut dyn Renderer, rect: Rect)` and usually `type Body = Never`.
3. Wrap renderable content in `renderer.push_vnode(rect, "...")` and `renderer.pop_vnode()` when hit testing or accessibility identity matters.
4. Trace the full render path and verify the final visual appears.
5. Add pixel or command tests when possible; compile checks alone are insufficient.
6. Run the relevant demo or headless test.

Useful commands:

```bash
cargo fmt --all --check
cargo check --workspace
cargo clippy --workspace --all-targets
cargo check -p cvkg-render-gpu --tests
cargo test -p cvkg-render-gpu
cargo run -p berserker
```

## Component Implementation

Use this path for reusable widgets and component audits.

1. Choose a logical module name. Avoid catch-all names such as `misc.rs`, `extras.rs`, `modern_missing.rs`, or `new_components.rs`.
2. Add `#[derive(Clone)]`, public fields, a constructor, and builder methods.
3. Every builder method must return `Self`.
4. Implement `View`. Primitive components use `type Body = Never`; structural components compose children.
5. Draw real content in `render()`. Do not leave empty render bodies.
6. Use `register_handler` for interactive behavior.
7. Export from the module and from `cvkg-components/src/lib.rs`.
8. Verify with crate and workspace checks.

Useful commands:

```bash
cargo fmt --all --check
cargo check -p cvkg-components
cargo clippy -p cvkg-components --all-targets
cargo test -p cvkg-components
cargo check --workspace
cargo test --workspace
```

## Frontend Design

Use this path when the task is about visual quality, design systems, accessibility, or motion.

- Choose one clear visual direction: tactical HUD, catalog explorer, command console, agent cockpit, data wall, or minimal control panel.
- Build hierarchy before decoration.
- Use theme tokens consistently.
- Use glass/backdrop effects where they add hierarchy, not everywhere.
- Ensure focus, keyboard paths, text contrast, hover states, loading states, empty states, error states, and overflow states are considered.
- Keep motion purposeful and readable.
- Avoid emoji, em dashes, random neon gradients, and generic AI-slop visuals.

Useful commands:

```bash
cargo fmt --all --check
cargo check -p cvkg-components
cargo test -p cvkg-components
cargo run -p berserker
cargo check -p adele-web-demo
cargo check -p berserker-fire-web-demo
```

## App Design

Use this path for complete application architecture.

1. Define user goal, workflows, and target platform.
2. Choose the CVKG crate set:
   - `cvkg` facade for simple entry points.
   - `cvkg-core` for traits, geometry, state, and environment values.
   - `cvkg-components` for widgets.
   - `cvkg-layout` for stacks, grids, and sizing.
   - `cvkg-anim` for spring motion.
   - `cvkg-vdom` for logical UI tree and events.
   - `cvkg-compositor` for retained layer orchestration.
   - `cvkg-render-gpu` for GPU rendering.
   - `cvkg-render-native` for desktop windowing.
   - `cvkg-themes` for semantic design tokens.
   - `cvkg-test` for visual regression support.
3. Define state ownership, navigation, data flow, and telemetry.
4. Keep rendering out of the app layer.
5. Verify the claimed target: native, web, or both.

Useful commands:

```bash
cargo check -p berserker
cargo run -p berserker
cargo check -p adele-web-demo
cargo check -p berserker-fire-web-demo
cargo build --target wasm32-unknown-unknown --features web --release
cargo fmt --all --check
cargo check --workspace
cargo clippy --workspace --all-targets
cargo test --workspace
```

## WebAssembly

Use this path for browser or WASI targets.

Browser build workflow:

```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --features web --release
ls target/wasm32-unknown-unknown/release/*.wasm
python -m http.server 8000
# or
cvkg-webkit-server --port 8000 --root ./dist
```

Browser checklist:

- Canvas exists before `init()` runs.
- Generated JS glue is served next to the `.wasm`.
- Browser supports WebGPU or WebGL2 fallback.
- Console has no initialization errors.
- Canvas sizing and resize behavior work.
- No native-only APIs are referenced by browser code.

WASI checklist:

- Use `wasm32-wasip1` for headless WASI.
- Treat WASI as composition and compile validation, not visual rendering.
- Stable exports such as `cvkg_init`, `cvkg_update`, and `cvkg_render` are useful for host runtimes.

## Git Delivery

When the user asks to commit and push the project:

1. Inspect status first: `git status --short`.
2. Stage the intended project changes: `git add -A` when the user means the whole project.
3. Commit with a concise message that describes the work.
4. Push explicitly to main:

```bash
git push origin HEAD:main -v
```

If `git push HEAD:main` fails by trying the wrong remote, the durable fix is to specify the remote explicitly: `git push origin HEAD:main -v`.

## Verification Standard

A CVKG employment task is complete when:

- The relevant code or docs are read directly.
- The intended feature path is implemented or clearly audited.
- Relevant checks/tests pass.
- Native or web targets are exercised when claimed.
- No temporary stubs, TODOs, or disabled features remain unless explicitly accepted.
- Git delivery, if requested, is confirmed on `main`.

## Related Skills

- `wgsl-wgpu-shader-pipeline` for shader, bind group, and pipeline debugging.
- `rendering-architecture-audit` for GPU architecture audits and capability mapping.
- `frontend-design` for general visual design direction outside CVKG specifics.
- `tdd-workflow` when tests should drive a new feature or fix.

## References

- `references/skill-map.md` - CVKG skill-library map and umbrella policy.
- `references/demo-runbook.md` - Native and web demo commands.
- `references/git-delivery.md` - Commit and push pattern for CVKG.
