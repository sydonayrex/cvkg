//! Core data types, internal structs, and rendering contexts.
use cvkg_core::Rect;
use crate::vertex::Vertex;

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
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ShadowState {
    pub radius: f32,
    pub color: [f32; 4],
    pub _offset: [f32; 2],
}

#[allow(dead_code)]
pub(crate) struct SurfaceContext {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) scene_texture: wgpu::TextureView,
    pub(crate) scene_bind_group: wgpu::BindGroup,
    pub(crate) scene_texture_bind_group: wgpu::BindGroup,
    pub(crate) depth_texture_view: wgpu::TextureView,
    // Dedicated backdrop blur textures - used only for glass backdrop blur
    // Stores raw Texture (for mip view creation) and default view (for binding)
    pub(crate) blur_tex_a: wgpu::Texture,
    pub(crate) blur_texture_a: wgpu::TextureView,
    pub(crate) blur_tex_b: wgpu::Texture,
    pub(crate) blur_texture_b: wgpu::TextureView,
    pub(crate) blur_bind_group_a: wgpu::BindGroup,
    pub(crate) blur_bind_group_b: wgpu::BindGroup,
    pub(crate) blur_env_bind_group_a: wgpu::BindGroup,
    // Dedicated bloom textures - used only for bloom extraction and blur
    pub(crate) bloom_tex_a: wgpu::Texture,
    pub(crate) bloom_texture_a: wgpu::TextureView,
    pub(crate) bloom_tex_b: wgpu::Texture,
    pub(crate) bloom_texture_b: wgpu::TextureView,
    pub(crate) bloom_bind_group_a: wgpu::BindGroup,
    pub(crate) bloom_bind_group_b: wgpu::BindGroup,
    pub(crate) bloom_env_bind_group_a: wgpu::BindGroup,
    pub(crate) scale_factor: f32,
    pub(crate) sampler: wgpu::Sampler,
}

/// HeadlessContext — A rendering target for surface-less execution.
#[allow(dead_code)]
pub struct HeadlessContext {
    pub scene_texture: wgpu::TextureView,
    pub scene_bind_group: wgpu::BindGroup,
    pub scene_texture_bind_group: wgpu::BindGroup,
    pub depth_texture_view: wgpu::TextureView,
    // Dedicated backdrop blur textures - used only for glass backdrop blur
    pub blur_tex_a: wgpu::Texture,
    pub blur_texture_a: wgpu::TextureView,
    pub blur_tex_b: wgpu::Texture,
    pub blur_texture_b: wgpu::TextureView,
    pub blur_bind_group_a: wgpu::BindGroup,
    pub blur_bind_group_b: wgpu::BindGroup,
    pub blur_env_bind_group_a: wgpu::BindGroup,
    // Dedicated bloom textures - used only for bloom extraction and blur
    pub bloom_tex_a: wgpu::Texture,
    pub bloom_texture_a: wgpu::TextureView,
    pub bloom_tex_b: wgpu::Texture,
    pub bloom_texture_b: wgpu::TextureView,
    pub bloom_bind_group_a: wgpu::BindGroup,
    pub bloom_bind_group_b: wgpu::BindGroup,
    pub bloom_env_bind_group_a: wgpu::BindGroup,
    pub scale_factor: f32,
    pub sampler: wgpu::Sampler,
    pub width: u32,
    pub height: u32,
    pub output_texture: wgpu::Texture,
    pub output_view: wgpu::TextureView,
}

pub(crate) const MAX_VERTICES: usize = 100_000;
pub(crate) const MAX_INDICES: usize = 150_000;
