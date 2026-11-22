#!/usr/bin/env python3
"""Fail when a workspace path dependency pin differs from the workspace version."""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WORKSPACE = ROOT / "Cargo.toml"


def workspace_version() -> str:
    text = WORKSPACE.read_text()
    section = text.split("[workspace.package]", 1)[1].split("\n[", 1)[0]
    match = re.search(r'^version\s*=\s*"([^"]+)"', section, re.M)
    if not match:
        raise SystemExit("missing [workspace.package] version")
    return match.group(1)


def main() -> int:
    expected = workspace_version()
    bad = []
    for manifest in sorted(ROOT.glob("**/Cargo.toml")):
        if "target" in manifest.parts:
            continue
        text = manifest.read_text()
        for line in text.splitlines():
            if "cvkg" not in line or "path" not in line:
                continue
            match = re.search(r'version\s*=\s*"([^"]+)"', line)
            if match and match.group(1) != expected:
                bad.append((manifest.relative_to(ROOT), match.group(1), line.strip()))
    for manifest, version, line in bad:
        print(f"{manifest}: expected {expected}, found {version}: {line}")
    if bad:
        return 1
    print(f"OK all workspace path pins equal {expected}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
