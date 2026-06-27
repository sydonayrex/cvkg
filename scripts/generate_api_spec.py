#!/usr/bin/env python3
"""Extract public component names, constructor fields, and types from cvkg-components."""

import os
import re
import json

COMPONENTS_DIR = "cvkg-components/src"
OUTPUT_FILE = "docs/api-spec.json"

def extract_structs(filepath):
    """Extract pub struct definitions with their fields."""
    with open(filepath) as f:
        content = f.read()
    
    results = []
    lines = content.split('\n')
    
    for i, line in enumerate(lines):
        stripped = line.strip()
        # Match pub struct Name { or pub struct Name<T> {
        m = re.match(r'pub struct (\w+)', stripped)
        if not m:
            continue
        
        struct_name = m.group(1)
        
        # Skip internal/modifier/state types
        if struct_name.endswith(('Modifier', 'State', 'Visual', 'Engine', 'Builder')):
            continue
        
        # Check if next line is `{` (open struct body) or has generics
        # Collect fields
        fields = []
        for j in range(i + 1, min(i + 30, len(lines))):
            field_line = lines[j].strip()
            if field_line == '}' or field_line == '};':
                break
            if field_line.startswith('pub struct') or field_line.startswith('pub enum'):
                break
            if not field_line.startswith('pub ') or field_line.startswith('pub fn'):
                continue
            
            # Extract pub field_name: Type,
            fm = re.match(r'pub (\w+)\s*:\s*(.+?)(?:,\s*$|$)', field_line)
            if fm:
                field_name = fm.group(1)
                field_type = fm.group(2).strip().rstrip(',').strip()
                fields.append({"name": field_name, "type": field_type})
        
        if fields:
            results.append({
                "name": struct_name,
                "file": os.path.basename(filepath),
                "fields": fields
            })
    
    return results

def main():
    all_components = []
    
    for root, dirs, files in os.walk(COMPONENTS_DIR):
        # Skip hidden and test directories
        dirs[:] = [d for d in dirs if not d.startswith('.')]
        for f in files:
            if not f.endswith('.rs'):
                continue
            filepath = os.path.join(root, f)
            components = extract_structs(filepath)
            all_components.extend(components)
    
    # Sort by name for consistency
    all_components.sort(key=lambda c: c["name"])
    
    spec = {
        "generated_at": "2024-06-24",
        "source": "cvkg-components/src",
        "component_count": len(all_components),
        "components": all_components
    }
    
    os.makedirs(os.path.dirname(OUTPUT_FILE), exist_ok=True)
    with open(OUTPUT_FILE, 'w') as f:
        json.dump(spec, f, indent=2)
    
    print(f"Generated {OUTPUT_FILE} with {len(all_components)} components")

if __name__ == "__main__":
    main()
