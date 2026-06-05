import re

with open('../cvkg-render-gpu/src/lib.rs', 'r') as f:
    lines = f.readlines()

with open('map.txt', 'r') as f:
    map_lines = f.readlines()

# Define the assignment rules
modules = {
    'types.rs': [],
    'vertex.rs': [],
    'renderer.rs': [],
    'setup.rs': [],
    'passes.rs': [],
    'draw.rs': [],
    'api.rs': [],
    'lib.rs': [] # for things that must stay
}

def assign(kind, name, start, end):
    # Some things are exact matches
    if name in ['SvgModel', 'SvgAnimation', 'DrawCall', 'ShadowState', 'SurfaceContext', 'HeadlessContext']:
        modules['types.rs'].append((start, end, name))
        return
    if name in ['Vertex', 'InstanceData', 'SceneVertexConstructor', 'CustomStrokeVertexConstructor']:
        modules['vertex.rs'].append((start, end, name))
        return
    if name == 'SurtrRenderer':
        modules['renderer.rs'].append((start, end, name))
        return
        
    # Trait impls
    if kind == 'TRAIT_IMPL':
        if name == 'Drop':
            modules['renderer.rs'].append((start, end, name))
        elif name == 'Renderer':
            modules['api.rs'].append((start, end, name))
        elif name in ['FrameRenderer', 'ElapsedTime']:
            modules['api.rs'].append((start, end, name))
        return
        
    if kind == 'METHOD':
        n = name
        if n in ['begin_frame', 'begin_frame_headless', 'end_frame', 'register_window', 'resize', 'reset_time', 'reclaim_vram', 'update_vram_telemetry', 'get_telemetry', 'prewarm_vram', 'submit_buckets', 'submit_routed', 'capture_frame']:
            modules['renderer.rs'].append((start, end, name))
        elif n in ['forge', 'forge_internal', 'forge_headless', 'create_headless_context', 'create_surface_context', 'rebuild_texture_array_bind_group']:
            modules['setup.rs'].append((start, end, name))
        elif n.startswith('execute_pass_'):
            modules['passes.rs'].append((start, end, name))
        else:
            modules['draw.rs'].append((start, end, name))
        return
        
    if kind == 'FN':
        modules['draw.rs'].append((start, end, name))
        return
        
    if kind == 'IMPL':
        if start == 232 or start == 258 or start == 5439 or start == 5462:
            modules['vertex.rs'].append((start, end, name))
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
    with open(f'../cvkg-render-gpu/src/{mod}', 'w') as f:
        f.write("#![allow(unused_imports, dead_code, clippy::type_complexity, clippy::unwrap_or_default)]\n")
        f.write("use super::*;\n")
        f.write("use std::sync::Arc;\n")
        f.write("use std::num::NonZeroUsize;\n")
        f.write("use cvkg_core::*;\n")
        f.write("use lru::LruCache;\n")
        f.write("use crate::types::*;\n")
        f.write("use crate::vertex::*;\n")
        f.write("use crate::renderer::*;\n")
        f.write("use crate::atlas::*;\n")
        
        # When creating the impl blocks for SurtrRenderer, we need to wrap the methods
        methods = [r for r in rngs if r[2] != '' and r[2] != 'SvgModel'] # just a hacky check
        # Let's just output the methods wrapped in `impl SurtrRenderer { ... }`
        
        structs_and_traits = []
        methods_to_wrap = []
        
        for r in rngs:
            # If it's a method, we wrap it.
            # We can tell by looking at the line itself.
            first_line = lines[r[0]-1].strip()
            if first_line.startswith('pub fn') or first_line.startswith('fn') or first_line.startswith('pub async fn') or first_line.startswith('async fn'):
                # Wait, global fn like parse_svg_animations shouldn't be wrapped in impl
                if r[2] in ['parse_svg_animations', 'usvg_to_lyon']:
                    structs_and_traits.append(r)
                else:
                    methods_to_wrap.append(r)
            else:
                structs_and_traits.append(r)
                
        for (start, end, name) in structs_and_traits:
            f.writelines(lines[start-1:end])
            f.write("\n")
            
        if methods_to_wrap:
            f.write("impl SurtrRenderer {\n")
            for (start, end, name) in methods_to_wrap:
                f.writelines(lines[start-1:end])
                f.write("\n")
            f.write("}\n")

print("Files generated!")
