import re

with open('/D/rex/projects/cvkg/cvkg-vdom/src/lib.rs', 'r') as f:
    content = f.read()

# Revert unused variable renaming
content = content.replace('_rect: cvkg_core::Rect', 'rect: cvkg_core::Rect')
content = content.replace('_size: f32', 'size: f32')

# Fix node.layout on line 558 or anywhere
content = re.sub(r',\s*node\.layout', '', content)

# Also fix intensity
content = content.replace('intensit_y: f32', 'intensity: f32')
content = content.replace('intensit_y', 'intensity')

with open('/D/rex/projects/cvkg/cvkg-vdom/src/lib.rs', 'w') as f:
    f.write(content)

print("Fixed compile errors")
