//! Atlas loading and text shaping utilities.
use crate::renderer::SurtrRenderer;
use cvkg_core::{Rect, Renderer};


impl SurtrRenderer {
    /// load_image_to_atlas — Packs a raw asset into the Mega-Atlas.
    /// This is used for common icons to enable aggressive batching (1 draw call).
    pub fn load_image_to_atlas(&mut self, name: &str, data: &[u8]) {
        if self.image_uv_registry.contains(name) {
            return;
        }
        let img_result = image::load_from_memory(data);
        let img = match img_result {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                log::error!("Failed to load image {} to atlas: {}", name, e);
                return;
            }
        };
        let (width, height) = img.dimensions();

        // Pack into atlas
        if let Some((x, y)) = self.atlas_packer.pack(width, height) {
            let uv_rect = Rect {
                x: x as f32 / 4096.0,
                y: y as f32 / 4096.0,
                width: width as f32 / 4096.0,
                height: height as f32 / 4096.0,
            };

            // Upload to GPU
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.mega_atlas_tex,
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
            // Index 0 = mega-atlas texture (stored in texture_views[0])
            self.texture_registry.put(name.to_string(), 0);
            log::debug!(
                "[Surtr] Packed '{}' into Mega-Atlas at ({}, {})",
                name,
                x,
                y
            );
        } else {
            log::warn!(
                "ATLAS_FULL: Failed to pack '{}' into Mega-Atlas. Falling back to Texture Array.",
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
    pub(crate) fn shape_text_with_stack(&mut self, text: &str, size: f32) -> cvkg_runic_text::ShapedText {
        let mut style = cvkg_runic_text::TextStyle::new("SF Pro Text", size);
        style.fallback_families = vec![
            "SF Pro".to_string(),
            "Inter".to_string(),
            "Helvetica Neue".to_string(),
            "Helvetica".to_string(),
            "Arial".to_string(),
            "sans-serif".to_string(),
        ];
        let spans = vec![cvkg_runic_text::TextSpan::new(text, style)];
        self.text_engine
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
            })
    }
}

