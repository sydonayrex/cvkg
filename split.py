import re

with open('cvkg-render-gpu/src/lib.rs', 'r') as f:
    content = f.read()

# We can parse top-level items using a simple tokenizer/brace matcher
def get_top_level_items(code):
    items = []
    current_item = []
    brace_level = 0
    in_comment = False
    in_string = False
    string_char = None
    
    i = 0
    start_idx = 0
    
    while i < len(code):
        char = code[i]
        
        # Simple string handling
        if in_string:
            if char == '\\':
                i += 2
                continue
            elif char == string_char:
                in_string = False
        else:
            if char == '"' or char == "'":
                in_string = True
                string_char = char
            elif char == '/' and i + 1 < len(code) and code[i+1] == '/':
                # Line comment
                while i < len(code) and code[i] != '\n':
                    i += 1
                continue
            elif char == '{':
                brace_level += 1
            elif char == '}':
                brace_level -= 1
                if brace_level == 0:
                    # End of an item like struct or impl
                    items.append(code[start_idx:i+1])
                    start_idx = i+1
            elif char == ';' and brace_level == 0:
                # End of a statement item like type or use
                items.append(code[start_idx:i+1])
                start_idx = i+1
                
        i += 1
        
    if start_idx < len(code):
        items.append(code[start_idx:])
        
    return items

items = get_top_level_items(content)
for i, item in enumerate(items):
    lines = item.strip().split('\n')
    if not lines or not lines[0]: continue
    first_line = lines[0].strip()
    if first_line.startswith('#[') and len(lines) > 1:
        first_line = lines[1].strip()
    print(f"Item {i}: {first_line[:80]}")

