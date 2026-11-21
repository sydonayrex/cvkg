#!/usr/bin/env python3
"""Shader hygiene checks for CVKG (P0-3 / P0-4).

Two modes:

  python3 scripts/validate_shaders.py --check       # CI: validate, do not write
  python3 scripts/validate_shaders.py --sync-wasm  # regenerate wasm variants

The wasm variants are derived deterministically from their native counterparts
with a small, explicit set of textual substitutions. Keeping them generated (and
checking them in CI) prevents the drift that let stale wasm shaders ship while
naga validation only covered individual native files.

This script does NOT compile WGSL. Composed-program parsing/validation is done
by the Rust tests in `cvkg-render-gpu/src/shader_composition.rs`, which CI runs
separately (see `.github/workflows/ci.yml`).
"""

from __future__ import annotations

import argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SHADERS = ROOT / "cvkg-render-gpu" / "src" / "shaders"
SOURCE = ROOT / "cvkg-render-gpu" / "src"

# native -> (wasm, [(old, new), ...], note)
WASM_VARIANTS: dict[str, tuple[str, list[tuple[str, str]], str]] = {
    "common.wgsl": (
        "common_wasm.wgsl",
        [("var t_diffuse: binding_array<texture_2d<f32>, 32>;", "var t_diffuse: texture_2d<f32>;")],
        "t_diffuse is a single texture (WebGL2 has no binding arrays).",
    ),
    "material_opaque.wgsl": (
        "material_opaque_wasm.wgsl",
        [("t_diffuse[in.tex_index]", "t_diffuse")],
        "in.tex_index is ignored; only texture slot 0 exists.",
    ),
    "bloom.wgsl": (
        "bloom_wasm.wgsl",
        [("t_diffuse[0]", "t_diffuse"), ("t_diffuse[in.tex_index]", "t_diffuse")],
        "t_diffuse is a single texture.",
    ),
}


def expected_wasm(native_name: str) -> str:
    wasm_name, replacements, note = WASM_VARIANTS[native_name]
    body = (SHADERS / native_name).read_text()
    for old, new in replacements:
        if old not in body:
            raise RuntimeError(f"{native_name}: expected substitution target not found: {old!r}")
        body = body.replace(old, new)
    header = (
        "// P1-12 WASM variant — regenerated 2026-11-21 (P0-3).\n"
        f"// Derived from {native_name}; kept in sync so composed WGSL validation\n"
        f"// covers the wasm target. {note}\n"
        "// Do not hand-edit: regenerate with scripts/validate_shaders.py --sync-wasm.\n\n"
    )
    return header + body


def check() -> int:
    failures: list[str] = []

    for native_name in WASM_VARIANTS:
        wasm_name = WASM_VARIANTS[native_name][0]
        expected = expected_wasm(native_name)
        actual = (SHADERS / wasm_name).read_text()
        if actual != expected:
            failures.append(
                f"{wasm_name} is stale or hand-edited; run "
                "`python3 scripts/validate_shaders.py --sync-wasm`"
            )

    # P0-4: no SPIR-V artifacts should be present in active source paths.
    spirv_suffixes = (".spv", ".spvasm")
    artifacts = [
        p.relative_to(ROOT)
        for p in ROOT.rglob("*")
        if p.is_file()
        and p.name.endswith(spirv_suffixes)
        and "target" not in p.parts
        and "old" not in p.parts
        and ".git" not in p.parts
    ]
    if artifacts:
        failures.append(f"unexpected SPIR-V artifacts in source paths: {artifacts}")

    # Drift warning: every shader should be referenced from Rust source.
    all_rust = "\n".join(p.read_text(errors="replace") for p in SOURCE.rglob("*.rs"))
    unreferenced = [p.name for p in sorted(SHADERS.glob("*.wgsl")) if p.name not in all_rust]

    for failure in failures:
        print(f"FAIL: {failure}")
    if unreferenced:
        print(f"WARNING: shader files not referenced from Rust source: {unreferenced}")
    if failures:
        return 1
    print(f"OK: {len(WASM_VARIANTS)} wasm variants in sync; no SPIR-V artifacts in source paths")
    return 0


def sync() -> int:
    for native_name in WASM_VARIANTS:
        wasm_name = WASM_VARIANTS[native_name][0]
        path = SHADERS / wasm_name
        path.write_text(expected_wasm(native_name))
        print(f"wrote {path.relative_to(ROOT)}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--check", action="store_true", help="validate without writing (default)")
    group.add_argument("--sync-wasm", action="store_true", help="regenerate wasm shader variants")
    args = parser.parse_args()
    return sync() if args.sync_wasm else check()


if __name__ == "__main__":
    raise SystemExit(main())
