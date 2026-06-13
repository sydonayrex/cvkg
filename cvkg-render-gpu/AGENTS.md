# cvkg-render-gpu AGENTS.md

## Purpose
Own the GPU rendering pipeline for CVKG: SurtrRenderer, Kvasir render graph, WGSL shaders, GPU resources, compositor integration, and render validation.

## Ownership
- `src/renderer.rs` - renderer lifecycle, surface/device setup, pipeline creation, frame execution, submit path.
- `src/kvasir/` - render graph nodes, planner, resource registry.
- `src/passes/` - geometry, glass, backdrop, UI, bloom, composite, accessibility, SVG filter effects.
- `src/shaders/` - WGSL shader modules and shared uniforms.
- `tests/hello_world.rs` - render graph and shader behavior tests.

## Local Contracts
- Trace GPU bugs from draw call through material, pipeline, bind group layout, WGSL bindings, and render graph order before changing code.
- Do not disable render features to silence errors. Complete or correctly wire them.
- Keep Rust structs and WGSL structs byte-aligned and field-name consistent.
- Avoid per-frame GPU allocations in hot paths; cache bind groups and reuse buffers where possible.
- UI readiness blockers must be reported as code, wiring, shader, dependency, or validation gaps with concrete fixes.

## Work Guidance
- Tahoe-level UI requires crisp text, MSAA geometry, high-quality glass/backdrop blur, stable frame timing, accessibility ordering, HDR/color-management hooks, and platform-native window chrome support.
- The Kvasir graph should express pass dependencies explicitly and avoid implicit ordering hidden in renderer.rs.
- Prefer typed effect parameters over shared opaque uniform arrays.

## Verification
- Run `cargo check -p cvkg-render-gpu --tests`.
- Run `cargo test -p cvkg-render-gpu`.
- For visual changes, run the render tests that compare pixel output and run a native demo when possible.
- For dependency or graph changes, regenerate the Mermaid graph and verify it includes the render-gpu closure.

## Child DOX Index
None.
