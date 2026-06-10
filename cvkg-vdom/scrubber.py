import re

with open('/D/rex/projects/cvkg/cvkg-vdom/src/lib.rs', 'r') as f:
    lines = f.read().split('\n')

new_lines = []
skip = 0

for i, line in enumerate(lines):
    if skip > 0:
        skip -= 1
        continue

    # Remove LayoutRect struct definition
    if line.startswith('/// Represents the computed layout bounds of a component in the Virtual DOM.'):
        if lines[i+1].startswith('#[derive(') and lines[i+2].startswith('pub struct LayoutRect'):
            skip = 12
            continue

    # Remove layout: LayoutRect { ... } initialization
    if 'layout: LayoutRect {' in line:
        skip = 5
        continue
    
    # Remove `pub layout: LayoutRect,`
    if 'pub layout: LayoutRect,' in line:
        continue
    if 'layout: Option<LayoutRect>,' in line:
        continue

    # Remove let layout_changed ...
    if 'let layout_changed = old_node.layout != new_node.layout;' in line:
        continue
    
    # Remove layout: if layout_changed ...
    if 'layout: if layout_changed {' in line:
        skip = 4
        continue

    # Remove layout: layout_changed.then_some(new_node.layout),
    if 'layout: layout_changed.then_some(new_node.layout),' in line:
        continue
    
    # Remove from to_accesskit_node
    if 'node.set_bounds(accesskit::Rect {' in line:
        if 'self.layout.x' in lines[i+1]:
            new_lines.append('        node.set_bounds(accesskit::Rect { x0: 0.0, y0: 0.0, x1: 0.0, y1: 0.0 });')
            skip = 5
            continue

    # Remove VNode diff layout checks
    if '&& self.layout == other.layout' in line:
        continue
    if '.field("layout", &self.layout)' in line:
        continue
    if '.field("layout", layout)' in line:
        continue

    # Remove sdf_distance layout usage and layout field in Update
    if 'layout,' in line.strip() and len(line.strip()) == 7: # just `layout,`
        continue
    if 'layout:' in line and '&LayoutRect' in line:
        continue
    if 'state.serialize_field("layout", layout)?;' in line:
        continue

    # Hit test layout usage
    if 'let dist = Self::sdf_distance(node.sdf_shape.as_ref(), &node.layout, x, y);' in line:
        new_lines.append('        let dist = 1000.0; // Hit testing geometry deferred to physics/taffy')
        continue

    if 'let tolerance = 0.5;' in line:
        if 'if (vnode.layout.x' in lines[i+1]:
            skip = 5
            continue

    new_lines.append(line)

with open('/D/rex/projects/cvkg/cvkg-vdom/src/lib.rs', 'w') as f:
    f.write('\n'.join(new_lines))

print("Scrubbed layout")
