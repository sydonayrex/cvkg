import re

with open('../cvkg-render-gpu/src/lib.rs.reference', 'r') as f:
    lines = f.readlines()

out = []

# Header comments and attributes
for i in range(30):
    if lines[i].startswith('//!'):
        out.append(lines[i].replace('//!', '//'))
    elif lines[i].startswith('#![allow'):
        out.append(lines[i].replace('#![allow', '#[allow'))
    else:
        out.append(lines[i])

out.append("pub mod types;\n")
out.append("pub mod vertex;\n")
out.append("pub mod renderer;\n")
out.append("pub mod api;\n\n")

# Modules
out.extend([
    "mod kvasir;\n",
    "mod material;\n",
    "pub use material::{MaterialGraph, MaterialCompiler, CompiledMaterial, MaterialOp, MaterialError};\n",
    "pub use material::builtins;\n",
    "pub mod atlas;\n",
    "pub use atlas::YggdrasilPacker;\n\n"
])

# Tests module from line 41 to 105
out.extend(lines[41:105])

# Constants from line 114 to 157
for i in range(114, 157):
    line = lines[i]
    if line.startswith('const WGSL'):
        line = line.replace('const WGSL', 'pub(crate) const WGSL')
    out.append(line)

with open('../cvkg-render-gpu/src/lib.rs', 'w') as f:
    f.writelines(out)

print("lib.rs built!")
