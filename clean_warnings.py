import re
import os

def remove_block(filepath, pattern):
    with open(filepath, 'r') as f:
        content = f.read()
    
    # Simple regex to remove a function or block, assuming standard indentation
    # e.g., `    fn focus_order_hash() -> u64 {\n        ...\n    }`
    # We match the signature and everything until the matching closing brace.
    new_content = re.sub(pattern, '', content, flags=re.MULTILINE | re.DOTALL)
    
    with open(filepath, 'w') as f:
        f.write(new_content)

def main():
    # autocomplete.rs: remove `text: String,` and `text: String::new(),`
    with open('cvkg-components/src/autocomplete.rs', 'r') as f:
        c = f.read()
    c = c.replace('text: String,', '')
    c = c.replace('text: String::new(),', '')
    with open('cvkg-components/src/autocomplete.rs', 'w') as f:
        f.write(c)

    # richtext.rs: remove trait RichTextExt
    with open('cvkg-components/src/richtext.rs', 'r') as f:
        lines = f.readlines()
    with open('cvkg-components/src/richtext.rs', 'w') as f:
        skip = False
        for line in lines:
            if 'trait RichTextExt' in line:
                skip = True
            if skip and line.startswith('}'):
                skip = False
                continue
            if not skip:
                f.write(line)

    # datepicker.rs: remove set_open_state and set_displayed_month
    remove_block('cvkg-components/src/datepicker.rs', r'    fn set_open_state\(&self, open: bool\) \{.*?\n    \}')
    remove_block('cvkg-components/src/datepicker.rs', r'    fn set_displayed_month\(&self, month: u32, year: u32\) \{.*?\n    \}')

    # keyboard_nav.rs: remove focus_order_hash and current_focus_hash
    remove_block('cvkg-components/src/keyboard_nav.rs', r'fn focus_order_hash\(\) -> u64 \{.*?\n\}')
    remove_block('cvkg-components/src/keyboard_nav.rs', r'fn current_focus_hash\(\) -> u64 \{.*?\n\}')

    # popover.rs: remove set_open_state
    remove_block('cvkg-components/src/popover.rs', r'    fn set_open_state\(&self, open: bool\) \{.*?\n    \}')

    # toast.rs: remove is_expired, purge_expired, init_timestamps
    remove_block('cvkg-components/src/toast.rs', r'    fn is_expired\(&self, current_time: f32\) -> bool \{.*?\n    \}')
    remove_block('cvkg-components/src/toast.rs', r'    fn purge_expired\(&mut self, current_time: f32\) \{.*?\n    \}')
    remove_block('cvkg-components/src/toast.rs', r'    fn init_timestamps\(&mut self, current_time: f32\) \{.*?\n    \}')

    # cvkg-render-gpu/src/lib.rs: remove clear, mut insert_idx
    with open('cvkg-render-gpu/src/lib.rs', 'r') as f:
        c = f.read()
    c = c.replace('let mut insert_idx = idx;', 'let insert_idx = idx;')
    with open('cvkg-render-gpu/src/lib.rs', 'w') as f:
        f.write(c)
    remove_block('cvkg-render-gpu/src/lib.rs', r'    fn clear\(&mut self\) \{.*?\n    \}')

if __name__ == "__main__":
    main()
