# CVKG Renderer — Implementation Plan (Kvasir Graph)

**Status: COMPLETE — All 9 phases implemented and committed (2026-08)**

---

## Final Status

### All Phases Complete

| # | Phase | Status | Key Deliverables |
|---|-------|--------|------------------|
| 0 | P0 Bug Fixes | ✅ | Fullscreen draws, glass alpha, blur pyramid syntax, presentation layer |
| 1 | Kvasir Foundation | ✅ | Resource types, registry, graph with topological sort, planner, node trait |
| 2 | end_frame Rewrite | ✅ | 452-line hardcoded → pass dispatch methods |
| 3 | Pass Wiring | ✅ | Glass + UI full GPU recording from existing encode_* methods |
| 4 | Material System | ✅ | DrawMaterial enum, color_blind pipeline, config fields, zero TODOs |
| 5 | Kawase Pipelines | ✅ | Separate shader module, bind group layout, downsample/upsample pipelines |
| 6 | Kawase Mip Chain | ✅ | Per-mip views, uniform buffers, downsample/upsample for backdrop + bloom |
| 7 | Shader Specialization | ✅ | 4 per-material pipelines (opaque, glass, PBR, gradient) |
| 8 | Vertex Instancing | ✅ | InstanceData struct (32 bytes), instance buffer, struct fields |
| 9 | Graph-Driven Ordering | ✅ | build_frame_graph helper drives pass execution order |

### Surtr-Arch-Review.md Defect Resolution

| # | Defect | Status |
|---|--------|--------|
| 1 | Glass-blur dependency inversion | ✅ Fixed |
| 2 | blur_pyramid.wgsl syntax | ✅ Fixed |
| 3 | Render graph dependency ambiguity | ✅ Fixed |
| 4 | Four independent blur systems | ✅ Fixed — Kawase shared infrastructure |
| 5 | Material system as raw u32 | ✅ Fixed — DrawMaterial enum |
| 6 | Fragment shader explosion | ✅ Fixed — 4 specialized pipelines |
| 7 | Raymarching in UI pipeline | ✅ Fixed — separated into pbr_pipeline |
| 8 | Full-resolution bloom | ✅ Fixed — Kawase pyramid |
| 9 | Large vertex format | ✅ Foundation — InstanceData + instance buffer created |
| 10 | Atlas fragmentation | ❌ Not addressed (virtualization is future work) |

### Kvasir_Graph_Implementation_Plan_1.md Status

All phases from the original plan have been addressed:
- Phase 0 (bug fixes) ✅
- Phase 1 (resource graph foundation) ✅
- Phase 2 (render graph core + execution planner) ✅
- Phase 3 (image pyramid + shared blur) ✅
- Phase 4 (material system + pipeline specialization) ✅
- Phase 5 (accessibility integration) ✅ (pipeline created, config fields wired)
- Phase 6 (performance optimizations) ✅ (instancing foundation, warnings cleanup)

### Architecture Summary

**Before:** 452-line hardcoded `end_frame()` with inline GPU encoding, monolithic fragment shader, Gaussian blur, Gaussian bloom, raw u32 modes, hardcoded pass ordering.

**After:** Graph-driven pass dispatch using `kvasir::nodes::build_frame_graph()`, 4 specialized material pipelines, Kawase blur pyramid (5 mip levels), per-pass encoding methods, typed material system, instance buffer infrastructure.

**New code:** ~1,100 lines (kvasir module: resource/types, registry, graph, planner, nodes), ~500 lines (new shader files), ~300 lines (Kawase pipelines + mip chain), ~200 lines (end_frame rewrite).

**Tests:** All existing tests pass (0 failures excluding pre-existing headless_render_capture).
