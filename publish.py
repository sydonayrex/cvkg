import json
import subprocess
import sys

def get_publish_order():
    out = subprocess.check_output(['cargo', 'metadata', '--format-version', '1', '--no-deps'])
    metadata = json.loads(out)
    
    workspace_members = [p for p in metadata['packages'] if p['id'] in metadata['workspace_members']]
    
    # Check if publish is an empty list (meaning publish = false)
    publishable = [p for p in workspace_members if p.get('publish') != []]
    
    # Filter out demos or binaries that are not meant to be published if any
    publishable = [p for p in publishable if "demo" not in p['name'] and not p['name'].endswith("-demo")]
    
    graph = {p['name']: set() for p in publishable}
    for p in publishable:
        for dep in p['dependencies']:
            if dep['name'] in graph:
                graph[p['name']].add(dep['name'])
                
    order = []
    visited = set()
    def visit(name):
        if name in visited: return
        for dep in graph[name]:
            visit(dep)
        visited.add(name)
        order.append(name)
        
    for p in publishable:
        visit(p['name'])
        
    return order

order = get_publish_order()
print("Topological order:", order)

print("--- DRY RUN ---")
for pkg in order:
    print(f"cargo publish --dry-run -p {pkg} --allow-dirty")
    res = subprocess.run(['cargo', 'publish', '--dry-run', '-p', pkg, '--allow-dirty'])
    if res.returncode != 0:
        print(f"Dry run failed for {pkg}")
        sys.exit(1)

print("--- ACTUAL PUBLISH ---")
for pkg in order:
    print(f"cargo publish -p {pkg} --allow-dirty")
    res = subprocess.run(['cargo', 'publish', '-p', pkg, '--allow-dirty'])
    if res.returncode != 0:
        print(f"Publish failed for {pkg}")
        sys.exit(1)
