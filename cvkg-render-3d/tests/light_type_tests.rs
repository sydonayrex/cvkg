use cvkg_render_3d::types::{DirectionalLight, Light, PointLight, ShadowQuality};
use glam::Vec3;

/// Verify light types construct correctly with sane defaults.

#[test]
fn test_directional_light_defaults() {
    let light = DirectionalLight::default();
    assert_eq!(light.shadow_map_size, 1024); // Medium quality default
    assert!((light.shadow_bias - 0.005).abs() < 1e-6);
}

#[test]
fn test_point_light_range_positive() {
    let light = PointLight {
        position: Vec3::ZERO,
        color: [1.0, 1.0, 1.0],
        intensity: 1000.0,
        range: 50.0,
        shadow_map_size: 512,
    };
    assert!(light.range > 0.0);
}

#[test]
fn test_shadow_quality_variants() {
    assert_eq!(ShadowQuality::Low.size(), 512);
    assert_eq!(ShadowQuality::Medium.size(), 1024);
    assert_eq!(ShadowQuality::High.size(), 2048);
    assert_eq!(ShadowQuality::Ultra.size(), 4096);
}

#[test]
fn test_light_enum_dispatch() {
    let d = Light::Directional(DirectionalLight::default());
    let p = Light::Point(PointLight {
        position: Vec3::ZERO,
        color: [1.0; 3],
        intensity: 100.0,
        range: 10.0,
        shadow_map_size: 256,
    });
    match d {
        Light::Directional(_) => {}
        _ => panic!("expected Directional"),
    }
    match p {
        Light::Point(_) => {}
        _ => panic!("expected Point"),
    }
}