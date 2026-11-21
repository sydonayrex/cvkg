# Numeric rigor policy

**Status:** addresses fix-plan task P1-1 (numeric rigor standardization), first
pilot completed 2026-11-21. Updated 2026-11-21.

## 1. Problem

The 2026-11-20 audit found no unified floating-point policy. CVKG performs
heavy float math in layout, animation, physics, text shaping, color/theme
conversion and WGSL shaders. Guards existed, but each module used its own
tolerance and its own answer for NaN/Inf, which produces inconsistent and
sometimes unstable behavior.

## 2. Policy

The authoritative implementation is `cvkg-core/src/numeric.rs`. New code and
migrated hotspots should follow these rules:

1. **Sanitize at the boundary.** Every `f32`/`f64` entering a subsystem from
   input, config, deserialization or a platform API is checked with
   `finite_or` / `clamp_finite` before arithmetic.
2. **Never persist NaN/Inf.** Use `safe_div` at division, normalization,
   inverse square root and logarithm sites; write only finite results back to
   state.
3. **Compare with an epsilon.** Reserve direct `==` for genuine sentinels
   (e.g. `0.0` meaning "unset"). Use `approx_eq` / `approx_zero` elsewhere.
   NaN is never approximately equal to anything.
4. **Guard division and normalization.** Zero, subnormal or non-finite
   denominators return an explicit fallback.
5. **Clamp domains, not just results.** Inputs to `sqrt`, `pow`, `acos`,
   `asin`, `log` and color conversions are clamped before the operation.
6. **GPU mirrors CPU.** Shader code applies equivalent clamps/fallbacks; the
   mapping below records which shader sites have been reviewed.

## 3. Shared helpers (`cvkg-core::numeric`)

| Helper | Contract |
|---|---|
| `finite_or(v, fallback)` | `v` if finite, else a finite fallback (falls back to `0.0` if the fallback is also non-finite) |
| `finite_or_f64` | `f64` counterpart |
| `clamp_finite(v, min, max, fallback)` | finite + clamped; reversed/non-finite bounds yield the fallback |
| `safe_div(n, d, fallback)` | fallback on zero/subnormal/non-finite denominator or overflow |
| `approx_eq(a, b, eps)` | absolute + relative tolerance; NaN compares false |
| `approx_eq_f64` | `f64` counterpart |
| `approx_zero(v, eps)` | finite and within `eps` of zero |
| `finite_or_zero(v)` | shorthand for `finite_or(v, 0.0)` |

Defaults: `EPSILON_F32 = 1e-6`, `EPSILON_F64 = 1e-12`,
`REL_TOL_F32 = 1e-6`.

## 4. Completed pilot — spring animation

`cvkg-core/src/animation.rs` is the first migrated area:

- `SpringValue::is_near_zero` implementations now delegate to
  `numeric::approx_zero` for `f32`, `Vec2` and `Vec4`, replacing three
  hand-written `abs() < epsilon` checks. NaN now fails convergence cleanly
  instead of being compared.
- `SpringConfig::sanitized()` clamps `period` to `[1e-3, 10]`, `damping_ratio`
  to `[0, 4]` and `epsilon` to `[0, 1]`. The public `config()` still returns
  the caller's original values; only the integrator uses the sanitized copy.
- `Animated::update` clamps `dt` to `[0, 0.25]`, treats non-finite/negative/zero
  `dt` as "no time passed", and computes `omega` through `safe_div` so a zero
  period cannot produce infinity.
- Explicit Euler is only stable while `omega * step < 2`. Long frames are now
  split into substeps bounded by a tenth of the sanitized period, which keeps
  that invariant for any valid period and any (clamped) `dt`.

Regression tests added in `animation.rs`:

- `non_finite_dt_does_not_poison_state`
- `invalid_config_is_sanitized_integrator_side`
- `huge_dt_is_clamped`

`numeric.rs` itself has six unit tests covering NaN/Inf fallbacks, reversed
bounds, division guards, relative tolerance and NaN comparison semantics.

### Second pilot — layout dimensions (`cvkg-core/src/geometry_modifiers.rs`)

`FrameModifier::transform_proposal` / `transform_size` clamped dimensions with
`f32::clamp`, which **panics when `min > max`** and propagates NaN through the
public builder setters. Added a private `FrameModifier::bounded()` helper that
sanitizes both limits and delegates to `numeric::clamp_finite`, plus
`numeric::finite_or` for explicit fixed sizes. Four regression tests cover
reversed bounds, NaN/Inf bounds, valid clamping and non-finite explicit sizes.

## 5. Shader-side status

Shared WGSL helpers now live in `common.wgsl` (and therefore in the generated
`common_wasm.wgsl`): `is_finite_f32`, `safe_div_f32` and
`safe_normalize_vec3`. Naga 29 has no `isFinite` builtin, so finiteness is
checked with `v == v` plus a magnitude bound.

`material_pbr.wgsl` is the first migrated shader (P1-2 pilot): guarded the
Schlick-GGX denominator, irradiance-volume spacing, shadow homogeneous divide,
shadow texel size, and all PBR normal/light/view/half-vector normalizations.
All composed native and wasm variants still pass naga validation.

Second migration pass (P1-2 continuation):

- `material_glass.wgsl` — Snell IOR divide, clamped sqrt radicand, GGX NDF and
  Smith denominators, and screen-space normalizations.
- `common.wgsl` — SDF box/extent divides, squircle exponent, segment parameter,
  smooth-min denominator, bloom denominator, raymarch step monotonicity and
  gradient normalization.
- `material_pbr.wgsl` — reflection homogeneous divide, mode-21 march step
  monotonicity, cube gradient normalization.

All composed native and wasm variants continue to pass naga validation after
each change. Note that this proves structure and types only; it does not prove
numeric stability on real GPU backends, and no visual snapshot tests exist yet.

## 6. Verification

```bash
cargo test -p cvkg-core --lib numeric
cargo test -p cvkg-core --lib animation
```

CI runs the full workspace test suite (`.github/workflows/ci.yml`), which
includes both modules.

## 7. Adoption checklist for new code

- [ ] Every external float goes through `finite_or`/`clamp_finite`.
- [ ] Every division/normalization uses `safe_div` or an equivalent guard.
- [ ] Float comparisons use `approx_eq`/`approx_zero` unless testing a sentinel.
- [ ] Edge-case tests include `0.0`, `NaN`, `±Inf` and extreme magnitudes.
- [ ] Shader equivalents are documented in the PR and covered by composed
      shader validation.
