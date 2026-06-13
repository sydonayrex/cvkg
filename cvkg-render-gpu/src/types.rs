//! Core data types, internal structs, and rendering contexts.
use crate::vertex::Vertex;
use cvkg_core::Rect;

/// SvgModel — A collection of tessellated triangles representing a vector icon.
#[derive(Clone, Debug)]
pub struct SvgModel {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub view_box: Rect,
    pub animations: Vec<SvgAnimation>,
}

#[derive(Clone, Debug)]
pub struct SvgAnimation {
    pub target_id: String,
    pub attribute_name: String,
    pub from_val: f32,
    pub to_val: f32,
    pub duration: f32,
    pub vertex_range: std::ops::Range<usize>,
}

/// Represents a single batched GPU draw call.
/// Batches are broken whenever the active texture or primitive mode changes.
#[derive(Debug, Clone)]
pub(crate) struct DrawCall {
    pub texture_id: Option<u32>,
    pub scissor_rect: Option<Rect>,
    pub index_start: u32,
    pub index_count: u32,
    /// Material routing tag — determines which pass this draw call is routed to
    /// in the multi-pass Backdrop Capture pipeline.
    pub material: cvkg_core::DrawMaterial,
    pub target_id: Option<u64>,
    pub instance_start: u32,
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

/// HeadlessContext — A rendering target for surface-less execution.
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
