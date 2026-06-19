# cvkg-svg-serialize TLDR.md

## Purpose
Own the SVG serialization: converting CVKG visual trees to SVG format, supporting both export and interchange use cases.

## Ownership
- `src/lib.rs` — SVG document generation, element serialization
- CSS style embedding, coordinate system mapping

## Local Contracts
- Output must be valid SVG 1.1 or SVG 2.0.
- Must round-trip through SVG parsers without data loss.
- Coordinate system must match CVKG's origin-top-left convention.

## Verification
- Run `cargo test -p cvkg-svg-serialize`
- Run `cargo check --workspace`
