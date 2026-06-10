use crate::color_blindness::ColorBlindUniforms;
use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::nodes::{PassId, RES_SCENE};
use crate::kvasir::resource::ResourceId;

pub struct AccessibilityNode {
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
}

impl AccessibilityNode {
    pub fn new() -> Self {
        Self {
            // Note: inputs declares RES_SCENE (not RES_SWAPCHAIN) because the
            // execute() method samples from the scene texture, not the swapchain.
            // The swapchain is only used as the render target (target_view).
            inputs: vec![RES_SCENE],
            outputs: vec![RES_SCENE],
        }
    }
}

impl KvasirNode for AccessibilityNode {
    fn label(&self) -> &'static str {
        "Accessibility"
    }

    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }

    fn outputs(&self) -> &[ResourceId] {
        &self.outputs
    }

    fn pass_id(&self) -> PassId {
        PassId::Accessibility
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        if ctx.renderer.color_blind_mode.is_identity() {
            return;
        }

        let uniforms = ColorBlindUniforms::new(
            ctx.renderer.color_blind_mode,
            ctx.renderer.color_blind_intensity,
        );
        ctx.queue.write_buffer(
            &ctx.renderer.color_blind_uniform_buffer,
            0,
            bytemuck::bytes_of(&uniforms),
        );

        // For simplicity during refactor, we sample from scene texture.
        // In a true graph, this would sample from the previous composite target and ping-pong.
        let scene_texture = ctx
            .registry
            .get_texture_view(crate::kvasir::nodes::RES_SCENE)
            .unwrap();
        let target_view = ctx.target_view;

        let color_blind_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Color Blind Bind Group"),
            layout: &ctx.renderer.color_blind_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&scene_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&ctx.renderer.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &ctx.renderer.color_blind_uniform_buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(
                            std::mem::size_of::<ColorBlindUniforms>() as u64
                        ),
                    }),
                },
            ],
        });

        let mut p = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surtr Accessibility"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
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

        p.set_pipeline(&ctx.renderer.color_blind_pipeline);
        p.set_bind_group(0, &color_blind_bind_group, &[]);
        p.draw(0..3, 0..1);
    }
}
