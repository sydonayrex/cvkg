import os
import re

def bump_versions():
    old_ver = "0.1.12"
    new_ver = "0.1.13"
    
    files_to_update = []
    for root, dirs, files in os.walk("."):
        if "target" in root:
            continue
        for file in files:
            if file == "Cargo.toml" or file == "README.md" or file == "cvkg-prp.md":
                files_to_update.append(os.path.join(root, file))

    for file_path in files_to_update:
        with open(file_path, "r", encoding="utf-8") as f:
            content = f.read()
        
        # In Cargo.toml, only replace version = "0.1.11" or cvkg-* = { version = "0.1.11" ... }
        if file_path.endswith("Cargo.toml"):
            new_content = content.replace(f'version = "{old_ver}"', f'version = "{new_ver}"')
        else:
            new_content = content.replace(old_ver, new_ver)
            
        if new_content != content:
            print(f"Updating {file_path}")
            with open(file_path, "w", encoding="utf-8") as f:
                f.write(new_content)

if __name__ == "__main__":
    bump_versions()
