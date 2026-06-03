# Kvasir Graph Implementation Plan — Revised Edition

**Source:** Kvasir_Graph_Implementation_Plan_2.md with corrections from kvasir_update.md
**Subject:** Renderer 3.0 — Unified Visual Computation Graph (Corrected Architecture)
**Date:** 2026-06-02

---

## Executive Summary (Revised)

The Kvasir architecture is technically achievable in Rust with wgpu and represents a well-understood class of systems. This revised plan incorporates critique feedback to address

```text
One Universal Graph → Multiple Specialized Graph Domains
```

The correction establishes **five independent but synchronized graphs** under a unified runtime layer, eliminating the monolithic graph anti-pattern identified in the critique.

**Phase 0 remains the priority** — the seven critical blocking bugs must be resolved before any Kvasir work begins.

---

## New Architectural Principles

| Principle | Description |
|-----------|-------------|
| **KVASIR-012** | Kvasir Is A Runtime, Not A Graph |
| **KVASIR-013** | Multiple Specialized Graph Domains |
| **KVASIR-014** | Temporal Graph Is First-Class |
| **KVASIR-015** | Material Graph Compiles Through IR |
| **KVASIR-016** | Accessibility Split Into Visual and Semantic |
| **KVASIR-017** | Resource Virtualization Moves To Phase 1 |
| **KVASIR-018** | Execution Plans Are Cached |
| **KVASIR-019** | AI Generates Declarative Descriptions, Not Runtime Nodes |
| **KVASIR-020** | Kvasir Runtime Becomes Its Own Crate |

---

## Revised Core Architecture

```text
Application Layer
        │
        ▼

Kvasir Runtime
├── Scene Graph      (Geometry topology, retained state)
├── Execution Graph  (Render pass scheduling, barriers)
├── Resource Graph   (Lifetime management, virtualization)
├── Material Graph   (Shader generation, IR compilation)
├── Temporal Graph   (Animation, physics, cross-frame deps)
└── Accessibility Layer
        │
        ▼

Execution Planner
        │
        ▼

Raster / Compute / Hybrid
        │
        ▼

GPU Backend
```

---

## Phase 0 — Fix Blocking Bugs (Sprint 1–2)

Unchanged from original plan. Seven critical bugs must be resolved:

| Bug | Fix |
|-----|-----|
| `fs_copy` reads Mega-Atlas instead of scene texture | Change to sample `t_env` (bloom.wgsl:L3) |
| `stroke_path` DrawCall uses vertex cursor as index cursor | Capture `base_index = self.indices.len()` before tessellation (lib.rs:L4639-4650) |
| Parallel rayon passes share `ctx_scene_texture` without barrier | Encode Glass and UI passes sequentially (lib.rs:L2947-3139) |
| Bloom extract overwrites backdrop blur texture | Allocate separate `bloom_tex_a/b` (lib.rs:L3174-3199) |
| `vs_fullscreen` draws 6 vertices — second triangle is degenerate | Change all fullscreen draws to `p.draw(0..3, 0..1)` (common.wgsl:L87-92) |
| Glyph atlas fallback writes to `(0,0)` on full atlas | Return early with error log (lib.rs:L3845-3852, L3977-3983) |
| SVG tessellation `.unwrap()` panics on malformed paths | Propagate as `Result`, skip degenerate paths (lib.rs:L5118-5132) |
| Clip SDF uses `clip_position.xy` as NDC | Remove the `* 0.5 + 0.5 * resolution` transform (shapes.wgsl:L63-74) |
| Bifrost glass alpha is 1–3% | Fix the fresnel alpha calculation (shapes.wgsl:L100-171) |

---

## Phase 1 — Resource Graph Foundation (Sprint 3–6)
