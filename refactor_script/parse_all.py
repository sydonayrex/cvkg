import re

with open('../cvkg-render-gpu/src/lib.rs', 'r') as f:
    text = f.read()

def get_top_level_items(text):
    items = []
    brace_level = 0
    in_string = False
    string_char = None
    start_idx = 0
    i = 0
    while i < len(text):
        char = text[i]
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
            elif char == '/' and i + 1 < len(text) and text[i+1] == '/':
                while i < len(text) and text[i] != '\n':
                    i += 1
                continue
            elif char == '{':
                brace_level += 1
            elif char == '}':
                brace_level -= 1
                if brace_level == 0:
                    items.append(text[start_idx:i+1])
                    start_idx = i+1
            elif char == ';' and brace_level == 0:
                items.append(text[start_idx:i+1])
                start_idx = i+1
        i += 1
    if start_idx < len(text):
        items.append(text[start_idx:])
    return items

items = get_top_level_items(text)
for idx, item in enumerate(items):
    lines = item.strip().split('\n')
    if not lines or not lines[0]: continue
    first = lines[0].strip()
    if first.startswith('#['):
        if len(lines) > 1:
            first = lines[1].strip()
    # print up to brace
    short = first.split('{')[0].strip()
    print(f"{idx}: {short[:80]}")

