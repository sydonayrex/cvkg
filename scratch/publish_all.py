import os
import subprocess
import time
import re

WORKSPACE_ROOT = "/D/rex/projects/cvkg"

# The correct topological order of ALL crates (ignoring dev-dependencies)
ORDER = [
    "cvkg-runic-text",
    "cvkg-core",
    "cvkg-anim",
    "cvkg-themes",
    "cvkg-scene",
    "cvkg-vdom",
    "cvkg-layout",
    "cvkg-components",
    "cvkg-render-web",
    "cvkg-compositor",
    "cvkg-svg-serialize",
    "cvkg-svg-filters",
    "cvkg-render-gpu",
    "cvkg-macros",
    "cvkg-render-native",
    "cvkg",
    "cvkg-physics",
    "cvkg-cli",
    "cvkg-webkit-server",
    "cvkg-flow"
]

# Crates already published based on previous task logs
ALREADY_PUBLISHED = {
    "cvkg-runic-text",
    "cvkg-core",
    "cvkg-anim",
    "cvkg-themes",
    "cvkg-scene",
    "cvkg-vdom",
    "cvkg-layout",
    "cvkg-macros"
}

def strip_dev_dependencies(toml_path):
    with open(toml_path, "r", encoding="utf-8") as f:
        content = f.read()

    # Remove inline dev-dependencies blocks like [dev-dependencies.cvkg-render-gpu]
    content = re.sub(r'\[dev-dependencies\.[^\]]+\][^\[]*', '', content)
    
    # Remove lines under [dev-dependencies] that reference local crates
    # This is a bit tricky, so we'll just remove the whole [dev-dependencies] section 
    # and anything under it until the next section.
    content = re.sub(r'\[dev-dependencies\][^\[]*', '', content)

    with open(toml_path, "w", encoding="utf-8") as f:
        f.write(content)

def main():
    os.chdir(WORKSPACE_ROOT)
    
    for crate in ORDER:
        if crate in ALREADY_PUBLISHED:
            print(f"Skipping {crate} (already published)")
            continue
            
        print(f"\n======================================")
        print(f"Publishing {crate}...")
        print(f"======================================")
        
        toml_path = os.path.join(WORKSPACE_ROOT, crate, "Cargo.toml")
        if not os.path.exists(toml_path):
            # Special case for 'cvkg' which is at the root
            if crate == "cvkg":
                toml_path = os.path.join(WORKSPACE_ROOT, "Cargo.toml")
            else:
                print(f"Could not find Cargo.toml for {crate}")
                continue
                
        # Strip dev dependencies
        strip_dev_dependencies(toml_path)
        
        max_retries = 5
        success = False
        
        for attempt in range(max_retries):
            # Run cargo publish
            result = subprocess.run(["cargo", "publish", "--allow-dirty", "-p", crate])
            if result.returncode == 0:
                success = True
                break
            else:
                print(f"Publish failed, retrying in 10 seconds... ({attempt+1}/{max_retries})")
                time.sleep(10)
                
        # Restore Cargo.toml
        subprocess.run(["git", "restore", toml_path])
        
        if not success:
            print(f"Failed to publish {crate} after {max_retries} retries.")
            exit(1)
            
        print(f"Published {crate} successfully. Waiting 15s for index propagation...")
        time.sleep(15)
        
    print("All remaining crates published successfully!")

if __name__ == "__main__":
    main()
