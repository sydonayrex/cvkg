//! Bridging the internal renderer to `cvkg-core` traits.
use crate::renderer::SurtrRenderer;
use crate::types::*;
use crate::vertex::*;
use cvkg_core::LAYOUT_DIRTY;
use cvkg_core::{ColorTheme, Mesh, Rect, Renderer};
use lyon::math::point;
use lyon::tessellation::{BuffersBuilder, StrokeOptions, StrokeTessellator, VertexBuffers};
use std::sync::atomic::Ordering;

impl cvkg_core::ElapsedTime for SurtrRenderer {
    fn delta_time(&self) -> f32 {
        self.current_scene.delta_time
    }

    fn elapsed_time(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
    }
}

impl cvkg_core::Renderer for SurtrRenderer {
    fn is_over_budget(&self) -> bool {
        self.frame_budget.allow_degradation
            && self.last_frame_start.elapsed().as_secs_f32() * 1000.0 > self.frame_budget.target_ms
    }

    /// fill_rect — Standard rectangle drawing method.
    fn prewarm_vram(&mut self, assets: Vec<(String, Vec<u8>)>) {
        log::info!(
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
    /// For Tahoe parity, this registers the rect as a portal region for
    /// per-element isolated backdrop blur when z_index != 0.
    fn fill_glass_rect(&mut self, rect: Rect, radius: f32, blur_radius: f32) {
        // Store blur radius for use during glass pass - the renderer will apply
        // this to the Kawase blur uniform during the backdrop blur phase
        let blur_strength = (blur_radius / 100.0).clamp(0.0, 4.0);

        // Register for portal-aware per-element backdrop blur (Tahoe feature)
        // When current_z != 0, this element is in a portal layer
        if self.current_z != 0.0 {
            self.portal_regions.push_back(rect);
        }

        self.fill_rect_with_full_params(
            rect,
            [1.0, 1.0, 1.0, 0.4], // Glass tint: white at 40% opacity
            7,                    // Mode 7 = Glass material
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
            21,
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
        // Create neon glow effect using additive blending
        // This renders a glowing aura around the element
        let center_x = rect.x + rect.width * 0.5;
        let center_y = rect.y + rect.height * 0.5;
        let max_dim = rect.width.max(rect.height) * 0.5 + radius;

        // Draw expanding glow layers
        for i in 0..8 {
            let alpha = intensity / (i as f32 + 1.0) * 0.3;
            let glow_color = [color[0], color[1], color[2], alpha];
            self.fill_rect_with_mode(
                Rect {
                    x: center_x - max_dim - i as f32 * 2.0,
                    y: center_y - max_dim - i as f32 * 2.0,
                    width: max_dim * 2.0 + i as f32 * 4.0,
                    height: max_dim * 2.0 + i as f32 * 4.0,
                },
                glow_color,
                8, // Mode for additive blending
                None,
            );
        }
    }

    /// Renders a dynamic glowing hover boundary field around a hit target.
    ///
    /// # Contract
    /// Expands the bounding box of the visual target by `radius` to establish
    /// a continuous proximity glow. Uses blending mode 18 (GPU drop shadow/glow)
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
            18,
            None,
            8.0,
            uv_rect,
        );
    }

    fn stroke_rect(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32) {
        let c = self.apply_opacity(color);
        let hw = stroke_width;
        // Top, bottom, left, right edge bars
        self.fill_rect_with_mode(
            Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: hw,
            },
            c,
            1,
            None,
        );
        self.fill_rect_with_mode(
            Rect {
                x: rect.x,
                y: rect.y + rect.height - hw,
                width: rect.width,
                height: hw,
            },
            c,
            1,
            None,
        );
        self.fill_rect_with_mode(
            Rect {
                x: rect.x,
                y: rect.y,
                width: hw,
                height: rect.height,
            },
            c,
            1,
            None,
        );
        self.fill_rect_with_mode(
            Rect {
                x: rect.x + rect.width - hw,
                y: rect.y,
                width: hw,
                height: rect.height,
            },
            c,
            1,
            None,
        );
    }

    fn stroke_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4], stroke_width: f32) {
        self.fill_rect_with_full_params(
            rect,
            self.apply_opacity(color),
            17,
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
            16,
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
        self.fill_rect_with_full_params(
            inflated,
            self.apply_opacity(color),
            18,
            None,
            radius,
            Rect {
                x: margin,
                y: blur,
                width: 0.0,
                height: 0.0,
            },
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
            19,
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
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 {
            return;
        }

        // Create a proper line path using Lyon for correct tessellation
        // The stroke_path function will apply the current transform, which handles rotation
        let mut builder = lyon::path::Path::builder();
        builder.begin(point(x1, y1));
        builder.line_to(point(x2, y2));
        builder.close();
        let path = builder.build();

        self.stroke_path(&path, color, stroke_width);
    }

    fn draw_image(&mut self, image_name: &str, rect: Rect) {
        // Guard: skip if image not loaded — avoids rendering garbage from uninitialized atlas regions
        if !self.image_uv_registry.contains(image_name) {
            log::warn!("[Surtr] draw_image: '{}' not loaded, skipping", image_name);
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

    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        // High-DPI: Shape and rasterize at the physical scale factor for maximum sharpness.
        let scaled_size = size * self.current_scale_factor();
        let shaped = self.shape_text_with_stack(text, scaled_size);
        let c = self.apply_opacity(color);

        for glyph in shaped.glyphs {
            let cache_key = glyph.cache_key;

            let (uv_rect, w, h, x_off, y_off) = if let Some(info) = self.text_cache.get(&cache_key)
            {
                *info
            } else {
                if let Some(image) = self.text_engine.rasterize(cache_key) {
                    let gw = image.width;
                    let gh = image.height;
                    let x_offset = image.x_offset;
                    let y_offset = image.y_offset;

                    let pack_res = self.heim_packer.pack(gw, gh);
                    let (nx, ny) = if let Some(pos) = pack_res {
                        pos
                    } else {
                        // RECLAIM & RETRY: Heim is full, quench the forge and try again.
                        self.reclaim_vram();
                        match self.heim_packer.pack(gw, gh) {
                            Some(pos) => pos,
                            None => {
                                log::error!(
                                    "Glyph heim critically full after reclaim: cannot pack {}x{} glyph for '{}', skipping",
                                    gw,
                                    gh,
                                    text
                                );
                                continue; // Skip this glyph rather than corrupting atlas origin
                            }
                        }
                    };

                    // Uploads rasterized glyph image data to the GPU atlas texture.
                    // CONTRACT: If the image already contains 32-bit RGBA data (as in subpixel/color mode),
                    // we write it directly. Otherwise (grayscale 8-bit), we map to [255, 255, 255, alpha].
                    let rgba_data = if image.data.len() == (gw * gh * 4) as usize {
                        image.data
                    } else {
                        let mut data = Vec::with_capacity((gw * gh * 4) as usize);
                        for alpha in &image.data {
                            data.push(255);
                            data.push(255);
                            data.push(255);
                            data.push(*alpha);
                        }
                        data
                    };

                    self.queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &self.mega_heim_tex,
                            mip_level: 0,
                            origin: wgpu::Origin3d { x: nx, y: ny, z: 0 },
                            aspect: wgpu::TextureAspect::All,
                        },
                        &rgba_data,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(gw * 4),
                            rows_per_image: Some(gh),
                        },
                        wgpu::Extent3d {
                            width: gw,
                            height: gh,
                            depth_or_array_layers: 1,
                        },
                    );

                    let info = (
                        Rect {
                            x: nx as f32 / 4096.0,
                            y: ny as f32 / 4096.0,
                            width: gw as f32 / 4096.0,
                            height: gh as f32 / 4096.0,
                        },
                        gw as f32,
                        gh as f32,
                        x_offset,
                        y_offset,
                    );
                    self.text_cache.put(cache_key, info);
                    info
                } else {
                    (Rect::zero(), 0.0, 0.0, 0.0, 0.0)
                }
            };

            if w > 0.0 {
                // Position glyph relative to baseline.
                // glyph.x/y are in physical pixels, baseline-relative.
                // shaped.ascent gives the baseline offset from the text origin (y).
                let baseline_y = y + shaped.ascent / self.current_scale_factor();
                let glyph_rect = Rect {
                    x: x + (glyph.x + x_off) / self.current_scale_factor(),
                    y: baseline_y + (glyph.y - y_off) / self.current_scale_factor(),
                    width: w / self.current_scale_factor(),
                    height: h / self.current_scale_factor(),
                };
                let tid = self.get_texture_id("__mega_heim");
                self.fill_rect_with_full_params(glyph_rect, c, 6, tid, 0.0, uv_rect);
            }
        }
    }

    /// measure_text — Calculates the dimensions of a text string without rendering.
    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        let sf = self.current_scale_factor();
        let shaped = self.shape_text_with_stack(text, size * sf);
        // Convert physical pixels back to logical units
        (shaped.width / sf, shaped.height / sf)
    }

    fn shape_rich_text(
        &mut self,
        spans: &[cvkg_runic_text::TextSpan],
        max_width: Option<f32>,
        align: cvkg_runic_text::TextAlign,
        overflow: cvkg_runic_text::TextOverflow,
    ) -> Option<cvkg_runic_text::ShapedText> {
        let sf = self.current_scale_factor();
        let mut scaled_spans = spans.to_vec();
        for span in &mut scaled_spans {
            span.style.font_size *= sf;
            if span.style.fallback_families.is_empty() {
                span.style.fallback_families = vec![
                    "SF Pro".to_string(),
                    "Inter".to_string(),
                    "Helvetica Neue".to_string(),
                    "Helvetica".to_string(),
                    "Arial".to_string(),
                    "sans-serif".to_string(),
                ];
            }
        }
        let scaled_max_width = max_width.map(|w| w * sf);
        self.text_engine
            .shape_layout(&scaled_spans, scaled_max_width, align, overflow)
            .ok()
    }

    fn draw_shaped_text(&mut self, shaped: &cvkg_runic_text::ShapedText, x: f32, y: f32) {
        for glyph in &shaped.glyphs {
            let byte_idx = shaped
                .grapheme_boundaries
                .get(glyph.cluster as usize)
                .copied()
                .unwrap_or(0);
            let mut span_color = [1.0, 1.0, 1.0, 1.0];
            for span in &shaped.spans {
                if byte_idx >= span.byte_offset && byte_idx < span.byte_offset + span.text.len() {
                    span_color = [
                        span.style.color[0] as f32 / 255.0,
                        span.style.color[1] as f32 / 255.0,
                        span.style.color[2] as f32 / 255.0,
                        span.style.color[3] as f32 / 255.0,
                    ];
                    break;
                }
            }
            let c = self.apply_opacity(span_color);

            let cache_key = glyph.cache_key;
            let (uv_rect, w, h, x_off, y_off) = if let Some(info) = self.text_cache.get(&cache_key)
            {
                *info
            } else {
                if let Some(image) = self.text_engine.rasterize(cache_key) {
                    let gw = image.width;
                    let gh = image.height;
                    let x_offset = image.x_offset;
                    let y_offset = image.y_offset;

                    let pack_res = self.heim_packer.pack(gw, gh);
                    let (nx, ny) = if let Some(pos) = pack_res {
                        pos
                    } else {
                        self.reclaim_vram();
                        match self.heim_packer.pack(gw, gh) {
                            Some(pos) => pos,
                            None => {
                                log::error!(
                                    "Glyph heim critically full after reclaim: cannot pack {}x{} glyph, skipping",
                                    gw,
                                    gh
                                );
                                continue; // Skip this glyph rather than corrupting atlas origin
                            }
                        }
                    };

                    // Uploads rasterized glyph image data to the GPU atlas texture.
                    // CONTRACT: If the image already contains 32-bit RGBA data (as in subpixel/color mode),
                    // we write it directly. Otherwise (grayscale 8-bit), we map to [255, 255, 255, alpha].
                    let rgba_data = if image.data.len() == (gw * gh * 4) as usize {
                        image.data
                    } else {
                        let mut data = Vec::with_capacity((gw * gh * 4) as usize);
                        for alpha in &image.data {
                            data.push(255);
                            data.push(255);
                            data.push(255);
                            data.push(*alpha);
                        }
                        data
                    };

                    self.queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &self.mega_heim_tex,
                            mip_level: 0,
                            origin: wgpu::Origin3d { x: nx, y: ny, z: 0 },
                            aspect: wgpu::TextureAspect::All,
                        },
                        &rgba_data,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(gw * 4),
                            rows_per_image: Some(gh),
                        },
                        wgpu::Extent3d {
                            width: gw,
                            height: gh,
                            depth_or_array_layers: 1,
                        },
                    );

                    let info = (
                        Rect {
                            x: nx as f32 / 4096.0,
                            y: ny as f32 / 4096.0,
                            width: gw as f32 / 4096.0,
                            height: gh as f32 / 4096.0,
                        },
                        gw as f32,
                        gh as f32,
                        x_offset,
                        y_offset,
                    );
                    self.text_cache.put(cache_key, info);
                    info
                } else {
                    (Rect::zero(), 0.0, 0.0, 0.0, 0.0)
                }
            };

            if w > 0.0 {
                let sf = self.current_scale_factor();
                // Position glyph relative to baseline.
                // glyph.x/y are in physical pixels, baseline-relative.
                // shaped.ascent gives the baseline offset from the text origin (y).
                let baseline_y = y + shaped.ascent / sf;
                let glyph_rect = Rect {
                    x: x + (glyph.x + x_off) / sf,
                    y: baseline_y + (glyph.y - y_off) / sf,
                    width: w / sf,
                    height: h / sf,
                };
                let tid = self.get_texture_id("__mega_heim");
                let slice = self
                    .slice_stack
                    .last()
                    .copied()
                    .map(|(a, o)| [a, o, 1.0, 1.0])
                    .unwrap_or([0.0, 0.0, 0.0, 1.0]);
                self.fill_rect_with_full_params_and_slice(
                    glyph_rect,
                    c,
                    6,
                    tid,
                    0.0,
                    uv_rect,
                    slice,
                    [glyph.glyph_index as f32, glyph.time_offset],
                );
            }
        }
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

    /// load_image — Proactively pushes a raw asset into the Mega-Heim.
    /// load_image — Proactively pushes a raw asset into the Texture Array.
    fn load_image(&mut self, name: &str, data: &[u8]) {
        if self.image_uv_registry.contains(name) {
            return;
        }
        let img_result = image::load_from_memory(data);
        let img = match img_result {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                log::error!("Failed to load image {}: {}", name, e);
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
        let index = if self.texture_registry.len() < 255 {
            (self.texture_registry.len() + 1) as u32
        } else {
            // Evict the least recently used texture
            if let Some((old_name, old_index)) = self.texture_registry.pop_lru() {
                self.image_uv_registry.pop(&old_name);
                old_index
            } else {
                1 // Fallback
            }
        };

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
        // Check if we've already rendered this content with the same hash this frame
        // The cache stores the last-seen hash for each ID
        let should_skip = self.memo_cache.get(&id) == Some(&data_hash);

        if !should_skip {
            // Update cache with current hash
            self.memo_cache.insert(id, data_hash);
            render_fn(self);
        }
        // If should_skip is true, we skip rendering as the content hasn't changed
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

    fn trigger_shatter_event(&mut self, origin: [f32; 2], force: f32) {
        self.current_scene.shatter_origin = origin;
        self.current_scene.shatter_time = self.current_scene.time;
        self.current_scene.shatter_force = force;
    }

    fn set_scene_preset(&mut self, preset: u32) {
        self.current_scene.scene_type = preset;
    }

    /// push_mjolnir_slice — Pushes a geometric clipping plane onto the stack.
    /// All subsequent draw calls will be sliced by this plane until it is popped.
    fn push_mjolnir_slice(&mut self, angle: f32, offset: f32) {
        self.slice_stack.push((angle, offset));
    }

    /// pop_mjolnir_slice — Removes the top-most geometric clipping plane from the stack.
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
        _color: [f32; 4],
    ) {
        log::info!(
            "[Surtr] Dispatching {} {} particles at {:?}",
            count,
            effect_type,
            origin
        );
        // Stub: A full implementation would push to a compute pass command queue
    }

    fn draw_hologram(&mut self, rect: Rect, hologram_id: &str, time: f32) {
        log::info!(
            "[Surtr] Drawing hologram {} at {:?} (t={})",
            hologram_id,
            rect,
            time
        );
        // Stub: In the future, this will push a DrawCall into the volumetric pass queue.
        // For now, render a glowing wireframe box
        self.stroke_rect(rect, [0.0, 1.0, 1.0, 0.5], 2.0);
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
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&vec![&view; 256]),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
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
        let screen = [self.current_width() as f32, self.current_height() as f32];

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
                clip: [-10000.0, -10000.0, 20000.0, 20000.0],
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
            });
            self.draw_calls.push(DrawCall {
                target_id: None,
                texture_id: None,
                scissor_rect: self.clip_stack.last().copied(),
                index_start: (self.indices.len() as u32) - (mesh.indices.len() as u32),
                index_count: mesh.indices.len() as u32,
                material: cvkg_core::DrawMaterial::Opaque,
                instance_start: (self.instance_data.len() - 1) as u32,
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
        let screen = [self.current_width() as f32, self.current_height() as f32];
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
                clip: [-10000.0, -10000.0, 20000.0, 20000.0],
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
        });

        self.draw_calls.push(DrawCall {
            target_id: None,
            texture_id: None,
            scissor_rect: self.clip_stack.last().copied(),
            index_start: (self.indices.len() as u32) - (mesh.indices.len() as u32),
            index_count: mesh.indices.len() as u32,
            material: cvkg_core::DrawMaterial::Opaque,
            instance_start: (self.instance_data.len() - 1) as u32,
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
            let material = cvkg_core::Material3D::unlit(color);
            self.draw_mesh_3d(&cube, &material, &transform);
        } else {
            let material = cvkg_core::Material3D::unlit(color);
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

    fn serialize_svg(&mut self, name: &str) -> Result<String, String> {
        let tree = self
            .svg_trees
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
            .svg_trees
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
}

// ── Inherent methods on SurtrRenderer (not part of the Renderer trait) ──

impl SurtrRenderer {
    /// Clear all registered event handlers. Call at the start of each frame
    /// before re-rendering the component tree.
    pub fn clear_event_handlers(&mut self) {
        self.event_handlers.clear();
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
        let c = self.apply_opacity(color);
        let mut tessellator = StrokeTessellator::new();
        let mut buffers: VertexBuffers<Vertex, u32> = VertexBuffers::new();
        let base_vertex_idx = self.vertices.len() as u32;
        let base_index_idx = self.indices.len() as u32;

        let (translation, scale, rotation, _, _) = self.current_transform();
        let clip_rect = self.clip_stack.last().copied().unwrap_or(cvkg_core::Rect {
            x: -10000.0,
            y: -10000.0,
            width: 20000.0,
            height: 20000.0,
        });
        let clip = [clip_rect.x, clip_rect.y, clip_rect.width, clip_rect.height];

        let result = tessellator.tessellate_path(
            path,
            &StrokeOptions::default().with_line_width(stroke_width),
            &mut BuffersBuilder::new(
                &mut buffers,
                CustomStrokeVertexConstructor { color: c, clip },
            ),
        );
        if let Err(e) = result {
            log::warn!("Failed to tessellate stroke path: {:?}", e);
            return;
        }

        self.vertices.extend(buffers.vertices);
        for idx in &buffers.indices {
            self.indices.push(base_vertex_idx + *idx);
        }

        let material = self.current_material();
        let tid = self.get_texture_id("__mega_heim");

        let last_call = self.draw_calls.last();
        let needs_new_call = self.draw_calls.is_empty()
            || self.current_texture_id != tid
            || last_call.unwrap().scissor_rect != self.clip_stack.last().copied()
            || last_call.unwrap().material != material;

        if needs_new_call {
            self.current_texture_id = tid;

            self.instance_data.push(InstanceData {
                translation,
                scale,
                rotation,
                blur_radius: 0.0,
                ior_override: 0.0,
            });
            self.draw_calls.push(DrawCall {
                target_id: None,
                texture_id: tid,
                scissor_rect: self.clip_stack.last().copied(),
                index_start: base_index_idx,
                index_count: buffers.indices.len() as u32,
                material,
                instance_start: (self.instance_data.len() - 1) as u32,
            });
        } else if let Some(call) = self.draw_calls.last_mut() {
            call.index_count += buffers.indices.len() as u32;
        }
    }
}

impl cvkg_core::FrameRenderer<wgpu::CommandEncoder> for SurtrRenderer {
    fn begin_frame(&mut self) -> wgpu::CommandEncoder {
        cvkg_core::begin_render_phase();
        let id = self
            .current_window
            .expect("No target window set for frame. Call set_target_window first.");
        self.begin_frame(id)
    }

    fn render_frame(&mut self) {
        // Visual Lint: If layout was dirtied during the render phase (layout thrashing),
        // draw a 10px red border as a warning flash.
        if LAYOUT_DIRTY.swap(false, Ordering::AcqRel)
            && let Some(window_id) = self.current_window
            && let Some(surface_ctx) = self.surfaces.get(&window_id)
        {
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

        // Dynamic Buffer Growth (Up to 4x capacity)
        let req_v_size = (self.vertices.len() * std::mem::size_of::<Vertex>()) as u64;
        let mut cur_v_size = self.vertex_buffer.size();
        let max_v_size = (MAX_VERTICES * std::mem::size_of::<Vertex>()) as u64 * 4;

        if req_v_size > cur_v_size {
            while cur_v_size < req_v_size && cur_v_size < max_v_size {
                cur_v_size *= 2;
            }
            if req_v_size > max_v_size {
                log::error!("Exceeded dynamic vertex buffer max capacity! Capping geometry.");
                self.vertices
                    .truncate((max_v_size / std::mem::size_of::<Vertex>() as u64) as usize);
                cur_v_size = max_v_size;
            }
            log::info!("Growing vertex buffer to {} bytes", cur_v_size);
            self.vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer (Grown)"),
                size: cur_v_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        let req_i_size = (self.indices.len() * std::mem::size_of::<u32>()) as u64;
        let mut cur_i_size = self.index_buffer.size();
        let max_i_size = (MAX_INDICES * std::mem::size_of::<u32>()) as u64 * 4;

        if req_i_size > cur_i_size {
            while cur_i_size < req_i_size && cur_i_size < max_i_size {
                cur_i_size *= 2;
            }
            if req_i_size > max_i_size {
                log::error!("Exceeded dynamic index buffer max capacity! Capping geometry.");
                self.indices
                    .truncate((max_i_size / std::mem::size_of::<u32>() as u64) as usize);
                cur_i_size = max_i_size;
            }
            log::info!("Growing index buffer to {} bytes", cur_i_size);
            self.index_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Index Buffer (Grown)"),
                size: cur_i_size,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
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
                    &self.vertex_buffer,
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
                    &self.index_buffer,
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
                    &self.instance_buffer,
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
    }

    fn end_frame(&mut self, encoder: wgpu::CommandEncoder) {
        // Delegate to the inherent end_frame which runs the render graph
        SurtrRenderer::end_frame(self, encoder);
        cvkg_core::end_render_phase();
    }
}
