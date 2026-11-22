#!/usr/bin/env python3
"""Generate the component source manifest consumed by `cvkg-cli component`."""

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "cvkg-components" / "src"
OUTPUT = ROOT / "cvkg-cli" / "manifest.json"


def main() -> None:
    entries = {}
    for path in sorted(SOURCE.rglob("*.rs")):
        text = path.read_text(errors="ignore")
        names = re.findall(r"^pub struct ([A-Z][A-Za-z0-9_]*)\b", text, re.M)
        if not names:
            continue
        category = path.parent.name
        for name in names:
            entries.setdefault(
                name,
                {
                    "name": name,
                    "path": path.relative_to(ROOT).as_posix(),
                    "category": category,
                },
            )
    payload = {"version": 1, "components": [entries[k] for k in sorted(entries)]}
    OUTPUT.write_text(json.dumps(payload, indent=2) + "\n")
    print(f"wrote {len(payload['components'])} components to {OUTPUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
