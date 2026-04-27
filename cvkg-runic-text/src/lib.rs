//! # cvkg-runic-text
//!
//! Natively integrated Cyber Viking text shaping and layout engine for CVKG.
//! This crate provides a stateless, high-performance text pipeline.

pub use rustybuzz;
pub use swash;
pub use fontdb;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use fontdb::Database;
use rustybuzz::{Face, UnicodeBuffer, Direction};
use swash::scale::{ScaleContext, Render};
use unicode_segmentation::UnicodeSegmentation;
use unicode_bidi::BidiInfo;
use lru::LruCache;
use std::num::NonZeroUsize;

/// CacheKey uniquely identifies a glyph in the atlas.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CacheKey {
    pub glyph_id: u32,
    pub font_id: fontdb::ID,
    pub size: u32,
}

/// A span of text with identical styling.
#[derive(Debug, Clone)]
pub struct TextSpan<'a> {
    pub text: &'a str,
    pub font_family: &'a str,
    pub size: f32,
    pub line_height: f32,
}

impl<'a> TextSpan<'a> {
    pub fn new(text: &'a str, font_family: &'a str, size: f32) -> Self {
        Self {
            text,
            font_family,
            size,
            line_height: size * 1.2,
        }
    }
}

/// RunicTextEngine manages font resources and provides text shaping capabilities.
pub struct RunicTextEngine {
    pub db: Database,
    pub scale_context: ScaleContext,
    fallback_cache: HashMap<char, fontdb::ID>,
    shape_cache: LruCache<u64, ShapedText>,
}

impl RunicTextEngine {
    pub fn new() -> Self {
        let mut db = Database::new();
        db.load_system_fonts();
        Self {
            db,
            scale_context: ScaleContext::new(),
            fallback_cache: HashMap::new(),
            shape_cache: LruCache::new(NonZeroUsize::new(256).unwrap()),
        }
    }

    /// Shape a single string (legacy wrapper)
    pub fn shape(&mut self, text: &str, font_family: &str, size: f32) -> ShapedText {
        self.shape_layout(&[TextSpan::new(text, font_family, size)], f32::MAX)
    }

    fn get_fallback_font(&mut self, c: char) -> Option<fontdb::ID> {
        if let Some(&id) = self.fallback_cache.get(&c) {
            return Some(id);
        }

        let mut found_id = None;
        for face_info in self.db.faces() {
            let id = face_info.id;
            let supports = self.db.with_face_data(id, |data, index| {
                if let Some(face) = Face::from_slice(data, index) {
                    let mut buffer = UnicodeBuffer::new();
                    buffer.push_str(&c.to_string());
                    let output = rustybuzz::shape(&face, &[], buffer);
                    output.glyph_infos().first().map(|i| i.glyph_id != 0).unwrap_or(false)
                } else {
                    false
                }
            }).unwrap_or(false);

            if supports {
                found_id = Some(id);
                break;
            }
        }

        if let Some(id) = found_id {
            self.fallback_cache.insert(c, id);
        }
        found_id
    }

    /// Shape and layout a rich text document with word wrapping, font fallback, and BiDi.
    pub fn shape_layout(&mut self, spans: &[TextSpan], max_width: f32) -> ShapedText {
        // Calculate hash for LRU caching
        let mut hasher = DefaultHasher::new();
        for span in spans {
            span.text.hash(&mut hasher);
            span.font_family.hash(&mut hasher);
            span.size.to_bits().hash(&mut hasher);
            span.line_height.to_bits().hash(&mut hasher);
        }
        max_width.to_bits().hash(&mut hasher);
        let cache_hash = hasher.finish();

        if let Some(cached) = self.shape_cache.get(&cache_hash) {
            return cached.clone();
        }

        let mut glyphs = Vec::new();
        let mut max_x = 0.0f32;
        let mut current_x = 0.0f32;
        let mut current_y = 0.0f32;
        let mut current_line_max_height = 0.0f32;
        let mut global_byte_offset = 0usize;

        for span in spans {
            let query = fontdb::Query {
                families: &[fontdb::Family::Name(span.font_family), fontdb::Family::SansSerif],
                weight: fontdb::Weight::NORMAL,
                stretch: fontdb::Stretch::Normal,
                style: fontdb::Style::Normal,
            };

            let primary_font_id = self.db.query(&query).unwrap_or_else(|| {
                self.db.faces().next().map(|f| f.id).expect("No fonts found in system")
            });

            current_line_max_height = current_line_max_height.max(span.line_height);

            let lines: Vec<&str> = span.text.split('\n').collect();

            for (line_idx, line) in lines.iter().enumerate() {
                if line_idx > 0 {
                    current_y += current_line_max_height;
                    current_x = 0.0;
                    current_line_max_height = span.line_height;
                    global_byte_offset += 1; // For the '\n'
                }

                let bidi_info = BidiInfo::new(line, None);
                let mut line_byte_offset = 0;
                let words = line.split_word_bounds();
                
                for word in words {
                    let level = if line_byte_offset < bidi_info.levels.len() {
                        bidi_info.levels[line_byte_offset]
                    } else {
                        unicode_bidi::Level::ltr()
                    };
                    
                    let is_rtl = level.is_rtl();
                    let word_start_global = global_byte_offset + line_byte_offset;

                    // Chunk the word by font support
                    let mut chunks = Vec::new();
                    let mut current_chunk = String::new();
                    let mut current_font_id = primary_font_id;
                    let mut chunk_start_offset = 0;

                    for c in word.chars() {
                        let mut needs_fallback = false;
                        self.db.with_face_data(primary_font_id, |data, index| {
                            if let Some(face) = Face::from_slice(data, index) {
                                let mut buf = UnicodeBuffer::new();
                                buf.push_str(&c.to_string());
                                let out = rustybuzz::shape(&face, &[], buf);
                                if out.glyph_infos().first().is_some_and(|info| info.glyph_id == 0 && !c.is_whitespace()) {
                                    needs_fallback = true;
                                }
                            }
                        });

                        let target_font = if needs_fallback {
                            self.get_fallback_font(c).unwrap_or(primary_font_id)
                        } else {
                            primary_font_id
                        };

                        if target_font != current_font_id && !current_chunk.is_empty() {
                            chunks.push((current_chunk.clone(), current_font_id, chunk_start_offset));
                            chunk_start_offset += current_chunk.len();
                            current_chunk.clear();
                        }
                        current_chunk.push(c);
                        current_font_id = target_font;
                    }
                    if !current_chunk.is_empty() {
                        chunks.push((current_chunk, current_font_id, chunk_start_offset));
                    }

                    // Measure total word width
                    let mut word_width = 0.0;
                    for (chunk_text, chunk_font_id, _) in &chunks {
                        self.db.with_face_data(*chunk_font_id, |data, index| {
                            if let Some(face) = Face::from_slice(data, index) {
                                let scale = span.size / face.units_per_em() as f32;
                                let mut buffer = UnicodeBuffer::new();
                                buffer.push_str(chunk_text);
                                if is_rtl {
                                    buffer.set_direction(Direction::RightToLeft);
                                } else {
                                    buffer.set_direction(Direction::LeftToRight);
                                }
                                let output = rustybuzz::shape(&face, &[], buffer);
                                for pos in output.glyph_positions() {
                                    word_width += pos.x_advance as f32 * scale;
                                }
                            }
                        });
                    }

                    // Wrap if needed
                    if current_x + word_width > max_width && current_x > 0.0 && !word.trim().is_empty() {
                        current_y += current_line_max_height;
                        current_x = 0.0;
                    }

                    // Shape and emit
                    for (chunk_text, chunk_font_id, chunk_offset) in chunks {
                        self.db.with_face_data(chunk_font_id, |data, index| {
                            if let Some(face) = Face::from_slice(data, index) {
                                let scale = span.size / face.units_per_em() as f32;
                                let mut buffer = UnicodeBuffer::new();
                                buffer.push_str(&chunk_text);
                                if is_rtl {
                                    buffer.set_direction(Direction::RightToLeft);
                                } else {
                                    buffer.set_direction(Direction::LeftToRight);
                                }
                                let output = rustybuzz::shape(&face, &[], buffer);
                                let positions = output.glyph_positions();
                                let infos = output.glyph_infos();

                                for (pos, info) in positions.iter().zip(infos.iter()) {
                                    let cache_key = CacheKey {
                                        glyph_id: info.glyph_id,
                                        font_id: chunk_font_id,
                                        size: (span.size * 64.0) as u32,
                                    };

                                    // Subpixel positioning is maintained natively by keeping exact f32 floats
                                    // rather than rounding advances.
                                    let glyph_global_cluster = word_start_global + chunk_offset + info.cluster as usize;

                                    glyphs.push(GlyphInstance {
                                        id: info.glyph_id,
                                        cache_key,
                                        x: current_x + pos.x_offset as f32 * scale,
                                        y: current_y + pos.y_offset as f32 * scale,
                                        advance_x: pos.x_advance as f32 * scale,
                                        cluster: glyph_global_cluster,
                                        line_height: current_line_max_height,
                                    });
                                    current_x += pos.x_advance as f32 * scale;
                                }
                            }
                        });
                    }
                    max_x = max_x.max(current_x);
                    line_byte_offset += word.len();
                }
                global_byte_offset += line.len();
            }
        }

        let result = ShapedText {
            glyphs,
            width: max_x,
            height: current_y + current_line_max_height,
        };

        self.shape_cache.put(cache_hash, result.clone());
        result
    }

    /// Rasterize a glyph into a bitmap.
    pub fn rasterize(&mut self, key: CacheKey) -> Option<GlyphImage> {
        self.db.with_face_data(key.font_id, |data, index| {
            let font = swash::FontRef::from_index(data, index as usize)?;
            // Use subpixel metrics in the scaler by avoiding aggressive hinting
            let mut scaler = self.scale_context.builder(font)
                .size(key.size as f32 / 64.0)
                .hint(false) 
                .build();
            
            let image = Render::new(&[
                swash::scale::Source::ColorOutline(0),
                swash::scale::Source::Outline,
            ])
            .render(&mut scaler, swash::GlyphId::from(key.glyph_id as u16))?;

            Some(GlyphImage {
                data: image.data,
                width: image.placement.width,
                height: image.placement.height,
                left: image.placement.left,
                top: image.placement.top,
            })
        }).flatten()
    }
}

pub struct GlyphImage {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub left: i32,
    pub top: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct GlyphInstance {
    pub id: u32,
    pub cache_key: CacheKey,
    pub x: f32,
    pub y: f32,
    pub advance_x: f32,
    pub cluster: usize,
    pub line_height: f32,
}

#[derive(Debug, Clone)]
pub struct ShapedText {
    pub glyphs: Vec<GlyphInstance>,
    pub width: f32,
    pub height: f32,
}

impl ShapedText {
    /// Hit Testing: Position-to-Index
    /// Maps a visual (X, Y) coordinate to the logical byte index (cluster) of the text.
    pub fn hit_test(&self, x: f32, y: f32) -> Option<usize> {
        for glyph in &self.glyphs {
            // Check if Y is within this line's vertical bounds
            if y >= glyph.y && y <= glyph.y + glyph.line_height {
                // Check if X is within the glyph's horizontal advance
                if x >= glyph.x && x <= glyph.x + glyph.advance_x {
                    // Determine if the click was on the left or right half of the glyph
                    // to accurately place the cursor.
                    if x < glyph.x + (glyph.advance_x / 2.0) {
                        return Some(glyph.cluster);
                    } else {
                        // Return the next index (naively assuming +1, 
                        // in a full engine we'd need the next cluster offset)
                        return Some(glyph.cluster + 1);
                    }
                }
            }
        }
        
        // If out of bounds, maybe return the end of the string.
        self.glyphs.last().map(|g| g.cluster + 1)
    }

    /// Cursor Mapping: Index-to-Position
    /// Maps a logical byte index (cluster) back to the visual (X, Y) coordinate.
    pub fn cursor_position(&self, byte_index: usize) -> Option<(f32, f32)> {
        for glyph in &self.glyphs {
            if glyph.cluster >= byte_index {
                return Some((glyph.x, glyph.y));
            }
        }
        
        // If index is past the end, place cursor at the end of the last glyph
        if let Some(last) = self.glyphs.last() {
            return Some((last.x + last.advance_x, last.y));
        }
        
        Some((0.0, 0.0))
    }
}

impl Default for RunicTextEngine {
    fn default() -> Self {
        Self::new()
    }
}
