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
            // Reads from RES_SCENE (scene texture sampled in the shader).
            // Writes to the swapchain render target (not a graph resource).
            inputs: vec![RES_SCENE],
            outputs: vec![], // render target write is implicit, not a graph edge
        }
    }
}

impl Default for AccessibilityNode {
    fn default() -> Self {
        Self::new()
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

        // Sample from the scene texture, render to the swapchain target.
        let scene_view = match ctx
            .registry
            .get_texture_view(crate::kvasir::nodes::RES_SCENE)
        {
            Some(v) => v,
            None => {
                tracing::error!("[Accessibility] Missing scene texture view");
                return;
            }
        };
        let target_view = ctx.target_view;

        let color_blind_bind_group = ctx.get_or_create_bind_group(
            (crate::kvasir::nodes::RES_SCENE, 99998, false),
            &ctx.renderer.color_blind_bind_group_layout,
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&scene_view),
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
            Some("Color Blind Bind Group"),
        );

        let mut p = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surtr Accessibility"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
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
