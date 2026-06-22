use crate::renderer::GpuRenderer;
use crate::types::{MAX_INDICES, MAX_VERTICES};
use cvkg_core::Renderer;
use cvkg_core::LAYOUT_DIRTY;
use std::sync::atomic::Ordering;

impl cvkg_core::FrameRenderer<wgpu::CommandEncoder> for GpuRenderer {
    fn begin_frame(&mut self) -> wgpu::CommandEncoder {
        cvkg_core::begin_render_phase();
        self.frame_rendered = false;
        self.app_drew_background = false;
        let id = self
            .current_window
            .expect("No target window set for frame. Call set_target_window first.");
        self.begin_frame(id)
    }

    fn render_frame(&mut self) {
        // Visual Lint: If layout was dirtied during the render phase (layout thrashing),
        // draw a 10px red border as a warning flash.
        if LAYOUT_DIRTY.swap(false, Ordering::AcqRel) {
            if let Some(window_id) = self.current_window {
                if let Some(surface_ctx) = self.surfaces.get(&window_id) {
                    let w = surface_ctx.config.width as f32;
                    let h = surface_ctx.config.height as f32;
                    let border_rect = cvkg_core::Rect {
                        x: 0.0,
                        y: 0.0,
                        width: w,
                        height: h,
                    };
                    // Draw a thick red border to signal layout-thrashing
                    self.stroke_rect(border_rect, [1.0, 0.0, 0.0, 1.0], 10.0);
                }
            }
        }

        // Dynamic Buffer Growth (Up to 4x capacity)
        let max_v_capacity = MAX_VERTICES * 4;
        let grown = self.geometry_buffers.grow_vertex_buffer(
            &self.device,
            self.vertices.len(),
            max_v_capacity,
        );
        if grown {
            log::info!("Grew vertex buffer to fit {} vertices", self.vertices.len());
        }
        if self.vertices.len() > max_v_capacity {
            log::error!("Exceeded dynamic vertex buffer max capacity! Capping geometry.");
            self.vertices.truncate(max_v_capacity);
        }

        let max_i_capacity = MAX_INDICES * 4;
        let grown = self.geometry_buffers.grow_index_buffer(
            &self.device,
            self.indices.len(),
            max_i_capacity,
        );
        if grown {
            log::info!("Grew index buffer to fit {} indices", self.indices.len());
        }
        if self.indices.len() > max_i_capacity {
            log::error!("Exceeded dynamic index buffer max capacity! Capping geometry.");
            self.indices.truncate(max_i_capacity);
        }

        // Forge Submission: Sync all geometry to GPU using StagingBelt with a dedicated encoder
        let mut staging_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Surtr Staging Encoder"),
                });

        let mut has_writes = false;

        if !self.vertices.is_empty() {
            let v_bytes = bytemuck::cast_slice(&self.vertices);
            self.staging_belt
                .write_buffer(
                    &mut staging_encoder,
                    &self.geometry_buffers.vertex_buffer,
                    0,
                    wgpu::BufferSize::new(v_bytes.len() as u64).unwrap(),
                )
                .copy_from_slice(v_bytes);
            has_writes = true;
        }

        if !self.indices.is_empty() {
            let i_bytes = bytemuck::cast_slice(&self.indices);
            self.staging_belt
                .write_buffer(
                    &mut staging_encoder,
                    &self.geometry_buffers.index_buffer,
                    0,
                    wgpu::BufferSize::new(i_bytes.len() as u64).unwrap(),
                )
                .copy_from_slice(i_bytes);
            has_writes = true;
        }

        if !self.instance_data.is_empty() {
            let inst_bytes = bytemuck::cast_slice(&self.instance_data);
            self.staging_belt
                .write_buffer(
                    &mut staging_encoder,
                    &self.geometry_buffers.instance_buffer,
                    0,
                    wgpu::BufferSize::new(inst_bytes.len() as u64).unwrap(),
                )
                .copy_from_slice(inst_bytes);
            has_writes = true;
        }

        if has_writes {
            self.staging_belt.finish();
            self.staging_command_buffers.push(staging_encoder.finish());
        }

        // Update Time & Uniforms (Direct write is fine for small uniforms)
        self.current_scene.time = self.start_time.elapsed().as_secs_f32();
        self.queue.write_buffer(
            &self.scene_buffer,
            0,
            bytemuck::bytes_of(&self.current_scene),
        );
        self.queue.write_buffer(
            &self.theme_buffer,
            0,
            bytemuck::bytes_of(&self.current_theme),
        );

        // Populate telemetry for this frame
        self.telemetry.draw_calls = self.draw_calls.len() as u32;
        self.telemetry.vertices = self.vertices.len() as u32;
        self.frame_rendered = true;

        log::debug!(
            "[Perf] draw_calls={} vertices={} instances={} staging_cmds={}",
            self.draw_calls.len(),
            self.vertices.len(),
            self.instance_data.len(),
            self.staging_command_buffers.len()
        );
    }

    fn end_frame(&mut self, encoder: wgpu::CommandEncoder) {
        GpuRenderer::end_frame(self, encoder);
        cvkg_core::end_render_phase();
    }
}
