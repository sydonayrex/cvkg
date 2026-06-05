import re
import os

with open('../cvkg-render-gpu/src/lib.rs.reference', 'r') as f:
    lines = f.readlines()

with open('map.txt', 'r') as f:
    map_lines = f.readlines()

modules = {
    'types.rs': [],
    'vertex.rs': [],
    'renderer/mod.rs': [],
    'renderer/setup.rs': [],
    'renderer/passes.rs': [],
    'renderer/draw.rs': [],
    'api.rs': [],
    'color_blindness.rs': [],
    'lib.rs': []
}

def assign(kind, name, start, end):
    entry = (start, end, name, kind)
    if name in ['SvgModel', 'SvgAnimation', 'DrawCall', 'ShadowState', 'SurfaceContext', 'HeadlessContext']:
        modules['types.rs'].append(entry)
        return
    if name in ['Vertex', 'InstanceData', 'SceneVertexConstructor', 'CustomStrokeVertexConstructor']:
        modules['vertex.rs'].append(entry)
        return
    if name == 'SurtrRenderer':
        modules['renderer/mod.rs'].append(entry)
        return
    if name in ['ColorBlindMode', 'ColorBlindUniforms']:
        modules['color_blindness.rs'].append(entry)
        return

    if kind == 'TRAIT_IMPL':
        if name == 'Drop':
            modules['renderer/mod.rs'].append(entry)
        elif name == 'Renderer':
            modules['api.rs'].append(entry)
        elif name in ['FrameRenderer', 'ElapsedTime']:
            modules['api.rs'].append(entry)
        return

    if kind == 'METHOD':
        n = name
        if n in ['begin_frame', 'begin_frame_headless', 'end_frame', 'register_window', 'resize', 'reset_time', 'reclaim_vram', 'update_vram_telemetry', 'get_telemetry', 'prewarm_vram', 'submit_buckets', 'submit_routed', 'capture_frame']:
            modules['renderer/mod.rs'].append(entry)
        elif n in ['forge', 'forge_internal', 'forge_headless', 'create_headless_context', 'create_surface_context', 'rebuild_texture_array_bind_group']:
            modules['renderer/setup.rs'].append(entry)
        elif n.startswith('execute_pass_'):
            modules['renderer/passes.rs'].append(entry)
        else:
            modules['renderer/draw.rs'].append(entry)
        return

    if kind == 'FN':
        if name in ['usvg_to_lyon', 'parse_svg_animations']:
            modules['renderer/draw.rs'].append(entry)
        elif name == 'align_to':
            modules['api.rs'].append(entry)
        return

    if kind == 'IMPL':
        if start in [232, 258, 5460, 5483]:
            modules['vertex.rs'].append(entry)
        elif start in [5247]:
            # Impl ColorBlindUniforms
            modules['color_blindness.rs'].append(entry)
        return

for line in map_lines:
    parts = line.strip().split('|')
    if len(parts) == 4:
        kind, name, start, end = parts
        assign(kind, name, int(start), int(end))
    elif len(parts) == 5:
        kind, tr, name, start, end = parts
        assign(kind, tr, int(start), int(end))

for mod, rngs in modules.items():
    if mod == 'lib.rs': continue
    filepath = f'../cvkg-render-gpu/src/{mod}'
    os.makedirs(os.path.dirname(filepath), exist_ok=True)
    with open(filepath, 'w') as f:
        f.write("#![allow(unused_imports, dead_code, clippy::type_complexity, clippy::unwrap_or_default)]\n")
        f.write("use std::sync::atomic::Ordering;\n")
        f.write("use bytemuck;\n")
        f.write("use lyon::tessellation::{\n")
        f.write("    StrokeTessellator, FillTessellator, StrokeOptions, FillOptions,\n")
        f.write("    BuffersBuilder, VertexBuffers, StrokeVertex, FillVertex\n")
        f.write("};\n")
        f.write("use lyon::math::point;\n")
        if mod != 'renderer/draw.rs':
            f.write("use crate::renderer::draw::{parse_svg_animations, usvg_to_lyon};\n")
        f.write("use super::*;\n")
        if mod.startswith('renderer/'):
            f.write("use crate::*;\n")
        f.write("use std::sync::Arc;\n")
        f.write("use std::num::NonZeroUsize;\n")
        f.write("use cvkg_core::*;\n")
        f.write("use lru::LruCache;\n")
        f.write("use crate::types::*;\n")
        f.write("use crate::vertex::*;\n")
        f.write("use crate::renderer::*;\n")
        f.write("use crate::atlas::*;\n")
        f.write("use crate::color_blindness::*;\n")
        if mod == 'renderer/mod.rs':
            f.write("\npub mod setup;\npub mod passes;\npub mod draw;\n\n")

        structs_and_traits = []
        methods_to_wrap = []

        for (start, end, name, kind) in rngs:
            if kind == 'METHOD':
                methods_to_wrap.append((start, end, name))
            else:
                structs_and_traits.append((start, end, name))

        for (start, end, name) in structs_and_traits:
            # We rewrite pub(crate) directly for structs
            block = lines[start-1:end]
            for i in range(len(block)):
                if block[i].startswith('    ') and ':' in block[i] and not block[i].strip().startswith('pub') and not block[i].strip().startswith('fn') and not block[i].strip().startswith('//'):
                    block[i] = re.sub(r'^    ([a-zA-Z0-9_]+): ', r'    pub(crate) \1: ', block[i])
                elif block[i].startswith('    fn ') or block[i].startswith('    async fn '):
                    block[i] = block[i].replace('    fn ', '    pub(crate) fn ').replace('    async fn ', '    pub(crate) async fn ')
            if name in ['usvg_to_lyon', 'parse_svg_animations']:
                # Ensure these functions are pub(crate)
                block[0] = block[0].replace('fn ', 'pub(crate) fn ')

            f.writelines(block)
            f.write("\n")

        if methods_to_wrap:
            f.write("impl SurtrRenderer {\n")
            for (start, end, name) in methods_to_wrap:
                block = lines[start-1:end]
                for i in range(len(block)):
                    if block[i].startswith('    fn ') or block[i].startswith('    async fn '):
                        block[i] = block[i].replace('    fn ', '    pub(crate) fn ').replace('    async fn ', '    pub(crate) async fn ')
                f.writelines(block)
                f.write("\n")
            f.write("}\n")

print("Files generated!")
