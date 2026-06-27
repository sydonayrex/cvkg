#!/usr/bin/env python3
"""Cargo publish dry run to determine publish order for CVKG crates."""

import subprocess
import sys
import os
import json
from pathlib import Path

WORKSPACE_ROOT = Path("/D/rex/projects/cvkg")

# Get all workspace members
result = subprocess.run(
    ["cargo", "metadata", "--format-version=1", "--no-deps"],
    capture_output=True, text=True, cwd=WORKSPACE_ROOT
)
if result.returncode != 0:
    print(f"ERROR: cargo metadata failed: {result.stderr}")
    sys.exit(1)

metadata = json.loads(result.stdout)

# Extract package info
packages = {}
for pkg in metadata.get("packages", []):
    name = pkg["name"]
    version = pkg["version"]
    # Use manifest_path to get the crate directory
    manifest = pkg["manifest_path"]
    path = os.path.dirname(manifest)
    deps = []
    for dep in pkg.get("dependencies", []):
        if dep.get("kind") is None or dep.get("kind") == "normal":
            deps.append(dep["name"])
    packages[name] = {
        "version": version,
        "path": path,
        "deps": deps,
    }

print(f"=== Found {len(packages)} packages ===\n")
for name, info in sorted(packages.items()):
    print(f"  {name} v{info['version']} ({len(info['deps'])} deps)")

# Determine publish order using topological sort
def topological_sort(packages):
    """Sort packages by dependency order (leaves first)."""
    # Filter to only workspace packages
    workspace_names = set(packages.keys())
    
    # Build adjacency list (only workspace deps)
    adj = {name: [] for name in packages}
    in_degree = {name: 0 for name in packages}
    
    for name, info in packages.items():
        for dep in info["deps"]:
            if dep in workspace_names:
                adj[dep].append(name)  # dep -> dependent
                in_degree[name] += 1
    
    # Kahn's algorithm
    queue = sorted([name for name, deg in in_degree.items() if deg == 0])
    order = []
    
    while queue:
        node = queue.pop(0)
        order.append(node)
        for dependent in sorted(adj[node]):
            in_degree[dependent] -= 1
            if in_degree[dependent] == 0:
                queue.append(dependent)
        queue.sort()
    
    if len(order) != len(packages):
        # Circular dependency detected
        remaining = set(packages.keys()) - set(order)
        print(f"\nWARNING: Circular dependency detected among: {remaining}")
        # Add remaining with a warning marker
        for name in sorted(remaining):
            order.append(name)
    
    return order

order = topological_sort(packages)

print(f"\n=== Publish Order (topological) ===\n")
for i, name in enumerate(order, 1):
    info = packages[name]
    workspace_deps = [d for d in info["deps"] if d in packages]
    print(f"  {i:2d}. {name:35s} v{info['version']:10s} (workspace deps: {', '.join(workspace_deps) or 'none'})")

# Now run dry-run publish for each crate
print(f"\n=== Running cargo publish --dry-run ===\n")

results = {}
for i, name in enumerate(order, 1):
    info = packages[name]
    pkg_path = Path(info["path"])
    
    print(f"  [{i}/{len(order)}] {name}...", end=" ", flush=True)
    
    result = subprocess.run(
        ["cargo", "publish", "--dry-run", "--allow-dirty"],
        capture_output=True, text=True, cwd=pkg_path
    )
    
    if result.returncode == 0:
        print("OK")
        results[name] = "OK"
    else:
        # Check for specific errors
        stderr = result.stderr
        if "no matching package" in stderr.lower():
            print(f"FAILED (dependency not found)")
            results[name] = "DEP_MISSING"
        elif "circular" in stderr.lower():
            print(f"FAILED (circular dependency)")
            results[name] = "CIRCULAR"
        else:
            print(f"FAILED")
            results[name] = "ERROR"
        # Print first few lines of error
        for line in stderr.split('\n')[:5]:
            if line.strip():
                print(f"         {line.strip()}")

print(f"\n=== Summary ===\n")
ok = sum(1 for v in results.values() if v == "OK")
failed = sum(1 for v in results.values() if v != "OK")
print(f"  OK: {ok}/{len(order)}")
print(f"  Failed: {failed}/{len(order)}")

if failed > 0:
    print(f"\n=== Failed Crates ===\n")
    for name, status in results.items():
        if status != "OK":
            print(f"  {name}: {status}")

# Output the order for the next step
print(f"\n=== Publish Command ===\n")
print("crates_to_publish = [")
for name in order:
    info = packages[name]
    path = info["path"]
    print(f'    {{"name": "{name}", "path": "{path}"}},')
print("]")
