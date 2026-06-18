//! Atlas loading and text shaping utilities.
use crate::renderer::SurtrRenderer;
use cvkg_core::{Rect, Renderer};

impl SurtrRenderer {
    /// load_image_to_heim -- Packs a raw asset into the Mega-Heim.
    /// This is used for common icons to enable aggressive batching (1 draw call).
    pub fn load_image_to_heim(&mut self, name: &str, data: &[u8]) {
        if self.image_uv_registry.contains(name) {
            log::info!("[Surtr] load_image_to_heim: '{}' already in registry, skipping", name);
            return;
        }
        log::info!("[Surtr] load_image_to_heim: decoding '{}' ({} bytes)", name, data.len());
        let img_result = image::load_from_memory(data);
        let img = match img_result {
            Ok(img) => {
                log::info!("[Surtr] decode OK: {}x{}", img.width(), img.height());
                img.to_rgba8()
            }
            Err(e) => {
                log::error!("[Surtr] Failed to load image {} to heim: {}", name, e);
                return;
            }
        };
        let (width, height) = img.dimensions();

        // Pack into heim
        if let Some((x, y)) = self.heim_packer.pack(width, height) {
            let tex_w = self.mega_heim_tex.width() as f32;
            let tex_h = self.mega_heim_tex.height() as f32;
            let uv_rect = Rect {
                x: x as f32 / tex_w,
                y: y as f32 / tex_h,
                width: width as f32 / tex_w,
                height: height as f32 / tex_h,
            };

            // Upload to GPU
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.mega_heim_tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x, y, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                &img,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );

            self.image_uv_registry.put(name.to_string(), uv_rect);
            // Index 0 = mega-heim texture (stored in texture_views[0])
            self.texture_registry.put(name.to_string(), 0);
            log::info!("[Surtr] Packed '{}' into Mega-Heim at ({}, {})", name, x, y);
            log::info!("[Surtr] Registry now contains '{}'", name);
        } else {
            log::warn!(
                "HEIM_FULL: Failed to pack '{}' into Mega-Heim. Falling back to Texture Array.",
                name
            );
            self.load_image(name, data);
        }
    }

    /// Shapes a text string using a predefined system font stack.
    ///
    /// # Contract
    /// Evaluates text shaping with fallbacks: queries "SF Pro Text", "SF Pro", "Inter",
    /// "Helvetica Neue", "Helvetica", "Arial", and defaults back to "sans-serif".
    /// This ensures visual typographic consistency across platforms where specific
    /// branding faces may or may not be installed.
    /// Shapes a text string using a default font stack.
    ///
    /// # Contract
    /// Resolves standard font families in order of system availability. Falls back from
    /// common system sans-serif aliases, to platform-specific sans-serif faces, and finally
    /// to the embedded "Jupiteroid" font as a last resort.
    /// Shapes a text string using a predefined system font stack.
    ///
    /// # Contract
    /// Evaluates text shaping with fallbacks: queries "SF Pro Text", "SF Pro", "Inter",
    /// "Helvetica Neue", "Helvetica", "Arial", and defaults back to "sans-serif".
    /// This ensures visual typographic consistency across platforms where specific
    /// branding faces may or may not be installed.
    ///
    /// The shaped text result is cached in `shaped_text_cache` by content and size.
    /// This layout cache guarantees sub-millisecond execution times for subsequent
    /// lookups, bypassing expensive font config fallback queries on repeating frames.
    pub(crate) fn shape_text_with_stack(
        &mut self,
        text: &str,
        size: f32,
    ) -> cvkg_runic_text::ShapedText {
        let cache_key = (text.to_string(), (size * 100.0) as u32);
        if let Some(shaped) = self.shaped_text_cache.get(&cache_key) {
            return shaped.clone();
        }

        let mut style = cvkg_runic_text::TextStyle::new("Jupiteroid", size);
        style.fallback_families = vec![
            "sans-serif".to_string(),
            // Linux-native (fontconfig standard aliases + common packages)
            "DejaVu Sans".to_string(),
            "Cantarell".to_string(),
            "Liberation Sans".to_string(),
            "Noto Sans".to_string(),
            "Adwaita Sans".to_string(),
            // macOS / Windows
            "SF Pro".to_string(),
            "SF Pro Text".to_string(),
            "Inter".to_string(),
            "Helvetica Neue".to_string(),
            "Helvetica".to_string(),
            "Arial".to_string(),
        ];
        style.render_mode = cvkg_runic_text::RenderMode::Grayscale;
        let spans = vec![cvkg_runic_text::TextSpan::new(text, style)];
        let shaped = self
            .text_engine
            .shape_layout(
                &spans,
                None,
                cvkg_runic_text::TextAlign::Start,
                cvkg_runic_text::TextOverflow::WordWrap,
            )
            .unwrap_or_else(|_| cvkg_runic_text::ShapedText {
                glyphs: Vec::new(),
                lines: Vec::new(),
                width: 0.0,
                height: 0.0,
                text: text.to_string(),
                spans: Vec::new(),
                has_rtl: false,
                ascent: 0.0,
                descent: 0.0,
                line_gap: 0.0,
                grapheme_boundaries: vec![],
            });

        self.shaped_text_cache.insert(cache_key, shaped.clone());
        shaped
    }
}
