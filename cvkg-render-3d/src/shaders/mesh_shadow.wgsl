//! Shadow pass vertex shader — depth-only rendering for shadow map generation.
//! Uses the VertexInput3D struct from common.wgsl (locations 0-3, 16-19).
//! Transforms vertices by the model matrix then by the light VP.

struct VertexOutputShadow {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput3D) -> VertexOutputShadow {
    var out: VertexOutputShadow;

    let model = mat4x4<f32>(
        in.model_row0,
        in.model_row1,
        in.model_row2,
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );

    let world_pos = model * vec4<f32>(in.position, 1.0);
    // scene.light_vp is the light's view-projection matrix
    out.clip_position = scene.light_vp * world_pos;

    return out;
}
