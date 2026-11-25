#!/usr/bin/env python3
"""Bump the workspace and internal path-dependency versions together."""

import argparse
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SECTION = re.compile(r"^\[[^\]]+\]", re.M)
VERSION_FIELD = re.compile(r'version\s*=\s*"[^"]+"')
INLINE_PIN = re.compile(r'(cvkg[^\n]*path[^\n]*version\s*=\s*")[^"]+(")')


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("version")
    args = parser.parse_args()

    changed = 0
    for manifest in sorted(ROOT.glob("**/Cargo.toml")):
        if "target" in manifest.parts:
            continue
        text = manifest.read_text()
        matches = list(SECTION.finditer(text))
        pieces = []
        cursor = 0
        for index, match in enumerate(matches):
            end = matches[index + 1].start() if index + 1 < len(matches) else len(text)
            pieces.append(text[cursor : match.start()])
            header = match.group(0)
            chunk = text[match.start() : end]
            if header in ("[package]", "[workspace.package]") or (
                "dependencies" in header and "cvkg" in header
            ):
                updated = VERSION_FIELD.sub(f'version = "{args.version}"', chunk)
            elif "cvkg" in chunk and "path" in chunk and "version" in chunk:
                lines = chunk.splitlines(keepends=True)
                updated_lines = []
                for line in lines:
                    if "cvkg" in line and "path" in line and VERSION_FIELD.search(line):
                        line = VERSION_FIELD.sub(f'version = "{args.version}"', line)
                    updated_lines.append(line)
                updated = "".join(updated_lines)
            else:
                updated = chunk
            if updated != chunk:
                changed += 1
            pieces.append(updated)
            cursor = end
        pieces.append(text[cursor:])
        manifest.write_text("".join(pieces))
    print(f"updated {changed} manifest sections to {args.version}")


if __name__ == "__main__":
    main()
