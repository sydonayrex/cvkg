#!/usr/bin/env python3
"""Fail when a public symbol has no production callers.

Read docs/quality/wiring-allowlist.txt for the symbols to check and
docs/quality/wiring-baseline.txt for the approved zero-caller set.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ALLOWLIST = ROOT / "docs/quality/wiring-allowlist.txt"
BASELINE = ROOT / "docs/quality/wiring-baseline.txt"
SKIP_DIRS = {"target", ".git", "old", "pkg", "node_modules", ".hermes"}


def read_symbols(path: Path) -> list[str]:
    if not path.exists():
        return []
    return [line.strip() for line in path.read_text().splitlines()
            if line.strip() and not line.startswith("#")]


def source_files() -> list[Path]:
    files = []
    for child in ROOT.iterdir():
        if not child.is_dir() or child.name in SKIP_DIRS:
            continue
        if not (child.name.startswith("cvkg") or child.name == "demos"):
            continue
        for path in child.rglob("*.rs"):
            if "src" not in path.parts:
                continue
            if not any(part in SKIP_DIRS for part in path.parts):
                files.append(path)
    return files


def main() -> int:
    symbols = read_symbols(ALLOWLIST)
    baseline = set(read_symbols(BASELINE))
    texts = [path.read_text(errors="ignore") for path in source_files()]
    found = {}
    for symbol in symbols:
        pattern = re.compile(r"\b" + re.escape(symbol) + r"\b")
        found[symbol] = sum(len(pattern.findall(text)) for text in texts)

    zero = {name for name, count in found.items() if count <= 1}
    unexpected = sorted(zero - baseline)
    resolved = sorted(baseline - zero)

    for name in sorted(found):
        print(f"{name}: {found[name]} occurrences")
    if resolved:
        print("RESOLVED baseline entries, remove them from the baseline:")
        for name in resolved:
            print(f"  {name}")
    if unexpected:
        print("NEW zero-caller symbols:")
        for name in unexpected:
            print(f"  {name}")
        return 1
    print(f"OK {len(baseline)} baseline symbols")
    return 0


if __name__ == "__main__":
    sys.exit(main())
