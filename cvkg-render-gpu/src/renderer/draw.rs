use super::GpuRenderer;
use super::context_helpers::create_surface_context;
use crate::types::{DrawCall, MAX_PARTICLES};
use crate::vertex::{InstanceData, Vertex};
use cvkg_core::{Rect, Renderer};
use std::sync::Arc;

impl GpuRenderer {
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

    /// Reset per-frame state shared by both `begin_frame` and `begin_frame_headless`.
    /// Factored out to avoid the copy-paste duplication hazard identified in the audit.
    fn reset_frame_state(&mut self) {
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
        let resolution = [self.current_width() as f32, self.current_height() as f32];
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
            self.memo_cache.retain(|_, entry| entry.frame_gen >= cutoff);
        }
        self.last_frame_start = std::time::Instant::now();
        self.telemetry.draw_calls = 0;
        self.telemetry.vertices = 0;
    }

    /// begin_frame -- Strike the flaming sword to begin a new GPU frame for a specific window.
    pub fn begin_frame(&mut self, window_id: winit::window::WindowId) -> wgpu::CommandEncoder {
        self.begin_frame_internal(window_id, true)
    }

    /// Begin a frame without resetting per-frame state.
    /// Used when reusing the previous frame's draw calls (view unchanged).
    pub fn begin_frame_reuse(
        &mut self,
        window_id: winit::window::WindowId,
    ) -> wgpu::CommandEncoder {
        self.begin_frame_internal(window_id, false)
    }

    fn begin_frame_internal(
        &mut self,
        window_id: winit::window::WindowId,
        reset_state: bool,
    ) -> wgpu::CommandEncoder {
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
    pub fn register_window(&mut self, window: Arc<winit::window::Window>) {
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

        let ctx = create_surface_context(
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

    pub(crate) fn shatter_internal(
        &mut self,
        rect: Rect,
        pieces: u32,
        force: f32,
        color: [f32; 4],
        material_id: u32,
    ) {
        // High-Fidelity Variable Particle Density
        let count = (pieces as f32).sqrt().ceil() as u32;
        let dw = rect.width / count as f32;
        let dh = rect.height / count as f32;

        let c = self.apply_opacity(color);

        let cx = rect.x + rect.width * 0.5;
        let cy = rect.y + rect.height * 0.5;

        for y in 0..count {
            for x in 0..count {
                let init_x = rect.x + x as f32 * dw;
                let init_y = rect.y + y as f32 * dh;

                // Center of the shard relative to the card center
                let dx = (init_x + dw * 0.5) - cx;
                let dy = (init_y + dh * 0.5) - cy;
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);

                // Normal direction outwards
                let nx = dx / dist;
                let ny = dy / dist;

                // Hash-based pseudo-random variations for dispersion
                let hash =
                    ((x as f32 * 12.9898 + y as f32 * 78.233).sin().fract() * 43_758.547).fract();
                let hash2 =
                    ((x as f32 * 37.11 + y as f32 * 149.87).sin().fract() * 23_412.19).fract();

                let speed_var = 0.5 + hash * 1.5;
                let angle = ny.atan2(nx) + (hash2 - 0.5) * 0.6;
                let disp_x = angle.cos() * force * 50.0 * speed_var;
                let disp_y = angle.sin() * force * 50.0 * speed_var;

                // Downward gravity-like drift over time/force
                let gravity = force * force * 20.0;

                // Shrink shard size as it scatters away
                // Assuming max force in demo is ~6.0
                let scale_factor = (1.0 - (force / 6.0).min(1.0)).max(0.0);
                let shard_w = dw * scale_factor;
                let shard_h = dh * scale_factor;

                let displaced_x = init_x + disp_x + (dw - shard_w) * 0.5;
                let displaced_y = init_y + disp_y + gravity + (dh - shard_h) * 0.5;

                let shard_rect = Rect {
                    x: displaced_x,
                    y: displaced_y,
                    width: shard_w,
                    height: shard_h,
                };

                let uv = Rect {
                    x: x as f32 / count as f32,
                    y: y as f32 / count as f32,
                    width: 1.0 / count as f32,
                    height: 1.0 / count as f32,
                };

                self.fill_rect_with_full_params(shard_rect, c, material_id, None, force, uv);
            }
        }
    }

    pub(crate) fn recursive_bolt(
        &mut self,
        from: [f32; 2],
        to: [f32; 2],
        depth: u32,
        color: [f32; 4],
    ) {
        if depth == 0 {
            self.draw_lightning_segment(from, to, color);
            return;
        }

        let mid_x = (from[0] + to[0]) * 0.5;
        let mid_y = (from[1] + to[1]) * 0.5;

        let dx = to[0] - from[0];
        let dy = to[1] - from[1];
        let len = (dx * dx + dy * dy).sqrt();

        if len < 1e-4 {
            return;
        }

        // Perpendicular offset for jaggedness
        let offset_scale = len * 0.15;
        let seed = (from[0] * 12.9898 + from[1] * 78.233 + (depth as f32) * 37.11)
            .sin()
            .fract();
        let offset_x = -dy / len * (seed - 0.5) * offset_scale;
        let offset_y = dx / len * (seed - 0.5) * offset_scale;

        let mid = [mid_x + offset_x, mid_y + offset_y];

        self.recursive_bolt(from, mid, depth - 1, color);
        self.recursive_bolt(mid, to, depth - 1, color);

        // 20% chance of a secondary branch
        if depth > 2 && seed > 0.8 {
            let branch_to = [
                mid[0] + offset_x * 2.0 + (seed * 100.0).sin() * 50.0,
                mid[1] + offset_y * 2.0 + (seed * 100.0).cos() * 50.0,
            ];
            self.recursive_bolt(mid, branch_to, depth - 2, color);
        }
    }

    pub(crate) fn draw_lightning_segment(&mut self, from: [f32; 2], to: [f32; 2], color: [f32; 4]) {
        let dx = to[0] - from[0];
        let dy = to[1] - from[1];
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 {
            return;
        }

        let glow_width = 32.0;
        let core_width = 4.0;
        let c = self.apply_opacity(color);

        // 1. Render Volumetric Glow (Cyan)
        let gnx = -dy / len * glow_width * 0.5;
        let gny = dx / len * glow_width * 0.5;
        let gp1 = [from[0] + gnx, from[1] + gny];
        let gp2 = [to[0] + gnx, to[1] + gny];
        let gp3 = [to[0] - gnx, to[1] - gny];
        let gp4 = [from[0] - gnx, from[1] - gny];
        self.push_oriented_quad(
            [gp1, gp2, gp3, gp4],
            c,
            9,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );

        // 2. Render Blinding Core (White)
        let cnx = -dy / len * core_width * 0.5;
        let cny = dx / len * core_width * 0.5;
        let cp1 = [from[0] + cnx, from[1] + cny];
        let cp2 = [to[0] + cnx, to[1] + cny];
        let cp3 = [to[0] - cnx, to[1] - cny];
        let cp4 = [from[0] - cnx, from[1] - cny];
        self.push_oriented_quad(
            [cp1, cp2, cp3, cp4],
            [1.0, 1.0, 1.0, c[3]],
            0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
    }

    pub(crate) fn push_oriented_quad(
        &mut self,
        points: [[f32; 2]; 4],
        color: [f32; 4],
        material_id: u32,
        uv_rect: Rect,
    ) {
        let scissor = self.clip_stack.last().copied();
        let texture_id = None; // Oriented quads like lightning don't use textures yet

        let (translation, scale_transform, rotation, _, _) = self.current_transform();
        let current_instance_data = InstanceData {
            translation,
            scale: scale_transform,
            rotation,
            blur_radius: 0.0,
            ior_override: 0.0,
            glass_intensity: 1.0,
        };

        // CRITICAL FIX: Only break batch on material/scissor/texture state changes.
        // Transform (translation/scale/rotation) is per-instance data.
        let material =
            Self::resolve_material_with_context(material_id, &self.current_draw_material);
        let final_material_id = match material {
            cvkg_core::DrawMaterial::Opaque => material_id,
            cvkg_core::DrawMaterial::TopUI => crate::renderer::material_id::TOP_UI,
            cvkg_core::DrawMaterial::Glass { .. } => crate::renderer::material_id::GLASS,
            cvkg_core::DrawMaterial::Blend { mode } => 7 + mode,
        };

        let last_call = self.draw_calls.last();
        let needs_new_call = self.draw_calls.is_empty()
            || self.current_texture_id != texture_id
            || last_call.unwrap().scissor_rect != scissor
            || last_call.unwrap().material != material
            || {
                let last_material = last_call.unwrap().material;
                matches!((material, last_material),
                    (cvkg_core::DrawMaterial::Glass { blur_radius: a, ior_override: b, glass_intensity: c },
                     cvkg_core::DrawMaterial::Glass { blur_radius: d, ior_override: e, glass_intensity: f })
                    if a != d || b != e || c != f)
            };

        if needs_new_call {
            self.current_texture_id = texture_id;
            self.instance_data.push(current_instance_data);
            self.draw_calls.push(DrawCall {
                target_id: None,
                texture_id,
                scissor_rect: scissor,
                index_start: self.indices.len() as u32,
                index_count: 0,
                instance_count: 1,
                material,
                instance_start: (self.instance_data.len() - 1) as u32,
                draw_order: 0,
            });
        } else {
            // Same batch - add instance data and increment instance count
            self.instance_data.push(current_instance_data);
            if let Some(call) = self.draw_calls.last_mut() {
                call.instance_count += 1;
            }
        }

        let uvs = [
            [uv_rect.x, uv_rect.y],
            [uv_rect.x + uv_rect.width, uv_rect.y],
            [uv_rect.x + uv_rect.width, uv_rect.y + uv_rect.height],
            [uv_rect.x, uv_rect.y + uv_rect.height],
        ];

        let rect = Rect {
            x: points[0][0],
            y: points[0][1],
            width: 1.0,
            height: 1.0,
        };

        for i in 0..4 {
            let px = points[i][0];
            let py = points[i][1];

            self.vertices.push(Vertex {
                position: [px, py, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: uvs[i],
                color,
                material_id: final_material_id,
                radius: 0.0,
                slice: [0.0, 0.0, 0.0, 1.0],
                logical: [px - rect.x, py - rect.y],
                size: [rect.width, rect.height],
                clip: [-f32::INFINITY, -f32::INFINITY, f32::INFINITY, f32::INFINITY],
                tex_index: 0,
            });
        }

        // Push indices for the quad (two triangles: 0-1-2 and 0-2-3)
        let base = self.vertices.len() as u32 - 4;
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        if let Some(call) = self.draw_calls.last_mut() {
            call.index_count += 6;
        }
    }

    pub(crate) fn get_texture_id(&mut self, name: &str) -> Option<u32> {
        self.texture_registry.get(name).copied()
    }

    /// fill_rect_with_mode -- Specialized rectangle drawing with mode-specific shader logic.
    pub fn fill_rect_with_mode(
        &mut self,
        rect: Rect,
        color: [f32; 4],
        material_id: u32,
        texture_id: Option<u32>,
    ) {
        self.fill_rect_with_full_params(
            rect,
            color,
            material_id,
            texture_id,
            0.0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
    }

    pub(crate) fn fill_rect_with_full_params(
        &mut self,
        rect: Rect,
        color: [f32; 4],
        material_id: u32,
        texture_id: Option<u32>,
        radius: f32,
        uv_rect: Rect,
    ) {
        // If a shadow is active, draw it first, offset by shadow._offset
        if let Some(shadow) = self.shadow_stack.last().copied()
            && shadow.color[3] > 0.001
        {
            let shadow_rect = Rect {
                x: rect.x + shadow._offset[0],
                y: rect.y + shadow._offset[1],
                width: rect.width,
                height: rect.height,
            };
            Renderer::draw_drop_shadow(
                self,
                shadow_rect,
                radius,
                shadow.color,
                shadow.radius,
                0.0, // Spread
            );
        }

        let slice = self
            .slice_stack
            .last()
            .copied()
            .map(|(a, o)| [a, o, 1.0, 1.0])
            .unwrap_or([0.0, 0.0, 0.0, 1.0]);
        self.fill_rect_with_full_params_and_slice(
            rect,
            color,
            material_id,
            texture_id,
            radius,
            uv_rect,
            slice,
            [0.0, 0.0],
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn fill_rect_with_full_params_and_slice(
        &mut self,
        mut rect: Rect,
        color: [f32; 4],
        material_id: u32,
        texture_id: Option<u32>,
        radius: f32,
        uv_rect: Rect,
        slice: [f32; 4],
        _glyph_time: [f32; 2],
    ) {
        // Pixel-snap rect coordinates to prevent sub-pixel blurring on high-DPI displays.
        // Only snap for non-glass materials where visual crispness matters.
        if material_id != crate::renderer::material_id::GLASS {
            let scale = self.current_scale_factor();
            let snap = |v: f32| (v * scale).round() / scale;
            rect.x = snap(rect.x);
            rect.y = snap(rect.y);
            rect.width = snap(rect.width);
            rect.height = snap(rect.height);
        }

        let scissor = self.clip_stack.last().copied();

        let material =
            Self::resolve_material_with_context(material_id, &self.current_draw_material);
        let final_material_id = match material {
            cvkg_core::DrawMaterial::Opaque => material_id,
            cvkg_core::DrawMaterial::TopUI => crate::renderer::material_id::TOP_UI,
            cvkg_core::DrawMaterial::Glass { .. } => crate::renderer::material_id::GLASS,
            cvkg_core::DrawMaterial::Blend { mode } => 7 + mode,
        };

        let (translation, scale_transform, rotation, _, _) = self.current_transform();
        let (blur_radius, ior_override, glass_intensity) = if let cvkg_core::DrawMaterial::Glass {
            blur_radius,
            ior_override,
            glass_intensity,
        } = material
        {
            (blur_radius, ior_override, glass_intensity)
        } else {
            (0.0, 0.0, 1.0)
        };

        let current_instance_data = InstanceData {
            translation,
            scale: scale_transform,
            rotation,
            blur_radius,
            ior_override,
            glass_intensity,
        };

        // Batching: check if we need to start a new DrawCall
        // With Texture Array, we no longer need to break batches when the texture changes,
        // as long as they are all part of the same array bind group (Group 0).
        // CRITICAL FIX: Only break batch on material/scissor/blur/glass state changes.
        // Transform (translation/scale/rotation) is per-instance data and should NOT
        // break the batch - multiple instances with different transforms can share a draw call.
        let last_call = self.draw_calls.last();
        let needs_new_call = self.draw_calls.is_empty()
            || last_call.unwrap().scissor_rect != scissor
            || last_call.unwrap().material != material
            || last_call.unwrap().texture_id != self.current_texture_id
            || {
                // Check if glass/blur state changed (these require pipeline changes)
                let last_material = last_call.unwrap().material;
                matches!((material, last_material),
                    (cvkg_core::DrawMaterial::Glass { blur_radius: a, ior_override: b, glass_intensity: c },
                     cvkg_core::DrawMaterial::Glass { blur_radius: d, ior_override: e, glass_intensity: f })
                    if a != d || b != e || c != f)
            };

        if needs_new_call {
            self.current_texture_id = Some(0); // All textures are now in the binding array at Group 0
            self.instance_data.push(current_instance_data);
            self.draw_calls.push(DrawCall {
                target_id: None,
                texture_id: self.current_texture_id,
                scissor_rect: scissor,
                index_start: self.indices.len() as u32,
                index_count: 0,
                instance_count: 1,
                material,
                instance_start: (self.instance_data.len() - 1) as u32,
                draw_order: 0,
            });
        } else {
            // Same batch - add instance data and increment instance count
            self.instance_data.push(current_instance_data);
            if let Some(call) = self.draw_calls.last_mut() {
                call.instance_count += 1;
            }
        }

        let scale = self.current_scale_factor();
        let snap = |v: f32| (v * scale).round() / scale;

        let base_idx = self.vertices.len() as u32;
        let x1 = snap(rect.x);
        let y1 = snap(rect.y);
        let x2 = snap(rect.x + rect.width);
        let y2 = snap(rect.y + rect.height);
        let z = self.current_z;
        let normal = [0.0, 0.0, 1.0];
        let clip_rect = self.clip_stack.last().copied().unwrap_or(cvkg_core::Rect {
            x: -10000.0,
            y: -10000.0,
            width: 20000.0,
            height: 20000.0,
        });
        let clip = [clip_rect.x, clip_rect.y, clip_rect.width, clip_rect.height];

        let tex_index = texture_id.unwrap_or(0);

        self.vertices.push(Vertex {
            position: [x1, y1, z],
            normal,
            uv: [uv_rect.x, uv_rect.y],
            color,
            material_id: final_material_id,
            radius,
            slice,
            logical: [0.0, 0.0],
            size: [rect.width, rect.height],
            clip,
            tex_index,
        });
        self.vertices.push(Vertex {
            position: [x2, y1, z],
            normal,
            uv: [uv_rect.x + uv_rect.width, uv_rect.y],
            color,
            material_id: final_material_id,
            radius,
            slice,
            logical: [rect.width, 0.0],
            size: [rect.width, rect.height],
            clip,
            tex_index,
        });
        self.vertices.push(Vertex {
            position: [x2, y2, z],
            normal,
            uv: [uv_rect.x + uv_rect.width, uv_rect.y + uv_rect.height],
            color,
            material_id: final_material_id,
            radius,
            slice,
            logical: [rect.width, rect.height],
            size: [rect.width, rect.height],
            clip,
            tex_index,
        });
        self.vertices.push(Vertex {
            position: [x1, y2, z],
            normal,
            uv: [uv_rect.x, uv_rect.y + uv_rect.height],
            color,
            material_id: final_material_id,
            radius,
            slice,
            logical: [0.0, rect.height],
            size: [rect.width, rect.height],
            clip,
            tex_index,
        });

        self.indices.extend_from_slice(&[
            base_idx,
            base_idx + 1,
            base_idx + 2,
            base_idx,
            base_idx + 2,
            base_idx + 3,
        ]);

        if let Some(call) = self.draw_calls.last_mut() {
            call.index_count += 6;
        }
    }

    /// Pass 1: Clear scene+depth, draw atmosphere, draw opaque geometry.
    /// end_frame -- Quench the blade by submitting the full Muspelheim multi-pass effect.
    ///
    /// Since the Renderer 3.0 migration, the pass sequence is driven by a Kvasir
    /// dependency graph rather than hardcoded ordering. The graph is built each
    /// frame (cheap -- just node/edge allocation), validated (cycle detection,
    /// input satisfiability), then executed. Conditional passes (glass, bloom,
    /// accessibility) are automatically eliminated when not needed.
    pub fn end_frame(&mut self, mut encoder: wgpu::CommandEncoder) {
        struct ActiveFrameResources {
            surface_texture: Option<wgpu::SurfaceTexture>,
            target_view: wgpu::TextureView,
            scene_texture: wgpu::TextureView,
            scene_msaa_texture: wgpu::TextureView,
            depth_texture_view: wgpu::TextureView,
            blur_env_bind_group_a: wgpu::BindGroup,
            blur_env_bind_group_b: wgpu::BindGroup,
            bloom_env_bind_group_a: wgpu::BindGroup,
            bloom_env_bind_group_b: wgpu::BindGroup,
        }

        let res = if let Some(window_id) = self.current_window {
            let Some(ctx) = self.surfaces.get(&window_id) else {
                log::error!("[GPU] Missing surface context for end_frame");
                return;
            };
            let frame = match ctx.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(t) => t,
                wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                    ctx.surface.configure(&self.device, &ctx.config);
                    t
                }
                other => {
                    log::warn!(
                        "[GPU] Surface texture acquisition failed ({:?}), reconfiguring surface",
                        other
                    );
                    ctx.surface.configure(&self.device, &ctx.config);
                    // Retry once after reconfiguration; if it fails again, skip the frame.
                    match ctx.surface.get_current_texture() {
                        wgpu::CurrentSurfaceTexture::Success(t) => t,
                        wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                            ctx.surface.configure(&self.device, &ctx.config);
                            t
                        }
                        retry_failed => {
                            log::error!(
                                "[GPU] Surface texture retry also failed ({:?}), skipping frame",
                                retry_failed
                            );
                            self.queue.submit(std::iter::once(encoder.finish()));
                            return;
                        }
                    }
                }
            };
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            ActiveFrameResources {
                surface_texture: Some(frame),
                target_view: view,
                scene_texture: ctx.scene_texture.clone(),
                scene_msaa_texture: ctx.scene_msaa_texture.clone(),
                depth_texture_view: ctx.depth_texture_view.clone(),
                blur_env_bind_group_a: ctx.blur_env_bind_group_a.clone(),
                blur_env_bind_group_b: ctx.blur_env_bind_group_b.clone(),
                bloom_env_bind_group_a: ctx.bloom_env_bind_group_a.clone(),
                bloom_env_bind_group_b: ctx.bloom_env_bind_group_b.clone(),
            }
        } else {
            let Some(ctx) = self.headless_context.as_ref() else {
                log::error!("[GPU] No headless context for end_frame");
                return;
            };

            ActiveFrameResources {
                surface_texture: None,
                target_view: ctx.output_view.clone(),
                scene_texture: ctx.scene_texture.clone(),
                scene_msaa_texture: ctx.scene_msaa_texture.clone(),
                depth_texture_view: ctx.depth_texture_view.clone(),
                blur_env_bind_group_a: ctx.blur_env_bind_group_a.clone(),
                blur_env_bind_group_b: ctx.blur_env_bind_group_b.clone(),
                bloom_env_bind_group_a: ctx.bloom_env_bind_group_a.clone(),
                bloom_env_bind_group_b: ctx.bloom_env_bind_group_b.clone(),
            }
        };

        // Auto-flush staging belt if render_frame() was not called but geometry was queued.
        // This ensures apps that forget render_frame() still see their draw calls rendered.
        if !self.frame_rendered && (!self.vertices.is_empty() || !self.indices.is_empty()) {
            log::debug!(
                "[GPU] Auto-flushing staging belt in end_frame (render_frame was not called)"
            );
            let mut staging_encoder =
                self.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Surtr Auto-Flush Staging Encoder"),
                    });
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
            }
            self.staging_belt.finish();
            self.staging_command_buffers.push(staging_encoder.finish());
        }

        // ── Build and execute the Kvasir frame graph ─────────────────────────────
        let has_glass = self
            .draw_calls
            .iter()
            .any(|c| matches!(c.material, cvkg_core::DrawMaterial::Glass { .. }));
        let has_bloom = self.bloom_enabled;
        let has_accessibility =
            self.color_blind_mode != crate::color_blindness::ColorBlindMode::Normal;

        // Build the frame graph using the Kvasir helper for correct pass ordering.
        // Conditional passes (glass, bloom, accessibility) are included/excluded based on frame state.
        // This replaces the hardcoded if/else pass dispatch with a data-driven approach:
        // the graph declares which passes exist and their ordering, and we execute only enabled ones.
        //
        // NOTE: Geometry is uploaded by render_frame() via StagingBelt into staging_command_buffers.
        // Those staging commands must be submitted before the render pass encoders below, which is
        // guaranteed by inserting the render encoders after the existing staging entries (see submit block).

        let (blur_id, bloom_id) = if let Some(window_id) = self.current_window {
            let ctx = self.surfaces.get(&window_id).unwrap();
            (ctx.blur_tex_a, ctx.bloom_tex_a)
        } else {
            let ctx = self.headless_context.as_ref().unwrap();
            (ctx.blur_tex_a, ctx.bloom_tex_a)
        };
        self.registry
            .alias(crate::kvasir::nodes::RES_BLUR_A, blur_id);
        self.registry
            .alias(crate::kvasir::nodes::RES_BLOOM_A, bloom_id);
        self.registry
            .alias_view(crate::kvasir::nodes::RES_SCENE, res.scene_texture.clone());
        self.registry.alias_view(
            crate::kvasir::nodes::RES_SCENE_MSAA,
            res.scene_msaa_texture.clone(),
        );

        let scale = self.current_scale_factor();
        let scale_bits = scale.to_bits();
        let active_offscreens_count = self.active_offscreens.len();
        let portal_regions_count = self.portal_regions.len();
        let width = self.current_width();
        let height = self.current_height();
        let has_volumetric = self.volumetric_enabled;

        // Compute content hashes for cache key (must match construction site)
        let mut offscreen_hash: u64 = 0;
        for offscreen in &self.active_offscreens {
            offscreen_hash = offscreen_hash.wrapping_add(
                offscreen.target_id.wrapping_mul(31)
                    ^ (offscreen.blend_mode as u64).wrapping_mul(17),
            );
        }
        let mut portal_hash: u64 = 0;
        for region in &self.portal_regions {
            portal_hash = portal_hash.wrapping_add(
                (region.x.to_bits() as u64)
                    .wrapping_mul(7)
                    .wrapping_add((region.y.to_bits() as u64).wrapping_mul(13))
                    .wrapping_add((region.width.to_bits() as u64).wrapping_mul(19))
                    .wrapping_add((region.height.to_bits() as u64).wrapping_mul(23)),
            );
        }

        let use_cache = if let Some(ref cached) = self.cached_graph_plan {
            cached.matches(
                has_glass,
                has_bloom,
                has_accessibility,
                has_volumetric,
                active_offscreens_count,
                offscreen_hash,
                portal_regions_count,
                portal_hash,
                width,
                height,
                scale_bits,
                self.material_compilation_hash,
            )
        } else {
            false
        };

        if !use_cache {
            let render_graph = crate::kvasir::nodes::build_render_graph(
                &crate::kvasir::nodes::RenderGraphConfig {
                    has_glass,
                    has_bloom,
                    has_accessibility,
                    has_volumetric,
                    active_offscreens: &self.active_offscreens,
                    portal_regions: &self.portal_regions.iter().cloned().collect::<Vec<_>>(),
                    width,
                    height,
                    scale,
                },
            );
            let planner = crate::kvasir::planner::ExecutionPlanner::new(&render_graph);
            let compiled_plan = match planner.compile() {
                Ok(plan) => plan,
                Err(e) => {
                    log::error!(
                        "[Kvasir] Render graph compilation failed ({}), skipping render passes",
                        e
                    );
                    // Present the frame with whatever was rendered (stale scene or blank).
                    if let Some(surface_texture) = res.surface_texture {
                        surface_texture.present();
                        log::info!("[Surtr] Frame presented (graph compilation fallback)");
                    }
                    return;
                }
            };

            // Reuse the already-computed hashes (computed above for cache matching)
            self.cached_graph_plan = Some(crate::kvasir::graph_cache::CachedGraphPlan {
                has_glass,
                has_bloom,
                has_accessibility,
                has_volumetric,
                active_offscreens_count,
                offscreen_content_hash: offscreen_hash,
                portal_regions_count,
                portal_content_hash: portal_hash,
                width,
                height,
                scale_bits,
                material_compilation_hash: self.material_compilation_hash,
                graph: render_graph,
                plan: compiled_plan,
            });
        }

        let cached = self.cached_graph_plan.as_ref().unwrap();
        let frame_start = self.last_frame_start;
        let budget_ms = self.frame_budget.target_ms;
        let allow_degradation = self.frame_budget.allow_degradation;

        for &node_key in &cached.plan {
            // Frame budget enforcement: if we're already over budget and degradation
            // is allowed, skip expensive COSMETIC passes (bloom, volumetric).
            //
            // P0-2 fix: BackdropBlur, BackdropRegion, and Accessibility are FUNCTIONAL
            // passes, not cosmetic effects:
            //   * BackdropBlur/BackdropRegion implement glassmorphism (frosted glass
            //     panels, modals, sidebars). Skipping them makes glass elements
            //     render as opaque solid rectangles, breaking the visual contract
            //     for any app using glass materials.
            //   * Accessibility is required for screen readers and other AT;
            //     skipping it makes the UI unusable for visually-impaired users.
            // Only BloomExtract/BloomBlur (post-processing glow) and Volumetric
            // (raymarched lighting) are true cosmetics and safe to degrade.
            if allow_degradation && budget_ms > 0.0 {
                let elapsed_ms = frame_start.elapsed().as_secs_f32() * 1000.0;
                if elapsed_ms > budget_ms
                    && let Some(node) = cached.graph.node(node_key)
                {
                    match node.pass_id() {
                        crate::kvasir::nodes::PassId::BloomExtract
                        | crate::kvasir::nodes::PassId::BloomBlur
                        | crate::kvasir::nodes::PassId::Volumetric => {
                            log::trace!(
                                "[Kvasir] Skipping {} (over budget: {:.1}ms > {:.1}ms)",
                                node.label(),
                                elapsed_ms,
                                budget_ms
                            );
                            continue;
                        }
                        _ => {} // Always run: Glass, BackdropBlur, BackdropRegion,
                                // Accessibility, Geometry, UI, Composite, Present, ...
                    }
                }
            }
            if let Some(node) = cached.graph.node(node_key) {
                log::trace!("[Kvasir] Executing node: {}", node.label());
                let mut ctx = crate::kvasir::node::ExecutionContext {
                    device: &self.device,
                    queue: &self.queue,
                    encoder: &mut encoder,
                    registry: &self.registry,
                    renderer: self,
                    target_view: &res.target_view,
                    depth_view: &res.depth_texture_view,
                    blur_env_bind_group_a: &res.blur_env_bind_group_a,
                    blur_env_bind_group_b: &res.blur_env_bind_group_b,
                    bloom_env_bind_group_a: &res.bloom_env_bind_group_a,
                    bloom_env_bind_group_b: &res.bloom_env_bind_group_b,
                    scale_factor: scale,
                };
                node.execute(&mut ctx);
            }
        }

        // ── Particle Compute Pass ──────────────────────────────────────────
        // Flush staged particles to GPU, then run compute integration.
        // Must run BEFORE the submit so particle positions are up-to-date.
        if !self.particles.staging.is_empty() || self.particles.count > 0 {
            // 1. Flush staged particles into the ring buffer
            if !self.particles.staging.is_empty() {
                let write_start = self.particles.write_head as usize;
                let write_count = self.particles.staging.len();
                let max = MAX_PARTICLES;

                // P1-6 fix: cap the write to max particles to prevent
                // wrap-around overlap. If write_count > max, only the
                // LAST `max` particles are kept (the most recent ones
                // are most relevant for particle effects, and the
                // earlier ones are dropped). Without this cap, if
                // write_count > max - write_start, the second chunk
                // would write past offset 0 and overlap the first
                // chunk, corrupting the buffer.
                let effective_count = write_count.min(max);
                let drop_count = write_count - effective_count;

                // Write particles in ring-buffer fashion
                let first_chunk = (max - write_start).min(effective_count);
                let bytes = bytemuck::cast_slice(
                    &self.particles.staging[drop_count..drop_count + first_chunk],
                );
                self.queue.write_buffer(
                    &self.particle_buffer,
                    (write_start * std::mem::size_of::<crate::types::GpuParticle>()) as u64,
                    bytes,
                );
                if first_chunk < effective_count {
                    let remaining = effective_count - first_chunk;
                    let bytes2 = bytemuck::cast_slice(
                        &self.particles.staging
                            [drop_count + first_chunk..drop_count + first_chunk + remaining],
                    );
                    self.queue.write_buffer(&self.particle_buffer, 0, bytes2);
                    self.particles.write_head = remaining as u32;
                } else {
                    self.particles.write_head = ((write_start + effective_count) % max) as u32;
                }
                self.particles.count =
                    (self.particles.count as usize + effective_count).min(max) as u32;
                self.particles.staging.clear();

                // Invalidate render bind group so it's recreated with new data
                self.particle_render_bind_group = None;
            }

            // 2. Run compute pass to integrate particle physics
            let dt = self.current_scene.delta_time;
            let uniforms = crate::types::ParticleUniforms { dt, _pad: [0.0; 7] };
            self.queue.write_buffer(
                &self.particle_uniform_buffer,
                0,
                bytemuck::bytes_of(&uniforms),
            );

            let compute_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Particle Compute BG"),
                layout: &self.particle_compute_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.particle_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.particle_uniform_buffer.as_entire_binding(),
                    },
                ],
            });

            let mut compute_encoder =
                self.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Particle Compute Encoder"),
                    });
            {
                let mut cpass = compute_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Particle Integration"),
                    ..Default::default()
                });
                cpass.set_pipeline(&self.particle_compute_pipeline);
                cpass.set_bind_group(0, &compute_bind_group, &[]);
                let workgroups = self.particles.count.div_ceil(64).max(1);
                cpass.dispatch_workgroups(workgroups, 1, 1);
            }
            self.staging_command_buffers.push(compute_encoder.finish());
        }

        // 3. Compact dead particles periodically (every 2 seconds)
        if self.particles.count > 0 && self.particles.last_compact.elapsed().as_secs_f32() > 2.0 {
            self.particles.last_compact = std::time::Instant::now();
            // Read back particle data to compact dead particles
            let read_size = (self.particles.count as usize
                * std::mem::size_of::<crate::types::GpuParticle>())
                as u64;
            let staging_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Particle Compact Staging"),
                size: read_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            let mut compact_encoder =
                self.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Particle Compact Copy"),
                    });
            compact_encoder.copy_buffer_to_buffer(
                &self.particle_buffer,
                0,
                &staging_buf,
                0,
                read_size,
            );
            self.staging_command_buffers.push(compact_encoder.finish());
            // Note: full GPU readback is expensive; in production we'd use a
            // compute compaction pass. For now, dead particles are simply
            // overwritten by new ones in the ring buffer (lifetime <= 0 causes
            // the vertex shader to output degenerate points behind the camera).
        }

        // ── Particle Render Pass ────────────────────────────────────────────
        // Render live particles as colored points to the swapchain target,
        // composited on top of the scene with additive blending.
        if self.particles.count > 0 {
            // Lazily (re)create the render bind group when staging changed
            if self.particle_render_bind_group.is_none() {
                self.particle_render_bind_group =
                    Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Particle Render BG"),
                        layout: &self.particle_render_bgl,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.particle_buffer.as_entire_binding(),
                        }],
                    }));
            }
            if let Some(bg) = &self.particle_render_bind_group {
                let mut render_encoder =
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Particle Render Encoder"),
                        });
                {
                    let mut rpass = render_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Particle Render"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &res.target_view,
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
                    rpass.set_pipeline(&self.particle_render_pipeline);
                    rpass.set_bind_group(0, bg, &[]);
                    rpass.draw(0..self.particles.count, 0..1);
                }
                self.staging_command_buffers.push(render_encoder.finish());
            }
        }

        // ── Submit ─────────────────────────────────────────────────────────────
        // staging_command_buffers already contains the geometry upload encoder from
        // render_frame() (StagingBelt). The render pass encoders must come AFTER it
        // so the GPU sees vertex/index data before the draw calls that reference it.
        self.staging_command_buffers.push(encoder.finish());

        // Skuld: Resolve timestamps (preserved from original)
        if let (Some(q), Some(b), Some(rb)) = (
            &self.skuld_queries,
            &self.skuld_buffer,
            &self.skuld_read_buffer,
        ) {
            let mut resolve_encoder =
                self.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Skuld Resolve Encoder"),
                    });
            resolve_encoder.resolve_query_set(q, 0..2, b, 0);
            resolve_encoder.copy_buffer_to_buffer(b, 0, rb, 0, 16);
            self.staging_command_buffers.push(resolve_encoder.finish());
        }

        let cmds = std::mem::take(&mut self.staging_command_buffers);
        self.queue.submit(cmds);
        self.telemetry.frame_time_ms = self.last_frame_start.elapsed().as_secs_f32() * 1000.0;
        self.update_vram_telemetry();

        // Evict transient frame resources (portal regions, offscreen effects) back into
        // the texture pool instead of leaking GPU memory when panels are closed.
        self.registry.evict_frame_resources();

        if let Some(f) = res.surface_texture {
            f.present();
            log::info!("[Surtr] Frame presented");
        }
    }

    /// Submit pre-routed draw command buckets from the cvkg-compositor.
    ///
    /// Accepts `CommandBuckets` produced by `CompositorEngine::flatten_and_route()`
    /// and submits draw calls in the correct pass order for the Backdrop Capture
    /// Architecture:
    /// 1. Scene commands (opaque) → Scene Capture pass
    /// 2. Glass commands → Material Composite pass (samples blur pyramid)
    /// 3. Overlay commands → Top-Level Foreground pass
    pub fn submit_buckets(&mut self, buckets: &cvkg_compositor::CommandBuckets) {
        // Scene pass -- opaque draw calls, sorted by (z_index, draw_order)
        let mut active_offscreens = Vec::new();
        let mut current_target_id = None;

        // Collect and sort scene commands by (z_index, draw_order) for correct painter's order.
        let mut sorted_scene: Vec<_> = buckets.scene_commands.iter().collect();
        sorted_scene.sort_by_key(|cmd| match cmd {
            cvkg_compositor::engine::RenderCommand::Draw(routed) => {
                (routed.z_index as i64, routed.draw_order as i64)
            }
            _ => (0, 0),
        });

        for cmd in sorted_scene {
            match cmd {
                cvkg_compositor::engine::RenderCommand::Draw(routed) => {
                    self.set_material(cvkg_core::DrawMaterial::Opaque);
                    self.submit_routed(routed, current_target_id);
                }
                cvkg_compositor::engine::RenderCommand::PushOffscreen {
                    source_layer,
                    material,
                    bounds,
                } => {
                    current_target_id = Some(source_layer.0);

                    // Pre-allocate the texture
                    let width = (bounds.width).max(1.0) as u32;
                    let height = (bounds.height).max(1.0) as u32;
                    self.registry
                        .allocate_offscreen(&self.device, source_layer.0, [width, height]);

                    if let cvkg_compositor::Material::ShaderEffect {
                        effect_name,
                        params_json: _,
                        ..
                    } = material
                    {
                        active_offscreens.push(crate::types::OffscreenEffectConfig {
                            target_id: source_layer.0,
                            effect: effect_name.clone(),
                            blend_mode: 0,          // Default blend
                            effect_args: [0.0; 16], // Need to parse params_json
                        });
                    }
                }
                cvkg_compositor::engine::RenderCommand::PopOffscreen => {
                    current_target_id = None;
                }
            }
        }
        self.active_offscreens = active_offscreens;

        // Glass pass -- glassmorphism draw calls sampling blur pyramid
        let mut sorted_glass: Vec<_> = buckets.glass_commands.iter().collect();
        sorted_glass.sort_by_key(|cmd| match cmd {
            cvkg_compositor::engine::RenderCommand::Draw(routed) => {
                (routed.z_index as i64, routed.draw_order as i64)
            }
            _ => (0, 0),
        });
        for cmd in sorted_glass {
            if let cvkg_compositor::engine::RenderCommand::Draw(routed) = cmd {
                self.set_material(Self::convert_compositor_material(&routed.material));
                self.submit_routed(routed, None);
            }
        }

        // Overlay pass -- foreground UI (crisp text, icons, edge lighting)
        let mut sorted_overlay: Vec<_> = buckets.overlay_commands.iter().collect();
        sorted_overlay.sort_by_key(|cmd| match cmd {
            cvkg_compositor::engine::RenderCommand::Draw(routed) => {
                (routed.z_index as i64, routed.draw_order as i64)
            }
            _ => (0, 0),
        });
        for cmd in sorted_overlay {
            if let cvkg_compositor::engine::RenderCommand::Draw(routed) = cmd {
                self.set_material(cvkg_core::DrawMaterial::TopUI);
                self.submit_routed(routed, None);
            }
        }
    }

    /// Submit a single routed draw command through the internal pipeline.
    pub(crate) fn submit_routed(
        &mut self,
        routed: &cvkg_compositor::RoutedDrawCommand,
        target_id: Option<u64>,
    ) {
        let cmd = &routed.command;
        if cmd.index_count == 0 {
            return;
        }
        let material = Self::convert_compositor_material(&routed.material);
        self.draw_calls.push(DrawCall {
            texture_id: cmd.texture_id,
            scissor_rect: cmd.scissor_rect,
            index_start: cmd.index_start,
            index_count: cmd.index_count,
            instance_count: 1,
            material,
            target_id,
            instance_start: cmd.instance_id,
            draw_order: 0,
        });
    }

    /// Returns the current effective opacity (product of all stacked values).
    pub(crate) fn apply_opacity(&self, mut color: [f32; 4]) -> [f32; 4] {
        if let Some(&alpha) = self.opacity_stack.last() {
            color[3] *= alpha;
        }
        color
    }

    /// Resolve a material_id to DrawMaterial with default parameters.
    /// Used by draw_svg which doesn't have a current_draw_material context.
    pub(crate) fn resolve_material(material_id: u32) -> cvkg_core::DrawMaterial {
        Self::resolve_material_with_context(material_id, &cvkg_core::DrawMaterial::Opaque)
    }

    /// Resolve a material_id to DrawMaterial, using current_draw_material as context
    /// for glass parameters. Centralizes the material routing logic used by both
    /// fill_rect_with_full_params_and_slice and emit_draw_call.
    pub(crate) fn resolve_material_with_context(
        material_id: u32,
        current: &cvkg_core::DrawMaterial,
    ) -> cvkg_core::DrawMaterial {
        use crate::renderer::material_id::*;

        // If current context is TopUI, route all non-glass elements to the overlay pass.
        // This ensures dropdowns, popovers, and menus render crisp text/shapes on top of other content.
        if matches!(current, cvkg_core::DrawMaterial::TopUI) && material_id != GLASS {
            return cvkg_core::DrawMaterial::TopUI;
        }

        // If current context has an active Blend mode, route standard opaque quads to that Blend mode.
        if let cvkg_core::DrawMaterial::Blend { mode } = current
            && material_id == 0
        {
            return cvkg_core::DrawMaterial::Blend { mode: *mode };
        }

        match material_id {
            GLASS => {
                if let cvkg_core::DrawMaterial::Glass {
                    blur_radius,
                    ior_override,
                    glass_intensity,
                } = current
                {
                    cvkg_core::DrawMaterial::Glass {
                        blur_radius: *blur_radius,
                        ior_override: *ior_override,
                        glass_intensity: *glass_intensity,
                    }
                } else {
                    cvkg_core::DrawMaterial::Glass {
                        blur_radius: 20.0,
                        ior_override: 0.0,
                        glass_intensity: 1.0,
                    }
                }
            }
            TOP_UI => cvkg_core::DrawMaterial::TopUI,
            BLEND_START..=BLEND_END => cvkg_core::DrawMaterial::Blend {
                mode: (material_id - 7),
            },
            _ => cvkg_core::DrawMaterial::Opaque,
        }
    }

    /// Convert a compositor Material to a core DrawMaterial.
    /// Centralizes the mapping used by submit_buckets and submit_routed.
    pub(crate) fn convert_compositor_material(
        mat: &cvkg_compositor::Material,
    ) -> cvkg_core::DrawMaterial {
        match mat {
            cvkg_compositor::Material::Glass { blur_radius, .. } => {
                cvkg_core::DrawMaterial::Glass {
                    blur_radius: *blur_radius,
                    ior_override: 0.0,
                    glass_intensity: 1.0,
                }
            }
            cvkg_compositor::Material::Overlay => cvkg_core::DrawMaterial::TopUI,
            cvkg_compositor::Material::Multiply => cvkg_core::DrawMaterial::Blend { mode: 1 },
            cvkg_compositor::Material::Screen => cvkg_core::DrawMaterial::Blend { mode: 2 },
            cvkg_compositor::Material::BlendOverlay => cvkg_core::DrawMaterial::Blend { mode: 3 },
            cvkg_compositor::Material::Darken => cvkg_core::DrawMaterial::Blend { mode: 4 },
            cvkg_compositor::Material::Lighten => cvkg_core::DrawMaterial::Blend { mode: 5 },
            cvkg_compositor::Material::ColorDodge => cvkg_core::DrawMaterial::Blend { mode: 6 },
            cvkg_compositor::Material::ColorBurn => cvkg_core::DrawMaterial::Blend { mode: 7 },
            cvkg_compositor::Material::HardLight => cvkg_core::DrawMaterial::Blend { mode: 8 },
            cvkg_compositor::Material::SoftLight => cvkg_core::DrawMaterial::Blend { mode: 9 },
            cvkg_compositor::Material::Difference => cvkg_core::DrawMaterial::Blend { mode: 10 },
            cvkg_compositor::Material::Exclusion => cvkg_core::DrawMaterial::Blend { mode: 11 },
            cvkg_compositor::Material::Hue => cvkg_core::DrawMaterial::Blend { mode: 12 },
            cvkg_compositor::Material::Saturation => cvkg_core::DrawMaterial::Blend { mode: 13 },
            cvkg_compositor::Material::Color => cvkg_core::DrawMaterial::Blend { mode: 14 },
            cvkg_compositor::Material::Luminosity => cvkg_core::DrawMaterial::Blend { mode: 15 },
            cvkg_compositor::Material::Opaque => cvkg_core::DrawMaterial::Opaque,
            _ => cvkg_core::DrawMaterial::Opaque,
        }
    }

    /// Helper: position vertices from SVG view_box into output rect.
    pub(crate) fn position_vertices(
        vertices: &mut [Vertex],
        view_box: Rect,
        rect: Rect,
        material_id: u32,
        clip: [f32; 4],
        snap: impl Fn(f32) -> f32,
    ) {
        for v in vertices.iter_mut() {
            let rel_x = (v.position[0] - view_box.x) / view_box.width;
            let rel_y = (v.position[1] - view_box.y) / view_box.height;
            v.position[0] = snap(rect.x + rel_x * rect.width);
            v.position[1] = snap(rect.y + rel_y * rect.height);
            v.position[2] = 0.0; // z will be set by transform stack
            v.logical = [v.position[0], v.position[1]];
            v.clip = clip;
            v.material_id = material_id;
        }
    }

    /// Helper: emit a draw call for a batch of vertices.
    pub(crate) fn emit_draw_call(
        renderer: &mut GpuRenderer,
        material: cvkg_core::DrawMaterial,
        texture_id: Option<u32>,
        scissor_rect: Rect,
        index_count: u32,
        base_vertex: u32,
    ) {
        let draw_order = renderer.current_draw_order;
        let (translation, scale_transform, rotation, _, _) = renderer.current_transform();
        let current_instance_data = InstanceData {
            translation,
            scale: scale_transform,
            rotation,
            blur_radius: 0.0,
            ior_override: 0.0,
            glass_intensity: 1.0,
        };
        // CRITICAL FIX: Only break batch on material/scissor/texture state changes.
        // Transform (translation/scale/rotation) is per-instance data.
        let last_call = renderer.draw_calls.last();
        let needs_new_call = renderer.draw_calls.is_empty()
            || renderer.current_texture_id != texture_id
            || last_call.unwrap().scissor_rect != renderer.clip_stack.last().copied()
            || last_call.unwrap().material != material
            || {
                let last_material = last_call.unwrap().material;
                matches!((material, last_material),
                    (cvkg_core::DrawMaterial::Glass { blur_radius: a, ior_override: b, glass_intensity: c },
                     cvkg_core::DrawMaterial::Glass { blur_radius: d, ior_override: e, glass_intensity: f })
                    if a != d || b != e || c != f)
            };

        if needs_new_call {
            renderer.current_texture_id = texture_id;
            renderer.instance_data.push(current_instance_data);
            renderer.draw_calls.push(DrawCall {
                target_id: None,
                texture_id,
                scissor_rect: renderer.clip_stack.last().copied(),
                index_start: (renderer.indices.len() - index_count as usize) as u32,
                index_count,
                instance_count: 1,
                material,
                instance_start: (renderer.instance_data.len() - 1) as u32,
                draw_order: 0,
            });
        } else {
            // Same batch - add instance data and increment instance count
            renderer.instance_data.push(current_instance_data);
            if let Some(call) = renderer.draw_calls.last_mut() {
                call.instance_count += 1;
            }
        }
    }

    /// capture_frame -- Read back the rendered frame as a byte buffer (RGBA8).
    pub async fn capture_frame(&self) -> Result<Vec<u8>, String> {
        let ctx = self
            .headless_context
            .as_ref()
            .ok_or("Headless context required for capture")?;

        let u32_size = std::mem::size_of::<u32>() as u32;
        let width = ctx.width;
        let height = ctx.height;
        let bytes_per_row = width * u32_size;
        let padding = (256 - (bytes_per_row % 256)) % 256;
        let padded_bytes_per_row = bytes_per_row + padding;

        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Capture Buffer"),
            size: (padded_bytes_per_row as u64 * height as u64),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Capture Encoder"),
            });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &ctx.output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = output_buffer.slice(..);
        let (sender, receiver) = futures::channel::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| {
            let _ = sender.send(v);
        });

        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });

        if let Ok(Ok(_)) = receiver.await {
            let data = buffer_slice.get_mapped_range();
            let mut result = Vec::with_capacity((width * height * 4) as usize);

            for y in 0..height {
                let start = (y * padded_bytes_per_row) as usize;
                let end = start + bytes_per_row as usize;
                result.extend_from_slice(&data[start..end]);
            }

            log::trace!(
                "[GPU] capture_frame: data len={}, first 4 bytes={:?}",
                data.len(),
                &data[0..4.min(data.len())]
            );

            drop(data);
            output_buffer.unmap();
            Ok(result)
        } else {
            Err("Failed to capture frame".to_string())
        }
    }

    /// Hash a set of gradient stops for cache lookup.
    /// Uses the position and color of each stop to produce a stable hash.
    fn hash_gradient_stops(stops: &[[f32; 4]]) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for stop in stops {
            for v in stop {
                v.to_bits().hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    /// Upload gradient stops as a 32x1 RGBA8 texture.
    /// RGB = stop color (linear-ish sRGB from the component), A = stop position (0-255 mapped to 0-1).
    /// The texture is cached by hash; stops are only re-uploaded when the hash changes.
    #[allow(clippy::collapsible_if)]
    pub(crate) fn upload_gradient_stops(&mut self, stops: &[[f32; 4]]) {
        if stops.is_empty() {
            return;
        }

        let hash = Self::hash_gradient_stops(stops);

        // Check if the texture is already cached with this hash
        if hash == self.gradient_stops_hash {
            if let Some((_, _, bg)) = self.gradient_texture_cache.get(&hash) {
                self.gradient_bind_group = bg.clone();
                return;
            }
        }

        // Check if we have a cached texture for this hash (from a previous frame)
        if let Some((_, view, bg)) = self.gradient_texture_cache.get(&hash) {
            self.gradient_stop_texture = view.texture().clone();
            self.gradient_stop_texture_view = view.clone();
            self.gradient_bind_group = bg.clone();
            self.gradient_stops_hash = hash;
            return;
        }

        // Upload stops into a 32x1 RGBA8 texture
        let max_stops = 32u32;
        let num_stops = stops.len().min(max_stops as usize) as u32;

        // Build RGBA8 data: pack position into alpha as u8
        let mut data = vec![0u8; (max_stops as usize) * 4];
        for (i, stop) in stops.iter().enumerate().take(max_stops as usize) {
            // Convert linear-ish float color to sRGB u8
            let r = (stop[0].clamp(0.0, 1.0) * 255.0).round() as u8;
            let g = (stop[1].clamp(0.0, 1.0) * 255.0).round() as u8;
            let b = (stop[2].clamp(0.0, 1.0) * 255.0).round() as u8;
            let a = (stop[3].clamp(0.0, 1.0) * 255.0).round() as u8;
            // Store position in the alpha channel (4th byte)
            // The color goes in RGB (bytes 0-2), position in byte 3
            #[allow(clippy::identity_op)]
            {
                data[i * 4 + 0] = r;
                data[i * 4 + 1] = g;
                data[i * 4 + 2] = b;
                data[i * 4 + 3] = a;
            }
        }

        // Create or reuse texture
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Gradient Stops Texture"),
            size: wgpu::Extent3d {
                width: max_stops,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(max_stops * 4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: max_stops,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.gradient_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.dummy_sampler),
                },
            ],
            label: Some("Gradient Bind Group"),
        });

        // Cache the texture
        self.gradient_stops_hash = hash;
        self.gradient_stop_texture = texture.clone();
        self.gradient_stop_texture_view = texture_view.clone();
        self.gradient_bind_group = bind_group.clone();
        self.gradient_texture_cache
            .insert(hash, (texture, texture_view, bind_group));
    }

    /// Draw a multi-stop gradient quad using the GPU shader.
    /// rect: bounding rectangle in logical pixels
    /// stops: array of [R, G, B, A] where A is the position (0.0-1.0)
    /// angle: gradient angle in radians (for linear gradients)
    /// is_radial: true for radial gradient, false for linear
    pub fn draw_gradient_multi(
        &mut self,
        rect: Rect,
        stops: &[[f32; 4]],
        angle: f32,
        is_radial: bool,
    ) {
        if stops.is_empty() {
            return;
        }

        // Upload gradient stops (cached by hash)
        self.upload_gradient_stops(stops);

        let num_stops = stops.len().min(32) as f32;
        let material_id = if is_radial { 31u32 } else { 30u32 };

        // Use a white base color; the shader reads stops from the texture
        let white = [1.0f32, 1.0, 1.0, 1.0];

        // slice.x = angle (for linear), slice.y = num_stops
        let slice = [angle, num_stops, 0.0, 1.0];

        self.fill_rect_with_full_params_and_slice(
            rect,
            white,
            material_id,
            None,
            0.0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            slice,
            [0.0, 0.0],
        );
    }
}
