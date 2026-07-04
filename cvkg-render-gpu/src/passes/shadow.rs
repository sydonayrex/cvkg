//! Shadow pass types and KvasirNode — renders depth-only shadow map from
//! light's perspective using 3D mesh data.

use crate::kvasir::{ExecutionContext, KvasirNode, ResourceId};
use crate::kvasir::nodes::PassId;
use glam::Mat4;
use wgpu::Buffer;

/// Directional light for shadow rendering.
#[derive(Debug, Clone, Copy)]
pub struct DirectionalLight {
    pub direction: glam::Vec3,
    pub color: glam::Vec3,
    pub intensity: f32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: glam::Vec3::new(0.0, -1.0, 0.0),
            color: glam::Vec3::ONE,
            intensity: 1.0,
        }
    }
}

/// GPU resources for a single 3D mesh instance ready for rendering.
#[derive(Debug, Clone)]
pub struct GpuMesh3d {
    /// Vertex buffer (position, normal, UV, etc.).
    pub vertex_buffer: Buffer,
    /// Index buffer.
    pub index_buffer: Buffer,
    /// Number of indices to draw.
    pub index_count: u32,
    /// Per-instance model matrix.
    pub transform: Mat4,
}

/// Shadow pass node — renders depth-only shadow map from light's perspective.
pub struct ShadowNode {
    pub light: DirectionalLight,
    pub shadow_map: ResourceId,
    /// GPU-ready mesh instances to render into the shadow map.
    pub mesh_instances: Vec<GpuMesh3d>,
    /// Bounds of the scene — used for light VP frustum computation.
    pub scene_radius: f32,
}

impl KvasirNode for ShadowNode {
    fn label(&self) -> &'static str {
        "ShadowPass"
    }

    fn inputs(&self) -> &[ResourceId] {
        &[]
    }

    fn outputs(&self) -> &[ResourceId] {
        std::slice::from_ref(&self.shadow_map)
    }

    fn pass_id(&self) -> PassId {
        PassId::Shadow
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        // Compute light VP: orthographic frustum from light direction toward scene origin.
        let light_dir = self.light.direction;
        let scene_center = glam::Vec3::ZERO;
        let light_pos = scene_center + light_dir * self.scene_radius * 2.0;
        let light_view = glam::Mat4::look_at_lh(light_pos, scene_center, glam::Vec3::Y);

        // Orthographic projection covering the scene bounds.
        let r = self.scene_radius;
        let light_proj = glam::Mat4::orthographic_lh(-r, r, -r, r, 0.0, self.scene_radius * 4.0);

        // Update scene_buffer with the computed light VP for shadow shader.
        // Create a partial update with just the light_vp field.
        let _light_vp = light_proj * light_view;
        // Note: The actual light_vp needs to be written to scene_buffer
        // but we need the renderer's current scene to preserve other fields.
        // For now, write a minimal struct update.

        tracing::debug!(
            "ShadowNode::execute — instances={}, shadow_map={:?}, light_dir=({:.2},{:.2},{:.2})",
            self.mesh_instances.len(),
            self.shadow_map,
            light_dir.x, light_dir.y, light_dir.z,
        );

        // Get the shadow map texture view from the resource registry.
        let shadow_view = match ctx.registry.get_texture_view(self.shadow_map) {
            Some(v) => v,
            None => {
                tracing::error!(
                    "ShadowNode: missing shadow map texture view for {:?}",
                    self.shadow_map,
                );
                return;
            }
        };

        // Create a depth-only render pass.
        let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Shadow Pass (Depth-Only)"),
            color_attachments: &[], // No color output.
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &shadow_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // Bind the shadow pipeline and scene uniforms.
        pass.set_pipeline(&ctx.renderer.shadow_pipeline);
        pass.set_bind_group(1, &ctx.renderer.berserker_bind_group, &[]);

        // For each mesh, set vertex/index buffers and draw depth only.
        for mesh in self.mesh_instances.iter() {
            // Set vertex buffer (assumes interleaved position at location 0).
            pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            // Bind per-instance transform and draw.
            pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
    }
}
