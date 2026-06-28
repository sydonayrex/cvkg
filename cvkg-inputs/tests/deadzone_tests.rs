//! Tests for deadzone math utilities.
//! Red-phase: these test the deadzone::apply() and deadzone::radial() functions.

use cvkg_inputs::deadzone;

#[test]
fn test_deadzone_clamps_small_values_to_zero() {
    assert_eq!(deadzone::apply(0.0, 0.1), 0.0);
    assert_eq!(deadzone::apply(0.05, 0.1), 0.0);
    assert_eq!(deadzone::apply(-0.05, 0.1), 0.0);
    assert_eq!(deadzone::apply(0.1, 0.1), 0.0);
    assert_eq!(deadzone::apply(-0.1, 0.1), 0.0);
}

#[test]
fn test_deadzone_preserves_full_deflection() {
    assert!((deadzone::apply(1.0, 0.1) - 1.0).abs() < f32::EPSILON);
    assert!((deadzone::apply(-1.0, 0.1) - (-1.0)).abs() < f32::EPSILON);
}

#[test]
fn test_deadzone_scales_midrange() {
    // At threshold=0.1, input=0.55 should map to (0.55-0.1)/(1.0-0.1) = 0.5
    let result = deadzone::apply(0.55, 0.1);
    assert!((result - 0.5).abs() < 0.01, "expected ~0.5, got {result}");
}

#[test]
fn test_deadzone_negative_midrange() {
    let result = deadzone::apply(-0.55, 0.1);
    assert!((result - (-0.5)).abs() < 0.01, "expected ~-0.5, got {result}");
}

#[test]
fn test_deadzone_zero_threshold_is_identity() {
    assert!((deadzone::apply(0.5, 0.0) - 0.5).abs() < f32::EPSILON);
    assert!((deadzone::apply(1.0, 0.0) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_deadzone_max_threshold_clamps_everything() {
    assert_eq!(deadzone::apply(0.5, 1.0), 0.0);
    assert_eq!(deadzone::apply(1.0, 1.0), 0.0);
    assert_eq!(deadzone::apply(-0.99, 1.0), 0.0);
}

#[test]
fn test_deadzone_nan_passthrough() {
    let result = deadzone::apply(f32::NAN, 0.1);
    assert!(result.is_nan());
}

#[test]
fn test_deadzone_infinity() {
    assert!((deadzone::apply(f32::INFINITY, 0.1) - 1.0).abs() < f32::EPSILON);
    assert!((deadzone::apply(f32::NEG_INFINITY, 0.1) - (-1.0)).abs() < f32::EPSILON);
}

#[test]
fn test_deadzone_subnormal() {
    let subnormal = f32::MIN_POSITIVE / 2.0;
    assert_eq!(deadzone::apply(subnormal, 0.1), 0.0);
}

#[test]
fn test_radial_deadzone_center_is_zero() {
    let (x, y) = deadzone::radial(0.0, 0.0, 0.1);
    assert_eq!(x, 0.0);
    assert_eq!(y, 0.0);
}

#[test]
fn test_radial_deadzone_small_vector_clamped() {
    let (x, y) = deadzone::radial(0.05, 0.05, 0.1);
    assert_eq!(x, 0.0);
    assert_eq!(y, 0.0);
}

#[test]
fn test_radial_deadzone_full_deflection() {
    let (x, y) = deadzone::radial(1.0, 0.0, 0.1);
    assert!((x - 1.0).abs() < 0.01, "expected x~1.0, got {x}");
    assert!(y.abs() < 0.01, "expected y~0, got {y}");
}

#[test]
fn test_radial_deadzone_diagonal() {
    let (x, y) = deadzone::radial(0.707, 0.707, 0.1);
    // Magnitude ~1.0, so both components should be non-zero
    assert!(x > 0.0, "expected x>0, got {x}");
    assert!(y > 0.0, "expected y>0, got {y}");
    assert!((x - y).abs() < 0.01, "expected symmetric diagonal");
}

#[test]
fn test_radial_deadzone_preserves_direction() {
    let (x, y) = deadzone::radial(0.8, 0.6, 0.1);
    let ratio = y / x;
    assert!((ratio - 0.75).abs() < 0.05, "expected y/x ~ 0.75, got {ratio}");
}

#[test]
fn test_radial_deadzone_nan() {
    let (x, y) = deadzone::radial(f32::NAN, 0.5, 0.1);
    assert!(x.is_nan() || x == 0.0);
    assert!(y.is_nan() || y == 0.0);
}

#[test]
fn test_radial_deadzone_zero_threshold() {
    let (x, y) = deadzone::radial(0.3, 0.4, 0.0);
    assert!(x > 0.0);
    assert!(y > 0.0);
}
