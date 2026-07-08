//! Deferred lighting resolve pass.
//! Reconstructs lighting by combining G-buffer values, SSAO, and shadow atlas.

use crate::kvasir::nodes::PassId;
use crate::kvasir::{ExecutionContext, KvasirNode, ResourceId};
use crate::passes::gbuffer::{RES_GBUFFER_ALBEDO, RES_GBUFFER_NORMAL};
use crate::passes::ssao::RES_SSAO_OUT;

/// Deferred Lighting Pass Node.
pub struct DeferredLightingNode {
    /// SSAO occlusion input texture.
    pub ssao_occlusion: ResourceId,
    /// Shadow map / atlas resource.
    pub shadow_atlas: ResourceId,
    /// Output scene target to resolve into.
    pub scene_output: ResourceId,
    /// Cached inputs slice container.
    pub inputs: [ResourceId; 2],
}

impl KvasirNode for DeferredLightingNode {
    fn label(&self) -> &'static str {
        "DeferredLightingPass"
    }

    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }

    fn outputs(&self) -> &[ResourceId] {
        std::slice::from_ref(&self.scene_output)
    }

    fn pass_id(&self) -> PassId {
        PassId::DeferredLighting
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        // Frame 0 check: skip rendering to avoid black frame before G-buffer is valid
        if ctx.renderer.frame_counter == 0 {
            return;
        }

        tracing::debug!("DeferredLightingNode::execute - Resolving deferred PBR shading equations");

        let dest_view = match ctx.registry.get_texture_view(self.scene_output) {
            Some(v) => v,
            None => return,
        };

        // Get the gbuffer and ssao texture views
        let albedo_view = match ctx.registry.get_texture_view(RES_GBUFFER_ALBEDO) {
            Some(v) => v,
            None => return,
        };
        let normal_view = match ctx.registry.get_texture_view(RES_GBUFFER_NORMAL) {
            Some(v) => v,
            None => return,
        };
        let ssao_view = match ctx.registry.get_texture_view(RES_SSAO_OUT) {
            Some(v) => v,
            None => return,
        };

        // Bind G-buffer textures + depth + ssao (group 1)

        let sampler = &ctx.renderer.sampler;
        let deferred_bg = ctx.get_or_create_bind_group(
            (RES_GBUFFER_ALBEDO, 0, false), // key for caching - use main G-buffer resource
            &ctx.renderer.deferred_bgl,
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&albedo_view),
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
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(ctx.depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&ssao_view),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            Some("Deferred Lighting Bind Group"),
        );

        // Render pass to draw full-screen quad evaluating PBR lighting
        let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Deferred Lighting Resolve Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &dest_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // Bind scene uniforms (group 0)
        // Bind G-buffer textures + ssao (group 1)
        // Bind GI uniforms (group 2)
        pass.set_bind_group(0, &ctx.renderer.berserker_bind_group, &[]);
        pass.set_bind_group(1, &deferred_bg, &[]);
        pass.set_bind_group(2, &ctx.renderer.gi_bind_group, &[]);

        // Use deferred lighting pipeline
        pass.set_pipeline(&ctx.renderer.deferred_lighting_pipeline);
        pass.draw(0..3, 0..1);
    }
}
