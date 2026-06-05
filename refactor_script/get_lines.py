import re

with open('../cvkg-render-gpu/src/lib.rs', 'r') as f:
    text = f.read()

# simple block matching:
# we look for `pub struct`, `struct`, `impl`, `pub fn`, `fn`
# we find the next `{`, then we find the matching `}`.
# we record the line numbers.

def find_blocks():
    blocks = []
    # match lines that start with these keywords (allowing indentation)
    pattern = re.compile(r'^(?: {0,4})(pub struct|struct|impl|pub fn|fn|pub async fn|async fn)\s+([^{]+)\{', re.MULTILINE)
    for m in pattern.finditer(text):
        start_idx = m.start()
        header = m.group(0)
        
        # find matching brace
        brace_level = 1
        i = m.end()
        in_string = False
        string_char = None
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
                        break
            i += 1
            
        end_idx = i + 1
        
        # include doc comments
        search_start = start_idx
        while True:
            prev_nl = text.rfind('\n', 0, search_start - 1)
            if prev_nl == -1: prev_nl = 0
            line = text[prev_nl:search_start].strip()
            if line.startswith('///') or line.startswith('//') or line.startswith('#['):
                search_start = prev_nl + 1 if prev_nl > 0 else 0
            elif line == '':
                search_start = prev_nl + 1 if prev_nl > 0 else 0
            else:
                break
                
        start_line = text.count('\n', 0, search_start) + 1
        end_line = text.count('\n', 0, end_idx) + 1
        
        name = m.group(2).strip()
        kind = m.group(1).strip()
        
        blocks.append({
            'kind': kind,
            'name': name,
            'start_line': start_line,
            'end_line': end_line,
            'start_idx': search_start,
            'end_idx': end_idx
        })
    return blocks

blocks = find_blocks()

# Now filter out nested blocks (like functions inside impl)
top_level_blocks = []
for b in blocks:
    is_nested = False
    for other in blocks:
        if b != other and other['start_line'] <= b['start_line'] and other['end_line'] >= b['end_line']:
            # check if it's strictly inside (to avoid identical blocks)
            if other['start_line'] < b['start_line'] or other['end_line'] > b['end_line']:
                is_nested = True
                break
    if not is_nested:
        top_level_blocks.append(b)
        print(f"{b['start_line']}-{b['end_line']}: {b['kind']} {b['name']}")
    else:
        # print nested functions of impl SurtrRenderer
        if b['kind'] in ('pub fn', 'fn', 'pub async fn', 'async fn'):
            print(f"  {b['start_line']}-{b['end_line']}: {b['kind']} {b['name']}")

