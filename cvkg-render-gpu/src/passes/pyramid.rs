use crate::pyramid::ImagePyramid;
use crate::renderer::SurtrRenderer;

impl SurtrRenderer {
    pub(crate) fn execute_pass_pyramid_build(
        &mut self,
        post_encoder: &mut wgpu::CommandEncoder,
        scene_color_view: &wgpu::TextureView,
        pyramid: &ImagePyramid,
    ) {
        let kawase_uniform = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Kawase Pyramid Uniform"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut views = Vec::new();
        for mip in 0..pyramid.levels as usize {
            views.push(self.registry.get_texture_view(pyramid.mips[mip]).unwrap());
        }

        let mut current_w = pyramid.width as f32;
        let mut current_h = pyramid.height as f32;

        for mip in 0..pyramid.levels as usize {
            let kernel_width = (mip as f32) + 1.0;
            let uniform_data: [f32; 8] = [
                current_w,
                current_h,
                mip as f32,
                kernel_width,
                0.0,
                0.0,
                0.0,
                0.0,
            ];
            self.queue
                .write_buffer(&kawase_uniform, 0, bytemuck::cast_slice(&uniform_data));

            let next_w = (current_w / 2.0).max(1.0);
            let next_h = (current_h / 2.0).max(1.0);

            let source_view = if mip == 0 {
                scene_color_view
            } else {
                &views[mip - 1]
            };
            let target_view = &views[mip];

            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("kawase_pyramid_bg_{}", mip)),
                layout: &self.kawase_bind_group_layout,
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
                        resource: wgpu::BindingResource::TextureView(source_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

            let mut p = post_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Kawase Pyramid Down {}", mip)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
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
            p.set_viewport(0.0, 0.0, next_w, next_h, 0.0, 1.0);
            p.set_pipeline(&self.kawase_down_pipeline);
            p.set_bind_group(0, &bg, &[]);
            p.draw(0..3, 0..1);

            current_w = next_w;
            current_h = next_h;
        }

        log::trace!("[Kvasir] ImagePyramid build: {} levels", pyramid.levels);
    }
}
