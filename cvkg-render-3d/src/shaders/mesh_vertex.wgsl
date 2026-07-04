//! Full MVP vertex shader for 3D meshes.
//! Uses the VertexInput3D struct from common.wgsl (locations 0-3, 16-21).
//! Reads the per-instance 3x4 model matrix and constructs a full mat4x4.

struct VertexOutput3D {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv:              vec2<f32>,
    @location(1) color:           vec4<f32>,
    @location(2) @interpolate(flat) material_id: u32,
    @location(3) world_pos:       vec3<f32>,
    @location(4) world_normal:    vec3<f32>,
    @location(5) material_uv_scale:  vec2<f32>,
    @location(6) material_uv_offset: vec2<f32>,
};

@vertex
fn vs_main_3d(in: VertexInput3D) -> VertexOutput3D {
    var out: VertexOutput3D;

    let model = mat4x4<f32>(
        in.model_row0,
        in.model_row1,
        in.model_row2,
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );

    let world_pos = model * vec4<f32>(in.position, 1.0);
    let view_proj = scene.proj * scene.view;

    out.clip_position = view_proj * world_pos;
    out.world_pos = world_pos.xyz;
    out.world_normal = normalize((model * vec4<f32>(in.normal, 0.0)).xyz);

    out.uv = in.uv;
    out.color = in.color;
    out.material_id = 13u; // OPAQUE_3D
    out.material_uv_scale = in.uv_scale;
    out.material_uv_offset = in.uv_offset;

    return out;
}
