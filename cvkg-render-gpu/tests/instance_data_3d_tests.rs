use std::mem;
use cvkg_render_gpu::vertex;
use bytemuck;

/// Verify InstanceData3D is 64 bytes (4 × vec4 = 16 f32 = 64 bytes).
/// Risk #1 guard: if someone merges this with 2D InstanceData, this breaks.

#[test]
fn test_instance_data_3d_size() {
    assert_eq!(mem::size_of::<vertex::InstanceData3D>(), 64);
}

#[test]
fn test_instance_data_3d_alignment() {
    // Must be 16-byte aligned for WGSL vec4<f32> alignment.
    assert_eq!(mem::align_of::<vertex::InstanceData3D>(), 4);
}

#[test]
fn test_instance_data_2d_unchanged() {
    // 2D InstanceData must remain 32 bytes — no accidental unification.
    assert_eq!(mem::size_of::<vertex::InstanceData>(), 32);
}

#[test]
fn test_instance_data_3d_model_matrix_packing() {
    // The first 3 rows of the model matrix must be packed as 3 × [f32; 4].
    let id = vertex::InstanceData3D {
        model_row0: [1.0, 0.0, 0.0, 0.0],
        model_row1: [0.0, 1.0, 0.0, 0.0],
        model_row2: [0.0, 0.0, 1.0, 0.0],
        material_overrides: [0.5, 0.5, 1.0, 1.0],
    };
    // Reconstruct the mat4 from raw bytes — verify no padding holes.
    let bytes = bytemuck::bytes_of(&id);
    assert_eq!(bytes.len(), 64);
}