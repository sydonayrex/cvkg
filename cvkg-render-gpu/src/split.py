#!/usr/bin/env python3
"""
One-shot script to split lib.rs into modules.
Run from cvkg-render-gpu/src/ directory.
Reads lib.rs, writes types.rs, vertex.rs, renderer.rs, setup.rs, passes.rs, draw.rs, api.rs.
Rewrites lib.rs to be just module declarations and re-exports.
"""
import re

with open('lib.rs', 'r') as f:
    lines = f.readlines()

total = len(lines)

def extract(start, end):
    """Extract lines start..end (1-indexed, inclusive)."""
    return lines[start-1:end]

def join(chunks):
    return ''.join(chunks)

# ──────────────────────────────────────────────────────────────
# types.rs — SvgModel, SvgAnimation, DrawCall, ShadowState, 
#             SurfaceContext, HeadlessContext, MAX_VERTICES/INDICES
# Lines: 159-166 (SvgModel), 168-176 (SvgAnimation), 238-256 (DrawCall, ShadowState)
#         449-505 (SurfaceContext, HeadlessContext), 507-508 (constants)
# ──────────────────────────────────────────────────────────────
types_rs = []
types_rs.append("//! Core data types, internal structs, and rendering contexts.\n")
types_rs.append("use cvkg_core::Rect;\n")
types_rs.append("use crate::vertex::Vertex;\n\n")

# SvgModel + SvgAnimation (lines 159-176)
for line in extract(159, 176):
    types_rs.append(line)
types_rs.append("\n")

# DrawCall (lines 238-249) — needs pub(crate)
for line in extract(238, 249):
    line = line.replace("struct DrawCall", "pub(crate) struct DrawCall")
    types_rs.append(line)
types_rs.append("\n")

# ShadowState (lines 251-256) — needs pub(crate) 
for line in extract(251, 256):
    line = line.replace("struct ShadowState", "pub(crate) struct ShadowState")
    types_rs.append(line)
types_rs.append("\n")

# SurfaceContext (lines 449-475) — needs pub(crate)
for line in extract(449, 475):
    line = line.replace("struct SurfaceContext", "pub(crate) struct SurfaceContext")
    # Make fields pub(crate)
    m = re.match(r'^    ([a-zA-Z_][a-zA-Z0-9_]*): ', line)
    if m and not line.strip().startswith('//'):
        line = line.replace(f'    {m.group(1)}: ', f'    pub(crate) {m.group(1)}: ')
    types_rs.append(line)
types_rs.append("\n")

# HeadlessContext (lines 477-505)
for line in extract(477, 505):
    types_rs.append(line)
types_rs.append("\n")

# Constants (lines 507-508)
for line in extract(507, 508):
    line = line.replace("const MAX_", "pub(crate) const MAX_")
    types_rs.append(line)

# ──────────────────────────────────────────────────────────────
# vertex.rs — Vertex, InstanceData, constructors, trait impls
# Lines: 200-284 (Vertex, InstanceData, impls)
#         5443-5503 (SceneVertexConstructor, CustomStrokeVertexConstructor, trait impls)
# ──────────────────────────────────────────────────────────────
vertex_rs = []
vertex_rs.append("//! Vertex layouts, instance data, and tessellation vertex constructors.\n")
vertex_rs.append("use lyon::tessellation::{FillVertex, FillVertexConstructor, StrokeVertex, StrokeVertexConstructor};\n\n")

# Vertex struct (lines 200-218) — already pub
for line in extract(200, 218):
    vertex_rs.append(line)
vertex_rs.append("\n")

# InstanceData (lines 220-236)
for line in extract(220, 236):
    vertex_rs.append(line)
vertex_rs.append("\n")

# impl Vertex (lines 258-284) — desc() needs pub(crate)
for line in extract(258, 284):
    line = line.replace("    fn desc()", "    pub(crate) fn desc()")
    vertex_rs.append(line)
vertex_rs.append("\n")

# SceneVertexConstructor (lines 5443-5448) — needs pub(crate)
for line in extract(5443, 5448):
    line = line.replace("struct SceneVertexConstructor", "pub(crate) struct SceneVertexConstructor")
    m = re.match(r'^    ([a-zA-Z_][a-zA-Z0-9_]*): ', line)
    if m and not line.strip().startswith('//'):
        line = line.replace(f'    {m.group(1)}: ', f'    pub(crate) {m.group(1)}: ')
    vertex_rs.append(line)
vertex_rs.append("\n")

# CustomStrokeVertexConstructor (lines 5450-5458) — needs pub(crate)
for line in extract(5450, 5458):
    line = line.replace("struct CustomStrokeVertexConstructor", "pub(crate) struct CustomStrokeVertexConstructor")
    m = re.match(r'^    ([a-zA-Z_][a-zA-Z0-9_]*): ', line)
    if m and not line.strip().startswith('//'):
        line = line.replace(f'    {m.group(1)}: ', f'    pub(crate) {m.group(1)}: ')
    vertex_rs.append(line)
vertex_rs.append("\n")

# impl StrokeVertexConstructor for CustomStrokeVertexConstructor (lines 5460-5481)
for line in extract(5460, 5481):
    vertex_rs.append(line)
vertex_rs.append("\n")

# impl FillVertexConstructor for SceneVertexConstructor (lines 5483-5503)
for line in extract(5483, 5503):
    vertex_rs.append(line)
vertex_rs.append("\n")

# ──────────────────────────────────────────────────────────────
# renderer.rs — SurtrRenderer struct + core lifecycle impl block
# Lines: 286-447 (struct), 510-3956 (first big impl block)
#         5505-5513 (Drop), 
#         5515-5573 (submit_buckets, submit_routed)
#         5710-6252 (apply_opacity, load_svg, tessellate_node, draw_svg, 
#                    forge_headless, capture_frame, current_width/height/scale, find_filter)
# ──────────────────────────────────────────────────────────────
renderer_rs = []
renderer_rs.append("//! The main SurtrRenderer struct and core frame lifecycle.\n")
renderer_rs.append("use cvkg_core::Rect;\n")
renderer_rs.append("use lru::LruCache;\n")
renderer_rs.append("use std::num::NonZeroUsize;\n")
renderer_rs.append("use std::sync::Arc;\n")
renderer_rs.append("use cvkg_core::{LAYOUT_DIRTY, Mesh, Renderer};\n")
renderer_rs.append("use std::sync::atomic::Ordering;\n")
renderer_rs.append("use bytemuck;\n")
renderer_rs.append("use crate::color_blindness::ColorBlindUniforms;\n")
renderer_rs.append("use lyon::tessellation::{\n")
renderer_rs.append("    BuffersBuilder, FillOptions, FillTessellator, FillVertex, FillVertexConstructor, StrokeOptions,\n")
renderer_rs.append("    StrokeTessellator, StrokeVertex, StrokeVertexConstructor, VertexBuffers,\n")
renderer_rs.append("};\n")
renderer_rs.append("use lyon::math::point;\n")
renderer_rs.append("use crate::types::*;\n")
renderer_rs.append("use crate::vertex::*;\n")
renderer_rs.append("use crate::atlas::YggdrasilPacker;\n")
renderer_rs.append("use crate::color_blindness::ColorBlindMode;\n")
renderer_rs.append("use cvkg_core::{ColorTheme, SceneUniforms};\n\n")

# SurtrRenderer struct (lines 286-447)
for line in extract(286, 447):
    # Make private fields pub(crate) so other modules can access
    m = re.match(r'^    ([a-zA-Z_][a-zA-Z0-9_]*): ', line)
    if m and not line.strip().startswith('pub') and not line.strip().startswith('//'):
        line = line.replace(f'    {m.group(1)}: ', f'    pub(crate) {m.group(1)}: ')
    renderer_rs.append(line)
renderer_rs.append("\n")

# First big impl block (lines 510-3956)
for line in extract(510, 3956):
    # Make private fn pub(crate) (but not trait methods)
    if line.startswith('    fn '):
        line = line.replace('    fn ', '    pub(crate) fn ')
    elif line.startswith('    async fn '):
        line = line.replace('    async fn ', '    pub(crate) async fn ')
    renderer_rs.append(line)
renderer_rs.append("\n")

# Drop impl (lines 5505-5513)
for line in extract(5505, 5513):
    renderer_rs.append(line)
renderer_rs.append("\n")

# submit_buckets/submit_routed block (lines 5515-5573)
for line in extract(5515, 5573):
    if line.startswith('    fn '):
        line = line.replace('    fn ', '    pub(crate) fn ')
    renderer_rs.append(line)
renderer_rs.append("\n")

# SVG + utility methods block (lines 5710-6252)
renderer_rs.append("impl SurtrRenderer {\n")
for line in extract(5711, 6251):
    if line.startswith('    fn '):
        line = line.replace('    fn ', '    pub(crate) fn ')
    elif line.startswith('    async fn '):
        line = line.replace('    async fn ', '    pub(crate) async fn ')
    renderer_rs.append(line)
renderer_rs.append("}\n")

# ──────────────────────────────────────────────────────────────
# passes.rs — execute_pass_* methods (lines 3968-4072)
# ──────────────────────────────────────────────────────────────
passes_rs = []
passes_rs.append("//! Individual GPU render passes and their command encoding logic.\n")
passes_rs.append("use crate::renderer::SurtrRenderer;\n")
passes_rs.append("use crate::types::*;\n")
passes_rs.append("use crate::color_blindness::ColorBlindUniforms;\n")
passes_rs.append("use bytemuck;\n\n")

for line in extract(3968, 4072):
    if line.startswith('    fn '):
        line = line.replace('    fn ', '    pub(crate) fn ')
    passes_rs.append(line)
passes_rs.append("\n")

# ──────────────────────────────────────────────────────────────
# draw.rs — free functions: parse_svg_animations, usvg_to_lyon
# Lines: 5341-5441
# ──────────────────────────────────────────────────────────────
draw_rs = []
draw_rs.append("//! SVG parsing helpers and free functions.\n")
draw_rs.append("use crate::types::SvgAnimation;\n")
draw_rs.append("use lyon::math::point;\n\n")

for line in extract(5341, 5441):
    draw_rs.append(line)
draw_rs.append("\n")

# ──────────────────────────────────────────────────────────────
# api.rs — trait impls: Renderer, ElapsedTime, FrameRenderer,
#           event handler methods
# Lines: 3958-3966 (ElapsedTime), 4074-5234 (Renderer),
#         5238-5339 (event handler methods),
#         5575-5708 (FrameRenderer)
# ──────────────────────────────────────────────────────────────
api_rs = []
api_rs.append("//! Bridging the internal renderer to `cvkg-core` traits.\n")
api_rs.append("use cvkg_core::{Mesh, Rect, Renderer, ColorTheme, SceneUniforms};\n")
api_rs.append("use crate::renderer::SurtrRenderer;\n")
api_rs.append("use crate::types::*;\n")
api_rs.append("use crate::vertex::*;\n")
api_rs.append("use bytemuck;\n")
api_rs.append("use std::sync::atomic::Ordering;\n")
api_rs.append("use cvkg_core::LAYOUT_DIRTY;\n")
api_rs.append("use lyon::tessellation::{\n")
api_rs.append("    BuffersBuilder, FillOptions, FillTessellator, StrokeOptions,\n")
api_rs.append("    StrokeTessellator, VertexBuffers,\n")
api_rs.append("};\n")
api_rs.append("use lyon::math::point;\n\n")

# ElapsedTime (lines 3958-3966)
for line in extract(3958, 3966):
    api_rs.append(line)
api_rs.append("\n")

# Renderer trait impl (lines 4074-5234)
for line in extract(4074, 5234):
    api_rs.append(line)
api_rs.append("\n")

# Event handler methods (lines 5236-5339)
for line in extract(5236, 5339):
    api_rs.append(line)
api_rs.append("\n")

# FrameRenderer (lines 5575-5708)
for line in extract(5575, 5708):
    api_rs.append(line)
api_rs.append("\n")

# ──────────────────────────────────────────────────────────────
# lib.rs — module declarations, re-exports, constants, tests
# ──────────────────────────────────────────────────────────────
lib_rs = []
# Keep the doc comments at top (lines 1-23), but convert //! to //
for line in extract(1, 23):
    lib_rs.append(line)

# Keep #![allow...] (line 24)
for line in extract(24, 24):
    lib_rs.append(line)
lib_rs.append("\n")

# Module declarations (lines 26-27 — kvasir, material)
for line in extract(26, 27):
    lib_rs.append(line)
lib_rs.append("\n")

# Material re-exports (lines 29-31)
for line in extract(29, 31):
    lib_rs.append(line)
lib_rs.append("\n")

# New module declarations
lib_rs.append("pub mod types;\n")
lib_rs.append("pub mod vertex;\n")
lib_rs.append("pub mod renderer;\n")
lib_rs.append("mod passes;\n")
lib_rs.append("mod draw;\n")
lib_rs.append("mod api;\n\n")

# atlas module (line 38-39)
for line in extract(38, 39):
    lib_rs.append(line)
lib_rs.append("\n")

# Tests (lines 41-105)
for line in extract(41, 105):
    # Fix test reference to parse_svg_animations (now in draw module)
    line = line.replace("parse_svg_animations(", "draw::parse_svg_animations(")
    lib_rs.append(line)
lib_rs.append("\n")

# WGSL constants (lines 109-151)
for line in extract(109, 151):
    lib_rs.append(line)
lib_rs.append("\n")

# color_blindness module (lines 154, 156-157)
for line in extract(154, 157):
    lib_rs.append(line)
lib_rs.append("\n")

# ShieldWall re-exports (lines 178-187)
for line in extract(178, 187):
    lib_rs.append(line)
lib_rs.append("\n")

# Re-export SurtrRenderer
lib_rs.append("pub use renderer::SurtrRenderer;\n")
lib_rs.append("pub use types::{SvgModel, SvgAnimation};\n")
lib_rs.append("pub use vertex::{Vertex, InstanceData};\n")

# Write all files
with open('types.rs', 'w') as f:
    f.write(join(types_rs))

with open('vertex.rs', 'w') as f:
    f.write(join(vertex_rs))

with open('renderer.rs', 'w') as f:
    f.write(join(renderer_rs))

with open('passes.rs', 'w') as f:
    f.write(join(passes_rs))

with open('draw.rs', 'w') as f:
    f.write(join(draw_rs))

with open('api.rs', 'w') as f:
    f.write(join(api_rs))

with open('lib.rs', 'w') as f:
    f.write(join(lib_rs))

print(f"Split complete. Original: {total} lines.")
for name in ['lib.rs', 'types.rs', 'vertex.rs', 'renderer.rs', 'passes.rs', 'draw.rs', 'api.rs']:
    with open(name) as f:
        count = sum(1 for _ in f)
    print(f"  {name}: {count} lines")
