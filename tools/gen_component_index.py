#!/usr/bin/env python3
"""Generate COMPONENTS.md — a component index for cvkg-components.

Scans every .rs file under cvkg-components/src/, extracts `pub struct` definitions
with their doc comments, and emits a Markdown table sorted by English alias name
(when one exists) or original struct name.

Usage:
    cd /D/rex/projects/cvkg
    python tools/gen_component_index.py > COMPONENTS.md
"""

import os
import re
import sys

COMPONENTS_SRC = os.path.join(os.path.dirname(__file__), "..", "cvkg-components", "src")
LIB_RS = os.path.join(COMPONENTS_SRC, "lib.rs")

# ── Known English aliases (from cvkg-components/src/lib.rs English API Aliases section) ──
# These are the type aliases defined in lib.rs, mapped: EnglishName → NorseName
ENGLISH_ALIASES: dict[str, str] = {}

def parse_english_aliases():
    """Read the English API Aliases section from lib.rs and module-level pub use as patterns."""
    if not os.path.isfile(LIB_RS):
        return
    with open(LIB_RS, encoding="utf-8") as f:
        content = f.read()
    # Match lines like: pub type Tabs = BifrostTabs;
    # Or: pub type Dialog = GeriDialog<cvkg_core::AnyView>;
    pattern = re.compile(r'pub type (\w+)\s*=\s*(\w+)')
    for m in pattern.finditer(content):
        eng, norse = m.group(1), m.group(2)
        ENGLISH_ALIASES[norse] = eng  # Norse → English
    
    # Also scan all .rs files for `pub use X as Y` module-level aliases
    for root, dirs, files in os.walk(COMPONENTS_SRC):
        dirs[:] = [d for d in dirs if not d.startswith("_") and d not in ("__pycache__",)]
        for fname in files:
            if not fname.endswith(".rs"):
                continue
            fpath = os.path.join(root, fname)
            with open(fpath, encoding="utf-8") as f:
                file_content = f.read()
            # Match lines like: pub use BifrostTabs as Tabs;
            pub_use_pattern = re.compile(r'pub use (\w+)\s+as\s+(\w+)')
            for m in pub_use_pattern.finditer(file_content):
                norse, eng = m.group(1), m.group(2)
                ENGLISH_ALIASES[norse] = eng  # Norse → English

def extract_components(filepath: str, rel_path: str) -> list[dict]:
    """Extract component definitions from a .rs file."""
    components = []
    with open(filepath, encoding="utf-8") as f:
        content = f.read()

    # Remove block comments to avoid false positives
    stripped = re.sub(r'/\*.*?\*/', '', content, flags=re.DOTALL)

    # Find all doc-comment + pub struct pairs
    # Pattern: optional doc comments, then pub struct Name
    pattern = re.compile(
        r'(?P<doc>(?:///[^\n]*\n\s*)*)pub\s+struct\s+(?P<name>\w+)',
    )
    for m in pattern.finditer(stripped):
        name = m.group("name")
        doc_raw = m.group("doc") or ""
        # Extract first meaningful sentence from doc
        doc_lines = []
        for line in doc_raw.split("\n"):
            line = line.strip()
            line = re.sub(r'^///\s?', '', line)
            if line and not line.startswith("#") and not line.startswith("//!"):
                doc_lines.append(line)
        first_sentence = ""
        for line in doc_lines:
            first_sentence += " " + line
            first_sentence = first_sentence.strip()
            # Take up to the first period or newline
            if "." in first_sentence:
                idx = first_sentence.index(".")
                first_sentence = first_sentence[:idx+1]
                break
        if not first_sentence and doc_lines:
            first_sentence = doc_lines[0][:80]

        # Skip internal / hidden types
        if name.startswith("_") or "Never" in name or name in ("Color", "FontWeight",
           "ButtonVariant", "ButtonSize", "InputState", "CheckboxState"):
            continue

        eng_name = ENGLISH_ALIASES.get(name, name)
        module = rel_path.replace("\\", "/").replace(".rs", "")
        if module.endswith("/mod"):
            module = module[:-4]

        components.append({
            "norse": name,
            "english": eng_name if eng_name != name else "",
            "display": eng_name,  # sort key
            "doc": first_sentence[:120] if first_sentence else "",
            "module": module,
        })

    return components


def main():
    parse_english_aliases()

    all_components = []

    for root, dirs, files in os.walk(COMPONENTS_SRC):
        # Skip subdirectories that are not module roots
        dirs[:] = [d for d in dirs if not d.startswith("_") and d not in ("__pycache__",)]
        for fname in sorted(files):
            if not fname.endswith(".rs"):
                continue
            fpath = os.path.join(root, fname)
            rel = os.path.relpath(fpath, COMPONENTS_SRC)
            components = extract_components(fpath, rel)
            all_components.extend(components)

    # Sort by display name (English alias if it exists, otherwise Norse)
    def sort_key(c):
        name = c["english"] or c["norse"]
        return name.lower()

    all_components.sort(key=sort_key)

    # Emit
    print("# CVKG Component Index")
    print()
    print(f"*Auto-generated by `tools/gen_component_index.py`. Total: {len(all_components)} components.*")
    print()
    print("| Component | Description | Module |")
    print("|-----------|-------------|--------|")
    for c in all_components:
        name = c["english"] if c["english"] else c["norse"]
        doc = c["doc"] or "*no description*"
        print(f"| `{name}` | {doc} | `{c['module']}` |")


if __name__ == "__main__":
    main()
