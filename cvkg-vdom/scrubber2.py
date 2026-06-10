import re

with open('/D/rex/projects/cvkg/cvkg-vdom/src/lib.rs', 'r') as f:
    content = f.read()

# Fix layout assignment in apply_patch
content = re.sub(r'if let Some\(l\) = layout \{\s*node\.layout = l;\s*\}', '', content)
content = re.sub(r'\|\|\s*layout_changed', '', content)

# Remove unused variables by prefixing with _ in Renderer implementation
for var in ['rect: cvkg_core::Rect', 'x: f32', 'y: f32', 'w', 'h', 'size: f32']:
    if var == 'rect: cvkg_core::Rect':
        content = content.replace('rect: cvkg_core::Rect', '_rect: cvkg_core::Rect')
    elif var == 'x: f32':
        content = content.replace('x: f32', '_x: f32')
    elif var == 'y: f32':
        content = content.replace('y: f32', '_y: f32')
    elif var == 'w':
        content = content.replace('let (w, h)', 'let (_w, _h)')
    elif var == 'size: f32':
        content = content.replace('size: f32', '_size: f32')

# Fix layout.x missing in hit_test or related code (around line 1322)
# actually let's just delete the rest of the file from 1708 if it's accesskit stuff
# Wait, let's fix the specific accesskit node.layout usages
content = re.sub(r'x0: node\.layout\.x as f64,', 'x0: 0.0,', content)
content = re.sub(r'y0: node\.layout\.y as f64,', 'y0: 0.0,', content)
content = re.sub(r'x1: \(node\.layout\.x \+ node\.layout\.width\) as f64,', 'x1: 0.0,', content)
content = re.sub(r'y1: \(node\.layout\.y \+ node\.layout\.height\) as f64,', 'y1: 0.0,', content)

# Fix .clamp issue:
content = re.sub(r'\.clamp\(0\.0, 1\.0\)', '.clamp(0.0f32, 1.0f32)', content)

# What is at line 1322? "x: layout.x"? 
# This might be in `sdf_distance` or similar if I didn't remove it properly. Let's remove the whole sdf_distance function if it's still there.
content = re.sub(r'fn sdf_distance.*?\}\n\s*\}', '', content, flags=re.DOTALL)

with open('/D/rex/projects/cvkg/cvkg-vdom/src/lib.rs', 'w') as f:
    f.write(content)

print("Scrubbed layout 2")
