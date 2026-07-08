//! Global Illumination (GI) and Baked Lightmaps / Irradiance Volumes pass structures.
//! Provides precomputed indirect diffuse and specular lighting inputs to PBR shaders.

use crate::kvasir::nodes::PassId;
use crate::kvasir::{ExecutionContext, KvasirNode, ResourceId};
use cvkg_core::GiUniforms;

/// Irradiance Volume configuration representing a grid of light probes.
#[derive(Debug, Clone)]
pub struct IrradianceVolume {
    /// Dimension bounds (min corner in world space).
    pub origin: glam::Vec3,
    /// Probe step distance/spacing.
    pub spacing: glam::Vec3,
    /// Probe count in x, y, and z.
    pub dimensions: [u32; 3],
}

/// Environment probe for local specular reflections.
#[derive(Debug, Clone)]
pub struct EnvironmentProbe {
    /// Center of the probe capture region.
    pub center: glam::Vec3,
    /// Influence bounds for box/sphere projection.
    pub bounds_half_size: glam::Vec3,
}

/// GI contribution pass node.
pub struct GlobalIlluminationNode {
    /// Irradiance Volume configuration parameters.
    pub volume: IrradianceVolume,
    /// Local specular environment probes active in the scene.
    pub environment_probes: Vec<EnvironmentProbe>,
    /// Global baked lightmap texture (if present).
    pub lightmap: Option<ResourceId>,
    /// GPU uniform buffer for SH coefficients.
    pub gi_uniforms: GiUniforms,
}

impl KvasirNode for GlobalIlluminationNode {
    fn label(&self) -> &'static str {
        "GlobalIlluminationPass"
    }

    fn inputs(&self) -> &[ResourceId] {
        &[]
    }

    fn outputs(&self) -> &[ResourceId] {
        &[]
    }

    fn pass_id(&self) -> PassId {
        PassId::Opaque3d
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        // Precondition: volume.probe_data populated (assert in debug builds)
        debug_assert!(
            self.volume.dimensions.iter().all(|&d| d > 0),
            "GI volume has zero dimensions"
        );

        // Postcondition: gi_header (uniform) and gi_probe_buffer (storage)
        // are kept in sync with the CPU-side GiUniforms. We write them
        // separately because the probe grid exceeds the 64 KB uniform
        // limit and lives in a storage buffer.
        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct GiHeaderGpu {
            volume_origin: [f32; 3],
            _pad0: f32,
            volume_spacing: [f32; 3],
            _pad1: f32,
            probe_dimensions: [u32; 3],
            _pad2: u32,
        }
        let header = GiHeaderGpu {
            volume_origin: self.gi_uniforms.volume_origin,
            _pad0: self.gi_uniforms._pad0,
            volume_spacing: self.gi_uniforms.volume_spacing,
            _pad1: self.gi_uniforms._pad1,
            probe_dimensions: self.gi_uniforms.probe_dimensions,
            _pad2: self.gi_uniforms._pad2,
        };
        ctx.queue.write_buffer(
            &ctx.renderer.gi_header_buffer,
            0,
            bytemuck::bytes_of(&header),
        );
        ctx.queue.write_buffer(
            &ctx.renderer.gi_probe_buffer,
            0,
            bytemuck::cast_slice(&self.gi_uniforms.probe_coefficients),
        );
        // Invariant: probe_coefficients[0] affects fragment lighting.
    }
}
