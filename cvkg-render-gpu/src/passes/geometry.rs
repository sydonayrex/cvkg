use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::nodes::{PassId, RES_SCENE};
use crate::kvasir::resource::ResourceId;

pub struct GeometryNode {
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
}

impl GeometryNode {
    pub fn new() -> Self {
        Self {
            inputs: vec![],
            outputs: vec![RES_SCENE],
        }
    }
}

impl KvasirNode for GeometryNode {
    fn label(&self) -> &'static str {
        "Geometry"
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
        let scale = ctx.renderer.current_scale_factor();
        let rt_w = ctx.renderer.current_width() as i32;
        let rt_h = ctx.renderer.current_height() as i32;

        let scene_view = match ctx.registry.get_texture_view(RES_SCENE) {
            Some(v) => v,
            None => {
                log::error!("Missing texture view for {}", stringify!(RES_SCENE));
                return;
            }
        };
        let msaa_view = match ctx
            .registry
            .get_texture_view(crate::kvasir::nodes::RES_SCENE_MSAA)
        {
            Some(v) => v,
            None => {
                log::error!(
                    "Missing texture view for {}",
                    stringify!(crate::kvasir::nodes::RES_SCENE_MSAA)
                );
                return;
            }
        };
        let depth_view = ctx.depth_view;

        let bg = ctx.renderer.default_background_color;
        let mut p = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surtr P1 Opaque Background"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &msaa_view,
                resolve_target: Some(&scene_view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: bg[0] as f64,
                        g: bg[1] as f64,
                        b: bg[2] as f64,
                        a: bg[3] as f64,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: ctx.renderer.skuld_queries.as_ref().map(|q| {
                wgpu::RenderPassTimestampWrites {
                    query_set: q,
                    beginning_of_pass_write_index: Some(0),
                    end_of_pass_write_index: None,
                }
            }),
            occlusion_query_set: None,
            multiview_mask: None,
        });

        if ctx.renderer.current_scene.scene_type == cvkg_core::SCENE_AURORA {
            p.set_pipeline(&ctx.renderer.background_pipeline);
            p.set_bind_group(0, &ctx.renderer.dummy_texture_bind_group, &[]);
            p.set_bind_group(1, &ctx.renderer.dummy_env_bind_group, &[]);
            p.set_bind_group(2, &ctx.renderer.berserker_bind_group, &[]);
            p.draw(0..3, 0..1);
        }

        if !ctx.renderer.draw_calls.is_empty() {
            log::trace!(
                "[Kvasir] GeometryNode: draw_calls={}",
                ctx.renderer.draw_calls.len()
            );
            for (i, call) in ctx.renderer.draw_calls.iter().enumerate() {
                log::trace!(
                    "[Kvasir]   call[{}]: material={:?}, target_id={:?}, index_start={}, index_count={}",
                    i,
                    call.material,
                    call.target_id,
                    call.index_start,
                    call.index_count
                );
            }
            p.set_vertex_buffer(0, ctx.renderer.vertex_buffer.slice(..));
            p.set_vertex_buffer(1, ctx.renderer.instance_buffer.slice(..));
            p.set_index_buffer(
                ctx.renderer.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            p.set_bind_group(1, &ctx.renderer.dummy_env_bind_group, &[]);
            p.set_bind_group(2, &ctx.renderer.berserker_bind_group, &[]);

            let mut opaque_calls_count = 0;
            for call in ctx.renderer.draw_calls.iter().filter(|c| {
                matches!(c.material, cvkg_core::DrawMaterial::Opaque) && c.target_id.is_none()
            }) {
                opaque_calls_count += 1;
                p.set_pipeline(&ctx.renderer.opaque_pipeline);

                // Non-trivial algorithm: Scissor Clipping and Resetting
                // WHY: Limits geometry drawing to bounds specified by push_clip_rect (e.g. clipping lightning to main demo canvas).
                // CONTRACT: If scissor_rect is Some, we calculate hardware coordinates and set it. Otherwise, we reset scissor to full texture.
                if let Some(rect) = call.scissor_rect
                    && rt_w > 0
                    && rt_h > 0
                {
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
                } else {
                    p.set_scissor_rect(0, 0, rt_w as u32, rt_h as u32);
                }

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
                    call.instance_start..call.instance_start + 1,
                );
            }
            log::trace!(
                "[Kvasir] GeometryNode: opaque_calls drawn={}",
                opaque_calls_count
            );
        }
    }
}
