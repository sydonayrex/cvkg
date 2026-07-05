#!/usr/bin/env python3
"""
CVKG Workspace Publisher Script
Determines the dependency order of workspace crates and publishes them sequentially.
Handles relaxing version constraints in dev-dependencies during the upload phase.
"""

import os
import re
import sys
import time
import argparse
import subprocess
from datetime import datetime, timezone

def extract_deps(content):
    parts = re.split(r'^(\[.*\])\s*$', content, flags=re.MULTILINE)
    deps = set()
    current_section = "[dependencies]"

    for part in parts:
        if not part:
            continue
        if part.startswith('['):
            current_section = part.strip()
            sec_match = re.match(r'^\[(?:dependencies|build-dependencies)\.(cvkg(?:-[a-z0-9-]+)?)(?:\..+)?\]', current_section)
            if sec_match:
                deps.add(sec_match.group(1))
        else:
            if 'dev-dependencies' in current_section:
                continue
            potential = re.findall(r'^\s*(cvkg(?:-[a-z0-9-]+)?)(?:\.[a-z0-9-]+)?\s*=', part, re.MULTILINE)
            for d in potential:
                deps.add(d)

    return deps

def get_workspace_crates(root_dir):
    crates = {}
    all_workspace_packages = set()
    for root, dirs, files in os.walk(root_dir):
        dirs[:] = [d for d in dirs if d not in ['.git', 'target', 'node_modules', 'demos', '.gemini', '.agents']]
        if 'Cargo.toml' in files and root != root_dir:
            cargo_path = os.path.join(root, 'Cargo.toml')
            try:
                with open(cargo_path, 'r', encoding='utf-8') as f:
                    content = f.read()

                name_match = re.search(r'^\s*name\s*=\s*"([^"]+)"', content, re.MULTILINE)
                if name_match:
                    crate_name = name_match.group(1)
                    all_workspace_packages.add(crate_name)

                    publish_match = re.search(r'^\s*publish\s*=\s*false', content, re.MULTILINE)
                    if publish_match:
                        continue

                    deps = extract_deps(content)
                    crates[crate_name] = {
                        'path': root,
                        'deps': deps
                    }
            except Exception as e:
                print(f"Warning: Could not parse {cargo_path}: {e}", file=sys.stderr)

    private_crates = all_workspace_packages - set(crates.keys())
    return crates, private_crates

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

def prepare_for_publish(cargo_path, workspace_packages, private_crates):
    """Relax version constraints and strip private dev-dependencies."""
    with open(cargo_path, 'r', encoding='utf-8') as f:
        content = f.read()

    backup_path = cargo_path + ".bak"
    with open(backup_path, 'w', encoding='utf-8') as f:
        f.write(content)

    lines = content.splitlines()
    new_lines = []
    in_dev_deps = False

    for line in lines:
        stripped = line.strip()

        if stripped.startswith('[dev-dependencies'):
            in_dev_deps = True
            new_lines.append(line)
        elif stripped.startswith('[') and in_dev_deps:
            in_dev_deps = False
            new_lines.append(line)
        elif in_dev_deps:
            # Skip private crate dev-deps entirely
            is_private = any(
                re.search(rf'^\s*{re.escape(p)}(?:\.[a-z0-9-]+)?\s*=', stripped)
                for p in private_crates
            )
            if is_private:
                continue

            # Relax version for workspace packages
            pkg_names = '|'.join(re.escape(p) for p in workspace_packages)
            relaxed = re.sub(
                rf'^(\s*(?:{pkg_names})(?:[.\-][a-z0-9-]+)?)(\s*=\s*)".*"$',
                r'\1\2">=0.3.1"',
                line
            )
            new_lines.append(relaxed)
        else:
            new_lines.append(line)

    with open(cargo_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(new_lines))

def restore_cargo_toml(cargo_path):
    backup_path = cargo_path + ".bak"
    if os.path.exists(backup_path):
        if os.path.exists(cargo_path):
            os.remove(cargo_path)
        os.rename(backup_path, cargo_path)

def main():
    parser = argparse.ArgumentParser(description="Publish CVKG workspace crates in dependency order.")
    parser.add_argument("--dry-run", action="store_true", help="Perform a dry-run publish.")
    parser.add_argument("--no-verify", action="store_true", help="Do not verify contents of the package (useful for local dry-runs).")
    parser.add_argument("--token", type=str, help="API token to use when publishing.")
    parser.add_argument("--yes", action="store_true", help="Skip confirmation prompt.")

    args = parser.parse_args()
    workspace_root = os.path.abspath(os.path.dirname(__file__))

    try:
        crates, private_crates = get_workspace_crates(workspace_root)
        order = topological_sort(crates)
    except Exception as e:
        print(f"Error sorting workspace: {e}", file=sys.stderr)
        sys.exit(1)

    print("Resolved Publishing Order:")
    for i, name in enumerate(order, 1):
        print(f"  {i:2d}. {name}")

    print(f"\nPrivate / Non-publishable Crates: {sorted(private_crates)}")
    print("  (These will be stripped from dev-dependencies)")

    cmd_base = ["cargo", "publish", "--no-verify", "--allow-dirty"]
    if args.dry_run:
        cmd_base.append("--dry-run")
    if args.token:
        cmd_base.extend(["--token", args.token])

    print(f"\nCommand: {' '.join(cmd_base)}")

    if not args.yes:
        confirm = input("Proceed? [y/N]: ").strip().lower()
        if confirm != 'y':
            print("Aborted.")
            sys.exit(0)

    for name in order:
        crate_info = crates[name]
        path = crate_info['path']
        cargo_path = os.path.join(path, "Cargo.toml")

        print(f"\nPublishing {name} from {path}...")

        prepare_for_publish(cargo_path, set(crates.keys()), private_crates)

        try:
            max_retries = 5
            for attempt in range(max_retries):
                res = subprocess.run(cmd_base, cwd=path, capture_output=True, text=True)
                if res.stdout:
                    print(res.stdout)
                if res.stderr:
                    print(res.stderr, file=sys.stderr)

                if res.returncode == 0:
                    print(f"  -> {name} published successfully")
                    break

                combined = res.stdout + res.stderr
                if any(x in combined for x in ["already uploaded", "already published", "already exists"]):
                    print(f"  -> {name} already published, skipping.")
                    break

                rate_limit_match = re.search(
                    r'try again after ([A-Za-z]+,\s+\d+\s+[A-Za-z]+\s+\d+\s+[\d:]+\s+GMT)',
                    combined
                )
                if rate_limit_match and attempt < max_retries - 1:
                    retry_str = rate_limit_match.group(1)
                    try:
                        retry_dt = datetime.strptime(retry_str, "%a, %d %b %Y %H:%M:%S GMT").replace(tzinfo=timezone.utc)
                        now = datetime.now(timezone.utc)
                        wait = max(0, (retry_dt - now).total_seconds()) + 5
                    except ValueError:
                        wait = 30
                    print(f"  Rate-limited. Retrying in {wait:.0f}s... ({attempt + 2}/{max_retries})")
                    time.sleep(wait)
                    continue

                print(f"Error: Failed to publish {name}", file=sys.stderr)
                sys.exit(res.returncode)
        finally:
            restore_cargo_toml(cargo_path)

    print("\nAll crates processed successfully!")

if __name__ == "__main__":
    main()