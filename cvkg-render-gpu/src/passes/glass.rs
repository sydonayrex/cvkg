use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::nodes::{PassId, RES_BLUR_A, RES_SCENE};
use crate::kvasir::resource::ResourceId;

pub struct BackdropCopyNode {
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
}

impl BackdropCopyNode {
    pub fn new() -> Self {
        Self {
            inputs: vec![RES_SCENE],
            outputs: vec![RES_BLUR_A],
        }
    }
}

impl KvasirNode for BackdropCopyNode {
    fn label(&self) -> &'static str {
        "Backdrop Copy"
    }

    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }

    fn outputs(&self) -> &[ResourceId] {
        &self.outputs
    }

    fn pass_id(&self) -> PassId {
        PassId::BackdropCopy
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        let target_texture = ctx.registry.get_texture(RES_BLUR_A).unwrap();
        let target_view = target_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("backdrop_copy_mip0"),
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let mut p = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surtr Backdrop Copy"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
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

        p.set_pipeline(&ctx.renderer.copy_pipeline);

        let scene_view = ctx.registry.get_texture_view(RES_SCENE).unwrap();
        let source_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("backdrop_copy_bg"),
            layout: &ctx.renderer.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&scene_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&ctx.renderer.dummy_sampler),
                },
            ],
        });

        p.set_bind_group(0, &source_bind_group, &[]);
        p.set_bind_group(1, &ctx.renderer.dummy_env_bind_group, &[]);
        p.set_bind_group(2, &ctx.renderer.berserker_bind_group, &[]);
        p.draw(0..3, 0..1);
    }
}

pub struct BackdropBlurNode {
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
    pub width: u32,
    pub height: u32,
}

impl BackdropBlurNode {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            inputs: vec![RES_BLUR_A],
            outputs: vec![RES_BLUR_A],
            width,
            height,
        }
    }
}

impl KvasirNode for BackdropBlurNode {
    fn label(&self) -> &'static str {
        "Backdrop Blur"
    }

    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }

    fn outputs(&self) -> &[ResourceId] {
        &self.outputs
    }

    fn pass_id(&self) -> PassId {
        PassId::BackdropBlur
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        let blur_tex = ctx.registry.get_texture(RES_BLUR_A).unwrap();

        let kawase_uniform = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Kawase Uniform"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Derive mip count from the actual texture, not hardcoded
        let num_mips = blur_tex.mip_level_count();
        let effective_mips = (num_mips as usize).min(5);
        if effective_mips < 2 {
            return;
        }

        let mip_views: Vec<wgpu::TextureView> = (0..effective_mips)
            .map(|mip| {
                blur_tex.create_view(&wgpu::TextureViewDescriptor {
                    label: Some(&format!("blur_mip_{}", mip)),
                    base_mip_level: mip as u32,
                    mip_level_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();

        let kawase_bind_groups: Vec<wgpu::BindGroup> = (0..effective_mips)
            .map(|mip| {
                ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("kawase_bg_{}", mip)),
                    layout: &ctx.renderer.kawase_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &kawase_uniform,
                                offset: 0,
                                size: wgpu::BufferSize::new(32),
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&mip_views[mip as usize]),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&ctx.renderer.sampler),
                        },
                    ],
                })
            })
            .collect();

        let blur_width = self.width;
        let blur_height = self.height;

        // Compute mip scales dynamically based on actual mip count
        let mip_scales: Vec<(f32, f32, f32)> = (0..effective_mips)
            .map(|i| {
                let div = (1u32 << i) as f32;
                (blur_width as f32 / div, blur_height as f32 / div, (i + 1) as f32)
            })
            .collect();

        for mip in 1..effective_mips {
            let kernel_width = mip_scales[mip as usize].2;
            let uniform_data: [f32; 8] = [
                mip_scales[(mip - 1) as usize].0,
                mip_scales[(mip - 1) as usize].1,
                (mip - 1) as f32,
                kernel_width,
                0.0,
                0.0,
                0.0,
                0.0,
            ];
            ctx.queue
                .write_buffer(&kawase_uniform, 0, bytemuck::cast_slice(&uniform_data[..8]));

            let w = mip_scales[mip as usize].0.max(1.0) as u32;
            let h = mip_scales[mip as usize].1.max(1.0) as u32;

            let mut p = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Kawase Down {}", mip)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &mip_views[mip as usize],
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
            p.set_bind_group(0, &kawase_bind_groups[(mip - 1) as usize], &[]);
            p.draw(0..3, 0..1);
        }

        for mip in (1..effective_mips).rev() {
            let kernel_width = mip_scales[mip as usize].2;
            let uniform_data: [f32; 8] = [
                mip_scales[mip as usize].0,
                mip_scales[mip as usize].1,
                mip as f32,
                kernel_width,
                0.0,
                0.0,
                0.0,
                0.0,
            ];
            ctx.queue
                .write_buffer(&kawase_uniform, 0, bytemuck::cast_slice(&uniform_data[..8]));

            let w = mip_scales[(mip - 1) as usize].0.max(1.0) as u32;
            let h = mip_scales[(mip - 1) as usize].1.max(1.0) as u32;

            let mut p = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Kawase Up {}", mip)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &mip_views[(mip - 1) as usize],
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            p.set_viewport(0.0, 0.0, w as f32, h as f32, 0.0, 1.0);
            p.set_pipeline(&ctx.renderer.kawase_up_pipeline);
            p.set_bind_group(0, &kawase_bind_groups[mip as usize], &[]);
            p.draw(0..3, 0..1);
        }

        log::trace!(
            "[Kvasir] backdrop_blur: Kawase pyramid ({}x{})",
            blur_width,
            blur_height
        );
    }
}

pub struct GlassNode {
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
    pub scale: f32,
}

impl GlassNode {
    pub fn new(scale: f32) -> Self {
        Self {
            inputs: vec![RES_SCENE, RES_BLUR_A],
            outputs: vec![RES_SCENE],
            scale,
        }
    }
}

impl KvasirNode for GlassNode {
    fn label(&self) -> &'static str {
        "Glass"
    }

    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }

    fn outputs(&self) -> &[ResourceId] {
        &self.outputs
    }

    fn pass_id(&self) -> PassId {
        PassId::Glass
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        let rt_w = ctx.renderer.current_width() as i32;
        let rt_h = ctx.renderer.current_height() as i32;

        let scene_view = ctx.registry.get_texture_view(RES_SCENE).unwrap();
        let blur_view = ctx.registry.get_texture_view(RES_BLUR_A).unwrap();

        let ctx_blur_env_bind_group_a = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("glass_blur_bg"),
            layout: &ctx.renderer.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&blur_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&ctx.renderer.dummy_sampler),
                },
            ],
        });

        let mut p = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surtr P3 Liquid Glass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &scene_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });

        p.set_pipeline(&ctx.renderer.glass_pipeline);
        p.set_vertex_buffer(0, ctx.renderer.vertex_buffer.slice(..));
        p.set_index_buffer(
            ctx.renderer.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        p.set_bind_group(1, &ctx_blur_env_bind_group_a, &[]);
        p.set_bind_group(2, &ctx.renderer.berserker_bind_group, &[]);

        let scale = self.scale;
        for call in ctx
            .renderer
            .draw_calls
            .iter()
            .filter(|c| matches!(c.material, cvkg_core::DrawMaterial::Glass { .. }))
        {
            let bg = if let Some(id) = call.texture_id {
                if id == 0 {
                    &ctx.renderer.mega_heim_bind_group
                } else {
                    ctx.renderer
                        .texture_bind_groups
                        .get(id as usize)
                        .unwrap_or(&ctx.renderer.dummy_texture_bind_group)
                }
            } else {
                &ctx.renderer.dummy_texture_bind_group
            };

            p.set_bind_group(0, bg, &[]);

            if let Some(rect) = call.scissor_rect {
                if rt_w > 0 && rt_h > 0 {
                    let x1 = (rect.x * scale).round() as i32;
                    let y1 = (rect.y * scale).round() as i32;
                    let x2 = ((rect.x + rect.width) * scale).round() as i32;
                    let y2 = ((rect.y + rect.height) * scale).round() as i32;
                    let w = (x2 - x1).clamp(0, rt_w);
                    let h = (y2 - y1).clamp(0, rt_h);
                    if w > 0 && h > 0 {
                        p.set_scissor_rect(x1 as u32, y1 as u32, w as u32, h as u32);
                    } else {
                        p.set_scissor_rect(0, 0, 1, 1);
                    }
                }
            }
            p.draw_indexed(
                call.index_start..call.index_start + call.index_count,
                0,
                0..1,
            );
        }
    }
}
