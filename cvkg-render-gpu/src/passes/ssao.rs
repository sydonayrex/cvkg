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
        PassId::Ssao
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        // Frame 0 check: skip rendering to avoid black frame
        if ctx.renderer.frame_counter == 0 {
            return;
        }

        tracing::debug!("SsaoNode::execute - Computing SSAO texture");

        let ssao_view = match ctx.registry.get_texture_view(RES_SSAO_OUT) {
            Some(v) => v,
            None => return,
        };

        // Get the normal g-buffer view
        let normal_view = match ctx.registry.get_texture_view(self.normal_buffer) {
            Some(v) => v,
            None => return,
        };

        let sampler = &ctx.renderer.sampler;

        // Create SSAO bind group with depth (from context) and normal texture
        let ssao_bg = ctx.get_or_create_bind_group(
            (ResourceId(403), 0, false), // key for caching (RES_SSAO_OUT-based)
            &ctx.renderer.ssao_bgl,
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(ctx.depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            Some("SSAO Bind Group"),
        );

        // Standard post-processing full-screen pass to evaluate occlusion factors
        let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

        // Bind scene uniforms at group(0) -- SSAO shader uses scene.resolution
        pass.set_bind_group(0, &ctx.renderer.berserker_bind_group, &[]);
        // Bind depth/normal textures at group(1)
        pass.set_bind_group(1, &ssao_bg, &[]);

        // Use the SSAO pipeline for full-screen triangle shader
        pass.set_pipeline(&ctx.renderer.ssao_pipeline);
        
        pass.draw(0..3, 0..1);
    }
}
