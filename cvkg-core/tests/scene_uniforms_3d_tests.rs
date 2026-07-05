/// SceneUniforms 3D extension tests — verifies that camera_pos, light_direction,
/// and light_color fields exist with correct defaults and alignment.
use cvkg_core::render_tier::SceneUniforms;

/// Verify camera_pos defaults to [0, 0, -5] (behind origin).
#[test]
fn test_scene_uniforms_camera_pos_default() {
    let s = SceneUniforms::new(800.0, 600.0);
    assert_eq!(s.camera_pos, [0.0, 0.0, -5.0]);
}

/// Verify light fields default to non-zero.
#[test]
fn test_scene_uniforms_light_defaults() {
    let s = SceneUniforms::new(800.0, 600.0);
    let len: f32 = s.light_direction.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!(len > 0.0, "light_direction must be non-zero");
    assert!(s.light_color[0] > 0.0);
}

/// Verify SceneUniforms is 16-byte aligned for wgpu uniform buffers.
#[test]
fn test_scene_uniforms_size_32byte_aligned() {
    let size = std::mem::size_of::<SceneUniforms>();
    assert_eq!(
        size % 16,
        0,
        "SceneUniforms must be 16-byte aligned, got {} bytes",
        size
    );
}

/// Verify camera_pos is writeable.
#[test]
fn test_scene_uniforms_camera_pos_writable() {
    let mut s = SceneUniforms::new(800.0, 600.0);
    s.camera_pos = [1.0, 2.0, 3.0];
    assert_eq!(s.camera_pos, [1.0, 2.0, 3.0]);
}

/// Verify light fields are writeable.
#[test]
fn test_scene_uniforms_light_fields_writable() {
    let mut s = SceneUniforms::new(800.0, 600.0);
    s.light_direction = [0.0, 1.0, 0.0];
    s.light_color = [1.0, 0.0, 0.0];
    assert_eq!(s.light_direction, [0.0, 1.0, 0.0]);
    assert_eq!(s.light_color, [1.0, 0.0, 0.0]);
}

/// Verify ambient_color defaults and writability.
#[test]
fn test_scene_uniforms_ambient_color() {
    let mut s = SceneUniforms::new(800.0, 600.0);
    assert_eq!(s.ambient_color, [0.06, 0.07, 0.1, 1.0]);
    s.ambient_color = [0.1, 0.2, 0.3, 0.5];
    assert_eq!(s.ambient_color, [0.1, 0.2, 0.3, 0.5]);
}
