//! 3D rendering types: lights, shadow maps, and mesh instances.
//!
//! Re-exports GPU-ready types from `cvkg-render-gpu` for convenience.
//! This module contains the *configuration* types (shadow config, quality presets)
//! while the actual GPU buffer types live in `cvkg-render-gpu`.

use glam::Mat4;
use wgpu::{Sampler, Texture, TextureView};

pub use cvkg_render_gpu::passes::shadow::GpuMesh3d;

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

/// Directional light configuration with shadow casting parameters.
///
/// This is the *high-level* config type that includes shadow map settings.
/// For the GPU-ready directional light (without shadow config), see
/// [`cvkg_render_gpu::passes::shadow::DirectionalLight`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectionalLightConfig {
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

impl Default for DirectionalLightConfig {
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

impl From<DirectionalLightConfig> for cvkg_render_gpu::passes::shadow::DirectionalLight {
    fn from(config: DirectionalLightConfig) -> Self {
        Self {
            direction: config.direction.normalize(),
            color: glam::Vec3::from(config.color),
            intensity: config.intensity,
        }
    }
}

/// Point light with shadow casting support.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointLight {
    /// World-space position of the point light.
    pub position: glam::Vec3,
    /// RGB color components.
    pub color: [f32; 3],
    /// Luminous intensity in candelas.
    pub intensity: f32,
    /// Maximum range of the light's influence.
    pub range: f32,
    /// Resolution of the cubemap face shadow maps.
    pub shadow_map_size: u32,
}

/// Spot light with conical beam and shadow casting support.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpotLight {
    /// World-space position of the spot light.
    pub position: glam::Vec3,
    /// Normalized world-space direction.
    pub direction: glam::Vec3,
    /// RGB color components.
    pub color: [f32; 3],
    /// Luminous intensity in candelas.
    pub intensity: f32,
    /// Maximum range of the light's influence.
    pub range: f32,
    /// Inner cone angle in radians.
    pub inner_cone_angle: f32,
    /// Outer cone angle in radians.
    pub outer_cone_angle: f32,
    /// Resolution of the shadow map.
    pub shadow_map_size: u32,
}

/// Light type enum containing all supported dynamic light sources.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Light {
    /// Directional light source (e.g. sun).
    Directional(DirectionalLightConfig),
    /// Omnidirectional point light source.
    Point(PointLight),
    /// Directional cone-shaped spot light source.
    Spot(SpotLight),
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
    pub mesh_id: cvkg_core::KvasirId,
    pub transform: Mat4,
}

impl Default for ShadowInstance {
    fn default() -> Self {
        Self {
            mesh_id: cvkg_core::KvasirId::default(),
            transform: Mat4::IDENTITY,
        }
    }
}
