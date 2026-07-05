//! G-Buffer pass - Renders scene geometry to intermediate buffers.
//! Used for deferred lighting, SSAO, and motion vector computation.

use crate::kvasir::nodes::PassId;
use crate::kvasir::{ExecutionContext, KvasirNode, ResourceId};

/// Resource IDs for the G-Buffer textures.
pub const RES_GBUFFER_ALBEDO: ResourceId = ResourceId(400);
pub const RES_GBUFFER_NORMAL: ResourceId = ResourceId(401);
pub const RES_GBUFFER_MOTION: ResourceId = ResourceId(402);

/// G-Buffer rendering node.
pub struct GBufferNode {
    /// Opaque mesh instances to draw.
    pub mesh_instances: Vec<crate::passes::shadow::GpuMesh3d>,
}

impl KvasirNode for GBufferNode {
    fn label(&self) -> &'static str {
        "GBufferPass"
    }

    fn inputs(&self) -> &[ResourceId] {
        &[]
    }

    fn outputs(&self) -> &[ResourceId] {
        &[RES_GBUFFER_ALBEDO, RES_GBUFFER_NORMAL, RES_GBUFFER_MOTION]
    }

    fn pass_id(&self) -> PassId {
        PassId::Geometry
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        tracing::debug!(
            "GBufferNode::execute - Rendering {} instances into G-Buffer",
            self.mesh_instances.len()
        );

        let albedo_view = match ctx.registry.get_texture_view(RES_GBUFFER_ALBEDO) {
            Some(v) => v,
            None => return,
        };
        let normal_view = match ctx.registry.get_texture_view(RES_GBUFFER_NORMAL) {
            Some(v) => v,
            None => return,
        };
        let motion_view = match ctx.registry.get_texture_view(RES_GBUFFER_MOTION) {
            Some(v) => v,
            None => return,
        };

        // Create a render pass rendering to three attachments + depth
        let mut _pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("G-Buffer Pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &albedo_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: &normal_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: &motion_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: ctx.depth_view,
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

        // Loop and draw instances with normal/position/uv shaders
    }
}
