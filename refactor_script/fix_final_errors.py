import os

def fix_file(filepath):
    if not os.path.exists(filepath): return
    with open(filepath, 'r') as f:
        content = f.read()
    
    # 1. include_str!("shaders/ -> include_str!("../shaders/
    #    include_bytes!("shaders/ -> include_bytes!("../shaders/
    if filepath.startswith('../cvkg-render-gpu/src/renderer/'):
        content = content.replace('include_str!("shaders/', 'include_str!("../shaders/')
        content = content.replace('include_bytes!("shaders/', 'include_bytes!("../shaders/')
        
    # 2. remove self imports
    filename = os.path.basename(filepath)
    modname = filename.replace('.rs', '')
    if modname == 'mod': modname = os.path.basename(os.path.dirname(filepath))
    content = content.replace(f"use crate::{modname}::*;\n", "")
    
    with open(filepath, 'w') as f:
        f.write(content)

for root, _, files in os.walk('../cvkg-render-gpu/src'):
    for f in files:
        if f.endswith('.rs'):
            fix_file(os.path.join(root, f))

# 3. Fix trait visibilities manually
def remove_vis(filepath, line_nums):
    with open(filepath, 'r') as f:
        lines = f.readlines()
    for ln in line_nums:
        lines[ln-1] = lines[ln-1].replace('pub(crate) fn', 'fn').replace('pub fn', 'fn')
    with open(filepath, 'w') as f:
        f.writelines(lines)

remove_vis('../cvkg-render-gpu/src/vertex.rs', [44, 95, 118])
remove_vis('../cvkg-render-gpu/src/kvasir.rs', [42])
remove_vis('../cvkg-render-gpu/src/material.rs', [228])

# 4. Fix lib.rs comments
with open('../cvkg-render-gpu/src/lib.rs', 'r') as f:
    lib_lines = f.readlines()
for i in range(len(lib_lines)):
    if lib_lines[i].startswith('//!'):
        lib_lines[i] = lib_lines[i].replace('//!', '//')
    if lib_lines[i].startswith('#![allow(clippy::type_complexity'):
        lib_lines[i] = lib_lines[i].replace('#![allow', '#[allow')
with open('../cvkg-render-gpu/src/lib.rs', 'w') as f:
    f.writelines(lib_lines)

print("Done")
