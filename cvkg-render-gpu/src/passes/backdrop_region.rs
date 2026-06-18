//! Per-element isolated backdrop blur pass.
//! Copies a scissored region from the scene texture into a
//! blur target the glass pass can sample, then runs a Kawase
//! downsample chain on the copied region.

use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::nodes::PassId;
use crate::kvasir::resource::ResourceId;

/// Copies a rectangular region from the scene texture into a
/// blur target resource, then runs a Kawase downsample chain.
pub struct BackdropRegionNode {
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
    /// Region in logical pixels.
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

        // Phase 1: GPU copy of the scissored region from scene to blur target.
        // Uses copy_texture_to_texture for an actual pixel-exact copy (no shader needed).
        let _src_extent = wgpu::Extent3d {
            width: scene_tex.width(),
            height: scene_tex.height(),
            depth_or_array_layers: 1,
        };
        let dst_extent = wgpu::Extent3d {
            width: rw,
            height: rh,
            depth_or_array_layers: 1,
        };
        ctx.encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &scene_tex,
                mip_level: 0,
                origin: wgpu::Origin3d { x: rx, y: ry, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &blur_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            dst_extent,
        );

        // Phase 2: Generate mips for the blurred backdrop region.
        // This lets the glass shader sample at different blur levels.
        // We do a simple blur via a Kawase-style approach on the copied region.
        let mip_count = blur_tex.mip_level_count().min(4);
        if mip_count >= 2 {
            // Reuse persistent uniform buffer (avoids per-frame GPU allocation)
            let kawase_uniform = &ctx.renderer.kawase_uniform;

            for mip in 1..mip_count {
                let src_view = {
                    let mut cache = ctx.renderer.texture_view_cache.lock().unwrap_or_else(|p| p.into_inner());
                    cache
                        .entry((self.output_id, (mip - 1)))
                        .or_insert_with(|| {
                            blur_tex.create_view(&wgpu::TextureViewDescriptor {
                                label: Some(&format!("blur_region_src_mip_{}", mip - 1)),
                                base_mip_level: mip - 1,
                                mip_level_count: Some(1),
                                ..Default::default()
                            })
                        })
                        .clone()
                };
                let dst_view = {
                    let mut cache = ctx.renderer.texture_view_cache.lock().unwrap_or_else(|p| p.into_inner());
                    cache
                        .entry((self.output_id, mip))
                        .or_insert_with(|| {
                            blur_tex.create_view(&wgpu::TextureViewDescriptor {
                                label: Some(&format!("blur_region_dst_mip_{}", mip)),
                                base_mip_level: mip,
                                mip_level_count: Some(1),
                                ..Default::default()
                            })
                        })
                        .clone()
                };

                let w = (rw >> mip).max(1);
                let h = (rh >> mip).max(1);
                let kernel = mip as f32;

                let uniform_data: [f32; 8] = [
                    w as f32,
                    h as f32,
                    (mip - 1) as f32,
                    kernel,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                ];
                ctx.queue
                    .write_buffer(kawase_uniform, 0, bytemuck::cast_slice(&uniform_data));

                let src_bg = ctx.get_or_create_bind_group(
                    (self.output_id, mip, false),
                    &ctx.renderer.kawase_bind_group_layout,
                    &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: kawase_uniform,
                                offset: 0,
                                size: wgpu::BufferSize::new(32),
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&src_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&ctx.renderer.sampler),
                        },
                    ],
                    Some(&format!("blur_region_kawase_bg_{}", mip)),
                );

                let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some(&format!("Backdrop Region Blur {}", mip)),
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
                pass.set_viewport(0.0, 0.0, w as f32, h as f32, 0.0, 1.0);
                pass.set_pipeline(&ctx.renderer.kawase_down_pipeline);
                pass.set_bind_group(0, &src_bg, &[]);
                pass.draw(0..3, 0..1);
            }

            // Upsample chain
            for mip in (1..mip_count).rev() {
                let src_view = {
                    let mut cache = ctx.renderer.texture_view_cache.lock().unwrap_or_else(|p| p.into_inner());
                    cache
                        .entry((self.output_id, mip))
                        .or_insert_with(|| {
                            blur_tex.create_view(&wgpu::TextureViewDescriptor {
                                label: Some(&format!("blur_region_src_mip_{}", mip)),
                                base_mip_level: mip,
                                mip_level_count: Some(1),
                                ..Default::default()
                            })
                        })
                        .clone()
                };
                let dst_view = {
                    let mut cache = ctx.renderer.texture_view_cache.lock().unwrap_or_else(|p| p.into_inner());
                    cache
                        .entry((self.output_id, (mip - 1)))
                        .or_insert_with(|| {
                            blur_tex.create_view(&wgpu::TextureViewDescriptor {
                                label: Some(&format!("blur_region_dst_mip_{}", mip - 1)),
                                base_mip_level: mip - 1,
                                mip_level_count: Some(1),
                                ..Default::default()
                            })
                        })
                        .clone()
                };

                let w = (rw >> (mip - 1)).max(1);
                let h = (rh >> (mip - 1)).max(1);
                let kernel = mip as f32;

                let uniform_data: [f32; 8] =
                    [w as f32, h as f32, mip as f32, kernel, 0.0, 0.0, 0.0, 0.0];
                ctx.queue
                    .write_buffer(kawase_uniform, 0, bytemuck::cast_slice(&uniform_data));

                let src_bg = ctx.get_or_create_bind_group(
                    (self.output_id, mip, true),
                    &ctx.renderer.kawase_bind_group_layout,
                    &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: kawase_uniform,
                                offset: 0,
                                size: wgpu::BufferSize::new(32),
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&src_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&ctx.renderer.sampler),
                        },
                    ],
                    Some(&format!("blur_region_kawase_up_bg_{}", mip)),
                );

                // Clear the destination view on load to prevent additive compounding of light during the upsample chain
                let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some(&format!("Backdrop Region Blur Up {}", mip)),
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
                pass.set_viewport(0.0, 0.0, w as f32, h as f32, 0.0, 1.0);
                pass.set_pipeline(&ctx.renderer.kawase_up_pipeline);
                pass.set_bind_group(0, &src_bg, &[]);
                pass.draw(0..3, 0..1);
            }
        }
    }
}
