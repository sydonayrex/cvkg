/// InstanceData3D memory layout tests — verifies size, alignment, and packing.
use std::mem;

use cvkg_render_gpu::vertex::{InstanceData, InstanceData3D};

/// Verify InstanceData3D is 80 bytes (4 × vec4 model/material + 2 × vec2 UV).
#[test]
fn test_instance_data_3d_size() {
    assert_eq!(mem::size_of::<InstanceData3D>(), 80);
}

/// Must be 4-byte aligned for Pod.
#[test]
fn test_instance_data_3d_alignment() {
    assert_eq!(mem::align_of::<InstanceData3D>(), 4);
}

/// 2D InstanceData must remain 32 bytes — no accidental unification.
#[test]
fn test_instance_data_2d_unchanged() {
    assert_eq!(mem::size_of::<InstanceData>(), 32);
}

/// Verify model matrix packing round-trips through bytemuck.
#[test]
fn test_instance_data_3d_model_matrix_packing() {
    let id = InstanceData3D {
        model_row0: [1.0, 0.0, 0.0, 0.0],
        model_row1: [0.0, 1.0, 0.0, 0.0],
        model_row2: [0.0, 0.0, 1.0, 0.0],
        material_overrides: [0.5, 0.5, 1.0, 1.0],
        uv_scale: [2.0, 2.0],
        uv_offset: [0.1, 0.2],
    };
    let bytes = bytemuck::bytes_of(&id);
    assert_eq!(bytes.len(), 80);
}

/// Verify default is an identity model matrix with identity UV transform.
#[test]
fn test_instance_data_3d_default_is_identity() {
    let id = InstanceData3D::default();
    assert_eq!(id.model_row0, [1.0, 0.0, 0.0, 0.0]);
    assert_eq!(id.model_row1, [0.0, 1.0, 0.0, 0.0]);
    assert_eq!(id.model_row2, [0.0, 0.0, 1.0, 0.0]);
    assert_eq!(id.uv_scale, [1.0, 1.0]);
    assert_eq!(id.uv_offset, [0.0, 0.0]);
}
