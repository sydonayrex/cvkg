#!/usr/bin/env python3
"""
CVKG Workspace Publisher Script
Determines the dependency order of workspace crates and publishes them sequentially.
"""

import os
import re
import sys
import argparse
import subprocess

def extract_deps(content):
    parts = re.split(r'^(\[.*\])\s*$', content, flags=re.MULTILINE)
    deps = set()
    current_section = "[dependencies]"
    
    for part in parts:
        if not part:
            continue
        if part.startswith('['):
            current_section = part.strip()
            # Match dependency table headers e.g., [dependencies.cvkg-core]
            sec_match = re.match(r'^\[(?:dependencies|build-dependencies)\.(cvkg-[a-z0-9-]+)(?:\..+)?\]', current_section)
            if sec_match:
                deps.add(sec_match.group(1))
        else:
            if 'dev-dependencies' in current_section:
                continue
            
            # Match standard dependencies e.g., cvkg-core = { ... } or cvkg-core.workspace = true
            potential = re.findall(r'^\s*(cvkg-[a-z0-9-]+)(?:\.[a-z0-9-]+)?\s*=', part, re.MULTILINE)
            for d in potential:
                deps.add(d)
                
    return deps

def get_workspace_crates(root_dir):
    crates = {}
    for root, dirs, files in os.walk(root_dir):
        # Skip directories we don't want to traverse
        dirs[:] = [d for d in dirs if d not in ['.git', 'target', 'node_modules', 'demos', '.gemini', '.agents']]
        if 'Cargo.toml' in files and root != root_dir:
            cargo_path = os.path.join(root, 'Cargo.toml')
            try:
                with open(cargo_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                # Check if package is publishable
                publish_match = re.search(r'^\s*publish\s*=\s*false', content, re.MULTILINE)
                if publish_match:
                    continue  # Skip private/non-publishable packages
                
                name_match = re.search(r'^\s*name\s*=\s*"([^"]+)"', content, re.MULTILINE)
                if name_match:
                    crate_name = name_match.group(1)
                    deps = extract_deps(content)
                    
                    crates[crate_name] = {
                        'path': root,
                        'deps': deps
                    }
            except Exception as e:
                print(f"Warning: Could not parse {cargo_path}: {e}", file=sys.stderr)
    return crates

def topological_sort(crates):
    in_degree = {name: 0 for name in crates}
    adj = {name: [] for name in crates}
    
    for name, info in crates.items():
        for dep in info['deps']:
            if dep in crates:
                adj[dep].append(name)
                in_degree[name] += 1
                
    queue = [name for name, deg in in_degree.items() if deg == 0]
    order = []
    
    while queue:
        queue.sort()
        curr = queue.pop(0)
        order.append(curr)
        for neighbor in adj[curr]:
            in_degree[neighbor] -= 1
            if in_degree[neighbor] == 0:
                queue.append(neighbor)
                
    if len(order) != len(crates):
        remaining = [name for name, deg in in_degree.items() if deg > 0]
        raise ValueError(f"Cycle detected or unresolvable dependencies in: {remaining}")
        
    return order

def main():
    parser = argparse.ArgumentParser(description="Publish CVKG workspace crates in dependency order.")
    parser.add_argument("--dry-run", action="store_true", help="Perform a dry-run publish.")
    parser.add_argument("--no-verify", action="store_true", help="Do not verify contents of the package (useful for local dry-runs).")
    parser.add_argument("--token", type=str, help="API token to use when publishing.")
    
    args = parser.parse_args()
    workspace_root = os.path.abspath(os.path.dirname(__file__))
    
    try:
        crates = get_workspace_crates(workspace_root)
        order = topological_sort(crates)
    except Exception as e:
        print(f"Error sorting workspace: {e}", file=sys.stderr)
        sys.exit(1)
        
    print("Resolved Publishing Order:")
    for i, name in enumerate(order, 1):
        print(f"  {i:2d}. {name}")
        
    cmd_base = ["cargo", "publish"]
    if args.dry_run:
        cmd_base.append("--dry-run")
        cmd_base.append("--allow-dirty")
    if args.no_verify:
        cmd_base.append("--no-verify")
    if args.token:
        cmd_base.extend(["--token", args.token])
        
    print(f"\nCommand base: {' '.join(cmd_base)}")
    confirm = input("Proceed? [y/N]: ").strip().lower()
    if confirm != 'y':
        print("Aborted.")
        sys.exit(0)
        
    for name in order:
        crate_info = crates[name]
        path = crate_info['path']
        print(f"\nPublishing {name} in {path}...")
        
        res = subprocess.run(cmd_base, cwd=path)
        if res.returncode != 0:
            print(f"Error: Failed to publish {name}", file=sys.stderr)
            sys.exit(res.returncode)
            
    print("\nAll crates processed successfully!")

if __name__ == "__main__":
    main()
