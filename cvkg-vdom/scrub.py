import re

with open('/D/rex/projects/cvkg/cvkg-vdom/src/lib.rs', 'r') as f:
    content = f.read()

# Remove layout diffing
content = re.sub(r'let layout_changed\s*=\s*old_node\.layout\s*!=\s*new_node\.layout;', '', content)
content = re.sub(r'layout:\s*if layout_changed.*?Some\(new_node\.layout\)\s*\} else \{\s*None\s*\},', '', content, flags=re.DOTALL)
content = re.sub(r'layout:\s*layout_changed\.then_some\(new_node\.layout\),', '', content)

# Remove LayoutRect missing type error
content = re.sub(r'layout:\s*&LayoutRect,', '', content)

# Remove sdf_distance layout usage
content = re.sub(r'layout:\s*LayoutRect\s*\{[^}]+\},', '', content)

# In hit_test_recursive, replace the sdf_distance call
# Let's just stub hit_test entirely
hit_test_stub = """
    pub fn hit_test(&self, x: f32, y: f32) -> Option<(NodeId, f32)> {
        // Hit testing is now deferred to the renderer/layout engine which knows absolute geometry.
        None
    }

    fn hit_test_recursive(&self, node_id: NodeId, x: f32, y: f32) -> Option<(NodeId, f32)> {
        None
    }
"""

content = re.sub(r'pub fn hit_test\(&self, x: f32, y: f32\).*?fn hit_test_recursive\(&self, node_id: NodeId, x: f32, y: f32\) -> Option<\(NodeId, f32\)> \{.*?None\n    \}', hit_test_stub, content, flags=re.DOTALL)

# Remove sdf_distance function
content = re.sub(r'fn sdf_distance\(.*?\}', '', content, flags=re.DOTALL)

with open('/D/rex/projects/cvkg/cvkg-vdom/src/lib.rs', 'w') as f:
    f.write(content)

print("Scrubbed layout")
