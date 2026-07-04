//! Opaque 3D render pass Kvasir node — renders 3D meshes with PBR shading
//! and reads the shadow map from the Shadow pass.

use crate::kvasir::{ExecutionContext, KvasirNode, ResourceId};
use crate::kvasir::nodes::PassId;
use crate::passes::shadow::{DirectionalLight, GpuMesh3d};

/// Opaque 3D render pass node — renders 3D meshes with PBR shading and PCF shadow sampling.
pub struct Opaque3dNode {
    /// GPU-ready mesh instances to render.
    pub mesh_instances: Vec<GpuMesh3d>,
    /// Active directional light for shading.
    pub light: DirectionalLight,
    /// Shadow map resource to sample for shadow attenuation.
    pub shadow_map: ResourceId,
}

impl KvasirNode for Opaque3dNode {
    fn label(&self) -> &'static str {
        "Opaque3d"
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
        tracing::debug!(
            "Opaque3dNode::execute — instances={}, shadow_map={:?}",
            self.mesh_instances.len(),
            self.shadow_map,
        );

        // Use the main scene render target (RES_SCENE) for color output.
        let scene_view = match ctx
            .registry
            .get_texture_view(crate::kvasir::nodes::RES_SCENE)
        {
            Some(v) => v,
            None => {
                tracing::error!("Opaque3dNode: missing scene color target");
                return;
            }
        };
        let depth_view = ctx.depth_view;

        // Default dark background color for 3D scenes.
        let bg = [0.02f32, 0.02, 0.05, 1.0];

        // Set up PBR render pass with color + depth attachments.
        let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Opaque3d Pass (PBR + Shadows)"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &scene_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: bg[0] as f64,
                        g: bg[1] as f64,
                        b: bg[2] as f64,
                        a: bg[3] as f64,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // Bind the PBR pipeline and required bind groups.
        pass.set_pipeline(&ctx.renderer.pbr_pipeline);
        pass.set_bind_group(2, &ctx.renderer.berserker_bind_group, &[]);

        // For each mesh instance, set vertex/index buffers and draw.
        for mesh in self.mesh_instances.iter() {
            pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
    }
}
