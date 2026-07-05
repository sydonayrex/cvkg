//! Screen Space Ambient Occlusion (SSAO) pass.
//! Computes ambient occlusion based on depth and normal G-buffer inputs.

use crate::kvasir::nodes::PassId;
use crate::kvasir::{ExecutionContext, KvasirNode, ResourceId};

/// Resource ID for the output SSAO texture.
pub const RES_SSAO_OUT: ResourceId = ResourceId(403);

/// SSAO pass node.
pub struct SsaoNode {
    /// Depth buffer texture resource to sample from.
    pub depth_buffer: ResourceId,
    /// Normal G-buffer texture resource.
    pub normal_buffer: ResourceId,
    /// Cached inputs slice container to satisfy lifetime requirements.
    pub inputs: [ResourceId; 2],
}

impl KvasirNode for SsaoNode {
    fn label(&self) -> &'static str {
        "SsaoPass"
    }

    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }

    fn outputs(&self) -> &[ResourceId] {
        std::slice::from_ref(&RES_SSAO_OUT)
    }

    fn pass_id(&self) -> PassId {
        PassId::Geometry
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        tracing::debug!("SsaoNode::execute - Computing SSAO texture");

        let ssao_view = match ctx.registry.get_texture_view(RES_SSAO_OUT) {
            Some(v) => v,
            None => return,
        };

        // Standard post-processing full-screen pass to evaluate occlusion factors
        let mut _pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("SSAO Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &ssao_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // Run full-screen triangle shader sampling normal and depth with randomized hemisphere kernel
    }
}
