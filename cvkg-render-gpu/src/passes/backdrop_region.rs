//! Per-element isolated backdrop blur pass.
//! Copies a scissored region from the scene texture into a
//! blur target the glass pass can sample.
//!
//! NOTE: Currently performs a scissored copy only. The Kawase downsample
//! chain will be added when the glass shader is updated to sample from
//! per-element blur textures.

use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::nodes::PassId;
use crate::kvasir::resource::ResourceId;

/// Copies a rectangular region from the scene texture into a
/// blur target resource. This gives each glass element its own
/// isolated backdrop region that can be independently blurred.
pub struct BackdropRegionNode {
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
    /// Region in logical pixels (from top-left).
    pub region: cvkg_core::Rect,
    /// Output resource ID (allocated by the graph builder).
    pub output_id: ResourceId,
}

impl BackdropRegionNode {
    pub fn new(region: cvkg_core::Rect, output_id: ResourceId) -> Self {
        Self {
            inputs: vec![crate::kvasir::nodes::RES_SCENE],
            outputs: vec![output_id],
            region,
            output_id,
        }
    }
}

impl KvasirNode for BackdropRegionNode {
    fn label(&self) -> &'static str {
        "Backdrop Region"
    }
    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }
    fn outputs(&self) -> &[ResourceId] {
        &self.outputs
    }
    fn pass_id(&self) -> PassId {
        PassId::BackdropRegion
    }
    fn execute(&self, ctx: &mut ExecutionContext) {
        // Get the source scene texture
        let scene_tex = ctx
            .registry
            .get_texture(crate::kvasir::nodes::RES_SCENE)
            .expect("scene texture must exist");
        let blur_tex = ctx
            .registry
            .get_texture(self.output_id)
            .expect("blur target texture must exist");

        let scale = ctx.scale_factor;
        let rx = (self.region.x * scale) as u32;
        let ry = (self.region.y * scale) as u32;
        let rw = (self.region.width * scale) as u32;
        let rh = (self.region.height * scale) as u32;

        // Scissored copy: only copy the region this glass element occupies
        let src_view = scene_tex.create_view(&wgpu::TextureViewDescriptor {
            label: Some("backdrop_region_src"),
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });
        let dst_view = blur_tex.create_view(&wgpu::TextureViewDescriptor {
            label: Some("backdrop_region_dst"),
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Backdrop Region Copy"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &dst_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            ..Default::default()
        });

        pass.set_scissor_rect(rx, ry, rw, rh);
        pass.set_pipeline(&ctx.renderer.copy_pipeline);

        let src_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("backdrop_region_src_bg"),
            layout: &ctx.renderer.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&ctx.renderer.dummy_sampler),
                },
            ],
        });
        pass.set_bind_group(0, &src_bind_group, &[]);
        pass.set_bind_group(1, &ctx.renderer.dummy_env_bind_group, &[]);
        pass.set_bind_group(2, &ctx.renderer.berserker_bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
