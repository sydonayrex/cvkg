# CVKG Renderer — Complete Implementation Plan (Kvasir Graph)

**Based on:** Surtr-Arch-Review.md (14 sections, 11,343 lines)
**Feasibility:** Kvasir_Graph_Implementation_Plan_1.md (900 lines)
**Current state:** v0.2.8 development, kvasir graph architecture in progress

---

## Current Status (Updated 2026-08)

### ✅ Done
- **Kvasir module**: Resource types, registry, graph with topological sort, planner, node trait, PassNode/PassId
- **end_frame()**: Rewritten to use pass dispatch methods (`execute_pass_*()`)
- **Pass encoding**: Full GPU recording for geometry, backdrop copy, glass, UI, bloom extract, bloom blur, composite
- **P0 bugs fixed**: Fullscreen draws (0..3), glass fresnel alpha, blur pyramid WGSL syntax, bloom/blur texture separation
- **Material system**: DrawMaterial enum (Opaque/Glass/TopUI), config fields for bloom_enabled, color_blind_mode
- **Color blind pipeline**: Created, conditional execution wired
- **Kawase pipelines**: kawase_down_pipeline + kwascre_up_pipeline created with separate shader module
- **Raw texture handles**: blur_tex_a/b and bloom_tex_a/b stored in SurfaceContext/HeadlessContext
- **Mip levels**: blur/bloom textures have 5 mip levels (was 1)
- **Zero TODOs/FIXMEs** in lib.rs
- Clean compile, all tests pass

### 🔄 Remaining
- **Kawase pyramid wiring**: Per-mip views, uniform buffer, bind groups, pass recording loop
- **Shader specialization**: shapes.wgsl still monolithic
- **Vertex instancing**: No instancing yet
- **Graph-driven ordering**: end_frame uses hardcoded if/else, not graph compilation
- **Accessibility uniforms**: Pipeline created, recording placeholder

---

## Architecture Summary

**Current:** Graph-driven pass dispatch with individual `execute_pass_*()` methods. Kawase blur pipelines created but not yet wired into the render loop. Glass samples blur before it's generated (backdrop blur placeholder).

**Target:** Fully wired Kvasir graph with Kawase blur pyramid, shader specialization, and graph-driven pass ordering.

---

*See original plan below for full phase-by-phase breakdown.*