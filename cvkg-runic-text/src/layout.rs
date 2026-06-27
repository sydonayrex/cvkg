use rustybuzz::{Direction, Feature, UnicodeBuffer};
use unicode_bidi::BidiInfo;
use unicode_segmentation::UnicodeSegmentation;

use crate::engine::{CacheKey, ResolvedFont, TextEngine, line_bidi_level, reorder_line_rtl};
use crate::global_cache;
use crate::path::{LayoutBoundary, TextPath};
use crate::span::{PortalAlignment, TextSpan, TextSpanKind};
use crate::style::{DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT, TextAlign, TextOverflow, TextStyle};
use crate::types::{GlyphInstance, LineInfo, ShapingError};
use fontdb::Style;

// ── ShapedText ───────────────────────────────────────────────────────────────

/// The result of shaping and laying out text.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapedText {
    /// All positioned glyphs.
    pub glyphs: Vec<GlyphInstance>,
    /// Line information.
    pub lines: Vec<LineInfo>,
    /// Total width of the layout.
    pub width: f32,
    /// Total height of the layout.
    pub height: f32,
    /// The text that was shaped.
    pub text: String,
    /// The spans that were used.
    pub spans: Vec<TextSpan>,
    /// Whether the text has RTL content.
    pub has_rtl: bool,
    /// Font ascent for the primary font.
    pub ascent: f32,
    /// Font descent for the primary font.
    pub descent: f32,
    /// Font line gap for the primary font.
    pub line_gap: f32,
    /// Precomputed grapheme cluster boundaries (byte offsets into `text`).
    pub grapheme_boundaries: Vec<usize>,
}

impl ShapedText {
    /// Find the glyph index for a given byte position in the text.
    pub fn hit_test(&self, byte_index: usize) -> (usize, u32) {
        if self.glyphs.is_empty() {
            return (0, 0);
        }

        let mut target_glyph_idx = 0;
        let mut target_cluster = 0;
        let mut found = false;

        for (i, glyph) in self.glyphs.iter().enumerate() {
            let start = glyph.cluster as usize;
            let end = self.next_grapheme_boundary(start);
            if byte_index >= start && byte_index < end {
                target_glyph_idx = i;
                target_cluster = glyph.cluster;
                found = true;
                break;
            }
        }

        if !found {
            // If out of bounds, return the last visual glyph on the last line
            let last_idx = self.glyphs.len() - 1;
            return (last_idx, self.glyphs[last_idx].cluster);
        }

        (target_glyph_idx, target_cluster)
    }

    /// Get the cursor position (x, line_index) for a byte index.
    pub fn cursor_position(&self, byte_index: usize) -> (f32, usize) {
        if self.glyphs.is_empty() {
            return (0.0, 0);
        }

        let target_glyph_idx;
        let is_after;

        if byte_index >= self.text.len() {
            let mut last_logical_idx = 0;
            let mut max_cluster = 0;
            for (i, glyph) in self.glyphs.iter().enumerate() {
                if glyph.cluster >= max_cluster {
                    max_cluster = glyph.cluster;
                    last_logical_idx = i;
                }
            }
            target_glyph_idx = last_logical_idx;
            is_after = true;
        } else {
            let (idx, _) = self.hit_test(byte_index);
            target_glyph_idx = idx;
            is_after = false;
        }

        let mut line_idx = 0;
        for (li, line) in self.lines.iter().enumerate() {
            if target_glyph_idx >= line.glyph_start && target_glyph_idx < line.glyph_end {
                line_idx = li;
                break;
            }
        }

        let glyph = &self.glyphs[target_glyph_idx];
        let line = &self.lines[line_idx];

        let mut x = line.x_offset + glyph.x;
        if is_after {
            if !glyph.is_rtl {
                x += glyph.advance_width;
            }
        } else {
            if glyph.is_rtl {
                x += glyph.advance_width;
            }
        }

        (x, line_idx)
    }

    /// Get selection rectangles for a byte range [start, end).
    pub fn selection_rects(&self, start: usize, end: usize) -> Vec<[f32; 4]> {
        if self.glyphs.is_empty() || start >= end {
            return vec![];
        }

        let mut rects = Vec::new();
        let mut current_rect: Option<[f32; 4]> = None;

        for (i, glyph) in self.glyphs.iter().enumerate() {
            let cluster_start = glyph.cluster as usize;
            let cluster_end = self.next_grapheme_boundary(cluster_start);

            // Check if this glyph's cluster overlaps with the selection
            if cluster_start < end && cluster_end > start {
                // Find the line for y/height
                let mut line_top = 0.0f32;
                let mut line_h = self.height;
                let mut line_x_offset = 0.0f32;
                for line in &self.lines {
                    if i >= line.glyph_start && i < line.glyph_end {
                        line_top = line.baseline_y - self.ascent;
                        line_h = line.height;
                        line_x_offset = line.x_offset;
                        break;
                    }
                }

                let x = line_x_offset + glyph.x;
                let w = glyph.advance_width.max(1.0);

                if let Some(ref mut rect) = current_rect {
                    if (rect[0] + rect[2] - x).abs() < 2.0 && (rect[1] - line_top).abs() < 1.0 {
                        // Extend current rect
                        rect[2] = (x + w) - rect[0];
                    } else {
                        // Start new rect
                        rects.push(*rect);
                        current_rect = Some([x, line_top, w, line_h]);
                    }
                } else {
                    current_rect = Some([x, line_top, w, line_h]);
                }
            }
        }

        if let Some(rect) = current_rect {
            rects.push(rect);
        }

        rects
    }

    /// Get the next grapheme boundary given a byte index.
    fn next_grapheme_boundary(&self, current: usize) -> usize {
        for &b in &self.grapheme_boundaries {
            if b > current {
                return b;
            }
        }
        self.text.len()
    }
}

// ── TextEngine layout methods ────────────────────────────────────────────────

impl TextEngine {
    /// Build rustybuzz Features from a TextStyle.
    pub(crate) fn build_features(style: &TextStyle) -> Vec<Feature> {
        use rustybuzz::ttf_parser::Tag;
        let mut features = vec![
            Feature::new(Tag::from_bytes(b"liga"), 1, 0..usize::MAX),
            Feature::new(Tag::from_bytes(b"kern"), 1, 0..usize::MAX),
            Feature::new(Tag::from_bytes(b"calt"), 1, 0..usize::MAX),
        ];

        for extra in &style.extra_features {
            features.push(Feature::new(
                Tag::from_bytes(&extra.tag.to_be_bytes()),
                extra.value,
                0..usize::MAX,
            ));
        }

        features
    }

    /// Computes a unique cache key for a glyph instance under a specific text style.
    ///
    /// # Contract
    /// Hashes the font identifier, quantized font size, glyph ID, and stylistic attributes
    /// (weight, stretch, style) into a single deterministic 64-bit unsigned integer to
    /// prevent texture atlas key collisions while keeping cache size bounded.
    pub(crate) fn calculate_glyph_cache_key(
        font_cache_key: u64,
        font_size: f32,
        glyph_id: u16,
        style: &TextStyle,
    ) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        font_cache_key.hash(&mut hasher);
        ((font_size * 2.0).round() as u32).hash(&mut hasher);
        glyph_id.hash(&mut hasher);
        style.weight.0.hash(&mut hasher);
        style.stretch.to_number().hash(&mut hasher);
        let style_discriminant = match style.style {
            Style::Normal => 0u8,
            Style::Italic => 1u8,
            Style::Oblique => 2u8,
        };
        style_discriminant.hash(&mut hasher);
        hasher.finish()
    }

    /// Shape a single run of text.
    pub(crate) fn shape_run(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: Direction,
    ) -> Result<Vec<GlyphInstance>, ShapingError> {
        let resolved = self.resolve_font(style)?;

        let features = Self::build_features(style);

        // Build cache key
        let cache_key = CacheKey::new(
            text,
            resolved.cache_key,
            style.font_size,
            style.weight,
            style.stretch,
            style.style,
            direction,
            style.letter_spacing,
            style.word_spacing,
        );

        // Check cache first
        if let Some(glyphs) = global_cache::global_cache_get(&cache_key) {
            return Ok(glyphs.clone());
        }

        // Create rustybuzz face
        let face = resolved
            .primary
            .face()
            .ok_or(ShapingError::InvalidFontData)?;

        // Build buffer
        let mut buffer = UnicodeBuffer::new();
        buffer.push_str(text);
        buffer.set_direction(direction);

        // Shape
        let output = rustybuzz::shape(&face, &features, buffer);

        let glyph_infos = output.glyph_infos();
        let glyph_positions = output.glyph_positions();

        let scale = style.font_size / (resolved.units_per_em as f32);

        let mut glyphs = Vec::new();
        let mut x_offset = 0.0f32;

        for (info, pos) in glyph_infos.iter().zip(glyph_positions.iter()) {
            let advance = (pos.x_advance as f32) * scale;
            let letter_space = if Self::is_space_cluster(text, info.cluster) {
                style.word_spacing
            } else {
                0.0
            };

            let glyph_cache_key = Self::calculate_glyph_cache_key(
                resolved.cache_key,
                style.font_size,
                info.glyph_id as u16,
                style,
            );

            glyphs.push(GlyphInstance {
                glyph_id: info.glyph_id as u16,
                x: x_offset + (pos.x_offset as f32) * scale,
                y: (pos.y_offset as f32) * scale,
                angle: 0.0,
                advance_width: advance + style.letter_spacing + letter_space,
                advance_height: (pos.y_advance as f32) * scale,
                cluster: info.cluster,
                is_rtl: direction == Direction::RightToLeft,
                cache_key: glyph_cache_key,
                glyph_index: 0,
                time_offset: 0.0,
            });

            x_offset += advance + style.letter_spacing + letter_space;
        }

        // Monospace Integrity Enforcement (P0-39)
        let is_monospace = style.family.to_lowercase().contains("mono")
            || style.family.to_lowercase() == "courier"
            || style.family.to_lowercase() == "consolas";

        if is_monospace && !glyphs.is_empty() {
            let mut max_advance = 0.0f32;
            for g in &glyphs {
                if g.advance_width > max_advance {
                    max_advance = g.advance_width;
                }
            }
            if max_advance > 0.0 {
                let mut current_x = 0.0;
                for g in &mut glyphs {
                    let offset = (max_advance - g.advance_width) / 2.0;
                    g.x = current_x + offset;
                    g.advance_width = max_advance;
                    current_x += max_advance;
                }
            }
        }

        // Apply font fallback for missing glyphs
        self.apply_fallbacks(&mut glyphs, text, style, &resolved, &features);

        // Update cache
        global_cache::global_cache_insert(cache_key, glyphs.clone());

        Ok(glyphs)
    }

    /// Check if a cluster represents a space character.
    fn is_space_cluster(text: &str, cluster: u32) -> bool {
        let byte_idx = cluster as usize;
        if byte_idx < text.len() {
            text[byte_idx..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_whitespace())
        } else {
            false
        }
    }

    /// Resolves missing glyphs in primary font by looking up fallback fonts.
    ///
    /// # Contract
    /// Evaluates glyph instances in place. For any glyph with ID 0, queries loaded
    /// fallback fonts sequentially and overrides the ID, position metrics, and calculates
    /// a new unique cache key using the fallback font identity if a match is found.
    fn apply_fallbacks(
        &mut self,
        glyphs: &mut [GlyphInstance],
        text: &str,
        style: &TextStyle,
        resolved: &ResolvedFont,
        features: &[Feature],
    ) {
        let len = glyphs.len();
        for i in 0..len {
            if glyphs[i].glyph_id == 0 {
                let glyph_cluster = glyphs[i].cluster;
                let glyph_is_rtl = glyphs[i].is_rtl;
                let glyph_x = glyphs[i].x;

                let byte_idx = glyph_cluster as usize;
                let grapheme = if byte_idx < text.len() {
                    use unicode_segmentation::UnicodeSegmentation;
                    text[byte_idx..]
                        .graphemes(true)
                        .next()
                        .unwrap_or("\u{FFFD}")
                } else {
                    "\u{FFFD}"
                };

                // Try each fallback font
                for fallback in &resolved.fallbacks {
                    if let Some(face) = fallback.face() {
                        let mut buf = UnicodeBuffer::new();
                        buf.push_str(grapheme);
                        buf.set_direction(if glyph_is_rtl {
                            Direction::RightToLeft
                        } else {
                            Direction::LeftToRight
                        });

                        let output = rustybuzz::shape(&face, features, buf);
                        let infos = output.glyph_infos();
                        let positions = output.glyph_positions();

                        if let (Some(info), Some(pos)) = (infos.first(), positions.first())
                            && info.glyph_id != 0
                        {
                            let scale = style.font_size / (resolved.units_per_em as f32);
                            glyphs[i].glyph_id = info.glyph_id as u16;
                            glyphs[i].x = glyph_x + (pos.x_offset as f32) * scale;
                            glyphs[i].y = (pos.y_offset as f32) * scale;
                            glyphs[i].advance_width = (pos.x_advance as f32) * scale;

                            let fallback_key = fallback.key;
                            glyphs[i].cache_key = Self::calculate_glyph_cache_key(
                                fallback_key,
                                style.font_size,
                                info.glyph_id as u16,
                                style,
                            );
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Shape and layout text with the given spans.
    pub fn shape_layout(
        &mut self,
        spans: &[TextSpan],
        max_width: Option<f32>,
        align: TextAlign,
        overflow: TextOverflow,
    ) -> Result<ShapedText, ShapingError> {
        self.shape_layout_ex(spans, max_width, align, overflow, None, None)
    }

    /// Shape and layout text with advanced capabilities (curved text paths and boundaries).
    ///
    /// # Contract
    /// Performs shaping over the spans and applies line breaking. If a `path` is provided,
    /// positions and rotates glyphs along the Bezier curve. If a `boundary` is provided,
    /// wrapping reflows dynamically to fit within the geometry.
    pub fn shape_layout_ex(
        &mut self,
        spans: &[TextSpan],
        max_width: Option<f32>,
        align: TextAlign,
        overflow: TextOverflow,
        path: Option<TextPath>,
        boundary: Option<LayoutBoundary>,
    ) -> Result<ShapedText, ShapingError> {
        if spans.is_empty() {
            return Ok(ShapedText {
                glyphs: vec![],
                lines: vec![],
                width: 0.0,
                height: 0.0,
                text: String::new(),
                spans: vec![],
                has_rtl: false,
                ascent: 0.0,
                descent: 0.0,
                line_gap: 0.0,
                grapheme_boundaries: vec![],
            });
        }

        // Concatenate all text
        let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();

        // Detect BiDi
        let bidi = unicode_bidi::BidiInfo::new(&full_text, Some(unicode_bidi::Level::ltr()));

        let mut all_glyphs: Vec<GlyphInstance> = Vec::new();
        let mut has_rtl = false;
        let mut primary_metrics = (0.0f32, 0.0f32, 0.0f32);
        let mut primary_line_height_px = DEFAULT_LINE_HEIGHT * DEFAULT_FONT_SIZE;
        let mut global_glyph_index = 0;

        let mut span_byte_offset = 0;
        for span in spans {
            let span_byte_end = span_byte_offset + span.text.len();

            let mut chunk_start = span_byte_offset;
            while chunk_start < span_byte_end {
                let current_level = if chunk_start < bidi.levels.len() {
                    bidi.levels[chunk_start]
                } else {
                    unicode_bidi::Level::ltr()
                };

                let mut chunk_end = chunk_start + 1;
                while chunk_end < span_byte_end {
                    let level = if chunk_end < bidi.levels.len() {
                        bidi.levels[chunk_end]
                    } else {
                        unicode_bidi::Level::ltr()
                    };
                    if level != current_level {
                        break;
                    }
                    chunk_end += 1;
                }

                let direction = if current_level.is_rtl() {
                    has_rtl = true;
                    Direction::RightToLeft
                } else {
                    Direction::LeftToRight
                };

                let chunk_text = &full_text[chunk_start..chunk_end];

                let mut run_glyphs = match &span.kind {
                    TextSpanKind::Text => self.shape_run(chunk_text, &span.style, direction)?,
                    TextSpanKind::Portal { width, height, .. } => {
                        vec![GlyphInstance {
                            glyph_id: 0xFFFF,
                            x: 0.0,
                            y: 0.0,
                            angle: 0.0,
                            advance_width: *width,
                            advance_height: *height,
                            cluster: 0,
                            is_rtl: direction == Direction::RightToLeft,
                            cache_key: 0,
                            glyph_index: 0,
                            time_offset: 0.0,
                        }]
                    }
                };
                if all_glyphs.is_empty() {
                    primary_metrics = (
                        span.style.font_size * 0.8,
                        span.style.font_size * 0.2,
                        span.style.font_size * 0.2,
                    );
                    if let Ok(resolved) = self.resolve_font(&span.style) {
                        primary_metrics = resolved.metrics_pixels(span.style.font_size);
                    }
                    primary_line_height_px = span.style.line_height.to_pixels(span.style.font_size);
                }

                // Adjust cluster offsets since shape_run thinks chunk_text starts at 0
                for glyph in &mut run_glyphs {
                    glyph.cluster += chunk_start as u32;
                    glyph.glyph_index = global_glyph_index;
                    glyph.time_offset = global_glyph_index as f32 * 0.05;
                    global_glyph_index += 1;
                    all_glyphs.push(*glyph);
                }

                chunk_start = chunk_end;
            }
            span_byte_offset = span_byte_end;
        }

        // Perform line breaking and layout
        let lines = self.layout_lines(
            &mut all_glyphs,
            &full_text,
            &bidi,
            max_width,
            align,
            overflow,
            primary_metrics.0,
            primary_metrics.1,
            primary_metrics.2,
            primary_line_height_px,
            path.as_ref(),
            boundary.as_ref(),
            spans,
        );

        // Compute total dimensions
        let mut total_width = 0.0f32;
        let total_height = lines.last().map(|l| l.baseline_y + l.height).unwrap_or(0.0);

        for line in &lines {
            if line.width > total_width {
                total_width = line.width;
            }
        }

        let grapheme_boundaries: Vec<usize> = full_text
            .grapheme_indices(true)
            .map(|(offset, _)| offset)
            .collect();

        Ok(ShapedText {
            glyphs: all_glyphs,
            lines,
            width: total_width,
            height: total_height,
            text: full_text,
            spans: spans.to_vec(),
            has_rtl,
            ascent: primary_metrics.0,
            descent: primary_metrics.1,
            line_gap: primary_metrics.2,
            grapheme_boundaries,
        })
    }

    /// Layout glyphs into lines with word wrapping and alignment.
    fn layout_lines(
        &self,
        glyphs: &mut Vec<GlyphInstance>,
        text: &str,
        bidi: &BidiInfo,
        max_width: Option<f32>,
        align: TextAlign,
        overflow: TextOverflow,
        ascent: f32,
        _descent: f32,
        _line_gap: f32,
        line_height_px: f32,
        path: Option<&TextPath>,
        boundary: Option<&LayoutBoundary>,
        spans: &[TextSpan],
    ) -> Vec<LineInfo> {
        let mut lines = Vec::new();
        let mut current_y = ascent;

        if glyphs.is_empty() {
            return lines;
        }

        if max_width.is_some() || boundary.is_some() {
            // Word wrapping mode
            let mut line_start_glyph = 0;
            let mut line_start_byte = 0;
            let mut last_word_break_glyph = 0usize;
            let mut last_word_break_byte = 0usize;

            for i in 0..glyphs.len() {
                let glyph = &glyphs[i];
                let char_at_cluster = text.chars().nth(glyph.cluster as usize).unwrap_or(' ');
                let is_space = char_at_cluster.is_ascii_whitespace();

                if is_space && i > line_start_glyph {
                    last_word_break_glyph = i + 1;
                    let mut byte_pos = 0;
                    let mut ci = 0u32;
                    let text_bytes = text.as_bytes();
                    while byte_pos < text_bytes.len() && ci <= glyph.cluster {
                        byte_pos += Self::utf8_len(text_bytes[byte_pos]);
                        ci += 1;
                    }
                    last_word_break_byte = byte_pos;
                }

                // Query constraints for the current line
                let (line_x_start, line_max_w) = if let Some(b) = boundary {
                    b.allowed_span(current_y)
                        .unwrap_or((0.0, max_width.unwrap_or(f32::MAX)))
                } else {
                    (0.0, max_width.unwrap_or(f32::MAX))
                };

                let glyph_right_edge = glyph.x + glyph.advance_width;
                let line_left = if line_start_glyph < glyphs.len() {
                    glyphs[line_start_glyph].x
                } else {
                    0.0
                };
                let line_content_width = glyph_right_edge - line_left;

                if line_content_width > line_max_w && i > line_start_glyph {
                    let break_glyph = if last_word_break_glyph > line_start_glyph {
                        last_word_break_glyph
                    } else {
                        i
                    };
                    let break_byte = if last_word_break_byte > line_start_byte {
                        last_word_break_byte
                    } else {
                        let mut bp = 0;
                        let mut ci2 = 0u32;
                        let tb = text.as_bytes();
                        while bp < tb.len()
                            && ci2 < glyphs[break_glyph.min(glyphs.len() - 1)].cluster
                        {
                            bp += Self::utf8_len(tb[bp]);
                            ci2 += 1;
                        }
                        bp
                    };

                    let line_width: f32 = glyphs[line_start_glyph..break_glyph]
                        .iter()
                        .map(|g| g.advance_width)
                        .sum();

                    let x_offset = line_x_start
                        + Self::compute_x_offset(
                            align,
                            line_max_w,
                            line_width,
                            glyphs,
                            line_start_glyph,
                            break_glyph,
                        );

                    // BiDi Visual Reordering (P0-41)
                    let line_range = line_start_byte..break_byte.min(text.len());
                    if !line_range.is_empty() && !bidi.paragraphs.is_empty() {
                        let para = bidi
                            .paragraphs
                            .iter()
                            .find(|p| {
                                p.range.start <= line_range.start && p.range.end >= line_range.end
                            })
                            .unwrap_or(&bidi.paragraphs[0]);

                        let (_, visual_runs) = bidi.visual_runs(para, line_range.clone());
                        let mut visual_glyphs = Vec::with_capacity(break_glyph - line_start_glyph);

                        for run in visual_runs {
                            for g in &glyphs[line_start_glyph..break_glyph] {
                                if run.contains(&(g.cluster as usize)) {
                                    visual_glyphs.push(*g);
                                }
                            }
                        }

                        if visual_glyphs.len() == break_glyph - line_start_glyph {
                            glyphs[line_start_glyph..break_glyph].clone_from_slice(&visual_glyphs);
                        }
                    }

                    // Position glyphs
                    let mut x = x_offset;
                    for g in &mut glyphs[line_start_glyph..break_glyph] {
                        g.x = x;
                        if g.glyph_id == 0xFFFF {
                            let mut portal_h = g.advance_height;
                            let mut alignment = PortalAlignment::Baseline;
                            for span in spans {
                                if let TextSpanKind::Portal {
                                    height,
                                    alignment: align_mode,
                                    ..
                                } = &span.kind
                                    && span.byte_offset as u32 == g.cluster
                                {
                                    portal_h = *height;
                                    alignment = *align_mode;
                                    break;
                                }
                            }
                            let y_offset = match alignment {
                                PortalAlignment::Baseline => 0.0,
                                PortalAlignment::Top => -ascent,
                                PortalAlignment::Center => {
                                    -ascent + (line_height_px - portal_h) / 2.0
                                }
                                PortalAlignment::Bottom => -ascent + line_height_px - portal_h,
                            };
                            g.y = current_y + y_offset;
                        } else {
                            g.y = current_y;
                        }
                        x += g.advance_width;
                    }

                    let line_text = text[line_start_byte..break_byte.min(text.len())].to_string();
                    lines.push(LineInfo {
                        glyph_start: line_start_glyph,
                        glyph_end: break_glyph,
                        baseline_y: current_y,
                        height: line_height_px,
                        width: line_width,
                        x_offset,
                        byte_offset: line_start_byte,
                        text: line_text,
                    });

                    current_y += line_height_px;
                    line_start_glyph = break_glyph;
                    line_start_byte = break_byte;
                }
            }

            // Last line
            if line_start_glyph < glyphs.len() {
                let (line_x_start, line_max_w) = if let Some(b) = boundary {
                    b.allowed_span(current_y)
                        .unwrap_or((0.0, max_width.unwrap_or(f32::MAX)))
                } else {
                    (0.0, max_width.unwrap_or(f32::MAX))
                };

                let line_width: f32 = glyphs[line_start_glyph..]
                    .iter()
                    .map(|g| g.advance_width)
                    .sum();

                let glyph_end = glyphs.len();
                let x_offset = line_x_start
                    + Self::compute_x_offset(
                        align,
                        line_max_w,
                        line_width,
                        glyphs,
                        line_start_glyph,
                        glyph_end,
                    );

                // BiDi Visual Reordering (P0-41)
                let line_range = line_start_byte..text.len();
                if !line_range.is_empty() && !bidi.paragraphs.is_empty() {
                    let para = bidi
                        .paragraphs
                        .iter()
                        .find(|p| {
                            p.range.start <= line_range.start && p.range.end >= line_range.end
                        })
                        .unwrap_or(&bidi.paragraphs[0]);

                    let (_, visual_runs) = bidi.visual_runs(para, line_range.clone());
                    let mut visual_glyphs = Vec::with_capacity(glyph_end - line_start_glyph);

                    for run in visual_runs {
                        for g in &glyphs[line_start_glyph..glyph_end] {
                            if run.contains(&(g.cluster as usize)) {
                                visual_glyphs.push(*g);
                            }
                        }
                    }

                    if visual_glyphs.len() == glyph_end - line_start_glyph {
                        glyphs[line_start_glyph..glyph_end].clone_from_slice(&visual_glyphs);
                    }
                }

                let mut x = x_offset;
                for g in &mut glyphs[line_start_glyph..] {
                    g.x = x;
                    if g.glyph_id == 0xFFFF {
                        let mut portal_h = g.advance_height;
                        let mut alignment = PortalAlignment::Baseline;
                        for span in spans {
                            if let TextSpanKind::Portal {
                                height,
                                alignment: align_mode,
                                ..
                            } = &span.kind
                                && span.byte_offset as u32 == g.cluster
                            {
                                portal_h = *height;
                                alignment = *align_mode;
                                break;
                            }
                        }
                        let y_offset = match alignment {
                            PortalAlignment::Baseline => 0.0,
                            PortalAlignment::Top => -ascent,
                            PortalAlignment::Center => -ascent + (line_height_px - portal_h) / 2.0,
                            PortalAlignment::Bottom => -ascent + line_height_px - portal_h,
                        };
                        g.y = current_y + y_offset;
                    } else {
                        g.y = current_y;
                    }
                    x += g.advance_width;
                }

                let remaining_text = text[line_start_byte.min(text.len())..].to_string();
                lines.push(LineInfo {
                    glyph_start: line_start_glyph,
                    glyph_end: glyphs.len(),
                    baseline_y: current_y,
                    height: line_height_px,
                    width: line_width,
                    x_offset,
                    byte_offset: line_start_byte,
                    text: remaining_text,
                });
            }
        } else {
            // No wrapping - single line
            let line_width: f32 = glyphs.iter().map(|g| g.advance_width).sum();

            let mut x = 0.0;
            for g in glyphs.iter_mut() {
                g.x = x;
                if g.glyph_id == 0xFFFF {
                    let mut portal_h = g.advance_height;
                    let mut alignment = PortalAlignment::Baseline;
                    for span in spans {
                        if let TextSpanKind::Portal {
                            height,
                            alignment: align_mode,
                            ..
                        } = &span.kind
                            && span.byte_offset as u32 == g.cluster
                        {
                            portal_h = *height;
                            alignment = *align_mode;
                            break;
                        }
                    }
                    let y_offset = match alignment {
                        PortalAlignment::Baseline => 0.0,
                        PortalAlignment::Top => -ascent,
                        PortalAlignment::Center => -ascent + (line_height_px - portal_h) / 2.0,
                        PortalAlignment::Bottom => -ascent + line_height_px - portal_h,
                    };
                    g.y = current_y + y_offset;
                } else {
                    g.y = current_y;
                }
                x += g.advance_width;
            }

            lines.push(LineInfo {
                glyph_start: 0,
                glyph_end: glyphs.len(),
                baseline_y: current_y,
                height: line_height_px,
                width: line_width,
                x_offset: 0.0,
                byte_offset: 0,
                text: text.to_string(),
            });
        }

        // Reorder glyphs within each line for BiDi
        for line_idx in 0..lines.len() {
            let line = &lines[line_idx];
            if line.glyph_start < line.glyph_end && line.glyph_end <= glyphs.len() {
                let level = line_bidi_level(bidi, line.byte_offset);
                if level.is_rtl() {
                    reorder_line_rtl(glyphs, line.glyph_start, line.glyph_end, bidi);
                }
            }
        }

        // Handle text overflow ellipsis
        if overflow == TextOverflow::Ellipsis
            && let Some(max_w) = max_width
        {
            for line_idx in 0..lines.len() {
                let line = &lines[line_idx];
                if line.width > max_w {
                    let mut trunc_width = 0.0f32;
                    let mut trunc_glyph_end = line.glyph_start;
                    let ellipsis_w = line_height_px * 0.6 * 3.0;

                    for gi in line.glyph_start..line.glyph_end {
                        if gi < glyphs.len() {
                            trunc_width += glyphs[gi].advance_width;
                            if trunc_width + ellipsis_w > max_w {
                                break;
                            }
                            trunc_glyph_end = gi + 1;
                        }
                    }

                    lines[line_idx].glyph_end = trunc_glyph_end;
                    lines[line_idx].width = trunc_width;
                }
            }
        }

        // Apply path layout constraint if present
        if let Some(tp) = path
            && let Some(last_glyph) = glyphs.last()
        {
            let total_x_len = last_glyph.x + last_glyph.advance_width;
            if total_x_len > 0.0 {
                for glyph in glyphs.iter_mut() {
                    let t = (glyph.x / total_x_len).clamp(0.0, 1.0);
                    let (pos, angle) = tp.sample(t);
                    let dy = glyph.y - ascent;
                    let perp_x = -angle.sin() * dy;
                    let perp_y = angle.cos() * dy;

                    glyph.x = pos.0 + perp_x;
                    glyph.y = pos.1 + perp_y;
                    glyph.angle = angle;
                }
            }
        }

        lines
    }

    /// Compute x offset for alignment.
    fn compute_x_offset(
        align: TextAlign,
        max_w: f32,
        line_width: f32,
        glyphs: &mut [GlyphInstance],
        start: usize,
        end: usize,
    ) -> f32 {
        match align {
            TextAlign::Start => 0.0,
            TextAlign::End => (max_w - line_width).max(0.0),
            TextAlign::Center => ((max_w - line_width) / 2.0).max(0.0),
            TextAlign::Justify => {
                if end <= start + 1 || max_w <= line_width {
                    return 0.0;
                }
                let extra = max_w - line_width;
                let space_count = glyphs[start..end]
                    .iter()
                    .filter(|g| g.glyph_id == 3)
                    .count();
                if space_count > 0 {
                    let add_per_space = extra / space_count as f32;
                    let mut x = 0.0f32;
                    for i in start..end {
                        glyphs[i].x = x;
                        if glyphs[i].glyph_id == 3 {
                            x += glyphs[i].advance_width + add_per_space;
                        } else {
                            x += glyphs[i].advance_width;
                        }
                    }
                }
                0.0
            }
        }
    }

    /// UTF-8 char length helper.
    fn utf8_len(first_byte: u8) -> usize {
        if first_byte < 0x80 {
            1
        } else if first_byte < 0xE0 {
            2
        } else if first_byte < 0xF0 {
            3
        } else {
            4
        }
    }
}
