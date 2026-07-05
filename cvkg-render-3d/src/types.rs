//! 3D rendering types: lights, shadow maps, and mesh instances.

use cvkg_core::KvasirId;
use glam::Mat4;
use wgpu::{Sampler, Texture, TextureView};

/// Quality preset for shadow map resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShadowQuality {
    #[default]
    Medium,
    Low,
    High,
    Ultra,
}

impl ShadowQuality {
    pub fn size(self) -> u32 {
        match self {
            ShadowQuality::Low => 512,
            ShadowQuality::Medium => 1024,
            ShadowQuality::High => 2048,
            ShadowQuality::Ultra => 4096,
        }
    }
}

/// Directional light with shadow casting support.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectionalLight {
    /// World-space direction (normalized).
    pub direction: glam::Vec3,
    /// RGB irradiance.
    pub color: [f32; 3],
    /// Intensity in lux.
    pub intensity: f32,
    /// Shadow map resolution.
    pub shadow_map_size: u32,
    /// Depth bias for shadow acne.
    pub shadow_bias: f32,
    /// Normal offset bias.
    pub shadow_normal_bias: f32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: glam::Vec3::new(-0.5, -0.8, -0.6).normalize(),
            color: [1.0, 1.0, 1.0],
            intensity: 100000.0,
            shadow_map_size: ShadowQuality::Medium.size(),
            shadow_bias: 0.005,
            shadow_normal_bias: 0.02,
        }
    }
}

/// Point light with shadow casting support.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointLight {
    pub position: glam::Vec3,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub shadow_map_size: u32,
}

/// Light type enum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Light {
    Directional(DirectionalLight),
    Point(PointLight),
}

/// Shadow map texture resource for a single light.
#[derive(Debug)]
pub struct ShadowMap {
    pub texture: Texture,
    pub view: TextureView,
    pub sampler: Sampler,
    pub size: u32,
    /// Light view-projection matrix used to render this shadow map.
    pub light_vp: Mat4,
}

/// Per-instance data for shadow pass rendering.
#[derive(Debug, Clone, Copy)]
pub struct ShadowInstance {
    pub mesh_id: KvasirId,
    pub transform: Mat4,
}

impl Default for ShadowInstance {
    fn default() -> Self {
        Self {
            mesh_id: KvasirId::default(),
            transform: Mat4::IDENTITY,
        }
    }
}

/// GPU resources for a single 3D mesh instance ready for rendering.
#[derive(Debug)]
pub struct GpuMesh3d {
    /// Vertex buffer (position, normal, UV, etc.).
    pub vertex_buffer: wgpu::Buffer,
    /// Index buffer.
    pub index_buffer: wgpu::Buffer,
    /// Number of indices to draw.
    pub index_count: u32,
    /// Per-instance model matrix.
    pub transform: Mat4,
}
