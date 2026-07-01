use cvkg_core::mesh::Material3D;

/// Verify Material3D gains texture fields with correct defaults.

#[test]
fn test_material3d_no_textures_by_default() {
    let m = Material3D::default();
    assert!(m.base_color_texture.is_none());
    assert!(m.normal_map_texture.is_none());
    assert!(m.metallic_roughness_texture.is_none());
}

#[test]
fn test_material3d_uv_defaults() {
    let m = Material3D::default();
    assert_eq!(m.uv_scale, [1.0, 1.0]);
    assert_eq!(m.uv_offset, [0.0, 0.0]);
}

#[test]
fn test_material3d_with_texture() {
    let m = Material3D {
        base_color_texture: Some("tile.png".into()),
        uv_scale: [2.0, 2.0],
        ..Default::default()
    };
    assert_eq!(m.base_color_texture.as_deref(), Some("tile.png"));
    assert_eq!(m.uv_scale, [2.0, 2.0]);
}