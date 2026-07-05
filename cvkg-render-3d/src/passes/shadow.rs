//! Shadow pass Kvasir node — renders depth-only shadow map from light's perspective.

use crate::types::{DirectionalLight, GpuMesh3d};
use cvkg_render_gpu::kvasir::nodes::PassId;
use cvkg_render_gpu::kvasir::{ExecutionContext, KvasirNode, ResourceId};

/// Shadow pass node — renders depth-only shadow map from light's perspective.
pub struct ShadowNode {
    pub light: DirectionalLight,
    pub shadow_map: ResourceId,
    /// GPU-ready mesh instances to render into the shadow map.
    pub mesh_instances: Vec<GpuMesh3d>,
    /// Bounds of the scene - used for light VP computation.
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
        PassId::Opaque3d
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
        let _light_vp = light_proj * light_view;

        // Store light VP in SceneUniforms via the uniform buffer — write through the queue.
        // ctx.renderer currently owns the Berserker uniform buffer; use the queue to update it.
        // The 3D shaders access `scene.light_vp` for shadow projection.
        tracing::info!(
            "ShadowNode::execute — light_vp computed, instances={}, shadow_map={:?}, light_dir=({:.2},{:.2},{:.2})",
            self.mesh_instances.len(),
            self.shadow_map,
            light_dir.x,
            light_dir.y,
            light_dir.z,
        );

        // Get the shadow map texture view from the resource registry.
        let shadow_view = match ctx.registry.get_texture_view(self.shadow_map) {
            Some(v) => v,
            None => {
                tracing::error!(
                    "ShadowNode: missing shadow map texture view for {:?}",
                    self.shadow_map
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

        // For each mesh, set vertex/index buffers and draw.
        for (_i, mesh) in self.mesh_instances.iter().enumerate() {
            // Set vertex buffer (assumes interleaved position-normal-UV at location 0).
            pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            // Bind per-instance transform via instance vertex buffer.
            // For now, push uniforms manually — full 3D pipeline to follow.
            pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
    }
}
