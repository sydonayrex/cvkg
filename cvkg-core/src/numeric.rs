//! Shared floating-point policy and helpers (P1-1).
//!
//! CVKG mixes CPU math (layout, animation, physics, text, color) with GPU
//! shaders. Without a shared policy each module invents its own tolerance and
//! its own answer to "what do we do with NaN?", which produces inconsistent and
//! occasionally unstable behavior. This module defines the rules that new code
//! should follow and that existing hotspots are being migrated to:
//!
//! 1. **Sanitize at the boundary.** Any `f32`/`f64` entering a subsystem from
//!    input, config, deserialization or a platform API is checked with
//!    [`finite_or`] / [`clamp_finite`] before it reaches arithmetic.
//! 2. **Never persist NaN/Inf.** Results written back to state must be finite;
//!    use [`safe_div`] and [`finite_or`] at division, normalization, inverse
//!    square root and logarithm sites.
//! 3. **Compare with an epsilon.** Direct `==` on floats is reserved for exact
//!    sentinel values (e.g. `0.0` meaning "unset"). Use [`approx_eq`] or
//!    [`approx_zero`] elsewhere. NaN comparisons are always false.
//! 4. **Guard division and normalization.** A zero, subnormal or non-finite
//!    denominator returns an explicit caller-supplied fallback; it never
//!    produces `inf`/`NaN` that later poisons a transform.
//! 5. **Clamp domains, not just results.** Inputs to `sqrt`, `pow`, `acos`,
//!    `asin`, `log` and color conversions are clamped before the operation.
//! 6. **GPU mirrors CPU.** Shader code applies the equivalent clamps and
//!    fallbacks; the mapping is documented in `docs/quality/numeric-rigor.md`.
//!
//! These helpers are intentionally small and dependency-free so they can be
//! used in `no_std`-adjacent and wasm paths alike.

/// Default absolute tolerance for `f32` comparisons.
pub const EPSILON_F32: f32 = 1e-6;
/// Default absolute tolerance for `f64` comparisons.
pub const EPSILON_F64: f64 = 1e-12;
/// Default relative tolerance (scaled by the larger operand magnitude).
pub const REL_TOL_F32: f32 = 1e-6;

/// Returns `value` when it is finite, otherwise `fallback`.
///
/// If `fallback` is also non-finite, returns `0.0` so the result is always
/// finite. This makes the helper safe to use on untrusted input without adding
/// a separate validation step.
#[inline]
pub fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else if fallback.is_finite() {
        fallback
    } else {
        0.0
    }
}

/// [`finite_or`] for `f64`.
#[inline]
pub fn finite_or_f64(value: f64, fallback: f64) -> f64 {
    if value.is_finite() {
        value
    } else if fallback.is_finite() {
        fallback
    } else {
        0.0
    }
}

/// Finite value clamped to `[min, max]`, with a guaranteed-finite fallback.
///
/// Reversed or non-finite bounds fall back to `fallback` (then to `0.0`) rather
/// than panicking or returning a range-invalid value.
#[inline]
pub fn clamp_finite(value: f32, min: f32, max: f32, fallback: f32) -> f32 {
    let fallback = finite_or(fallback, 0.0);
    let lo = finite_or(min, f32::MIN);
    let hi = finite_or(max, f32::MAX);
    if lo > hi {
        return fallback;
    }
    finite_or(value, fallback).clamp(lo, hi)
}

/// Division that never yields NaN or infinity.
///
/// Returns `fallback` when the denominator is zero, subnormal, or non-finite,
/// or when the quotient itself is non-finite (overflow).
#[inline]
pub fn safe_div(numerator: f32, denominator: f32, fallback: f32) -> f32 {
    let fallback = finite_or(fallback, 0.0);
    if !denominator.is_finite() || denominator.abs() <= f32::MIN_POSITIVE {
        return fallback;
    }
    finite_or(numerator / denominator, fallback)
}

/// Approximately-equal comparison for `f32`.
///
/// Uses both an absolute tolerance and a relative tolerance scaled by the
/// larger magnitude, so large values do not require an unrealistically large
/// absolute epsilon. `NaN` is never equal to anything (including itself);
/// equal infinities compare equal, which is useful for sentinel detection.
#[inline]
pub fn approx_eq(a: f32, b: f32, epsilon: f32) -> bool {
    if !a.is_finite() || !b.is_finite() {
        return a == b;
    }
    let abs_tol = finite_or(epsilon, EPSILON_F32).abs();
    let diff = (a - b).abs();
    diff <= abs_tol + REL_TOL_F32 * a.abs().max(b.abs())
}

/// [`approx_eq`] for `f64`.
#[inline]
pub fn approx_eq_f64(a: f64, b: f64, epsilon: f64) -> bool {
    if !a.is_finite() || !b.is_finite() {
        return a == b;
    }
    let abs_tol = finite_or_f64(epsilon, EPSILON_F64).abs();
    let diff = (a - b).abs();
    diff <= abs_tol + (REL_TOL_F32 as f64) * a.abs().max(b.abs())
}

/// True when `value` is finite and within `epsilon` of zero.
#[inline]
pub fn approx_zero(value: f32, epsilon: f32) -> bool {
    value.is_finite() && value.abs() <= finite_or(epsilon, EPSILON_F32).abs()
}

/// Convenience wrapper for [`finite_or`] with a zero fallback.
#[inline]
pub fn finite_or_zero(value: f32) -> f32 {
    finite_or(value, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_or_never_returns_non_finite() {
        assert_eq!(finite_or(1.5, 9.0), 1.5);
        assert_eq!(finite_or(f32::NAN, 9.0), 9.0);
        assert_eq!(finite_or(f32::INFINITY, 9.0), 9.0);
        assert_eq!(finite_or(f32::NAN, f32::NAN), 0.0);
        assert_eq!(finite_or_f64(f64::NEG_INFINITY, 2.0), 2.0);
    }

    #[test]
    fn clamp_finite_handles_bad_inputs_and_bounds() {
        assert_eq!(clamp_finite(5.0, 0.0, 10.0, 1.0), 5.0);
        assert_eq!(clamp_finite(-5.0, 0.0, 10.0, 1.0), 0.0);
        assert_eq!(clamp_finite(50.0, 0.0, 10.0, 1.0), 10.0);
        assert_eq!(clamp_finite(f32::NAN, 0.0, 10.0, 1.0), 1.0);
        assert_eq!(clamp_finite(5.0, 10.0, 0.0, 1.0), 1.0);
        assert!(clamp_finite(5.0, f32::NAN, f32::INFINITY, 1.0).is_finite());
    }

    #[test]
    fn safe_div_guards_zero_and_non_finite() {
        assert_eq!(safe_div(10.0, 2.0, -1.0), 5.0);
        assert_eq!(safe_div(10.0, 0.0, -1.0), -1.0);
        assert_eq!(safe_div(10.0, -0.0, -1.0), -1.0);
        assert_eq!(safe_div(10.0, f32::NAN, -1.0), -1.0);
        assert_eq!(safe_div(10.0, f32::INFINITY, -1.0), -1.0);
        assert_eq!(safe_div(f32::MAX, f32::MIN_POSITIVE, -1.0), -1.0); // overflow
        assert_eq!(safe_div(f32::NAN, 2.0, -1.0), -1.0);
    }

    #[test]
    fn approx_eq_scales_with_magnitude() {
        assert!(approx_eq(1.0, 1.0000001, EPSILON_F32));
        assert!(!approx_eq(1.0, 1.01, EPSILON_F32));
        assert!(approx_eq(1.0e6, 1.0e6 + 0.5, EPSILON_F32));
        assert!(approx_eq(0.0, -0.0, EPSILON_F32));
        assert!(approx_eq_f64(1.0, 1.0000000000001, EPSILON_F64));
    }

    #[test]
    fn nan_is_never_approximately_equal() {
        assert!(!approx_eq(f32::NAN, f32::NAN, EPSILON_F32));
        assert!(!approx_eq(0.0, f32::NAN, EPSILON_F32));
        assert!(!approx_zero(f32::NAN, EPSILON_F32));
        assert!(!approx_zero(f32::INFINITY, EPSILON_F32));
        assert!(approx_zero(1e-9, EPSILON_F32));
    }

    #[test]
    fn finite_or_zero_is_always_finite() {
        assert_eq!(finite_or_zero(f32::NAN), 0.0);
        assert_eq!(finite_or_zero(-3.0), -3.0);
    }
}
