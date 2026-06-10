use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::nodes::{PassId, RES_SCENE};
use crate::kvasir::resource::ResourceId;

pub struct UINode {
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
}

impl UINode {
    pub fn new() -> Self {
        Self {
            inputs: vec![RES_SCENE],
            outputs: vec![RES_SCENE],
        }
    }
}

impl KvasirNode for UINode {
    fn label(&self) -> &'static str {
        "UI"
    }

    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }

    fn outputs(&self) -> &[ResourceId] {
        &self.outputs
    }

    fn pass_id(&self) -> PassId {
        PassId::UI
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        let rt_w = ctx.renderer.current_width() as i32;
        let rt_h = ctx.renderer.current_height() as i32;
        let scale = ctx.renderer.current_scale_factor();

        let scene_view = ctx.registry.get_texture_view(RES_SCENE).unwrap();
        let depth_view = ctx.depth_view;

        let mut p = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surtr P4 UI Layer"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &scene_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });

        p.set_pipeline(&ctx.renderer.opaque_pipeline);
        p.set_vertex_buffer(0, ctx.renderer.vertex_buffer.slice(..));
        p.set_index_buffer(
            ctx.renderer.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        p.set_bind_group(1, &ctx.renderer.dummy_env_bind_group, &[]);
        p.set_bind_group(2, &ctx.renderer.berserker_bind_group, &[]);

        for call in ctx
            .renderer
            .draw_calls
            .iter()
            .filter(|c| matches!(c.material, cvkg_core::DrawMaterial::TopUI))
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
