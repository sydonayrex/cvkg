use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::nodes::{PassId, RES_SCENE};
use crate::kvasir::resource::ResourceId;

pub struct OffscreenGeometryNode {
    pub target_id: u64,
    pub output_texture: ResourceId,
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
}

impl OffscreenGeometryNode {
    pub fn new(target_id: u64, output_texture: ResourceId) -> Self {
        Self {
            target_id,
            output_texture,
            inputs: vec![],
            outputs: vec![output_texture],
        }
    }
}

impl KvasirNode for OffscreenGeometryNode {
    fn label(&self) -> &'static str {
        "OffscreenGeometry"
    }
    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }
    fn outputs(&self) -> &[ResourceId] {
        &self.outputs
    }
    fn pass_id(&self) -> PassId {
        PassId::Geometry
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        let view = match ctx.registry.get_texture_view(self.output_texture) {
            Some(v) => v,
            None => {
                log::error!(
                    "Missing texture view for {}",
                    stringify!(self.output_texture)
                );
                return;
            }
        };
        // Use a dummy depth view for offscreen passes for now (no depth testing)
        // or we need a dynamic depth texture in the registry.

        let mut p = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surtr Offscreen Geometry"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None, // No depth testing for now
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        if !ctx.renderer.draw_calls.is_empty() {
            p.set_vertex_buffer(0, ctx.renderer.geometry_buffers.vertex_buffer.slice(..));
            p.set_vertex_buffer(1, ctx.renderer.geometry_buffers.instance_buffer.slice(..));
            p.set_index_buffer(
                ctx.renderer.geometry_buffers.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            p.set_bind_group(1, &ctx.renderer.dummy_env_bind_group, &[]);
            p.set_bind_group(2, &ctx.renderer.berserker_bind_group, &[]);
            p.set_bind_group(3, &ctx.renderer.gradient_bind_group, &[]);

            for call in ctx
                .renderer
                .draw_calls
                .iter()
                .filter(|c| c.target_id == Some(self.target_id))
            {
                p.set_pipeline(&ctx.renderer.opaque_pipeline);
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
                p.draw_indexed(
                    call.index_start..call.index_start + call.index_count,
                    0,
                    call.instance_start..call.instance_start + call.instance_count,
                );
            }
        }
    }
}

pub struct EffectCompositeNode {
    pub target_id: u64,
    pub input_texture: ResourceId,
    pub output_scene: ResourceId,
    pub effect: String,
    pub blend_mode: u32,
    pub effect_args: [f32; 16],
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
}

impl EffectCompositeNode {
    pub fn new(
        target_id: u64,
        input_texture: ResourceId,
        effect: String,
        blend_mode: u32,
        effect_args: [f32; 16],
    ) -> Self {
        Self {
            target_id,
            input_texture,
            output_scene: RES_SCENE,
            effect,
            blend_mode,
            effect_args,
            inputs: vec![input_texture],
            outputs: vec![RES_SCENE],
        }
    }
}

impl KvasirNode for EffectCompositeNode {
    fn label(&self) -> &'static str {
        "EffectComposite"
    }
    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }
    fn outputs(&self) -> &[ResourceId] {
        &self.outputs
    }
    fn pass_id(&self) -> PassId {
        PassId::PostProcess {
            pipeline_id: self.target_id,
        }
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        let input_view = match ctx.registry.get_texture_view(self.input_texture) {
            Some(v) => v,
            None => {
                log::error!(
                    "Missing texture view for {}",
                    stringify!(self.input_texture)
                );
                return;
            }
        };
        let scene_view = match ctx.registry.get_texture_view(self.output_scene) {
            Some(v) => v,
            None => {
                log::error!("Missing texture view for {}", stringify!(self.output_scene));
                return;
            }
        };

        // 1. Retrieve or create bind group for the input texture from cache
        let bind_group = ctx.get_or_create_bind_group(
            (self.input_texture, 0, false),
            &ctx.renderer.texture_bind_group_layout,
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&vec![&input_view; 32]),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&ctx.renderer.linear_sampler),
                },
            ],
            Some("Effect Input Bind Group"),
        );

        // 2. Map effect name to pipeline
        // For now, we will use a dummy pipeline, or compile shaders on the fly?
        // Wait, WGSL is compiled into ctx.renderer.effect_pipelines!
        // We'll need to look it up from ctx.renderer.effect_pipelines
        let pipeline = if let Some(p) = ctx.renderer.effect_pipelines.get(&self.effect) {
            p
        } else {
            return;
        };

        // 3. Write parameters to uniform buffer
        ctx.renderer.queue.write_buffer(
            &ctx.renderer.effect_params_buffer,
            0,
            bytemuck::cast_slice(&[crate::types::EffectUniforms {
                time: ctx.renderer.start_time.elapsed().as_secs_f32(),
                pad0: 0.0,
                size: [
                    ctx.renderer.current_width() as f32,
                    ctx.renderer.current_height() as f32,
                ],
                args: self.effect_args,
            }]),
        );

        let mut p = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surtr Effect Composite"),
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
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        p.set_pipeline(pipeline);
        p.set_bind_group(0, &bind_group, &[]);
        p.set_bind_group(1, &ctx.renderer.effect_params_bind_group, &[]);
        p.draw(0..3, 0..1); // Fullscreen triangle
    }
}
