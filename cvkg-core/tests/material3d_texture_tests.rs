/// Material3D texture field tests — verifies texture references and UV defaults.
use cvkg_core::mesh::Material3D;

/// Default material has no texture bindings.
#[test]
fn test_material3d_no_textures_by_default() {
    let m = Material3D::default();
    assert!(m.base_color_texture.is_none());
    assert!(m.normal_map_texture.is_none());
    assert!(m.metallic_roughness_texture.is_none());
}

/// Default UV scale is identity (1,1) and offset is zero (0,0).
#[test]
fn test_material3d_uv_defaults() {
    let m = Material3D::default();
    assert_eq!(m.uv_scale, [1.0, 1.0]);
    assert_eq!(m.uv_offset, [0.0, 0.0]);
}

/// Material with texture and custom UV tiling.
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

/// Material with all three texture slots populated.
#[test]
fn test_material3d_all_textures() {
    let m = Material3D {
        base_color_texture: Some("albedo.png".into()),
        normal_map_texture: Some("normal.png".into()),
        metallic_roughness_texture: Some("orm.png".into()),
        ..Default::default()
    };
    assert_eq!(m.base_color_texture.as_deref(), Some("albedo.png"));
    assert_eq!(m.normal_map_texture.as_deref(), Some("normal.png"));
    assert_eq!(m.metallic_roughness_texture.as_deref(), Some("orm.png"));
}

/// Unlit material has no textures.
#[test]
fn test_material3d_unlit_no_textures() {
    let m = Material3D::unlit([1.0, 0.0, 0.0, 1.0]);
    assert!(m.base_color_texture.is_none());
    assert!(m.normal_map_texture.is_none());
    assert!(m.metallic_roughness_texture.is_none());
    assert_eq!(m.uv_scale, [1.0, 1.0]);
    assert_eq!(m.uv_offset, [0.0, 0.0]);
}

/// Metallic material has no textures.
#[test]
fn test_material3d_metallic_no_textures() {
    let m = Material3D::metallic([0.8, 0.8, 0.8, 1.0], 0.2);
    assert!(m.base_color_texture.is_none());
    assert_eq!(m.uv_scale, [1.0, 1.0]);
}
