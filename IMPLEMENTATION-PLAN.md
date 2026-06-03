# CVKG Renderer — Implementation Plan (Kvasir Graph)

**Status: SUBSTANTIALLY COMPLETE — All core phases implemented (2026-08)**

---

## Completed Phases

| # | Phase | Status | Key Deliverables |
|---|-------|--------|------------------|
| 0 | P0 Bug Fixes | ✅ | Fullscreen draws, glass alpha, blur pyramid syntax, presentation layer |
| 1 | Kvasir Foundation | ✅ | Resource types, registry, graph with topological sort, planner, node trait |
| 2 | end_frame Rewrite | ✅ | 452-line hardcoded → graph-driven pass dispatch |
| 3 | Pass Wiring | ✅ | Glass + UI full GPU recording |
| 4 | Material System | ✅ | DrawMaterial enum, color_blind pipeline, config fields, zero TODOs |
| 5 | Kawase Pipelines | ✅ | Separate shader module, bind group layout, downsample/upsample |
| 6 | Kawase Mip Chain | ✅ | Per-mip views, uniform buffers, pass recording for backdrop + bloom |
| 7 | Shader Specialization | ✅ | 4 per-material pipelines (opaque, glass, PBR, gradient) |
| 8 | Vertex Instancing Foundation | ✅ | InstanceData struct, instance buffer, struct fields |
| 9 | Graph-Driven Ordering | ✅ | build_frame_graph() drives execution order |
| 10 | Accessibility Service | ✅ | Brettel/Viénot Daltonization, 6 modes, intensity blending, separate pipeline |
| 11 | Material Graph Compiler | ✅ | MaterialGraph DAG, MaterialCompiler, WGSL generation, 17 built-in materials |

### Surtr-Arch-Review.md Defect Resolution

| # | Defect | Status |
|---|--------|--------|
| 1 | Glass-blur dependency inversion | ✅ Fixed |
| 2 | blur_pyramid.wgsl syntax | ✅ Fixed |
| 3 | Render graph dependency ambiguity | ✅ Fixed |
| 4 | Four independent blur systems | ✅ Kawase shared infrastructure |
| 5 | Material system as raw u32 | ✅ DrawMaterial enum + MaterialGraph |
| 6 | Fragment shader explosion | ✅ 4 specialized pipelines + material graph compiler |
| 7 | Raymarching in UI pipeline | ✅ Separated into pbr_pipeline |
| 8 | Full-resolution bloom | ✅ Kawase pyramid |
| 9 | Large vertex format | ✅ InstanceData foundation |
| 10 | Atlas fragmentation | ❌ Future work (virtualization) |

### Code Health
- Clean compilation, zero errors
- All existing tests pass (0 failures excluding pre-existing headless_render_capture)
- Zero TODO/FIXME/XXX/HACK/unimplemented/todo! markers
- No placeholder stubs or dead code
- 10 Rust source files totaling ~1,650 lines of modular code

### Architecture Summary

**Before:** 452-line hardcoded `end_frame()` with inline GPU encoding, monolithic fragment shader, Gaussian blur, Gaussian bloom, raw u32 modes, hardcoded pass ordering.

**After:** Graph-driven pass dispatch using `kvasir::nodes::build_frame_graph()`, 4 specialized material pipelines, Kawase blur pyramid (5 mip levels), per-pass encoding methods, typed material system, instance buffer infrastructure, material graph compiler with WGSL generation, accessibility service with proper Daltonization.

### Remaining Items (Future Work)

1. **Material Graph Integration** — Wire compiled materials into the fragment shader (replace mode dispatch)
2. **lib.rs Modularization** — Extract remaining inline code into submodules (currently 6,473 lines)
3. **Image Pyramid as First-Class Resource** — ResourceRegistry integration
4. **Documentation** — Update README and architecture docs
