import re

with open('../cvkg-render-gpu/src/lib.rs', 'r') as f:
    lines = f.readlines()
    
# We want to keep everything from line 1 to 157, minus the huge `mod tests` block?
# Wait, `mod tests` is lines 41 to 105.
# Let's just keep everything that was NOT assigned to a module!

with open('map.txt', 'r') as f:
    map_lines = f.readlines()

assigned_ranges = []
for line in map_lines:
    parts = line.strip().split('|')
    start = 0
    end = 0
    if len(parts) == 4:
        start, end = int(parts[2]), int(parts[3])
    elif len(parts) == 5:
        start, end = int(parts[3]), int(parts[4])
        
    # Exclude mods from being deleted
    if parts[0] == 'MOD':
        continue
    # Exclude OTHER
    if parts[0] == 'OTHER':
        continue
        
    assigned_ranges.append((start, end))
    
# Sort and merge ranges
assigned_ranges.sort()
merged = []
for r in assigned_ranges:
    if not merged:
        merged.append(r)
    else:
        last = merged[-1]
        if r[0] <= last[1] + 1: # overlap or adjacent
            merged[-1] = (last[0], max(last[1], r[1]))
        else:
            merged.append(r)

# Write out the non-deleted parts
with open('../cvkg-render-gpu/src/lib.new.rs', 'w') as f:
    f.write("pub mod types;\n")
    f.write("pub mod vertex;\n")
    f.write("pub mod renderer;\n")
    f.write("pub mod setup;\n")
    f.write("pub mod passes;\n")
    f.write("pub mod draw;\n")
    f.write("pub mod api;\n\n")
    
    current_line = 1
    for (start, end) in merged:
        if current_line < start:
            f.writelines(lines[current_line-1:start-1])
        current_line = end + 1
        
    if current_line <= len(lines):
        f.writelines(lines[current_line-1:])

