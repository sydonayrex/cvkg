//! Shadow pass Kvasir node — renders depth-only shadow map from light's perspective.

use cvkg_render_gpu::kvasir::nodes::PassId;
use cvkg_render_gpu::kvasir::{ExecutionContext, KvasirNode, ResourceId};
use cvkg_render_gpu::passes::shadow::{DirectionalLight, GpuMesh3d};

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
        PassId::Shadow
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        let light_dir = self.light.direction.normalize();
        let scene_center = glam::Vec3::ZERO;
        let light_pos = scene_center + light_dir * self.scene_radius * 2.0;
        let light_view = glam::Mat4::look_at_lh(light_pos, scene_center, glam::Vec3::Y);

        let r = self.scene_radius;
        let light_proj = glam::Mat4::orthographic_lh(-r, r, -r, r, 0.0, self.scene_radius * 4.0);
        let light_vp = light_proj * light_view;

        tracing::info!(
            "ShadowNode::execute — light_vp computed, instances={}, shadow_map={:?}, light_dir=({:.2},{:.2},{:.2})",
            self.mesh_instances.len(),
            self.shadow_map,
            light_dir.x,
            light_dir.y,
            light_dir.z,
        );

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

        // Upload light VP to the renderer's scene uniforms
        ctx.queue.write_buffer(
            ctx.renderer.scene_buffer(),
            // Offset to light_vp field (after view: Mat4 (64B) + proj: Mat4 (64B) + other fields)
            // Let's calculate: SceneUniforms has view (16), proj (16), time(4), delta_time(4), resolution(8),
            // mouse(8), mouse_velocity(8), shatter_origin(8), shatter_time(4), shatter_force(4),
            // berzerker_rage(4), berzerker_mode(4), scroll_offset(4), scale_factor(4), scene_type(4),
            // _pad_vec2_align(4), fireball_pos(8), camera_pos(12+4), light_direction(12+4),
            // light_color(12+4), ibl_enabled(4), shadow_map_size(4), shadow_bias(4), _pad_shadow(8)
            // = 144 bytes before light_vp
            144,
            bytemuck::bytes_of(&light_vp),
        );

        let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Shadow Pass (Depth-Only)"),
            color_attachments: &[],
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

        // Bind the shadow pipeline from the renderer
        pass.set_pipeline(ctx.renderer.shadow_pipeline());

        for mesh in self.mesh_instances.iter() {
            pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
    }
}
