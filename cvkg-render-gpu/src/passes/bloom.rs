use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::nodes::{PassId, RES_BLOOM_A, RES_SCENE};
use crate::kvasir::resource::ResourceId;

pub struct BloomExtractNode {
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
}

impl BloomExtractNode {
    pub fn new() -> Self {
        Self {
            inputs: vec![RES_SCENE],
            outputs: vec![RES_BLOOM_A],
        }
    }
}

impl KvasirNode for BloomExtractNode {
    fn label(&self) -> &'static str {
        "Bloom Extract"
    }

    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }

    fn outputs(&self) -> &[ResourceId] {
        &self.outputs
    }

    fn pass_id(&self) -> PassId {
        PassId::BloomExtract
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        let bloom_texture = match ctx.registry.get_texture(RES_BLOOM_A) {
            Some(v) => v,
            None => {
                log::error!("Missing texture for {}", stringify!(RES_BLOOM_A));
                return;
            }
        };
        // Create a single-mip view for the render pass (mip 0 only)
        let bloom_view = bloom_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("bloom_extract_mip0"),
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });

        // Get scene view and create cached bind group BEFORE render pass (avoids borrow conflict)
        let scene_view = match ctx.registry.get_texture_view(RES_SCENE) {
            Some(v) => v,
            None => {
                log::error!("Missing texture view for {}", stringify!(RES_SCENE));
                return;
            }
        };
        let bg = ctx.get_or_create_bind_group(
            (RES_SCENE, 0, false),
            &ctx.renderer.texture_bind_group_layout,
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&vec![&scene_view; 32]),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&ctx.renderer.dummy_sampler),
                },
            ],
            Some("bloom_extract_scene_bg"),
        );

        let mut p = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surtr Bloom Extract"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &bloom_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            ..Default::default()
        });

        p.set_pipeline(&ctx.renderer.bloom_extract_pipeline);
        p.set_bind_group(0, &bg, &[]);
        p.set_bind_group(1, &ctx.renderer.dummy_env_bind_group, &[]);
        p.set_bind_group(2, &ctx.renderer.berserker_bind_group, &[]);
        p.draw(0..3, 0..1);
    }
}

pub struct BloomBlurNode {
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
    pub width: u32,
    pub height: u32,
}

impl BloomBlurNode {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            inputs: vec![RES_BLOOM_A],
            outputs: vec![RES_BLOOM_A],
            width,
            height,
        }
    }
}

impl KvasirNode for BloomBlurNode {
    fn label(&self) -> &'static str {
        "Bloom Blur"
    }

    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }

    fn outputs(&self) -> &[ResourceId] {
        &self.outputs
    }

    fn pass_id(&self) -> PassId {
        PassId::BloomBlur
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        let bloom_tex = match ctx.registry.get_texture(RES_BLOOM_A) {
            Some(v) => v,
            None => {
                log::error!("Missing texture for {}", stringify!(RES_BLOOM_A));
                return;
            }
        };

        // Derive mip count from the actual texture, not hardcoded
        let num_mips = bloom_tex.mip_level_count();
        if num_mips < 2 {
            return;
        }

        // Reuse persistent uniform buffer (avoids per-frame GPU allocation)
        let kawase_uniform = &ctx.renderer.kawase_uniform;

        // Create per-mip views based on actual mip count
        let effective_mips = (num_mips as usize).min(5);
        let mip_views: Vec<wgpu::TextureView> = (0..effective_mips)
            .map(|mip| {
                bloom_tex.create_view(&wgpu::TextureViewDescriptor {
                    label: Some(&format!("bloom_mip_{}", mip)),
                    base_mip_level: mip as u32,
                    mip_level_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();

        let bloom_width = self.width;
        let bloom_height = self.height;

        // Compute mip scales dynamically
        let mut mip_scales = Vec::with_capacity(effective_mips);
        for i in 0..effective_mips {
            let div = (1u32 << i) as f32;
            mip_scales.push((
                bloom_width as f32 / div,
                bloom_height as f32 / div,
                (i + 1) as f32,
            ));
        }

        // Downsample chain
        for mip in 1..effective_mips {
            let kernel_width = mip_scales[mip].2;
            let uniform_data: [f32; 8] = [
                mip_scales[(mip - 1)].0,
                mip_scales[(mip - 1)].1,
                (mip - 1) as f32,
                kernel_width,
                0.0,
                0.0,
                0.0,
                0.0,
            ];
            ctx.queue
                .write_buffer(kawase_uniform, 0, bytemuck::cast_slice(&uniform_data));

            let w = mip_scales[mip].0.max(1.0) as u32;
            let h = mip_scales[mip].1.max(1.0) as u32;

            // Cache bind group per mip level (texture views + sampler are frame-stable)
            let bg = ctx.get_or_create_bind_group(
                (RES_BLOOM_A, mip as u32, false),
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
                        resource: wgpu::BindingResource::TextureView(&mip_views[(mip - 1)]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&ctx.renderer.sampler),
                    },
                ],
                Some(&format!("kawase_bloom_bg_{}", mip)),
            );

            let mut p = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Kawase Bloom Down {}", mip)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &mip_views[mip],
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            p.set_viewport(0.0, 0.0, w as f32, h as f32, 0.0, 1.0);
            p.set_pipeline(&ctx.renderer.kawase_down_pipeline);
            p.set_bind_group(0, &bg, &[]);
            p.draw(0..3, 0..1);
        }

        // Upsample chain
        for mip in (1..effective_mips).rev() {
            let kernel_width = mip_scales[mip].2;
            let uniform_data: [f32; 8] = [
                mip_scales[mip].0,
                mip_scales[mip].1,
                mip as f32,
                kernel_width,
                0.0,
                0.0,
                0.0,
                0.0,
            ];
            ctx.queue
                .write_buffer(kawase_uniform, 0, bytemuck::cast_slice(&uniform_data));

            let w = mip_scales[(mip - 1)].0.max(1.0) as u32;
            let h = mip_scales[(mip - 1)].1.max(1.0) as u32;

            // Cache bind group per mip level (upsample)
            let bg = ctx.get_or_create_bind_group(
                (RES_BLOOM_A, (mip + 100) as u32, false), // offset key to avoid collision with downsample
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
                        resource: wgpu::BindingResource::TextureView(&mip_views[mip]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&ctx.renderer.sampler),
                    },
                ],
                Some(&format!("kawase_bloom_up_{}", mip)),
            );

            // Clear the target mip level on load to prevent additive brightening from previous frames/passes
            let mut p = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Kawase Bloom Up {}", mip)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &mip_views[(mip - 1)],
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            p.set_viewport(0.0, 0.0, w as f32, h as f32, 0.0, 1.0);
            p.set_pipeline(&ctx.renderer.kawase_up_pipeline);
            p.set_bind_group(0, &bg, &[]);
            p.draw(0..3, 0..1);
        }

        log::trace!(
            "[Kvasir] bloom_blur: Kawase pyramid ({}x{})",
            bloom_width,
            bloom_height
        );
    }
}
