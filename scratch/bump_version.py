import os

WORKSPACE_ROOT = "/D/rex/projects/cvkg"

def bump_versions():
    count = 0
    for root, dirs, files in os.walk(WORKSPACE_ROOT):
        # Skip target and hidden directories
        dirs[:] = [d for d in dirs if d not in ("target", ".git", ".github") and not d.startswith(".")]
        
        if "Cargo.toml" in files:
            path = os.path.join(root, "Cargo.toml")
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()
            
            if "0.2.8" in content:
                # Replace exact string "0.2.8" with "0.2.9"
                new_content = content.replace("0.2.8", "0.2.9")
                with open(path, "w", encoding="utf-8") as f:
                    f.write(new_content)
                count += 1
                print(f"Bumped version in {os.path.relpath(path, WORKSPACE_ROOT)}")
                
    print(f"Bumped 0.2.8 to 0.2.9 in {count} Cargo.toml files.")

if __name__ == "__main__":
    bump_versions()
