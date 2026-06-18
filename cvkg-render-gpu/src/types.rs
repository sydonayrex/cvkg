//! Core data types, internal structs, and rendering contexts.
use crate::vertex::Vertex;
use cvkg_core::Rect;

/// SvgModel -- A collection of tessellated triangles representing a vector icon.
/// Paths are stored as independent sub-models, each with its own vertex range
/// and local transform, enabling per-path manipulation (e.g. in an SVG editor).
#[derive(Clone, Debug)]
pub struct SvgModel {
    /// All vertices for all paths in this SVG.
    pub vertices: Vec<Vertex>,
    /// All indices for all paths in this SVG.
    pub indices: Vec<u32>,
    /// The SVG viewBox defining the coordinate space.
    pub view_box: Rect,
    /// Per-path sub-models, each with its own vertex range and local transform.
    pub paths: Vec<SvgPath>,
    /// Animations parsed from SVG `<animate>` elements.
    pub animations: Vec<SvgAnimation>,
}

/// A single path within an SVG model, with its own vertex range and local transform.
/// Multiple paths can share the same underlying vertex buffer but are drawn
/// independently with different transforms.
#[derive(Clone, Debug)]
pub struct SvgPath {
    /// The element id from the SVG (e.g. "t1", "path2").
    pub id: String,
    /// Range into SvgModel.vertices for this path's vertices.
    pub vertex_range: std::ops::Range<usize>,
    /// Range into SvgModel.indices for this path's indices.
    pub index_range: std::ops::Range<usize>,
    /// Local transform offset applied when drawing this path.
    /// This allows per-path positioning, rotation, and scaling.
    pub local_transform: SvgTransform,
}

/// A 2D affine transform for SVG path positioning.
#[derive(Clone, Debug, Default)]
pub struct SvgTransform {
    /// Translation in SVG user units.
    pub translate: [f32; 2],
    /// Rotation in degrees.
    pub rotation: f32,
    /// Scale factor (1.0 = no scaling).
    pub scale: f32,
}

#[derive(Clone, Debug)]
pub struct SvgAnimation {
    pub target_id: String,
    pub attribute_name: String,
    /// Keyframe values. For 2-value animations, this is [from, to].
    /// For multi-keyframe animations (values="v0;v1;..."), this stores all values.
    pub keyframe_values: Vec<f32>,
    /// Optional keyTimes (normalized 0..1). If empty, uniform spacing is assumed.
    pub key_times: Vec<f32>,
    pub duration: f32,
    pub vertex_range: std::ops::Range<usize>,
}

impl SvgAnimation {
    /// Get the interpolated value at normalized time t (0..1).
    pub fn evaluate(&self, t: f32) -> f32 {
        let vals = &self.keyframe_values;
        if vals.is_empty() {
            return 0.0;
        }
        if vals.len() == 1 {
            return vals[0];
        }
        if vals.len() == 2 {
            return vals[0] + (vals[1] - vals[0]) * t;
        }
        // Multi-keyframe: find the active segment
        let times = if self.key_times.len() == vals.len() {
            &self.key_times
        } else {
            // Uniform spacing
            return self.evaluate_uniform(t);
        };
        // Find the segment containing t
        let t = t.clamp(0.0, 1.0);
        for i in 0..times.len() - 1 {
            if t >= times[i] && t <= times[i + 1] {
                let seg_t = (t - times[i]) / (times[i + 1] - times[i]);
                return vals[i] + (vals[i + 1] - vals[i]) * seg_t;
            }
        }
        vals[vals.len() - 1]
    }

    fn evaluate_uniform(&self, t: f32) -> f32 {
        let vals = &self.keyframe_values;
        let n = vals.len() - 1;
        let t = t.clamp(0.0, 1.0);
        let idx_f = t * n as f32;
        let idx = idx_f.floor() as usize;
        let frac = idx_f - idx as f32;
        if idx >= n {
            vals[n]
        } else {
            vals[idx] + (vals[idx + 1] - vals[idx]) * frac
        }
    }
}

/// Represents a single batched GPU draw call.
/// Batches are broken whenever the active texture or primitive mode changes.
#[derive(Debug, Clone)]
pub(crate) struct DrawCall {
    pub texture_id: Option<u32>,
    pub scissor_rect: Option<Rect>,
    pub index_start: u32,
    pub index_count: u32,
    /// Material routing tag -- determines which pass this draw call is routed to
    /// in the multi-pass Backdrop Capture pipeline.
    pub material: cvkg_core::DrawMaterial,
    pub target_id: Option<u64>,
    pub instance_start: u32,
    /// Draw order for sorting within the same pass. Higher = later (on top).
    /// Convention: 0 = background, 100 = UI chrome, 200 = SVG content, 300 = overlays.
    pub draw_order: i32,
}

pub struct OffscreenEffectConfig {
    pub target_id: u64,
    pub effect: String,
    pub blend_mode: u32,
    pub effect_args: [f32; 16],
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ShadowState {
    pub radius: f32,
    pub color: [f32; 4],
    pub _offset: [f32; 2],
}

pub(crate) struct SurfaceContext {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) scene_texture: wgpu::TextureView,
    pub(crate) scene_msaa_texture: wgpu::TextureView,
    pub(crate) scene_bind_group: wgpu::BindGroup,
    pub(crate) scene_texture_bind_group: wgpu::BindGroup,
    pub(crate) depth_texture_view: wgpu::TextureView,
    pub(crate) blur_tex_a: crate::kvasir::resource::ResourceId,
    pub(crate) blur_tex_b: crate::kvasir::resource::ResourceId,
    pub(crate) bloom_tex_a: crate::kvasir::resource::ResourceId,
    pub(crate) bloom_tex_b: crate::kvasir::resource::ResourceId,
    pub(crate) blur_env_bind_group_a: wgpu::BindGroup,
    pub(crate) blur_env_bind_group_b: wgpu::BindGroup,
    pub(crate) bloom_env_bind_group_a: wgpu::BindGroup,
    pub(crate) bloom_env_bind_group_b: wgpu::BindGroup,
    pub(crate) scale_factor: f32,
    pub(crate) sampler: wgpu::Sampler,
}

/// HeadlessContext -- A rendering target for surface-less execution.
pub struct HeadlessContext {
    pub scene_texture: wgpu::TextureView,
    pub scene_msaa_texture: wgpu::TextureView,
    pub scene_bind_group: wgpu::BindGroup,
    pub scene_texture_bind_group: wgpu::BindGroup,
    pub depth_texture_view: wgpu::TextureView,
    pub blur_tex_a: crate::kvasir::resource::ResourceId,
    pub blur_tex_b: crate::kvasir::resource::ResourceId,
    pub bloom_tex_a: crate::kvasir::resource::ResourceId,
    pub bloom_tex_b: crate::kvasir::resource::ResourceId,
    pub blur_env_bind_group_a: wgpu::BindGroup,
    pub blur_env_bind_group_b: wgpu::BindGroup,
    pub bloom_env_bind_group_a: wgpu::BindGroup,
    pub bloom_env_bind_group_b: wgpu::BindGroup,
    pub scale_factor: f32,
    pub sampler: wgpu::Sampler,
    pub width: u32,
    pub height: u32,
    pub output_texture: wgpu::Texture,
    pub output_view: wgpu::TextureView,
}

pub(crate) const MAX_VERTICES: usize = 100_000;
pub(crate) const MAX_INDICES: usize = 150_000;

/// Maximum number of GPU particles (ring-buffer capacity).
pub(crate) const MAX_PARTICLES: usize = 65536;

/// A single GPU particle: 32 bytes matching the WGSL Particle struct layout.
/// pos_vel: xy = position, zw = velocity.
/// color_life: xyz = RGB color, w = remaining lifetime in seconds.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuParticle {
    pub pos_vel: [f32; 4],
    pub color_life: [f32; 4],
}

/// Per-frame uniforms for the particle compute shader.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleUniforms {
    pub dt: f32,
    pub _pad: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EffectUniforms {
    pub time: f32,
    pub pad0: f32,
    pub size: [f32; 2],
    pub args: [f32; 16],
}

/// Per-draw-call glass instance parameters.
/// Passed as push constants (fast path, no buffer allocation) or via
/// a dedicated bind group for per-element blur sampling.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlassInstanceUniforms {
    /// Local tint override: [r, g, b, weight].
    /// weight=0 = use theme tint only, weight=1 = use local tint only.
    pub tint_override: [f32; 4],
    /// Per-instance IOR override. 0.0 = use theme default (1.45).
    pub ior_override: f32,
    /// Blur strength multiplier. 1.0 = normal, 2.0 = double blur.
    pub blur_multiplier: f32,
    /// Frost intensity override. 0.0 = theme default.
    pub frost_override: f32,
    /// Scissor rect in physical pixels: [x, y, width, height].
    /// Used for per-element backdrop blur sampling.
    pub scissor_px: [f32; 4],
    /// Portal index: which per-element blur texture to sample.
    /// 0 = main scene blur (default), 1+ = portal region blur.
    pub portal_index: f32,
    pub _pad: f32,
}

impl Default for GlassInstanceUniforms {
    fn default() -> Self {
        Self {
            tint_override: [0.0; 4],
            ior_override: 0.0,
            blur_multiplier: 1.0,
            frost_override: 0.0,
            scissor_px: [0.0; 4],
            portal_index: 0.0,
            _pad: 0.0,
        }
    }
}
