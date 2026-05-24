import re
import os

def remove_block(filepath, pattern):
    with open(filepath, 'r') as f:
        content = f.read()
    new_content = re.sub(pattern, '', content, flags=re.MULTILINE | re.DOTALL)
    with open(filepath, 'w') as f:
        f.write(new_content)

def main():
    # cvkg-components/src/interactive.rs: hover_index, left_items, right_items
    with open('cvkg-components/src/interactive.rs', 'r') as f:
        c = f.read()
    c = c.replace('hover_index: Option<usize>,', '')
    c = c.replace('hover_index: None,', '')
    
    c = c.replace('left_items: Vec<T>,', '')
    c = c.replace('right_items: Vec<T>,', '')
    c = c.replace('left_items: Vec::new(),', '')
    c = c.replace('right_items: Vec::new(),', '')
    with open('cvkg-components/src/interactive.rs', 'w') as f:
        f.write(c)

    # cvkg-components/src/multi_agent_orchestrator.rs: unused functions
    file = 'cvkg-components/src/multi_agent_orchestrator.rs'
    remove_block(file, r'fn r\(x: f32, y: f32, w: f32, h: f32\) -> Rect \{.*?\n\}')
    remove_block(file, r'fn render_output_panel\(.*?\n\}') # might have multiple nested braces, better to just let clippy or manually fix.
    
if __name__ == "__main__":
    main()
