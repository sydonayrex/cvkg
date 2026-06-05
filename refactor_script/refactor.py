import re
import os

with open('../cvkg-render-gpu/src/lib.rs', 'r') as f:
    content = f.read()
    
# We want to identify the boundaries of methods inside `impl SurtrRenderer`
# Since rustfmt is standard, we can match `    pub fn ` or `    fn ` at exactly 4 spaces.
def find_functions(text):
    funcs = []
    # match lines that look like function declarations at exactly 4 spaces indentation
    for m in re.finditer(r'^    (pub async fn |pub fn |fn |async fn )([a-zA-Z0-9_]+)', text, re.MULTILINE):
        start_idx = m.start()
        name = m.group(2)
        # Find the matching closing brace for this function
        brace_level = 0
        in_string = False
        string_char = None
        started = False
        end_idx = -1
        i = start_idx
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
                    started = True
                elif char == '}':
                    brace_level -= 1
                    if started and brace_level == 0:
                        end_idx = i + 1
                        break
            i += 1
        
        # Also grab preceding doc comments
        # look backwards for `    ///` or `    //`
        search_start = start_idx
        while True:
            # find previous newline
            prev_nl = text.rfind('\n', 0, search_start - 1)
            if prev_nl == -1:
                prev_nl = 0
            line = text[prev_nl:search_start]
            if line.strip() == '' or line.strip().startswith('///') or line.strip().startswith('//') or line.strip().startswith('#['):
                search_start = prev_nl + 1 if prev_nl > 0 else 0
            else:
                break
                
        if end_idx != -1:
            funcs.append({
                'name': name,
                'start': search_start,
                'end': end_idx,
                'code': text[search_start:end_idx] + '\n'
            })
    return funcs

funcs = find_functions(content)
for f in funcs:
    print(f['name'])

