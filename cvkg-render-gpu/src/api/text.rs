use crate::renderer::GpuRenderer;
use crate::vertex::{InstanceData, Vertex};
use cvkg_core::Rect;
use std::sync::Arc;

impl GpuRenderer {
    /// Inherent method: clear the text shaping cache.
    pub fn clear_text_cache_impl(&mut self) {
        self.text.shaped_cache.clear();
    }

    /// Measure text using the shaped text cache.
    pub(crate) fn measure_text_impl(&mut self, text: &str, size: f32) -> (f32, f32) {
        let cache_key = (text.to_string(), (size * 100.0) as u32);
        if let Some(shaped) = self.text.shaped_cache.get(&cache_key) {
            return (shaped.width, shaped.height);
        }
        let style = cvkg_runic_text::TextStyle::new("Inter", size);
        let spans = [cvkg_runic_text::TextSpan::new(text, style)];
        if let Some(shaped) = self.shape_rich_text_impl(
            &spans,
            None,
            cvkg_runic_text::TextAlign::Start,
            cvkg_runic_text::TextOverflow::Visible,
        ) {
            let shaped = std::sync::Arc::new(shaped);
            let result = (shaped.width, shaped.height);
            self.text.shaped_cache.put(cache_key, shaped);
            result
        } else {
            (0.0, 0.0)
        }
    }

    /// Shape rich text with support for scale factor and fallbacks.
    pub(crate) fn shape_rich_text_impl(
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
        self.text
            .engine
            .shape_layout(&scaled_spans, scaled_max_width, align, overflow)
            .ok()
    }

    /// Draw shaped text to the renderer buffers.
    pub(crate) fn draw_shaped_text_impl(
        &mut self,
        shaped: &cvkg_runic_text::ShapedText,
        x: f32,
        y: f32,
    ) {
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
            let (uv_rect, w, h, x_off, y_off) = if let Some(info) =
                self.text.glyph_cache.get(&cache_key)
            {
                *info
            } else {
                if let Some(image) = self.text.engine.rasterize(cache_key) {
                    let glyph_id = image.glyph_id;
                    let data_len = image.data.len();
                    let gw = image.width;
                    let gh = image.height;
                    let x_offset = image.x_offset;
                    let y_offset = image.y_offset;
                    let (rgba_data, gw, gh) = glyph_image_to_rgba(image);
                    if gw == 0 || gh == 0 {
                        let info = (Rect::zero(), 0.0, 0.0, 0.0, 0.0);
                        self.text.glyph_cache.put(cache_key, info);
                        continue;
                    }
                    if rgba_data.is_empty() {
                        tracing::warn!(
                            "Glyph rasterizer returned unsupported pixel format for glyph {} ({} bytes, {}x{}), skipping",
                            glyph_id,
                            data_len,
                            gw,
                            gh
                        );
                        continue;
                    }

                    let pack_res = self.heim_packer.pack(gw, gh);
                    let (nx, ny) = if let Some(pos) = pack_res {
                        pos
                    } else {
                        self.reclaim_vram();
                        match self.heim_packer.pack(gw, gh) {
                            Some(pos) => pos,
                            None => {
                                tracing::error!(
                                    "Glyph heim critically full after reclaim: cannot pack {}x{} glyph, skipping",
                                    gw,
                                    gh
                                );
                                continue;
                            }
                        }
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

                    let tex_w = self.mega_heim_tex.width() as f32;
                    let tex_h = self.mega_heim_tex.height() as f32;
                    let info = (
                        Rect {
                            x: nx as f32 / tex_w,
                            y: ny as f32 / tex_h,
                            width: gw as f32 / tex_w,
                            height: gh as f32 / tex_h,
                        },
                        gw as f32,
                        gh as f32,
                        x_offset,
                        y_offset,
                    );
                    self.text.glyph_cache.put(cache_key, info);
                    info
                } else {
                    (Rect::zero(), 0.0, 0.0, 0.0, 0.0)
                }
            };

            if w > 0.0 {
                let sf = self.current_scale_factor();
                let glyph_rect = Rect {
                    x: x + (glyph.x + x_off) / sf,
                    y: y + (glyph.y - y_off) / sf,
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

    /// Draw text using shaped text cache lookups.
    pub(crate) fn draw_text_impl(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        color: [f32; 4],
    ) {
        let cache_key = (text.to_string(), (size * 100.0) as u32);
        let r = (color[0] * 255.0).clamp(0.0, 255.0) as u8;
        let g = (color[1] * 255.0).clamp(0.0, 255.0) as u8;
        let b = (color[2] * 255.0).clamp(0.0, 255.0) as u8;
        let a = (color[3] * 255.0).clamp(0.0, 255.0) as u8;
        let cached = self.text.shaped_cache.get(&cache_key).cloned();
        if let Some(shaped) = cached {
            let color_matches = shaped
                .spans
                .first()
                .map(|s| s.style.color == [r, g, b, a])
                .unwrap_or(false);
            if color_matches {
                self.draw_shaped_text_impl(&shaped, x, y);
                return;
            }
            let mut shaped = (*shaped).clone();
            for span in &mut shaped.spans {
                span.style.color = [r, g, b, a];
            }
            self.draw_shaped_text_impl(&shaped, x, y);
            return;
        }
        let mut style = cvkg_runic_text::TextStyle::new("Inter", size);
        style.color = [r, g, b, a];
        let spans = [cvkg_runic_text::TextSpan::new(text, style)];
        if let Some(shaped) = self.shape_rich_text_impl(
            &spans,
            None,
            cvkg_runic_text::TextAlign::Start,
            cvkg_runic_text::TextOverflow::Visible,
        ) {
            let shaped = std::sync::Arc::new(shaped);
            self.draw_shaped_text_impl(&shaped, x, y);
            self.text.shaped_cache.put(cache_key, shaped);
        }
    }
}

fn glyph_image_to_rgba(image: cvkg_runic_text::GlyphImage) -> (Vec<u8>, u32, u32) {
    let width = image.width;
    let height = image.height;
    let pixels = width.saturating_mul(height) as usize;

    if pixels == 0 || image.data.is_empty() {
        return (Vec::new(), width, height);
    }

    let (bytes_per_pixel, remainder) = (image.data.len() / pixels, image.data.len() % pixels);
    if remainder != 0 {
        tracing::warn!(
            "Glyph rasterizer returned {} bytes for {}x{} glyph; expected whole pixels ({} bytes per pixel)",
            image.data.len(),
            width,
            height,
            bytes_per_pixel
        );
        return (Vec::new(), width, height);
    }

    let rgba_data = match bytes_per_pixel {
        1 => {
            let mut data = Vec::with_capacity(pixels * 4);
            for alpha in &image.data {
                data.push(255);
                data.push(255);
                data.push(255);
                data.push(*alpha);
            }
            data
        }
        3 => {
            let mut data = Vec::with_capacity(pixels * 4);
            for rgb in image.data.chunks_exact(3) {
                let alpha = rgb.iter().copied().max().unwrap_or(0);
                data.push(255);
                data.push(255);
                data.push(255);
                data.push(alpha);
            }
            data
        }
        4 => {
            let mut data = image.data;
            for chunk in data.chunks_exact_mut(4) {
                if chunk[3] == 0 && (chunk[0] > 0 || chunk[1] > 0 || chunk[2] > 0) {
                    chunk[3] = chunk[0].max(chunk[1]).max(chunk[2]);
                }
            }
            data
        }
        _ => {
            tracing::warn!(
                "Glyph rasterizer returned unsupported {} bytes per pixel for {}x{} glyph ({} bytes total)",
                bytes_per_pixel,
                width,
                height,
                image.data.len()
            );
            Vec::new()
        }
    };

    (rgba_data, width, height)
}

#[cfg(test)]
mod tests {
    use super::glyph_image_to_rgba;

    #[test]
    fn glyph_image_to_rgba_keeps_rgba_color_data() {
        let image = cvkg_runic_text::GlyphImage {
            glyph_id: 1,
            width: 2,
            height: 1,
            data: vec![1, 2, 3, 4, 5, 6, 7, 8],
            x_offset: 0.0,
            y_offset: 0.0,
            cache_key: 42,
        };

        assert_eq!(
            glyph_image_to_rgba(image),
            (vec![1, 2, 3, 4, 5, 6, 7, 8], 2, 1)
        );
    }

    #[test]
    fn glyph_image_to_rgba_expands_grayscale_alpha() {
        let image = cvkg_runic_text::GlyphImage {
            glyph_id: 1,
            width: 3,
            height: 1,
            data: vec![0, 128, 255],
            x_offset: 0.0,
            y_offset: 0.0,
            cache_key: 42,
        };

        assert_eq!(
            glyph_image_to_rgba(image),
            (
                vec![255, 255, 255, 0, 255, 255, 255, 128, 255, 255, 255, 255],
                3,
                1
            )
        );
    }

    #[test]
    fn glyph_image_to_rgba_collapses_subpixel_rgb_to_alpha() {
        let image = cvkg_runic_text::GlyphImage {
            glyph_id: 1,
            width: 2,
            height: 1,
            data: vec![0, 128, 255, 255, 0, 64],
            x_offset: 0.0,
            y_offset: 0.0,
            cache_key: 42,
        };

        assert_eq!(
            glyph_image_to_rgba(image),
            (vec![255, 255, 255, 255, 255, 255, 255, 255], 2, 1)
        );
    }
}
