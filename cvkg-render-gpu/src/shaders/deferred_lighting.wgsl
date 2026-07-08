// Shader: deferred_lighting.wgsl
// Purpose: Deferred PBR lighting resolve pass.
// Pipeline layout: [berserker@0, deferred_bgl@1, gi@2]

struct SceneUniforms {
    view:            mat4x4<f32>,
    proj:            mat4x4<f32>,
    time:            f32,
    delta_time:      f32,
    resolution:      vec2<f32>,
    mouse:           vec2<f32>,
    mouse_velocity:  vec2<f32>,
    shatter_origin:  vec2<f32>,
    shatter_time:    f32,
    shatter_force:   f32,
    berzerker_rage:  f32,
    berzerker_mode:  u32,
    scroll_offset:   f32,
    scale_factor:    f32,
    scene_type:      u32,
    _pad_vec2_align: u32,
    fireball_pos:    vec2<f32>,
    camera_pos:      vec3<f32>,
    _pad2:           f32,
    light_direction: vec3<f32>,
    _pad3:           f32,
    light_color:     vec3<f32>,
    ibl_enabled:     u32,
    shadow_map_size: f32,
    shadow_bias:     f32,
    _pad_shadow:     u32,
    _pad_shadow2:    u32,
    light_vp:        mat4x4<f32>,
    ambient_color:   vec4<f32>,
};

// Group 0 = berserker_bind_group_layout (from deferred pipeline layout)
// Theme is @binding(0), scene is @binding(1), csm is @binding(2)
@group(0) @binding(1) var<uniform> scene: SceneUniforms;

// Group 1 = deferred_bgl (G-buffer textures)
@group(1) @binding(0) var t_albedo: texture_2d<f32>;
@group(1) @binding(1) var s_albedo: sampler;
@group(1) @binding(2) var t_normal: texture_2d<f32>;
@group(1) @binding(3) var s_normal: sampler;
@group(1) @binding(4) var t_depth: texture_depth_2d;
@group(1) @binding(5) var s_depth: sampler;
@group(1) @binding(6) var t_ssao: texture_2d<f32>;
@group(1) @binding(7) var s_ssao: sampler;

// Group 2 = gi_bind_group_layout (GI uniforms for probe sampling).
// Header is a small uniform buffer. Probe coefficients live in a separate
// read-only storage buffer (256 KB exceeds the 64 KB uniform-buffer limit).
struct GiHeader {
    volume_origin: vec3<f32>,
    _pad0: f32,
    volume_spacing: vec3<f32>,
    _pad1: f32,
    probe_dimensions: vec3<u32>,
    _pad2: u32,
};
@group(2) @binding(0) var<uniform> gi: GiHeader;
@group(2) @binding(1) var<storage, read> gi_probes: array<array<vec3<f32>, 4>>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vertex_index) / 2) * 4.0 - 1.0;
    let y = f32(i32(vertex_index) % 2) * 4.0 - 1.0;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

@fragment
fn fs_deferred_resolve(in: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(t_albedo, s_albedo, in.uv);
    let normal_val = textureSample(t_normal, s_normal, in.uv).xyz;
    let normal = normalize(normal_val * 2.0 - 1.0);
    let depth = textureSample(t_depth, s_depth, in.uv);

    if (depth >= 1.0) {
        return vec4<f32>(0.02, 0.02, 0.05, 1.0);
    }

    let ssao = textureSample(t_ssao, s_ssao, in.uv).r;

    let light_dir = normalize(scene.light_direction);
    let n_dot_l = max(dot(normal, light_dir), 0.0);

    let diffuse = albedo.rgb * n_dot_l * scene.light_color;
    let ambient = albedo.rgb * scene.ambient_color.rgb * scene.ambient_color.w * ssao;

    let final_color = ambient + diffuse;
    return vec4<f32>(final_color, albedo.a);
}