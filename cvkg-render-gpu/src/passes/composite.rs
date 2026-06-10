use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::nodes::{PassId, RES_BLOOM_A, RES_SCENE, RES_SWAPCHAIN};
use crate::kvasir::resource::ResourceId;

pub struct CompositeNode {
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
    pub has_bloom: bool,
    /// If true, clear the target before rendering (first pass to touch the swapchain).
    /// If false, load existing content (e.g., accessibility pass already wrote to it).
    pub clear_target: bool,
}

impl CompositeNode {
    pub fn new(has_bloom: bool, clear_target: bool) -> Self {
        Self {
            inputs: if has_bloom {
                vec![RES_SCENE, RES_BLOOM_A]
            } else {
                vec![RES_SCENE]
            },
            outputs: vec![RES_SWAPCHAIN],
            has_bloom,
            clear_target,
        }
    }
}

impl KvasirNode for CompositeNode {
    fn label(&self) -> &'static str {
        "Composite"
    }

    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }

    fn outputs(&self) -> &[ResourceId] {
        &self.outputs
    }

    fn pass_id(&self) -> PassId {
        PassId::Composite
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        let target_view = ctx.target_view;

        let mut p = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surtr P7 Composite"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: if self.clear_target {
                        wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        })
                    } else {
                        // Load existing content (e.g., accessibility pass output)
                        wgpu::LoadOp::Load
                    },
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: ctx.renderer.skuld_queries.as_ref().map(|q| {
                wgpu::RenderPassTimestampWrites {
                    query_set: q,
                    beginning_of_pass_write_index: None,
                    end_of_pass_write_index: Some(1),
                }
            }),
            occlusion_query_set: None,
            multiview_mask: None,
        });

        p.set_pipeline(&ctx.renderer.composite_pipeline);

        let scene_view = ctx.registry.get_texture_view(RES_SCENE).unwrap();
        let scene_texture_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("composite_scene_bg"),
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

        let dummy_bg = &ctx.renderer.dummy_env_bind_group;
        let bloom_bg = if self.has_bloom {
            let bloom_view = ctx.registry.get_texture_view(RES_BLOOM_A).unwrap();
            ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("composite_bloom_bg"),
                layout: &ctx.renderer.env_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&bloom_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&ctx.renderer.dummy_sampler),
                    },
                ],
            })
        } else {
            // No bloom texture needed — use dummy bind group for pass compatibility
            dummy_bg.clone()
        };

        p.set_bind_group(0, &scene_texture_bind_group, &[]);
        p.set_bind_group(1, &bloom_bg, &[]);
        p.set_bind_group(2, &ctx.renderer.berserker_bind_group, &[]);
        p.draw(0..3, 0..1);
    }
}
