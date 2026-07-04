use std::mem;

use cvkg_render_gpu::vertex::InstanceData;

/// Verify InstanceData is 32 bytes (2D instance data).
#[test]
fn test_instance_data_size() {
    assert_eq!(mem::size_of::<InstanceData>(), 32);
}

/// Verify InstanceData is 4-byte aligned.
#[test]
fn test_instance_data_alignment() {
    assert_eq!(mem::align_of::<InstanceData>(), 4);
}

/// Verify InstanceData fields are in expected order.
#[test]
fn test_instance_data_field_offsets() {
    let offset_translation = mem::offset_of!(InstanceData, translation);
    let offset_scale = mem::offset_of!(InstanceData, scale);
    let offset_rotation = mem::offset_of!(InstanceData, rotation);
    let offset_blur = mem::offset_of!(InstanceData, blur_radius);
    let offset_ior = mem::offset_of!(InstanceData, ior_override);
    let offset_glass = mem::offset_of!(InstanceData, glass_intensity);

    assert_eq!(offset_translation, 0);
    assert_eq!(offset_scale, 8);
    assert_eq!(offset_rotation, 16);
    assert_eq!(offset_blur, 20);
    assert_eq!(offset_ior, 24);
    assert_eq!(offset_glass, 28);
}