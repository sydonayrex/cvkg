//! Bridging the internal renderer to `cvkg-core` traits.
use crate::renderer::GpuRenderer;

pub mod frame;
pub mod shapes;
pub mod text;

use crate::renderer::material_id;
use crate::types::*;
use crate::vertex::*;
use cvkg_core::LAYOUT_DIRTY;
use cvkg_core::{ColorTheme, Mesh, Rect, RenderStateSnapshot, Renderer};
use lyon::math::point;
use lyon::tessellation::{BuffersBuilder, StrokeOptions, StrokeTessellator, VertexBuffers};
use std::hash::{Hash, Hasher};
use std::sync::atomic::Ordering;

impl cvkg_core::ElapsedTime for GpuRenderer {
    fn delta_time(&self) -> f32 {
        self.current_scene.delta_time
    }

    fn elapsed_time(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
    }
}

impl cvkg_core::RendererErrorHandler for GpuRenderer {
    fn on_render_error(&mut self, error: &cvkg_core::CvkgError) {
        tracing::error!("[GpuRenderer] {error}");
        self.render_error_count += 1;
    }

    fn on_fatal_error(&mut self, error: &cvkg_core::CvkgError) {
        tracing::error!("[GpuRenderer FATAL] {error}");
        self.has_fatal_error = true;
    }

    fn has_error(&self) -> bool {
        self.has_fatal_error
    }
}

impl cvkg_core::Renderer for GpuRenderer {
    fn is_over_budget(&self) -> bool {
        self.frame_budget.allow_degradation
            && self.last_frame_start.elapsed().as_secs_f32() * 1000.0 > self.frame_budget.target_ms
    }

    fn text_scale_factor(&self) -> f32 {
        self.current_scale_factor()
    }

    fn prewarm_vram(&mut self, assets: Vec<(String, Vec<u8>)>) {
        tracing::info!(
            "[Surtr] Pre-warming Mega-Heim with {} assets...",
            assets.len()
        );
        for (name, data) in assets {
            self.load_image_to_heim(&name, &data);
        }
    }

    fn fill_rect(&mut self, rect: Rect, color: [f32; 4]) {
        self.fill_rect_with_mode(rect, self.apply_opacity(color), 0, None);
    }

    fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4]) {
        self.fill_rect_with_full_params(
            rect,
            self.apply_opacity(color),
            3,
            None,
            radius,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
    }

    /// Fill a rounded rect with glass material for frosted backdrop effect.
    /// This is the proper way to render glass cards that need macOS Tahoe-style blur.
    /// The blur_radius controls the intensity of the backdrop blur.
    /// The glass_intensity controls overall glass effect strength (0.0 = solid, 1.0 = full glass).
    /// For Tahoe parity, this registers the rect as a portal region for
    /// per-element isolated backdrop blur when z_index != 0.
    fn fill_glass_rect(&mut self, rect: Rect, radius: f32, blur_radius: f32) {
        self.fill_glass_rect_with_intensity(rect, radius, blur_radius, 1.0);
    }

    /// Fill a rounded rect with glass material with explicit intensity control.
    /// `glass_intensity` ranges from 0.0 (solid, no glass effect) to 1.0 (full glass).
    /// This allows per-component control over glass strength.
    fn fill_glass_rect_with_intensity(
        &mut self,
        rect: Rect,
        radius: f32,
        blur_radius: f32,
        glass_intensity: f32,
    ) {
        // Default tint: neutral white with moderate alpha, matching pre-tint behavior
        self.fill_glass_rect_with_tint(
            rect,
            radius,
            blur_radius,
            [1.0, 1.0, 1.0, 0.4],
            glass_intensity,
        );
    }

    /// Fill a rounded rect with glass material with explicit tint color and intensity.
    /// `tint_color` is the glass base color (RGBA). `glass_intensity` controls effect strength.
    fn fill_glass_rect_with_tint(
        &mut self,
        rect: Rect,
        radius: f32,
        blur_radius: f32,
        tint_color: [f32; 4],
        glass_intensity: f32,
    ) {
        let gi = glass_intensity.clamp(0.0, 1.0);
        // Per-instance blur_radius drives the shader's blur_mip level.
        // Scale: 0-100 input maps to 0-4 mip levels for the Kawase blur chain.
        let blur_strength = (blur_radius / 25.0).clamp(0.0, 4.0) * gi;

        // Register for portal-aware per-element backdrop blur (Tahoe feature)
        if self.current_z != 0.0 {
            self.portal_regions.push_back(rect);
        }

        // Temporary Material Override Binding
        let prev_material = self.current_draw_material;
        self.current_draw_material = cvkg_core::DrawMaterial::Glass {
            blur_radius: blur_strength,
            ior_override: 0.0,
            glass_intensity: gi,
        };

        // Tint color alpha is modulated by intensity so intensity=0 gives a near-invisible fill
        let fill_color = [
            tint_color[0],
            tint_color[1],
            tint_color[2],
            tint_color[3] * gi,
        ];

        self.fill_rect_with_full_params(
            rect,
            fill_color,
            7, // Mode 7 = Glass material
            None,
            radius,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );

        self.current_draw_material = prev_material;
    }

    fn fill_glass_rect_with_pressure(
        &mut self,
        rect: Rect,
        radius: f32,
        blur_radius: f32,
        pressure: f32,
    ) {
        // Pressure scales both blur and tint: full pressure = full glass effect
        let p = pressure.clamp(0.0, 1.0);
        self.fill_glass_rect_with_intensity(rect, radius, blur_radius * p, p);
    }

    /// Set the default background color for the canvas.
    /// This color is used when the app does not draw its own background.
    /// Default: `[0.02, 0.02, 0.05, 1.0]` (Deep Void).
    fn set_default_background_color(&mut self, color: [f32; 4]) {
        self.default_background_color = color;
    }

    /// Fill a squircle (superellipse) for Apple-style icon silhouettes.
    /// `n` controls the squareness: 2.0 = rounded rect, 4.0 = classic squircle, higher = more square.
    fn fill_squircle(&mut self, rect: Rect, n: f32, color: [f32; 4]) {
        let prev_material = self.current_draw_material;
        self.current_draw_material = cvkg_core::DrawMaterial::Opaque;
        self.fill_rect_with_full_params(
            rect,
            self.apply_opacity(color),
            0,
            None,
            rect.width.min(rect.height) * 0.22 * (n / 4.0),
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
        self.current_draw_material = prev_material;
    }

    /// Stroke a squircle (superellipse) outline.
    fn stroke_squircle(&mut self, rect: Rect, n: f32, color: [f32; 4], stroke_width: f32) {
        let prev_material = self.current_draw_material;
        self.current_draw_material = cvkg_core::DrawMaterial::Opaque;
        self.fill_rect_with_full_params(
            rect,
            self.apply_opacity(color),
            material_id::SQUIRCLE_STROKE,
            None,
            rect.width.min(rect.height) * 0.22 * (n / 4.0),
            Rect {
                x: stroke_width,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
        );
        self.current_draw_material = prev_material;
    }

    /// Draw a focus ring around a rect (for keyboard navigation accessibility).
    /// `offset` is the gap between the rect and the ring, `width` is the ring thickness.
    fn draw_focus_ring(
        &mut self,
        rect: Rect,
        radius: f32,
        offset: f32,
        width: f32,
        color: [f32; 4],
    ) {
        let ring_rect = Rect {
            x: rect.x - offset,
            y: rect.y - offset,
            width: rect.width + 2.0 * offset,
            height: rect.height + 2.0 * offset,
        };
        self.stroke_squircle(ring_rect, 4.0, color, width);
    }

    fn fill_ellipse(&mut self, rect: Rect, color: [f32; 4]) {
        self.fill_rect_with_full_params(
            rect,
            self.apply_opacity(color),
            4,
            None,
            0.0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
    }

    fn draw_3d_cube(&mut self, rect: Rect, color: [f32; 4], rotation: [f32; 3]) {
        self.fill_rect_with_full_params_and_slice(
            rect,
            self.apply_opacity(color),
            material_id::MESH_3D,
            None,
            0.0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            [rotation[0], rotation[1], rotation[2], 0.0],
            [0.0, 0.0],
        );
    }

    fn bifrost(&mut self, rect: Rect, blur: f32, _saturation: f32, opacity: f32) {
        // Calculate screen-space UVs for high-fidelity global refraction
        let logical_w = self.current_width() as f32 / self.current_scale_factor();
        let logical_h = self.current_height() as f32 / self.current_scale_factor();
        let screen_uv = Rect {
            x: rect.x / logical_w,
            y: rect.y / logical_h,
            width: rect.width / logical_w,
            height: rect.height / logical_h,
        };
        // Use mode 7 for high-fidelity background blur sampling
        // Use the blur parameter as corner radius for the glass panel
        self.fill_rect_with_full_params(rect, [1.0, 1.0, 1.0, opacity], 7, None, blur, screen_uv);
    }

    fn gungnir(&mut self, rect: Rect, color: [f32; 4], radius: f32, intensity: f32) {
        // Single draw call via SDF glow material instead of 4 additive rects
        let margin = radius;
        let glow_rect = Rect {
            x: rect.x - margin,
            y: rect.y - margin,
            width: rect.width + 2.0 * margin,
            height: rect.height + 2.0 * margin,
        };
        let glow_color = [color[0], color[1], color[2], intensity * 0.3];
        self.fill_rect_with_full_params(
            glow_rect,
            self.apply_opacity(glow_color),
            material_id::DROP_SHADOW,
            None,
            8.0,
            Rect {
                x: margin,
                y: radius,
                width: 0.0,
                height: 0.0,
            },
        );
    }

    /// Soft glow variant -- half the intensity of gungnir().
    /// Use for hover highlights, non-critical indicators.
    fn gungnir_soft(&mut self, rect: Rect, color: [f32; 4], radius: f32, intensity: f32) {
        self.gungnir(rect, color, radius, intensity * 0.5);
    }

    /// Renders a dynamic glowing hover boundary field around a hit target.
    ///
    /// # Contract
    /// Expands the bounding box of the visual target by `radius` to establish
    /// a continuous proximity glow. Uses the drop shadow/glow SDF material
    /// to rasterize the glow with specialized radius-to-margin uv coordinate mappings.
    fn mani_glow(&mut self, rect: Rect, color: [f32; 4], radius: f32) {
        let margin = radius;
        let glow_rect = Rect {
            x: rect.x - margin,
            y: rect.y - margin,
            width: rect.width + 2.0 * margin,
            height: rect.height + 2.0 * margin,
        };
        let uv_rect = Rect {
            x: margin,
            y: radius,
            width: 0.0,
            height: 0.0,
        };
        self.fill_rect_with_full_params(
            glow_rect,
            self.apply_opacity(color),
            material_id::DROP_SHADOW,
            None,
            8.0,
            uv_rect,
        );
    }

    fn stroke_rect(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32) {
        let c = self.apply_opacity(color);
        // Single draw call via SDF stroke material instead of 4 edge bars
        self.fill_rect_with_full_params(
            rect,
            c,
            material_id::SQUIRCLE_STROKE,
            None,
            0.0, // radius = 0 for sharp rect corners
            Rect {
                x: stroke_width,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
        );
    }

    fn stroke_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4], stroke_width: f32) {
        self.fill_rect_with_full_params(
            rect,
            self.apply_opacity(color),
            material_id::SQUIRCLE_STROKE,
            None,
            radius,
            Rect {
                x: stroke_width,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
        );
    }

    fn stroke_ellipse(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32) {
        // Tessellate an ellipse stroke using Lyon's StrokeTessellator.
        let cx = rect.x + rect.width / 2.0;
        let cy = rect.y + rect.height / 2.0;
        let rx = rect.width / 2.0;
        let ry = rect.height / 2.0;

        // Build an ellipse path using Lyon
        let mut builder = lyon::path::Path::builder();
        if rx > 0.0 && ry > 0.0 {
            // Approximate ellipse with 64 segments
            let segments = 64;
            for i in 0..segments {
                let angle = 2.0 * std::f32::consts::PI * (i as f32) / (segments as f32);
                let x = cx + rx * angle.cos();
                let y = cy + ry * angle.sin();
                if i == 0 {
                    builder.begin(lyon::math::point(x, y));
                } else {
                    builder.line_to(lyon::math::point(x, y));
                }
            }
            builder.close();
        }
        let path = builder.build();
        self.stroke_path(&path, color, stroke_width);
    }

    fn draw_linear_gradient(
        &mut self,
        rect: Rect,
        start_color: [f32; 4],
        end_color: [f32; 4],
        angle: f32,
    ) {
        self.fill_rect_with_full_params_and_slice(
            rect,
            self.apply_opacity(start_color),
            15,
            None,
            0.0,
            Rect {
                x: angle,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            end_color,
            [0.0, 0.0],
        );
    }

    fn draw_radial_gradient(&mut self, rect: Rect, inner_color: [f32; 4], outer_color: [f32; 4]) {
        self.fill_rect_with_full_params_and_slice(
            rect,
            self.apply_opacity(inner_color),
            material_id::RADIAL_GRADIENT,
            None,
            0.0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            outer_color,
            [0.0, 0.0],
        );
    }

    fn draw_linear_gradient_multi(&mut self, rect: Rect, stops: &[[f32; 4]], angle: f32) {
        self.draw_gradient_multi(rect, stops, angle, false);
    }

    fn draw_radial_gradient_multi(&mut self, rect: Rect, stops: &[[f32; 4]]) {
        self.draw_gradient_multi(rect, stops, 0.0, true);
    }

    fn draw_drop_shadow(
        &mut self,
        rect: Rect,
        radius: f32,
        color: [f32; 4],
        blur: f32,
        spread: f32,
    ) {
        let margin = blur + spread;
        let inflated = Rect {
            x: rect.x - margin,
            y: rect.y - margin,
            width: rect.width + margin * 2.0,
            height: rect.height + margin * 2.0,
        };
        // uv.x = total margin (for SDF offset), uv.y = blur width (for falloff)
        self.fill_rect_with_full_params_and_slice(
            inflated,
            self.apply_opacity(color),
            material_id::DROP_SHADOW,
            None,
            radius,
            Rect {
                x: margin,
                y: blur,
                width: 0.0,
                height: 0.0,
            },
            [0.0, 0.0, 0.0, 1.0],
            [0.0, 0.0],
        );
    }

    fn stroke_dashed_rounded_rect(
        &mut self,
        rect: Rect,
        radius: f32,
        color: [f32; 4],
        width: f32,
        dash: f32,
        gap: f32,
    ) {
        self.fill_rect_with_full_params(
            rect,
            self.apply_opacity(color),
            material_id::DASHED_STROKE,
            None,
            radius,
            Rect {
                x: width,
                y: dash,
                width: gap,
                height: 0.0,
            },
        );
    }

    fn draw_9slice(
        &mut self,
        image_name: &str,
        rect: Rect,
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
    ) {
        let c = self.apply_opacity([1.0, 1.0, 1.0, 1.0]);
        let tid = self.get_texture_id(image_name);
        self.fill_rect_with_full_params(
            rect,
            c,
            20,
            tid,
            bottom,
            Rect {
                x: left,
                y: top,
                width: right,
                height: 0.0,
            },
        );
    }

    fn draw_line(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: [f32; 4],
        stroke_width: f32,
    ) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len_sq = dx * dx + dy * dy;
        if len_sq < 0.000001 {
            return;
        }
        let len = len_sq.sqrt();
        let half_w = stroke_width * 0.5;
        // Perpendicular unit vector
        let nx = -dy / len * half_w;
        let ny = dx / len * half_w;
        // Build 4 corner points of the line quad
        let points = [
            [x1 + nx, y1 + ny],
            [x2 + nx, y2 + ny],
            [x2 - nx, y2 - ny],
            [x1 - nx, y1 - ny],
        ];
        self.push_oriented_quad(
            points,
            color,
            1,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
    }

    fn draw_image(&mut self, image_name: &str, rect: Rect) {
        // Guard: skip if image not loaded -- avoids rendering garbage from uninitialized atlas regions
        if !self.image_uv_registry.contains(image_name) {
            tracing::warn!("[Surtr] draw_image: '{}' not loaded, skipping", image_name);
            return;
        }
        let tid = self
            .get_texture_id(image_name)
            .or_else(|| self.get_texture_id("__mega_heim"));
        let uv_rect = self
            .image_uv_registry
            .get(image_name)
            .copied()
            .unwrap_or(Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            });
        self.fill_rect_with_full_params(rect, [1.0, 1.0, 1.0, 1.0], 2, tid, 0.0, uv_rect);
    }

    fn shape_rich_text(
        &mut self,
        spans: &[cvkg_runic_text::TextSpan],
        max_width: Option<f32>,
        align: cvkg_runic_text::TextAlign,
        overflow: cvkg_runic_text::TextOverflow,
    ) -> Option<cvkg_runic_text::ShapedText> {
        self.shape_rich_text_impl(spans, max_width, align, overflow)
    }

    fn draw_shaped_text(&mut self, shaped: &cvkg_runic_text::ShapedText, x: f32, y: f32) {
        self.draw_shaped_text_impl(shaped, x, y);
    }

    fn draw_texture(&mut self, texture_id: u32, rect: Rect) {
        self.fill_rect_with_full_params_and_slice(
            rect,
            [1.0, 1.0, 1.0, 1.0],
            2,
            Some(texture_id),
            0.0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            [0.0, 0.0, 0.0, 1.0],
            [0.0, 0.0],
        );
    }

    /// load_image -- Proactively pushes a raw asset into the Mega-Heim.
    /// load_image -- Proactively pushes a raw asset into the Texture Array.
    fn load_image(&mut self, name: &str, data: &[u8]) {
        if self.image_uv_registry.contains(name) {
            return;
        }
        let img_result = image::load_from_memory(data);
        let img = match img_result {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                tracing::error!("Failed to load image {}: {}", name, e);
                image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 255, 255, 255]))
            }
        };
        let (width, height) = img.dimensions();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("Texture Array Layer: {}", name)),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
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
            &img,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Slot allocation (Skip index 0 which is the dummy/atlas)
        // texture_views is a fixed 32-element Vec; indices 1..=31 are usable.
        let index = if self.texture_registry.len() < 31 {
            (self.texture_registry.len() + 1) as u32
        } else {
            // Evict the least recently used texture and reuse its slot.
            // The bind group cache is invalidated below by rebuilding.
            if let Some((old_name, old_index)) = self.texture_registry.pop_lru() {
                self.image_uv_registry.pop(&old_name);
                old_index
            } else {
                tracing::warn!("[GPU] texture registry full and no LRU entry to evict");
                return;
            }
        };

        // Bounds guard: index must be in 1..32 (index 0 is the atlas).
        if index == 0 || index as usize >= self.texture_views.len() {
            tracing::error!(
                "[GPU] load_image: invalid texture index {} (registry has {} entries)",
                index,
                self.texture_registry.len()
            );
            return;
        }

        self.texture_views[index as usize] = view;
        self.image_uv_registry.put(
            name.to_string(),
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
        self.texture_registry.put(name.to_string(), index);
        self.rebuild_texture_array_bind_group();
    }

    fn push_clip_rect(&mut self, rect: Rect) {
        self.clip_stack.push(rect);
    }

    fn pop_clip_rect(&mut self) {
        self.clip_stack.pop();
    }

    fn current_clip_rect(&self) -> Rect {
        self.clip_stack.last().copied().unwrap_or(Rect::new(
            0.0,
            0.0,
            self.current_width() as f32,
            self.current_height() as f32,
        ))
    }

    fn memoize(&mut self, id: u64, data_hash: u64, render_fn: &dyn Fn(&mut dyn Renderer)) {
        // P0-4 fix: actually cache and replay GPU draw commands.
        //
        // The previous implementation only cached `(data_hash, frame_generation)`
        // and emitted ZERO draw calls on the skip path. Any content using
        // `memoize` rendered once and then vanished on every subsequent frame.
        //
        // The fix: on first call (or when hash changes), record the vertex/
        // index/instance buffers and DrawCall list produced by `render_fn`,
        // with offsets remapped relative to the captured slice. On replay,
        // append the cached buffers to the current buffer state and shift
        // the cached DrawCall offsets by the current buffer length so the
        // replayed commands reference the freshly-appended data.
        use crate::types::{DrawCall, MemoEntry};

        let should_skip = self
            .memo_cache
            .get(&id)
            .is_some_and(|entry| entry.hash == data_hash);

        if should_skip {
            // Replay path: append cached buffers and remap cached DrawCall offsets.
            if let Some(entry) = self.memo_cache.get(&id) {
                let i_offset = self.indices.len() as u32;
                let inst_offset = self.instance_data.len() as u32;

                self.vertices.extend_from_slice(&entry.vertices);
                self.indices.extend_from_slice(&entry.indices);
                self.instance_data.extend_from_slice(&entry.instance_data);

                for dc in &entry.draw_calls {
                    let mut replayed = dc.clone();
                    // Offsets stored relative to the captured slice start;
                    // shift them by the current buffer lengths so they
                    // reference the freshly-appended data.
                    replayed.index_start += i_offset;
                    replayed.instance_start += inst_offset;
                    self.draw_calls.push(replayed);
                }
            }
        } else {
            // Capture path: snapshot lengths, render, then record deltas.
            let v_start = self.vertices.len();
            let i_start = self.indices.len();
            let inst_start = self.instance_data.len();
            let dc_start = self.draw_calls.len();

            render_fn(self);

            // Remap DrawCall offsets to be relative to the captured slice.
            let draw_calls: Vec<DrawCall> = self.draw_calls[dc_start..]
                .iter()
                .map(|dc| {
                    let mut remapped = dc.clone();
                    // saturating_sub guards against underflow if a draw call
                    // somehow already had an offset below the slice start
                    // (should not happen, but defensive).
                    remapped.index_start = remapped.index_start.saturating_sub(i_start as u32);
                    remapped.instance_start =
                        remapped.instance_start.saturating_sub(inst_start as u32);
                    remapped
                })
                .collect();

            let entry = MemoEntry {
                hash: data_hash,
                frame_gen: self.frame_generation,
                vertices: self.vertices[v_start..].to_vec(),
                indices: self.indices[i_start..].to_vec(),
                instance_data: self.instance_data[inst_start..].to_vec(),
                draw_calls,
            };

            self.memo_cache.insert(id, entry);
        }
    }

    fn snapshot_render_state(&self) -> RenderStateSnapshot {
        RenderStateSnapshot {
            clip_depth: self.clip_stack.len() as u32,
            opacity_depth: self.opacity_stack.len() as u32,
            slice_depth: self.slice_stack.len() as u32,
            shadow_depth: self.shadow_stack.len() as u32,
            transform_depth: self.transform_stack.len() as u32,
            vnode_depth: self.vnode_stack.len() as u32,
        }
    }

    fn restore_render_state(&mut self, snap: RenderStateSnapshot) {
        // Idempotent: pop only items pushed beyond the snapshot point.
        while self.clip_stack.len() as u32 > snap.clip_depth {
            self.clip_stack.pop();
        }
        while self.opacity_stack.len() as u32 > snap.opacity_depth {
            self.opacity_stack.pop();
        }
        while self.slice_stack.len() as u32 > snap.slice_depth {
            self.slice_stack.pop();
        }
        while self.shadow_stack.len() as u32 > snap.shadow_depth {
            self.shadow_stack.pop();
        }
        while self.transform_stack.len() as u32 > snap.transform_depth {
            self.transform_stack.pop();
        }
        while self.vnode_stack.len() as u32 > snap.vnode_depth {
            self.vnode_stack.pop();
        }
    }

    fn push_opacity(&mut self, opacity: f32) {
        let current = self.opacity_stack.last().copied().unwrap_or(1.0);
        self.opacity_stack.push(current * opacity);
    }

    fn pop_opacity(&mut self) {
        self.opacity_stack.pop();
    }

    fn push_shadow(&mut self, radius: f32, color: [f32; 4], offset: [f32; 2]) {
        self.shadow_stack.push(ShadowState {
            radius,
            color,
            _offset: offset,
        });
    }

    fn pop_shadow(&mut self) {
        self.shadow_stack.pop();
    }

    fn push_transform(&mut self, translation: [f32; 2], scale: [f32; 2], rotation: f32) {
        let c = rotation.cos();
        let sn = rotation.sin();
        let affine = glam::Mat3::from_cols(
            glam::Vec3::new(c * scale[0], sn * scale[0], 0.0),
            glam::Vec3::new(-sn * scale[1], c * scale[1], 0.0),
            glam::Vec3::new(translation[0], translation[1], 1.0),
        );

        let parent = self
            .transform_stack
            .last()
            .copied()
            .unwrap_or(glam::Mat3::IDENTITY);
        self.transform_stack.push(parent * affine);
    }

    fn push_affine(&mut self, transform: [f32; 6]) {
        let affine = glam::Mat3::from_cols(
            glam::Vec3::new(transform[0], transform[1], 0.0),
            glam::Vec3::new(transform[2], transform[3], 0.0),
            glam::Vec3::new(transform[4], transform[5], 1.0),
        );
        let parent = self
            .transform_stack
            .last()
            .copied()
            .unwrap_or(glam::Mat3::IDENTITY);
        self.transform_stack.push(parent * affine);
    }

    fn pop_transform(&mut self) {
        self.transform_stack.pop();
    }

    fn set_theme(&mut self, theme: ColorTheme) {
        self.current_theme = theme;
        self.queue
            .write_buffer(&self.theme_buffer, 0, bytemuck::bytes_of(&theme));
    }

    fn set_rage(&mut self, rage: f32) {
        self.current_scene.berzerker_rage = rage;
        // scene_buffer is updated every frame in begin_frame, so no need to write here
    }

    fn set_fireball_pos(&mut self, pos: [f32; 2]) {
        self.current_scene.fireball_pos = pos;
    }

    fn trigger_shatter_event(&mut self, origin: [f32; 2], force: f32) {
        self.current_scene.shatter_origin = origin;
        self.current_scene.shatter_time = self.current_scene.time;
        self.current_scene.shatter_force = force;
    }

    fn set_scene_preset(&mut self, preset: u32) {
        self.current_scene.scene_type = preset;
    }

    /// push_mjolnir_slice -- Pushes a geometric clipping plane onto the stack.
    /// All subsequent draw calls will be sliced by this plane until it is popped.
    fn push_mjolnir_slice(&mut self, angle: f32, offset: f32) {
        self.slice_stack.push((angle, offset));
    }

    /// pop_mjolnir_slice -- Removes the top-most geometric clipping plane from the stack.
    fn pop_mjolnir_slice(&mut self) {
        self.slice_stack.pop();
    }

    fn mjolnir_shatter(&mut self, rect: Rect, pieces: u32, force: f32, color: [f32; 4]) {
        self.shatter_internal(rect, pieces, force, color, 8);
    }

    fn mjolnir_fluid_shatter(&mut self, rect: Rect, pieces: u32, force: f32, color: [f32; 4]) {
        self.shatter_internal(rect, pieces, force, color, 11);
    }

    fn draw_mjolnir_bolt(&mut self, from: [f32; 2], to: [f32; 2], color: [f32; 4]) {
        self.recursive_bolt(from, to, 4, color);
    }

    fn dispatch_particles(
        &mut self,
        origin: [f32; 2],
        count: u32,
        effect_type: &str,
        color: [f32; 4],
    ) {
        use crate::types::{GpuParticle, MAX_PARTICLES};

        let dt = self.current_scene.delta_time;
        let now = std::time::Instant::now();

        // Determine spawn parameters based on effect type
        let (speed_range, life_range, spread_angle) = match effect_type {
            "firework" => (100.0..300.0, 1.0..2.5, std::f32::consts::TAU),
            "spark" => (50.0..150.0, 0.5..1.5, std::f32::consts::PI),
            "rain" => (20.0..80.0, 1.0..3.0, std::f32::consts::FRAC_PI_4),
            "data_stream" => (80.0..200.0, 0.8..2.0, std::f32::consts::FRAC_PI_6),
            "bubble" => (10.0..40.0, 2.0..4.0, std::f32::consts::TAU),
            _ => (30.0..120.0, 1.0..2.0, std::f32::consts::TAU),
        };

        let count = count.min((MAX_PARTICLES - self.particles.count as usize) as u32);
        if count == 0 {
            return;
        }

        let mut rng_state = (now.elapsed().as_nanos() as u64)
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let mut rand_f32 = |range: std::ops::Range<f32>| -> f32 {
            rng_state = rng_state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let t = (rng_state >> 33) as f32 / (1u64 << 31) as f32;
            range.start + t * (range.end - range.start)
        };

        for _ in 0..count {
            let angle = rand_f32(0.0..spread_angle);
            let speed = rand_f32(speed_range.clone());
            let life = rand_f32(life_range.clone());
            let vx = angle.cos() * speed;
            let vy = angle.sin() * speed;

            let particle = GpuParticle {
                pos_vel: [origin[0], origin[1], vx, vy],
                color_life: [color[0], color[1], color[2], life],
            };
            self.particles.staging.push(particle);
        }

        tracing::debug!(
            "[Surtr] dispatch_particles: {} {} particles at {:?} (staged, {} total pending)",
            count,
            effect_type,
            origin,
            self.particles.staging.len()
        );
    }

    fn draw_hologram(&mut self, rect: Rect, hologram_id: &str, time: f32) {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hologram_id.hash(&mut hasher);
        let id_hash = hasher.finish() as u32;

        tracing::debug!(
            "[Surtr] draw_hologram: {} at {:?} t={} (hologram pipeline)",
            hologram_id,
            rect,
            time
        );

        self.hologram_instances
            .push(crate::renderer::HologramInstance {
                rect,
                id_hash,
                time,
            });
        self.volumetric_enabled = true;
    }

    fn upload_data_texture(&mut self, id: &str, data: &[f32], width: u32, height: u32) {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(id),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
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
            bytemuck::cast_slice(data),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        // Reuse the renderer's pre-created linear sampler (ClampToEdge + Linear)
        // instead of allocating a new sampler on every upload.
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    // The layout requires 32 entries; only index 0 is the actual texture.
                    resource: wgpu::BindingResource::TextureViewArray(&vec![&view; 32]),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.linear_sampler),
                },
            ],
            label: Some(id),
        });
        self.texture_bind_groups.push(bind_group);
        let tid = (self.texture_bind_groups.len() - 1) as u32;
        self.texture_registry.put(id.to_string(), tid);
    }

    fn draw_heatmap(&mut self, texture_id: &str, rect: Rect, _palette: &str) {
        let tid = self.get_texture_id(texture_id);
        self.fill_rect_with_mode(rect, [1.0, 1.0, 1.0, 1.0], 12, tid);
    }

    fn draw_mesh(&mut self, mesh: &Mesh, color: [f32; 4], transform: glam::Mat4) {
        let base_idx = self.vertices.len() as u32;

        for i in 0..mesh.vertices.len() {
            let pos = transform.transform_point3(glam::Vec3::from(mesh.vertices[i]));
            let norm = transform.transform_vector3(glam::Vec3::from(mesh.normals[i]));

            self.vertices.push(Vertex {
                position: pos.to_array(),
                normal: norm.to_array(),
                uv: [0.0, 0.0],
                color,
                material_id: 13, // Material 13: 3D Surface
                radius: 0.0,
                slice: [0.0, 0.0, 0.0, 1.0],
                logical: [0.0, 0.0],
                size: [0.0, 0.0],
                clip: [-f32::INFINITY, -f32::INFINITY, f32::INFINITY, f32::INFINITY],
                tex_index: 0,
            });
        }

        for idx in &mesh.indices {
            self.indices.push(base_idx + idx);
        }

        let (translation, scale_transform, rotation, _, _) = self.current_transform();

        if self.draw_calls.is_empty() || self.current_texture_id.is_some() {
            self.current_texture_id = None;

            self.instance_data.push(InstanceData {
                translation,
                scale: scale_transform,
                rotation,
                blur_radius: 0.0,
                ior_override: 0.0,
                glass_intensity: 1.0,
            });
            self.draw_calls.push(DrawCall {
                target_id: None,
                texture_id: None,
                scissor_rect: self.clip_stack.last().copied(),
                index_start: (self.indices.len() as u32) - (mesh.indices.len() as u32),
                index_count: mesh.indices.len() as u32,
                instance_count: 1,
                material: cvkg_core::DrawMaterial::Opaque,
                instance_start: (self.instance_data.len() - 1) as u32,
                draw_order: 0,
            });
        } else {
            self.draw_calls.last_mut().unwrap().index_count += mesh.indices.len() as u32;
        }
    }

    fn draw_mesh_3d(
        &mut self,
        mesh: &Mesh,
        material: &cvkg_core::Material3D,
        transform: &cvkg_core::Transform3D,
    ) {
        let base_idx = self.vertices.len() as u32;
        let model_matrix = transform.to_matrix();

        for i in 0..mesh.vertices.len() {
            let pos = model_matrix.transform_point3(glam::Vec3::from(mesh.vertices[i]));
            let norm = model_matrix.transform_vector3(glam::Vec3::from(mesh.normals[i]));

            self.vertices.push(Vertex {
                position: [pos.x, pos.y, pos.z],
                normal: [norm.x, norm.y, norm.z],
                uv: [0.0, 0.0],
                color: material.base_color,
                material_id: 13, // Material 13: 3D Surface
                radius: 0.0,
                slice: [material.metallic, material.roughness, material.opacity, 1.0],
                logical: [0.0, 0.0],
                size: [0.0, 0.0],
                clip: [-f32::INFINITY, -f32::INFINITY, f32::INFINITY, f32::INFINITY],
                tex_index: 0,
            });
        }

        for idx in &mesh.indices {
            self.indices.push(base_idx + idx);
        }

        self.instance_data.push(InstanceData {
            translation: [0.0, 0.0],
            scale: [1.0, 1.0],
            rotation: 0.0,
            blur_radius: 0.0,
            ior_override: 0.0,
            glass_intensity: 1.0,
        });

        self.draw_calls.push(DrawCall {
            target_id: None,
            texture_id: None,
            scissor_rect: self.clip_stack.last().copied(),
            index_start: (self.indices.len() as u32) - (mesh.indices.len() as u32),
            index_count: mesh.indices.len() as u32,
            instance_count: 1,
            material: cvkg_core::DrawMaterial::Opaque,
            instance_start: (self.instance_data.len() - 1) as u32,
            draw_order: 0,
        });
    }

    fn set_camera_3d(&mut self, camera: &cvkg_core::Camera3D) {
        self.current_scene.proj = camera.projection_matrix();
        self.current_scene.view = camera.view_matrix();
    }

    fn push_transform_3d(&mut self, transform: &cvkg_core::Transform3D) {
        // Push a 2D-compatible transform for the existing pipeline
        // Use proper matrix decomposition to extract scale correctly (handles rotated matrices)
        let (translation, rotation_quat, scale_glam) =
            transform.to_matrix().to_scale_rotation_translation();
        let translation = [translation.x, translation.y];
        let scale = [scale_glam.x, scale_glam.y];
        let rotation = if rotation_quat.length_squared() > 0.0 {
            let (axis, angle) = rotation_quat.to_axis_angle();
            angle * axis.z.signum() // Radians (preserving Z-axis direction)
        } else {
            0.0
        };
        self.push_transform(translation, scale, rotation);
    }

    fn pop_transform_3d(&mut self) {
        // Only pop the single transform that was pushed - no double pop
        self.pop_transform();
    }

    /// Render a 3D scene graph node using the GPU backend.
    ///
    /// # Contract
    /// PBR lighting and opacity are computed using base color, metallic (0.0), and roughness (0.5)
    /// to support standard matte opaque 3D meshes.
    fn render_scene_node_3d(
        &mut self,
        position: [f32; 3],
        rotation: [f32; 4],
        scale: [f32; 3],
        color: [f32; 4],
        meshes: &[Mesh],
    ) {
        let transform = cvkg_core::Transform3D {
            position: glam::Vec3::from(position),
            rotation: glam::Quat::from_xyzw(rotation[0], rotation[1], rotation[2], rotation[3]),
            scale: glam::Vec3::from(scale),
        };
        // Use provided mesh or generate a default unit cube
        if meshes.is_empty() {
            // Generate a unit cube mesh on the stack
            let h = 0.5f32;
            let cube = Mesh {
                vertices: vec![
                    [-h, -h, -h],
                    [h, -h, -h],
                    [h, h, -h],
                    [-h, h, -h],
                    [-h, -h, h],
                    [h, -h, h],
                    [h, h, h],
                    [-h, h, h],
                ],
                normals: vec![
                    [0.0, 0.0, -1.0],
                    [0.0, 0.0, -1.0],
                    [0.0, 0.0, -1.0],
                    [0.0, 0.0, -1.0],
                    [0.0, 0.0, 1.0],
                    [0.0, 0.0, 1.0],
                    [0.0, 0.0, 1.0],
                    [0.0, 0.0, 1.0],
                    [0.0, -1.0, 0.0],
                    [0.0, -1.0, 0.0],
                    [0.0, -1.0, 0.0],
                    [0.0, -1.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [-1.0, 0.0, 0.0],
                    [-1.0, 0.0, 0.0],
                    [-1.0, 0.0, 0.0],
                    [-1.0, 0.0, 0.0],
                ],
                indices: vec![
                    0, 1, 2, 0, 2, 3, // front
                    5, 4, 7, 5, 7, 6, // back
                    4, 0, 3, 4, 3, 7, // left
                    1, 5, 6, 1, 6, 2, // right
                    3, 2, 6, 3, 6, 7, // top
                    4, 5, 1, 4, 1, 0, // bottom
                ],
            };
            let material = cvkg_core::Material3D {
                base_color: color,
                metallic: 0.0,
                roughness: 0.5,
                emissive: [0.0, 0.0, 0.0],
                opacity: color[3],
            };
            self.draw_mesh_3d(&cube, &material, &transform);
        } else {
            let material = cvkg_core::Material3D {
                base_color: color,
                metallic: 0.0,
                roughness: 0.5,
                emissive: [0.0, 0.0, 0.0],
                opacity: color[3],
            };
            self.draw_mesh_3d(&meshes[0], &material, &transform);
        }
    }

    fn register_shared_element(&mut self, id: &str, rect: Rect) {
        self.shared_elements.put(id.to_string(), rect);
    }

    fn set_z_index(&mut self, z: f32) {
        self.current_z = z;
    }

    fn set_material(&mut self, material: cvkg_core::DrawMaterial) {
        self.current_draw_material = material;
    }

    fn current_material(&self) -> cvkg_core::DrawMaterial {
        self.current_draw_material
    }

    fn get_z_index(&self) -> f32 {
        self.current_z
    }

    fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }

    // -- Portal / PhaseGate rendering -----------------------------------------

    /// Begin rendering into the portal root layer instead of the inline tree.
    /// All draw calls between `enter_portal` and `exit_portal` are collected
    /// into a separate buffer that is composited AFTER the main tree.
    ///
    /// WHY separate buffer: The main tree may have clipping, transforms, or
    /// opacity that should NOT affect overlays. The portal layer renders on top
    /// of everything, ignoring the local coordinate system.
    ///
    /// `z_index` controls the layer ordering for portal content.
    fn enter_portal(&mut self, z_index: i32) {
        // Portal rendering enables per-element backdrop blur for Tahoe glass
        // When z_index is 0, we're rendering normal glass cards
        // When z_index > 0, we're in a portal layer that will get special treatment
        self.current_z = z_index as f32;
    }

    /// Exit the portal layer and return to inline rendering.
    /// The portal content collected since `enter_portal` is now sealed --
    /// no more draw calls will be appended to it.
    fn exit_portal(&mut self) {
        self.current_z = 0.0;
    }

    fn push_vnode(&mut self, rect: Rect, name: &'static str) {
        self.vnode_stack.push((rect, name));
    }

    fn pop_vnode(&mut self) {
        self.vnode_stack.pop();
    }

    fn register_handler(
        &mut self,
        event_type: &str,
        handler: std::sync::Arc<dyn Fn(cvkg_core::Event) + Send + Sync>,
    ) {
        self.event_handlers
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(handler);
    }

    fn load_svg(&mut self, name: &str, svg_data: &[u8]) {
        GpuRenderer::load_svg(self, name, svg_data);
    }

    fn draw_svg(&mut self, name: &str, rect: Rect) {
        GpuRenderer::draw_svg(self, name, rect, None, 0);
    }
    fn draw_svg_with_offset(&mut self, name: &str, rect: Rect, animation_time_offset: f32) {
        GpuRenderer::draw_svg_with_offset(self, name, rect, None, 0, animation_time_offset);
    }

    /// Draw SVG content with explicit draw_order for z-sorting within the same pass.
    /// Use draw_order=200 for SVG content that should render above UI chrome (draw_order=0).
    fn draw_svg_with_order(&mut self, name: &str, rect: Rect, draw_order: i32) {
        GpuRenderer::draw_svg_with_order(self, name, rect, None, 0, 0.0, draw_order);
    }

    fn serialize_svg(&mut self, name: &str) -> Result<String, String> {
        let tree = self
            .svg
            .tree_cache
            .get(name)
            .ok_or_else(|| format!("SVG '{}' not found", name))?;
        let config = cvkg_svg_serialize::SerializerConfig::default();
        let mut serializer = cvkg_svg_serialize::SvgSerializer::with_config(config);
        serializer
            .serialize(tree)
            .map_err(|e| format!("SVG serialization failed: {}", e))
    }

    fn apply_svg_filter(
        &mut self,
        name: &str,
        filter_id: &str,
        _region: Rect,
    ) -> Result<String, String> {
        let tree = self
            .svg
            .tree_cache
            .get(name)
            .ok_or_else(|| format!("SVG '{}' not found", name))?;
        let _filter = Self::find_filter(tree, filter_id)
            .ok_or_else(|| format!("Filter '{}' not found in SVG '{}'", filter_id, name))?;
        let config = cvkg_svg_serialize::SerializerConfig::default();
        let mut serializer = cvkg_svg_serialize::SvgSerializer::with_config(config);
        serializer
            .serialize(tree)
            .map_err(|e| format!("SVG filter serialization failed: {}", e))
    }

    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        self.measure_text_impl(text, size)
    }

    fn draw_text(&mut self, text: &str, rect: &Rect, size: f32, color: [f32; 4], h_align: cvkg_core::TextHAlign, v_align: cvkg_core::TextVAlign) {
        self.draw_text_impl(text, rect, size, color, h_align, v_align);
    }
}

// ── Inherent methods on GpuRenderer (not part of the Renderer trait) ──

impl GpuRenderer {
    /// Clear all registered event handlers. Call at the start of each frame
    /// before re-rendering the component tree.
    pub fn clear_event_handlers(&mut self) {
        self.event_handlers.clear();
    }

    /// Phase 2.1: clear the text shaping cache at the start of each frame.
    pub fn clear_text_cache(&mut self) {
        self.clear_text_cache_impl();
    }

    /// Get all registered event handlers for a specific event type.
    pub fn get_handlers(
        &self,
        event_type: &str,
    ) -> Option<&Vec<std::sync::Arc<dyn Fn(cvkg_core::Event) + Send + Sync>>> {
        self.event_handlers.get(event_type)
    }

    /// Compute per-vertex transform values from the current matrix.
    /// Extracts translation, scale, rotation, and skew from the affine matrix
    /// so the existing vertex shader fields still work correctly.
    pub(crate) fn current_transform(&self) -> ([f32; 2], [f32; 2], f32, f32, f32) {
        // Returns (translation, scale, rotation,
        // skew_x, skew_y)
        let m = self
            .transform_stack
            .last()
            .copied()
            .unwrap_or(glam::Mat3::IDENTITY);
        let t = [m.z_axis.x, m.z_axis.y];
        // Extract scale and rotation from the 2x2 submatrix
        let a = m.x_axis.x;
        let b = m.x_axis.y;
        let c = m.y_axis.x;
        let d = m.y_axis.y;
        let sx = (a * a + b * b).sqrt();
        let sy = (c * c + d * d).sqrt();
        let rotation = b.atan2(a);
        // Skew: the angle between the basis vectors minus 90 degrees
        let skew_x = (a * c + b * d) / (sx * sy); // sin(skew)
        (t, [sx, sy], rotation, skew_x, 0.0)
    }

    pub fn stroke_path(&mut self, path: &lyon::path::Path, color: [f32; 4], stroke_width: f32) {
        self.stroke_path_impl(path, color, stroke_width);
    }
}
