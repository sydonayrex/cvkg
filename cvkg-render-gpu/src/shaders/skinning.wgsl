// Shader: skinning.wgsl
// Purpose: GPU-side vertex skinning and morph target blend shape interpolation.
//
// Bind Group 0: src_vertices (SkinnedVertex input) → dst_vertices (SkinnedOutput)
// The compute shader applies bone transforms to position and normal.
// UV, color, tangent are carried in a separate static buffer and read by the
// PBR vertex shader directly — they do not pass through the skinning compute.

struct SkinnedVertex {
    position: vec3<f32>,
    _pad_pos: f32,
    normal: vec3<f32>,
    _pad_norm: f32,
    joint_indices: vec4<u32>,
    joint_weights: vec4<f32>,
};

struct SkinnedOutput {
    position: vec3<f32>,
    _pad_pos_out: f32,
    normal: vec3<f32>,
    _pad_norm_out: f32,
};

// Bind Group 0: Mesh geometry data.
@group(0) @binding(0) var<storage, read> src_vertices: array<SkinnedVertex>;
@group(0) @binding(1) var<storage, read_write> dst_vertices: array<SkinnedOutput>;

// Bind Group 1: Skeletal joint matrices.
@group(1) @binding(0) var<storage, read> joint_matrices: array<mat4x4<f32>>;

// Bind Group 2: Morph Target displacement buffers (reserved).
@group(2) @binding(0) var<storage, read> morph_positions: array<vec3<f32>>;
@group(2) @binding(1) var<uniform> morph_weights: vec4<f32>;

// Compute shader entry point to perform skeletal evaluation.
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= arrayLength(&src_vertices)) {
        return;
    }

    let input = src_vertices[idx];

    // 1. Interpolate morph targets
    var position = input.position;
    var normal = input.normal;

    // Add morph target displacements scaled by active weights
    let morph_offset = idx * 2u;
    if (morph_offset + 1u < arrayLength(&morph_positions)) {
        position += morph_positions[morph_offset] * morph_weights.x;
        position += morph_positions[morph_offset + 1u] * morph_weights.y;
    }

    // 2. Perform skeletal skinning blend
    let bone_matrix =
        joint_matrices[input.joint_indices.x] * input.joint_weights.x +
        joint_matrices[input.joint_indices.y] * input.joint_weights.y +
        joint_matrices[input.joint_indices.z] * input.joint_weights.z +
        joint_matrices[input.joint_indices.w] * input.joint_weights.w;

    let skinned_pos = bone_matrix * vec4<f32>(position, 1.0);

    // NOTE: For correct normal transformation under non-uniform scaling, we should use the
    // inverse-transpose of the bone matrix. However, WGSL doesn't have a built-in matrix inverse.
    // The current approach (using the bone matrix directly) is correct for uniform scaling.
    // For non-uniform scaling, normals will be slightly distorted. A future improvement could
    // precompute inverse-transpose matrices on the CPU and pass them as a separate buffer.
    let skinned_norm = bone_matrix * vec4<f32>(normal, 0.0);

    dst_vertices[idx].position = skinned_pos.xyz;
    dst_vertices[idx].normal = normalize(skinned_norm.xyz);
}
