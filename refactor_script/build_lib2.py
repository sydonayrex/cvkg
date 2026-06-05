import re

with open('../cvkg-render-gpu/src/lib.rs.reference', 'r') as f:
    lines = f.readlines()

out = []

for i in range(40):
    if lines[i].startswith('//!'):
        out.append(lines[i].replace('//!', '//'))
    elif lines[i].startswith('#![allow'):
        out.append(lines[i].replace('#![allow', '#[allow'))
    else:
        out.append(lines[i])

out.append("pub mod types;\n")
out.append("pub mod vertex;\n")
out.append("pub mod renderer;\n")
out.append("pub mod api;\n")
out.append("pub mod color_blindness;\n\n")

# Find tests
start_test = -1
for i in range(len(lines)):
    if lines[i].startswith('#[cfg(test)]'):
        start_test = i
        break

if start_test != -1:
    end_test = start_test
    open_braces = 0
    found_brace = False
    for i in range(start_test, len(lines)):
        if '{' in lines[i]:
            open_braces += lines[i].count('{')
            found_brace = True
        if '}' in lines[i]:
            open_braces -= lines[i].count('}')
        if found_brace and open_braces == 0:
            end_test = i
            break

    out.extend(lines[start_test:end_test+1])
    out.append("\n")

# Find constants
for i in range(len(lines)):
    if lines[i].startswith('const WGSL'):
        out.append(lines[i].replace('const WGSL', 'pub(crate) const WGSL'))
    elif lines[i].startswith('    include_str!("shaders/'):
        out.append(lines[i].replace('include_str!("shaders/', 'include_str!("../shaders/'))
    elif lines[i].startswith(');'):
        # If the previous line was part of a const, we keep it
        if out[-1].strip().startswith('include_str!'):
            out.append(lines[i])
            
with open('../cvkg-render-gpu/src/lib.rs', 'w') as f:
    f.writelines(out)
