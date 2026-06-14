import json
import subprocess
import sys

def main():
    try:
        output = subprocess.check_output(['cargo', 'metadata', '--format-version', '1', '--no-deps'], universal_newlines=True)
    except subprocess.CalledProcessError as e:
        print("Failed to run cargo metadata", e)
        sys.exit(1)

    metadata = json.loads(output)
    packages = {pkg['id']: pkg for pkg in metadata['packages']}
    workspace_members = metadata['workspace_members']
    
    # Filter only publishable workspace members
    publishable = []
    for pid in workspace_members:
        pkg = packages[pid]
        # publish can be None (default, meaning publishable) or [] (unpublishable)
        if pkg.get('publish') is not None and len(pkg['publish']) == 0:
            continue
        publishable.append(pid)

    # Build dependency graph
    graph = {pid: set() for pid in publishable}
    for pid in publishable:
        pkg = packages[pid]
        for dep in pkg['dependencies']:
            # We need to find if this dependency is a workspace member
            # Dependency resolution can be tricky, let's just match by name
            dep_name = dep['name']
            for other_pid in publishable:
                if packages[other_pid]['name'] == dep_name:
                    graph[pid].add(other_pid)
                    break

    # Topological sort
    ordered = []
    visited = set()
    temp_mark = set()

    def visit(n):
        if n in temp_mark:
            raise Exception("Cycle detected!")
        if n not in visited:
            temp_mark.add(n)
            for m in graph[n]:
                visit(m)
            temp_mark.remove(n)
            visited.add(n)
            ordered.append(n)

    for pid in publishable:
        visit(pid)

    print("Publishing order:")
    for pid in ordered:
        pkg = packages[pid]
        print(f"  {pkg['name']} (version {pkg['version']})")
        
    print("\nRunning dry-run...")
    for pid in ordered:
        pkg = packages[pid]
        pkg_name = pkg['name']
        print(f"\n--- DRY RUN: {pkg_name} ---")
        res = subprocess.run(['cargo', 'publish', '--dry-run', '-p', pkg_name])
        if res.returncode != 0:
            print(f"Dry run failed for {pkg_name}!")
            sys.exit(1)
            
    print("\nAll dry-runs passed! Ready to publish.")
    sys.exit(0)

if __name__ == '__main__':
    main()
