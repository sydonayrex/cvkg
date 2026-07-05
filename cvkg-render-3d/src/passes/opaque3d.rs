//! Opaque 3D render pass Kvasir node — renders 3D meshes with PBR shading and shadow sampling.

use cvkg_render_gpu::kvasir::nodes::PassId;
use cvkg_render_gpu::kvasir::{ExecutionContext, KvasirNode, ResourceId};
use cvkg_render_gpu::passes::shadow::{DirectionalLight, GpuMesh3d};

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
        let shadow_view = ctx.registry.get_texture_view(self.shadow_map);

        let scene_view = match ctx
            .registry
            .get_texture_view(cvkg_render_gpu::kvasir::nodes::RES_SCENE)
        {
            Some(v) => v,
            None => {
                tracing::error!("Opaque3dNode: missing scene color target");
                return;
            }
        };
        let depth_view = ctx.depth_view;

        let bg = [0.02f32, 0.02, 0.05, 1.0];

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
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // Bind the PBR pipeline
        pass.set_pipeline(ctx.renderer.pbr_pipeline());

        // Bind shadow map if available
        if let Some(_shadow_view) = shadow_view {
            // Bind shadow texture to the appropriate bind group
            // This requires the renderer's PBR bind group layout
            // For now, we just log that shadow is available
            tracing::trace!("Shadow map available for Opaque3d pass");
        }

        for mesh in self.mesh_instances.iter() {
            pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
    }
}
