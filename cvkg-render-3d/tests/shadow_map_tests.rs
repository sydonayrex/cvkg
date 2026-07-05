use cvkg_render_3d::types::ShadowInstance;
use glam::Mat4;

/// Verify ShadowInstance defaults are sane.
#[test]
fn test_shadow_instance_default_transform() {
    let si = ShadowInstance::default();
    assert_eq!(si.transform, Mat4::IDENTITY);
}

#[test]
fn test_shadow_instance_custom_transform() {
    let m = Mat4::from_translation(glam::Vec3::new(1.0, 2.0, 3.0));
    let si = ShadowInstance {
        mesh_id: cvkg_core::KvasirId::default(),
        transform: m,
    };
    // Extract translation from column 3
    let t = si.transform.col(3).truncate();
    assert!((t.x - 1.0).abs() < 1e-6);
    assert!((t.y - 2.0).abs() < 1e-6);
    assert!((t.z - 3.0).abs() < 1e-6);
}

/// Verify ShadowQuality sizes are powers of two and increasing.
#[test]
fn test_shadow_quality_sizes_are_powers_of_two() {
    use cvkg_render_3d::types::ShadowQuality;
    let sizes = [
        ShadowQuality::Low.size(),
        ShadowQuality::Medium.size(),
        ShadowQuality::High.size(),
        ShadowQuality::Ultra.size(),
    ];
    for s in &sizes {
        assert!(*s > 0, "size must be positive");
        assert!(s & (s - 1) == 0, "size must be power of two, got {s}");
    }
    for window in sizes.windows(2) {
        assert!(window[1] > window[0], "sizes must be strictly increasing");
    }
}

/// Verify DirectionalLightConfig default direction is normalized.
#[test]
fn test_directional_light_default_direction_normalized() {
    use cvkg_render_3d::types::DirectionalLightConfig;
    let light = DirectionalLightConfig::default();
    let len = light.direction.length();
    assert!(
        (len - 1.0).abs() < 1e-5,
        "direction must be unit length, got {len}"
    );
}

/// Verify DirectionalLightConfig default biases are positive.
#[test]
fn test_directional_light_biases_positive() {
    use cvkg_render_3d::types::DirectionalLightConfig;
    let light = DirectionalLightConfig::default();
    assert!(light.shadow_bias > 0.0, "shadow_bias must be positive");
    assert!(
        light.shadow_normal_bias > 0.0,
        "shadow_normal_bias must be positive"
    );
}

/// Verify Light enum discriminants work for both variants.
#[test]
fn test_light_enum_both_variants() {
    use cvkg_render_3d::types::{DirectionalLightConfig, Light, PointLight};
    use glam::Vec3;

    let d = Light::Directional(DirectionalLightConfig::default());
    let p = Light::Point(PointLight {
        position: Vec3::new(1.0, 2.0, 3.0),
        color: [1.0, 0.8, 0.6],
        intensity: 500.0,
        range: 25.0,
        shadow_map_size: 1024,
    });

    // Both variants should be constructible and matchable
    match &d {
        Light::Directional(dl) => {
            assert!(dl.intensity > 0.0);
        }
        _ => panic!("expected Directional"),
    }
    match &p {
        Light::Point(pl) => {
            assert!(pl.range > 0.0);
            assert_eq!(pl.shadow_map_size, 1024);
        }
        _ => panic!("expected Point"),
    }
}
