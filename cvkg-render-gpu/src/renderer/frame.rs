//! Frame lifecycle methods for GpuRenderer.
//!
//! Extracted from draw.rs for modularization.

use crate::types::*;
use crate::vertex::Vertex;
use crate::renderer::GpuRenderer;
use cvkg_core::{Rect, Renderer};

impl GpuRenderer {
    /// Reset per-frame state shared by both `begin_frame` and `begin_frame_headless`.
    /// Factored out to avoid the copy-paste duplication hazard identified in the audit.
    pub(crate) fn reset_frame_state(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.instance_data.clear();
        self.draw_calls.clear();
        self.svg.clear_filter_batches();
        self.shared_elements.clear();
        self.current_texture_id = None;
        self.opacity_stack.clear();
        self.opacity_stack.push(1.0);
        self.clip_stack.clear();
        self.slice_stack.clear();
        self.transform_stack.clear();
        self.portal_regions.clear();
        self.hologram_instances.clear();
        self.current_z = 0.0;
        self.vnode_stack.clear();
        self.event_handlers.clear();
        // P2-13: Always update the volumetric time uniform, even if the
        // volumetric pass is skipped by the frame budget system. This prevents
        // a visible time pop when the pass resumes after being skipped.
        let current_time = self.current_time();
        let resolution = [
            self.current_width() as f32,
            self.current_height() as f32,
        ];
        let time_uniform: [f32; 4] = [
            current_time,
            resolution[0],
            resolution[1],
            0.0, // _pad
        ];
        self.queue.write_buffer(
            &self.volumetric_uniform_buffer,
            0,
            bytemuck::cast_slice(&time_uniform),
        );
        // Clear per-frame state but NOT memo_cache -- use generation counter instead
        self.frame_generation += 1;
        // Evict memo cache entries that are too old to prevent unbounded growth.
        const MAX_MEMO_AGE: u64 = 1000;
        if self.frame_generation > MAX_MEMO_AGE {
            let cutoff = self.frame_generation - MAX_MEMO_AGE;
            self.memo_cache
                .retain(|_, entry| entry.frame_gen >= cutoff);
        }
        self.last_frame_start = std::time::Instant::now();
        self.telemetry.draw_calls = 0;
        self.telemetry.vertices = 0;
    }

    /// begin_frame_headless -- Strike the flaming sword to begin a new GPU frame for headless rendering.
    pub fn begin_frame_headless(&mut self) -> wgpu::CommandEncoder {
        self.current_window = None;
        self.compositor_index_cursor = self.indices.len() as u32;
        self.reset_frame_state();

        // Recall staging belt buffers so they can be reused for vertex upload
        self.staging_belt.recall();

        let ctx = self
            .headless_context
            .as_ref()
            .expect("Headless context not initialized");
        let time = self.start_time.elapsed().as_secs_f32();
        let logical_w = ctx.width as f32 / ctx.scale_factor;
        let logical_h = ctx.height as f32 / ctx.scale_factor;
        let dt = time - self.current_scene.time;
        self.current_scene.time = time;
        self.current_scene.delta_time = dt;
        self.current_scene.resolution = [logical_w, logical_h];
        self.current_scene.scale_factor = ctx.scale_factor;
        self.current_scene.proj =
            glam::Mat4::orthographic_lh(0.0, logical_w, logical_h, 0.0, -1000.0, 1000.0);

        self.queue.write_buffer(
            &self.scene_buffer,
            0,
            bytemuck::bytes_of(&self.current_scene),
        );

        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Surtr Headless Command Encoder"),
            })
    }

    /// begin_frame -- Strike the flaming sword to begin a new GPU frame for a specific window.
    pub fn begin_frame(&mut self, window_id: winit::window::WindowId) -> wgpu::CommandEncoder {
        self.begin_frame_internal(window_id, true)
    }

    /// Begin a frame without resetting per-frame state.
    /// Used when reusing the previous frame's draw calls (view unchanged).
    pub fn begin_frame_reuse(&mut self, window_id: winit::window::WindowId) -> wgpu::CommandEncoder {
        self.begin_frame_internal(window_id, false)
    }

    fn begin_frame_internal(&mut self, window_id: winit::window::WindowId, reset_state: bool) -> wgpu::CommandEncoder {
        // Drain AI material channel
        if let Some(rx) = &self.ai_material_rx {
            while let Ok(res) = rx.try_recv() {
                match res {
                    Ok(_) => log::info!("[Surtr] Received AI generated material"),
                    Err(e) => log::warn!("[Surtr] AI material generation error: {:?}", e),
                }
            }
        }

        // Skuld timestamp query removed — was causing GPU sync stalls (10ms/frame)
        // and buffer mapping errors. GPU time can be profiled externally if needed.

        self.staging_belt.recall();
        self.current_window = Some(window_id);
        if reset_state {
            self.reset_frame_state();
        }

        let ctx = self
            .surfaces
            .get(&window_id)
            .expect("Window not registered");
        let time = self.start_time.elapsed().as_secs_f32();
        let logical_w = ctx.config.width as f32 / ctx.scale_factor;
        let logical_h = ctx.config.height as f32 / ctx.scale_factor;
        let dt = time - self.current_scene.time;
        self.current_scene.time = time;
        self.current_scene.delta_time = dt;
        self.current_scene.resolution = [logical_w, logical_h];
        self.current_scene.scale_factor = ctx.scale_factor;
        self.current_scene.proj =
            glam::Mat4::orthographic_lh(0.0, logical_w, logical_h, 0.0, -1000.0, 1000.0);

        self.queue.write_buffer(
            &self.scene_buffer,
            0,
            bytemuck::bytes_of(&self.current_scene),
        );

        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Surtr Command Encoder"),
            })
    }

    /// register_window -- Attaches a new OS window to the shared GPU context.
    pub fn register_window(&mut self, window: std::sync::Arc<winit::window::Window>) {
        let size = window.inner_size();
        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("Failed to create surface");
        let caps = surface.get_capabilities(&self.adapter);
        let format = caps.formats[0];

        // Dynamic present mode selection -- Mailbox not available on all platforms (e.g. Wayland)
        let present_mode = if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else {
            log::warn!("[GPU] Mailbox not supported, falling back to Fifo (V-Sync)");
            wgpu::PresentMode::Fifo
        };

        let alpha_mode = if caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PostMultiplied)
        {
            wgpu::CompositeAlphaMode::PostMultiplied
        } else if caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
        {
            wgpu::CompositeAlphaMode::PreMultiplied
        } else {
            caps.alpha_modes[0]
        };

        log::info!(
            "[GPU] Configuring surface: {}x{} | {:?} | {:?}",
            size.width,
            size.height,
            present_mode,
            alpha_mode
        );

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };
        surface.configure(&self.device, &config);

        let ctx = Self::create_surface_context(
            &self.device,
            surface,
            config,
            &self.env_bind_group_layout,
            &self.texture_bind_group_layout,
            window.scale_factor() as f32,
            self.quality_level.msaa_sample_count(),
            &mut self.registry,
        );

        self.surfaces.insert(window.id(), ctx);
    }
}