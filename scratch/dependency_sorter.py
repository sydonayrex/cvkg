import os
import re

WORKSPACE_ROOT = "/D/rex/projects/cvkg"

def parse_cargo_toml(path):
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    
    package_name_match = re.search(r'^name\s*=\s*"([^"]+)"', content, re.MULTILINE)
    if not package_name_match:
        return None
    name = package_name_match.group(1)
    
    publish_match = re.search(r'^publish\s*=\s*false', content, re.MULTILINE)
    is_publishable = not bool(publish_match)
    
    dependencies = set()
    
    # Parse dependencies, build-dependencies, and dev-dependencies
    dep_tables = re.findall(r'\[(?:dependencies|build-dependencies|dev-dependencies)\.([a-zA-Z0-9_-]+)\](.*?)(?=\[|$)', content, re.DOTALL)
    for dep_name, block in dep_tables:
        actual_name = dep_name
        pkg_match = re.search(r'package\s*=\s*"([^"]+)"', block)
        if pkg_match:
            actual_name = pkg_match.group(1)
        dependencies.add(actual_name)

    inline_sections = re.findall(r'\[(?:dependencies|build-dependencies|dev-dependencies)\](.*?)(?=\[|$)', content, re.DOTALL)
    for section in inline_sections:
        for line in section.splitlines():
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            match = re.match(r'^([a-zA-Z0-9_-]+)\s*=', line)
            if match:
                dep_name = match.group(1)
                actual_name = dep_name
                pkg_match = re.search(r'package\s*=\s*"([^"]+)"', line)
                if pkg_match:
                    actual_name = pkg_match.group(1)
                dependencies.add(actual_name)
                        
    return {
        "name": name,
        "publishable": is_publishable,
        "dependencies": list(dependencies)
    }

def main():
    # Find all cargo toml files
    crates = {}
    for root, dirs, files in os.walk(WORKSPACE_ROOT):
        dirs[:] = [d for d in dirs if d not in ("target", ".git", ".github") and not d.startswith(".")]
        if "Cargo.toml" in files:
            path = os.path.join(root, "Cargo.toml")
            if path == os.path.join(WORKSPACE_ROOT, "Cargo.toml"):
                continue
            res = parse_cargo_toml(path)
            if res:
                crates[res["name"]] = res

    # Filter out demos or non-publishable ones
    publishable = {name: c for name, c in crates.items() if c["publishable"] and not name.endswith("-demo") and not "demo" in name and not name in ["showcase", "berserker", "refactor_script"]}

    # Filter dependencies to only include publishable local crates
    for name, info in publishable.items():
        info["dependencies"] = [d for d in info["dependencies"] if d in publishable and d != name]

    print("--- RAW DEPENDENCIES (WITHOUT DEV-DEPS) ---")
    for name, info in sorted(publishable.items()):
        print(f"{name:<20} -> {info['dependencies']}")
    print("-------------------------------------------")

    visited = {}
    order = []
    
    def dfs(node, path=[]):
        if node in visited:
            if visited[node] == 1:
                cycle_path = path[path.index(node):] + [node]
                raise ValueError(f"Cycle detected: {' -> '.join(cycle_path)}")
            return
        
        visited[node] = 1
        for dep in publishable[node]["dependencies"]:
            dfs(dep, path + [node])
        visited[node] = 2
        order.append(node)

    for node in publishable:
        if node not in visited:
            dfs(node)
            
    print("Correct Publish Order:")
    for idx, name in enumerate(order, 1):
        info = publishable[name]
        print(f"{idx:02d}. {name:<20} -> deps: {info['dependencies']}")

if __name__ == "__main__":
    main()
