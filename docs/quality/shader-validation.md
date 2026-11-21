# Shader validation pipeline

**Status:** addresses fix-plan tasks P0-3 (composed shader validation) and P0-4
(SPIR-V/backend support clarification). Updated 2026-11-21.

## 1. Why this exists

CVKG's renderer never compiles an individual `.wgsl` file on its own. It
concatenates several files plus runtime-generated material functions into a
single shader module per pipeline. A file that parses in isolation can still
produce an invalid *composed* program: duplicate declarations, missing helper
functions, fields absent from a struct, or an indexed access into a value that
is not an array.

The 2026-11-20 audit found exactly this class of risk: the tests only parsed
`effects.wgsl` standalone and did textual checks on a few others, while the
wasm shader set had drifted from the native one. Converting the concatenation
into a shared, tested function immediately surfaced two real defects:

1. `shapes.wgsl` referenced `VertexOutput.logical`, `clip`, `slice` and
   `material_id`, none of which existed in the stale `common_wasm.wgsl`.
2. Generated material WGSL hardcoded `t_diffuse[<index>]`, which is invalid for
   the wasm single-texture binding declared in `common_wasm.wgsl`.

Both are fixed as of this document.

## 2. Source of truth

| Item | Location |
|---|---|
| Shader files | `cvkg-render-gpu/src/shaders/*.wgsl` |
| Target selection (`cfg`) | `cvkg-render-gpu/src/lib.rs`, `WGSL_*` constants (~lines 124–156) |
| Composition | `cvkg-render-gpu/src/shader_composition.rs` |
| Generated material WGSL | `cvkg-render-gpu/src/material.rs`, `generate_builtins_wgsl_*` |
| Runtime use | `cvkg-render-gpu/src/renderer/init.rs` |
| Tests | `cvkg-render-gpu/src/shader_composition.rs` (`mod tests`) |

`renderer/init.rs` calls `shader_composition::composed_variants()`, and the
tests call `shader_composition::compose()` with the same `ShaderSet` values.
This is deliberate: the string validated in CI is byte-for-byte the string
passed to `device.create_shader_module` (subject only to the target `cfg`).

## 3. Composed programs

All entries end with the generated material dispatch code.

| Variant | Concatenation |
|---|---|
| `main` | `common` + `shapes` + `bifrost` + `bloom` + `color_blind` + generated |
| `opaque` | `common` + `material_opaque` + `bifrost` + `bloom` + `color_blind` + generated |
| `glass` | `common` + `material_glass` + `bifrost` + `bloom` + `color_blind` + generated |
| `pbr` | `common` + `material_pbr` + `bifrost` + `bloom` + `color_blind` + generated |
| `shadow` | `common` + `material_shadow` + generated (no effect modules by design) |

Two complete sets exist:

- **native** — `common.wgsl` declares `t_diffuse` as
  `binding_array<texture_2d<f32>, 32>`; material/bloom files index it.
- **wasm** — `common_wasm.wgsl`, `material_opaque_wasm.wgsl` and
  `bloom_wasm.wgsl` declare/use a single `texture_2d<f32>` because WebGL2 has
  no binding arrays. Generated material code is emitted in single-texture form
  by `generate_builtins_wgsl_lossy_for(true)`.

The wasm files are generated from their native counterparts. Do not hand-edit
them; run `python3 scripts/validate_shaders.py --sync-wasm`. CI checks that
they are in sync.

## 4. Validation performed

Each composed variant is:

1. parsed with `naga::front::wgsl::Frontend`;
2. validated with `naga::valid::Validator` using all flags and capabilities;
3. checked to contain `fn dispatch_material` (i.e. generated material code was
   actually embedded).

Run locally:

```bash
cargo test -p cvkg-render-gpu --lib shader_composition
python3 scripts/validate_shaders.py --check
```

CI runs both as the `shader-validation` job in `.github/workflows/ci.yml`.

### What is *not* covered

- No wgpu device is created in CI, so runtime shader-module creation, pipeline
  layout compatibility and binding counts are **not** validated against a real
  GPU or software adapter. Naga validation is a structural/typecheck gate only.
- No browser/WebGPU validation is performed. The wasm variants are validated on
  the host with naga; actual Safari/Chrome execution is a manual matrix
  (see `docs/platform/webkit-compatibility.md`).
- No golden SPIR-V/MSL/DXIL outputs are stored or diffed.

## 5. SPIR-V and backend support statement (P0-4)

- WGSL is the only shader source language in this repository.
- There are no hand-authored SPIR-V binaries, SPIR-V assembly, or GLSL
  `.vert`/`.frag`/`.comp` shader artifacts.
- `Cargo.toml` enables wgpu's `spirv` feature and naga's `spv-out` feature, but
  this only makes the *capability* available; it is not an implemented or
  tested artifact pipeline. It should not be read as a support guarantee.
- GPU backend translation (Vulkan/Metal/DX12/WebGPU) is delegated to wgpu/Naga
  at runtime. CVKG does not ship or audit backend IR.
- Consequently, "correct GPU assembly / SPIR-V code" is **explicitly scoped
  out** rather than claimed. If SPIR-V support is added later, it must come
  with a generation path, checked-in golden outputs and CI validation before
  any documentation claim is made.

## 6. Maintenance

When adding a shader:

1. Add the file to `cvkg-render-gpu/src/shaders/` and an `include_str!` constant
   in `cvkg-render-gpu/src/lib.rs`.
2. If it belongs in a composed program, add it to `ShaderSet` and to the
   appropriate `compose()` entries.
3. If it needs a wasm variant, add the native file to `WASM_VARIANTS` in
   `scripts/validate_shaders.py`, run `--sync-wasm`, and commit both files.
4. Run the two commands in §4. CI will fail on an invalid composed program or
   a stale wasm variant.
