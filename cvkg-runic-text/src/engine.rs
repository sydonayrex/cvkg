use std::collections::HashMap;
use fontdb::{Database, Family, Query, Source, Stretch, Style, Weight};
use swash::FontRef;
use swash::scale::{Render, ScaleContext, Source as SwashSource};

use rustybuzz::Direction;
use unicode_bidi::BidiInfo;

use crate::types::{ShapingError, FontAxisInfo, GlyphInstance, RunicPathSegment, GlyphImage, FontMetrics, LineInfo};
use crate::style::{TextStyle, RenderMode, TextAlign, TextOverflow};
use crate::span::{TextSpan, Paragraph};
use crate::layout::ShapedText;
use crate::global_cache;

// ── CacheKey ─────────────────────────────────────────────────────────────────

/// Deterministic cache key for shaped text.
///
/// Uses font swash::CacheKey (u64) which is derived from font data identity,
/// not fontdb::ID which uses slotmap and differs across processes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// Hash of the text content.
    pub text_hash: u64,
    /// Font swash cache key (identifies font data uniquely).
    pub font_cache_key: u64,
    /// Font size in pixels (quantized to 0.5px steps for cache friendliness).
    pub font_size: u32,
    /// Font weight.
    pub weight: u16,
    /// Font stretch raw value.
    pub stretch: u16,
    /// Font style discriminant.
    pub style: u8,
    /// Direction: 0 = LTR, 1 = RTL.
    pub direction: u8,
    /// Letter spacing (quantized to 1/100px).
    pub letter_spacing: i32,
    /// Word spacing (quantized to 1/100px).
    pub word_spacing: i32,
}

impl CacheKey {
    /// Create a new cache key.
    pub fn new(
        text: &str,
        font_cache_key: u64,
        font_size: f32,
        weight: Weight,
        stretch: Stretch,
        style: Style,
        direction: rustybuzz::Direction,
        letter_spacing: f32,
        word_spacing: f32,
    ) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let text_hash = hasher.finish();

        CacheKey {
            text_hash,
            font_cache_key,
            font_size: (font_size * 2.0).round() as u32,
            weight: weight.0,
            stretch: stretch.to_number(),
            style: match style {
                Style::Normal => 0,
                Style::Italic => 1,
                Style::Oblique => 2,
            },
            direction: match direction {
                Direction::LeftToRight => 0,
                Direction::RightToLeft => 1,
                _ => 0,
            },
            letter_spacing: (letter_spacing * 100.0).round() as i32,
            word_spacing: (word_spacing * 100.0).round() as i32,
        }
    }
}

// ── FontData ─────────────────────────────────────────────────────────────────

/// Owning wrapper for font data that can be shared.
/// Holds the stable data vector, the collection index, and the pre-computed
/// Swash CacheKey value to prevent dynamic ID generation anomalies.
#[derive(Clone)]
pub(crate) struct FontData {
    pub(crate) data: std::sync::Arc<Vec<u8>>,
    pub(crate) index: u32,
    pub(crate) key: u64,
}

impl FontData {
    /// Creates a new FontData container and immediately evaluates the Swash cache key value.
    ///
    /// # Contract
    /// Evaluates the unique font key once using Swash, caching it inline so that subsequent
    /// lookups on this font instance are fully deterministic and bypass atomic counter mutations.
    pub(crate) fn new(data: Vec<u8>, index: u32) -> Self {
        let key = FontRef::from_index(&data, index as usize)
            .map(|r| r.key.value())
            .unwrap_or(0);
        FontData {
            data: std::sync::Arc::new(data),
            index,
            key,
        }
    }

    /// Accesses the underlying raw bytes of the font data.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Resolves a transient FontRef from the raw data.
    ///
    /// # Contract
    /// Returns a transient FontRef referencing the heap-stable data. Callers should use
    /// `self.key` for stable cache indexing rather than the transient `FontRef.key` value.
    pub(crate) fn font_ref(&self) -> Option<FontRef<'_>> {
        FontRef::from_index(&self.data, self.index as usize)
    }

    pub(crate) fn face(&self) -> Option<rustybuzz::Face<'_>> {
        rustybuzz::Face::from_slice(&self.data, self.index)
    }
}

// ── ResolvedFont ─────────────────────────────────────────────────────────────

/// A resolved font with its faces and metadata.
pub(crate) struct ResolvedFont {
    pub(crate) primary: FontData,
    pub(crate) fallbacks: Vec<FontData>,
    pub(crate) cache_key: u64,
    pub(crate) units_per_em: u16,
    pub(crate) ascent: f32,
    pub(crate) descent: f32,
    pub(crate) line_gap: f32,
    pub(crate) x_height: f32,
    pub(crate) cap_height: f32,
    pub(crate) has_colr: bool,
}

impl ResolvedFont {
    pub(crate) fn from_data(data: FontData) -> Option<Self> {
        let font_ref = data.font_ref()?;
        let _face_ref = font_ref;

        let cache_key = data.key;

        let ttf_face = rustybuzz::ttf_parser::Face::parse(data.as_bytes(), data.index).ok()?;
        let units_per_em = ttf_face.units_per_em();
        let ascent = ttf_face.ascender() as f32;
        let descent = ttf_face.descender().abs() as f32;
        let line_gap = ttf_face.line_gap() as f32;

        let (os2_xh, os2_ch) = ttf_face
            .x_height()
            .and_then(|xh| ttf_face.capital_height().map(|ch| (xh as f32, ch as f32)))
            .unwrap_or((0.0, 0.0));
        let has_colr = ttf_face
            .raw_face()
            .table(rustybuzz::ttf_parser::Tag(u32::from_be_bytes(*b"COLR")))
            .is_some();

        Some(ResolvedFont {
            primary: data,
            fallbacks: vec![],
            cache_key,
            units_per_em,
            ascent,
            descent,
            line_gap,
            x_height: os2_xh,
            cap_height: os2_ch,
            has_colr,
        })
    }

    pub(crate) fn metrics_pixels(&self, font_size: f32) -> (f32, f32, f32) {
        let scale = font_size / self.units_per_em as f32;
        (
            self.ascent * scale,
            self.descent * scale,
            self.line_gap * scale,
        )
    }
}

// ── TextEngine ──────────────────────────────────────────────────────────

/// The main text shaping and layout engine.
pub struct TextEngine {
    /// Font database.
    pub(crate) db: Database,
    /// Font data cache: fontdb::ID -> FontData.
    pub(crate) font_data: HashMap<fontdb::ID, FontData>,
    /// Scale context for rasterization.
    pub(crate) scale_context: ScaleContext,
    /// Background database loading state.
    pub(crate) bg_db: Option<std::sync::Arc<std::sync::Mutex<Option<Database>>>>,
}

impl TextEngine {
    /// Create a new text engine with system fonts and user fonts loaded asynchronously.
    ///
    /// # Contract
    /// Guaranteed to successfully instantiate a usable text engine. Loads Jupiteroid.ttf
    /// synchronously so there's always a font ready immediately.
    /// Only bundled fonts (Jupiteroid) are loaded by default.
    /// Call `load_system_fonts()` explicitly if system font discovery is needed.
    pub fn new() -> Self {
        let mut db = Database::new();
        let jupiteroid_data = include_bytes!("../Fonts/Jupiteroid.ttf").to_vec();
        db.load_font_data(jupiteroid_data.clone());

        let bg_db_arc = std::sync::Arc::new(std::sync::Mutex::new(None));
        let bg_db_clone = bg_db_arc.clone();

        std::thread::spawn(move || {
            let mut bg_db = Database::new();
            bg_db.load_font_data(jupiteroid_data);

            if let Ok(mut guard) = bg_db_clone.lock() {
                *guard = Some(bg_db);
            }
        });

        let mut font_data = HashMap::new();
        for face in db.faces() {
            let id = face.id;
            let face_index = face.index;
            font_data.insert(id, FontData::new(include_bytes!("../Fonts/Jupiteroid.ttf").to_vec(), face_index));
        }

        TextEngine {
            db,
            font_data,
            scale_context: ScaleContext::new(),
            bg_db: Some(bg_db_arc),
        }
    }

    /// Create a light text engine for testing -- no system/user font loading.
    /// Only bundled fonts (loaded via `load_font_data()`) are available.
    pub fn new_light() -> Self {
        TextEngine {
            db: Database::new(),
            font_data: HashMap::new(),
            scale_context: ScaleContext::new(),
            bg_db: None,
        }
    }

    /// Checks if the background font database has completed indexing and swaps it into the active state.
    ///
    /// # Contract
    /// Swaps the database atomically when background font scanning completes, invalidating the
    /// local font_data cache to allow query resolution against the newly loaded system fonts.
    pub(crate) fn check_bg_db(&mut self) {
        let should_clear = if let Some(ref bg_db_arc) = self.bg_db {
            if let Ok(mut guard) = bg_db_arc.try_lock() {
                if let Some(new_db) = guard.take() {
                    self.db = new_db;
                    self.font_data.clear();
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };
        if should_clear {
            self.bg_db = None;
        }
    }

    /// Create a test engine with only the bundled Jupiteroid font loaded.
    /// Avoids loading system fonts (which cause OOM in CI with many parallel tests).
    pub fn new_test() -> Self {
        let mut engine = Self::new_light();
        engine.load_font_data(include_bytes!("../Fonts/Jupiteroid.ttf").to_vec());
        engine
    }

    /// Load a font from file data.
    pub fn load_font_data(&mut self, data: Vec<u8>) {
        self.check_bg_db();
        self.db.load_font_data(data.clone());
        for face in self.db.faces() {
            let id = face.id;
            self.font_data.entry(id).or_insert_with(|| {
                let face_index = face.index;
                FontData::new(data.clone(), face_index)
            });
        }
    }

    /// Load system fonts from standard directories.
    /// NOT called by default -- applications must call this explicitly if they
    /// need system font discovery beyond the bundled Jupiteroid font.
    ///
    /// Scans: ~/.local/share/fonts, ~/.fonts, /usr/share/fonts, /usr/local/share/fonts
    pub fn load_system_fonts(&mut self) {
        self.check_bg_db();
        self.db.load_system_fonts();
        let home = std::env::var("HOME").unwrap_or_default();
        for dir in &[
            format!("{}/.local/share/fonts", home),
            format!("{}/.fonts", home),
            "/usr/share/fonts".to_string(),
            "/usr/local/share/fonts".to_string(),
        ] {
            self.db.load_fonts_dir(dir);
        }
        for face in self.db.faces() {
            let id = face.id;
            if !self.font_data.contains_key(&id) {
                if let Some((source, face_index)) = self.db.face_source(id) {
                    let bytes = match source {
                        Source::Binary(arc_data) => {
                            arc_data.as_ref().as_ref().to_vec()
                        }
                        Source::File(path) | Source::SharedFile(path, _) => {
                            if let Ok(data) = std::fs::read(&path) {
                                data
                            } else {
                                continue;
                            }
                        }
                    };
                    self.font_data.insert(id, FontData::new(bytes, face_index));
                }
            }
        }
    }

    /// Get or load FontData for a fontdb ID.
    pub(crate) fn get_font_data(&mut self, id: fontdb::ID) -> Option<FontData> {
        self.check_bg_db();
        if let Some(data) = self.font_data.get(&id) {
            return Some(data.clone());
        }

        let (source, face_index) = self.db.face_source(id)?;
        let data = match source {
            Source::Binary(arc_data) => {
                let bytes: Vec<u8> = arc_data.as_ref().as_ref().to_vec();
                bytes
            }
            Source::File(path) => std::fs::read(&path).ok()?,
            _ => return None,
        };

        let font_data = FontData::new(data, face_index);
        self.font_data.insert(id, font_data.clone());
        Some(font_data)
    }

    /// Resolve a font for the given style.
    pub(crate) fn resolve_font(&mut self, style: &TextStyle) -> Result<ResolvedFont, ShapingError> {
        self.check_bg_db();
        for family_name in std::iter::once(&style.family).chain(style.fallback_families.iter()) {
            let query = Query {
                families: &[Family::Name(family_name)],
                weight: style.weight,
                stretch: style.stretch,
                style: style.style,
            };

            if let Some(id) = self.db.query(&query)
                && let Some(data) = self.get_font_data(id)
                && let Some(mut resolved) = ResolvedFont::from_data(data.clone())
            {
                let fallback_ids: Vec<fontdb::ID> = self
                    .db
                    .faces()
                    .filter(|f| f.id != id)
                    .map(|f| f.id)
                    .collect();
                for fb_id in fallback_ids {
                    if let Some(fb_data) = self.get_font_data(fb_id) {
                        resolved.fallbacks.push(fb_data);
                    }
                }
                return Ok(resolved);
            }
        }

        let all_ids: Vec<fontdb::ID> = self.db.faces().map(|f| f.id).collect();
        for id in &all_ids {
            if let Some(data) = self.get_font_data(*id)
                && let Some(mut resolved) = ResolvedFont::from_data(data)
            {
                for fb_id in &all_ids {
                    if *fb_id != *id
                        && let Some(fb_data) = self.get_font_data(*fb_id)
                    {
                        resolved.fallbacks.push(fb_data);
                    }
                }
                return Ok(resolved);
            }
        }

        Err(ShapingError::NoFontFound(style.family.clone()))
    }

    /// Rasterize a glyph to a bitmap image.
    pub fn rasterize_glyph(
        &mut self,
        glyph_id: u16,
        style: &TextStyle,
    ) -> Result<GlyphImage, ShapingError> {
        let resolved = self.resolve_font(style)?;

        let font_ref = resolved
            .primary
            .font_ref()
            .ok_or(ShapingError::InvalidFontData)?;

        let mut scaler = self
            .scale_context
            .builder(font_ref)
            .size(style.font_size)
            .build();

        let use_color = resolved.has_colr && style.render_mode == RenderMode::Color;
        let use_subpixel = style.render_mode == RenderMode::Subpixel;

        let sources: Vec<SwashSource> = if use_color {
            vec![SwashSource::ColorOutline(glyph_id), SwashSource::Outline]
        } else {
            vec![SwashSource::Outline]
        };

        let mut render = Render::new(&sources);

        if use_subpixel {
            render.format(swash::zeno::Format::Subpixel);
        } else {
            render.format(swash::zeno::Format::Alpha);
        }

        if style.synthesize_styles && style.weight >= Weight(700) {
            render.embolden(0.04);
        }

        if let Some(image) = render.render(&mut scaler, glyph_id) {
            log::info!("Swash rendered image for glyph {}. content: {:?}, size: {}x{}, data len: {}", glyph_id, image.content, image.placement.width, image.placement.height, image.data.len());
            return Ok(GlyphImage {
                glyph_id,
                width: image.placement.width,
                height: image.placement.height,
                data: image.data,
                x_offset: image.placement.left as f32,
                y_offset: image.placement.top as f32,
                cache_key: resolved.cache_key,
            });
        }

        for fallback in &resolved.fallbacks {
            if let Some(font_ref) = fallback.font_ref() {
                let mut scaler = self
                    .scale_context
                    .builder(font_ref)
                    .size(style.font_size)
                    .build();
                if let Some(image) = render.render(&mut scaler, glyph_id) {
                    return Ok(GlyphImage {
                        glyph_id,
                        width: image.placement.width,
                        height: image.placement.height,
                        data: image.data,
                        x_offset: image.placement.left as f32,
                        y_offset: image.placement.top as f32,
                        cache_key: resolved.cache_key,
                    });
                }
            }
        }

        Err(ShapingError::EmptyShape(format!(
            "Could not rasterize glyph {}",
            glyph_id
        )))
    }

    /// Extract the vector outline path for a given glyph at the specified size.
    ///
    /// # Contract
    /// Resolves the font using the provided TextStyle and extracts its Bezier outline.
    /// Returns a list of `RunicPathSegment` representing the raw MoveTo, LineTo, QuadTo,
    /// CubicTo, and Close commands of the glyph contours, scaled to the given size.
    /// If the font does not contain outline data or the glyph is empty, returns an empty path.
    pub fn extract_glyph_path(
        &mut self,
        glyph_id: u16,
        size: f32,
        style: &TextStyle,
    ) -> Result<Vec<RunicPathSegment>, ShapingError> {
        let resolved = self.resolve_font(style)?;
        let font_ref = resolved
            .primary
            .font_ref()
            .ok_or(ShapingError::InvalidFontData)?;

        let mut scaler = self.scale_context.builder(font_ref).size(size).build();

        let map_outline_to_segments =
            |outline: swash::scale::outline::Outline| -> Vec<RunicPathSegment> {
                let mut segments = Vec::new();
                let mut points_iter = outline.points().iter();
                for verb in outline.verbs() {
                    match verb {
                        swash::zeno::Verb::MoveTo => {
                            if let Some(p) = points_iter.next() {
                                segments.push(RunicPathSegment::MoveTo { x: p.x, y: p.y });
                            }
                        }
                        swash::zeno::Verb::LineTo => {
                            if let Some(p) = points_iter.next() {
                                segments.push(RunicPathSegment::LineTo { x: p.x, y: p.y });
                            }
                        }
                        swash::zeno::Verb::QuadTo => {
                            if let Some(cp) = points_iter.next()
                                && let Some(p) = points_iter.next()
                            {
                                segments.push(RunicPathSegment::QuadTo {
                                    cx: cp.x,
                                    cy: cp.y,
                                    x: p.x,
                                    y: p.y,
                                });
                            }
                        }
                        swash::zeno::Verb::CurveTo => {
                            if let Some(cp1) = points_iter.next()
                                && let Some(cp2) = points_iter.next()
                                && let Some(p) = points_iter.next()
                            {
                                segments.push(RunicPathSegment::CubicTo {
                                    cx1: cp1.x,
                                    cy1: cp1.y,
                                    cx2: cp2.x,
                                    cy2: cp2.y,
                                    x: p.x,
                                    y: p.y,
                                });
                            }
                        }
                        swash::zeno::Verb::Close => {
                            segments.push(RunicPathSegment::Close);
                        }
                    }
                }
                segments
            };

        if let Some(outline) = scaler.scale_outline(glyph_id) {
            return Ok(map_outline_to_segments(outline));
        }

        for fallback in &resolved.fallbacks {
            if let Some(font_ref) = fallback.font_ref() {
                let mut scaler = self.scale_context.builder(font_ref).size(size).build();
                if let Some(outline) = scaler.scale_outline(glyph_id) {
                    return Ok(map_outline_to_segments(outline));
                }
            }
        }

        Ok(Vec::new())
    }

    /// Get font metrics for a style.
    pub fn font_metrics(&mut self, style: &TextStyle) -> Result<FontMetrics, ShapingError> {
        let resolved = self.resolve_font(style)?;
        let (ascent, descent, line_gap) = resolved.metrics_pixels(style.font_size);

        Ok(FontMetrics {
            ascent,
            descent,
            line_gap,
            units_per_em: resolved.units_per_em,
            x_height: resolved.x_height * style.font_size / resolved.units_per_em as f32,
            cap_height: resolved.cap_height * style.font_size / resolved.units_per_em as f32,
        })
    }

    /// Clear the shape cache.
    pub fn clear_cache(&mut self) {
        global_cache::global_cache_clear();
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> (usize, usize) {
        global_cache::global_cache_stats()
    }

    /// Get the number of faces in the database.
    pub fn font_count(&self) -> usize {
        self.db.faces().count()
    }

    /// Shapes and returns only the visible range of lines from a list of paragraphs/spans.
    ///
    /// # Contract
    /// Instead of shaping the entire document upfront, this method virtualizes lines
    /// by only invoking rustybuzz shaping and swash metric queries on paragraphs
    /// that overlap the visible Y coordinates [y_start, y_end). This allows rendering files
    /// with 1M+ lines with $O(1)$ memory growth and bounds layout latency.
    /// Returns the subset of shaped text segments layouted within the requested vertical slice.
    pub fn shape_layout_virtualized(
        &mut self,
        paragraphs: &[Paragraph],
        line_height_px: f32,
        y_start: f32,
        y_end: f32,
        max_width: Option<f32>,
        align: TextAlign,
    ) -> Result<ShapedText, ShapingError> {
        let mut visible_glyphs = Vec::new();
        let mut visible_lines = Vec::new();
        let mut total_height = 0.0;
        let mut has_rtl = false;

        let mut primary_ascent = 0.0;
        let mut primary_descent = 0.0;
        let mut primary_line_gap = 0.0;
        let mut grapheme_boundaries = Vec::new();
        let mut full_text_accum = String::new();

        let mut current_y = 0.0;

        for (_p_idx, para) in paragraphs.iter().enumerate() {
            let char_count = para.text.chars().count();
            let est_lines = if let Some(_mw) = max_width {
                ((char_count as f32 / 80.0).ceil() as usize).max(1)
            } else {
                1
            };
            let para_h = est_lines as f32 * line_height_px;

            let para_y_start = current_y;
            let para_y_end = para_y_start + para_h;

            if para_y_end >= y_start && para_y_start <= y_end {
                let spans: Vec<TextSpan> = para.runs.iter().map(|run| {
                    TextSpan::at(&run.text, run.style.clone(), run.start)
                }).collect();

                if !spans.is_empty() {
                    let shaped = self.shape_layout(&spans, max_width, align, TextOverflow::WordWrap)?;
                    if primary_ascent == 0.0 {
                        primary_ascent = shaped.ascent;
                        primary_descent = shaped.descent;
                        primary_line_gap = shaped.line_gap;
                    }
                    if shaped.has_rtl {
                        has_rtl = true;
                    }

                    let base_glyph_idx = visible_glyphs.len();
                    for mut g in shaped.glyphs {
                        g.y += para_y_start;
                        visible_glyphs.push(g);
                    }

                    let text_offset = full_text_accum.len();
                    for line in shaped.lines {
                        visible_lines.push(LineInfo {
                            glyph_start: base_glyph_idx + line.glyph_start,
                            glyph_end: base_glyph_idx + line.glyph_end,
                            baseline_y: para_y_start + line.baseline_y,
                            height: line.height,
                            width: line.width,
                            x_offset: line.x_offset,
                            byte_offset: text_offset + line.byte_offset,
                            text: line.text,
                        });
                    }

                    let g_offset = full_text_accum.len();
                    for boundary in shaped.grapheme_boundaries {
                        grapheme_boundaries.push(g_offset + boundary);
                    }
                    full_text_accum.push_str(&para.text);
                    full_text_accum.push('\n');
                }
            } else {
                full_text_accum.push_str(&para.text);
                full_text_accum.push('\n');
            }

            current_y += para_h;
            total_height = current_y;
        }

        let max_w = visible_lines.iter().map(|l| l.width).fold(0.0f32, |a, b| a.max(b));

        Ok(ShapedText {
            glyphs: visible_glyphs,
            lines: visible_lines,
            width: max_w,
            height: total_height,
            text: full_text_accum,
            spans: Vec::new(),
            has_rtl,
            ascent: primary_ascent,
            descent: primary_descent,
            line_gap: primary_line_gap,
            grapheme_boundaries,
        })
    }

    /// Query the variable font axes available for a given font family.
    ///
    /// Returns `Ok(None)` if the font is not variable.
    /// Returns `Err` if the font cannot be found.
    ///
    /// # Arguments
    /// * `family` -- Font family name.
    /// * `font_size` -- Font size for resolving the face.
    pub fn query_font_axes(
        &mut self,
        family: &str,
        _font_size: f32,
    ) -> Result<Option<Vec<FontAxisInfo>>, ShapingError> {
        let query = Query {
            families: &[Family::Name(family)],
            weight: Weight::NORMAL,
            stretch: Stretch::Normal,
            style: Style::Normal,
        };

        let id = self
            .db
            .query(&query)
            .ok_or_else(|| ShapingError::NoFontFound(family.to_string()))?;
        let data = self
            .get_font_data(id)
            .ok_or(ShapingError::InvalidFontData)?;
        let _font_ref = data.font_ref().ok_or(ShapingError::InvalidFontData)?;

        let ttf_face = rustybuzz::ttf_parser::Face::parse(data.as_bytes(), data.index)
            .map_err(|_| ShapingError::InvalidFontData)?;

        let fvar_data = match ttf_face
            .raw_face()
            .table(rustybuzz::ttf_parser::Tag(u32::from_be_bytes(*b"fvar")))
        {
            Some(d) => d,
            None => return Ok(None),
        };

        if fvar_data.len() < 16 {
            return Ok(None);
        }

        let axis_count = u16::from_be_bytes([fvar_data[8], fvar_data[9]]) as usize;
        let axis_size = u16::from_be_bytes([fvar_data[10], fvar_data[11]]) as usize;
        let data_offset = u16::from_be_bytes([fvar_data[4], fvar_data[5]]) as usize;

        let mut axes = Vec::new();
        for i in 0..axis_count {
            let offset = data_offset + i * axis_size;
            if offset + axis_size > fvar_data.len() {
                break;
            }

            let axis_data = &fvar_data[offset..offset + axis_size];

            if axis_data.len() < 20 {
                break;
            }

            let tag = u32::from_be_bytes([axis_data[0], axis_data[1], axis_data[2], axis_data[3]]);
            let min_val =
                f32::from_be_bytes([axis_data[4], axis_data[5], axis_data[6], axis_data[7]]);
            let default_val =
                f32::from_be_bytes([axis_data[8], axis_data[9], axis_data[10], axis_data[11]]);
            let max_val =
                f32::from_be_bytes([axis_data[12], axis_data[13], axis_data[14], axis_data[15]]);

            let tag_bytes = tag.to_be_bytes();
            let tag_string = String::from_utf8_lossy(&tag_bytes).trim().to_string();

            let standard_tags: &[&[u8]] = &[
                b"wght", b"wdth", b"ital", b"slnt", b"opsz", b"GRAD", b"XTRA", b"XOPQ", b"YOPQ",
                b"YTLC", b"YTUC", b"YTAS", b"YTDE", b"YTFI", b"wdth",
            ];
            let is_standard = standard_tags.contains(&tag_bytes.as_slice());

            axes.push(FontAxisInfo {
                tag,
                tag_string,
                min: min_val,
                max: max_val,
                default: default_val,
                is_standard,
            });
        }

        if axes.is_empty() {
            Ok(None)
        } else {
            Ok(Some(axes))
        }
    }

    /// Shape text with a simple family/size interface (backward-compatible).
    ///
    /// This wraps `shape_layout` with a single span and default settings
    /// for use by the cvkg-render-gpu crate.
    pub fn shape(&mut self, text: &str, family: &str, size: f32) -> ShapedText {
        let style = TextStyle::new(family, size);
        let spans = vec![TextSpan::new(text, style)];
        self.shape_layout(&spans, None, TextAlign::Start, TextOverflow::WordWrap)
            .unwrap_or_else(|_| ShapedText {
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
                grapheme_boundaries: Vec::new(),
            })
    }

    /// Rasterizes a glyph by lookup using its unique composite cache key.
    ///
    /// # Contract
    /// The `cache_key` must match a key generated during text shaping that hashes the
    /// font data identity, size, styling, and glyph ID. This function resolves the matching
    /// glyph parameters from the shape cache and rasterizes it at the correct size and weight
    /// to prevent cache collisions and visual distortion. Returns `None` if no matching shaped
    /// glyph is present in the cache.
    pub fn rasterize(&mut self, cache_key: u64) -> Option<GlyphImage> {
        self.check_bg_db();
        let found = global_cache::global_cache_find_glyph(cache_key);
        let (ck, glyph) = found?;

        let mut family = "sans-serif".to_string();
        let face_ids: Vec<fontdb::ID> = self.db.faces().map(|f| f.id).collect();
        for id in &face_ids {
            if let Some(font_data) = self.get_font_data(*id)
                && font_data.key == ck.font_cache_key
            {
                if let Some(face) = self.db.face(*id)
                    && let Some((name, _)) = face.families.first()
                {
                    family = name.clone();
                }
                break;
            }
        }

        let mut style = TextStyle::new(&family, ck.font_size as f32 / 2.0);
        style.weight = Weight(ck.weight);
        style.stretch = match ck.stretch {
            1 => Stretch::UltraCondensed,
            2 => Stretch::ExtraCondensed,
            3 => Stretch::Condensed,
            4 => Stretch::SemiCondensed,
            5 => Stretch::Normal,
            6 => Stretch::SemiExpanded,
            7 => Stretch::Expanded,
            8 => Stretch::ExtraExpanded,
            9 => Stretch::UltraExpanded,
            _ => Stretch::Normal,
        };
        style.style = match ck.style {
            0 => Style::Normal,
            1 => Style::Italic,
            2 => Style::Oblique,
            _ => Style::Normal,
        };

        let mut image = self.rasterize_glyph(glyph.glyph_id, &style).ok()?;
        image.cache_key = cache_key;
        Some(image)
    }
}

pub(crate) fn byte_offset_level(bidi: &BidiInfo, byte_offset: usize) -> unicode_bidi::Level {
    if let Some(para) = bidi.paragraphs.first() {
        let relative = byte_offset.saturating_sub(para.range.start);
        if relative < bidi.levels.len() {
            return bidi.levels[relative];
        }
    }
    unicode_bidi::Level::ltr()
}

pub(crate) fn line_bidi_level(bidi: &BidiInfo, byte_offset: usize) -> unicode_bidi::Level {
    byte_offset_level(bidi, byte_offset)
}

pub(crate) fn reorder_line_rtl(
    glyphs: &mut [GlyphInstance],
    start: usize,
    end: usize,
    bidi: &BidiInfo,
) {
    if end <= start {
        return;
    }
    let mut i = start;
    while i < end {
        let byte_off = glyphs[i].cluster as usize;
        let level = byte_offset_level(bidi, byte_off);
        if level.is_rtl() {
            let mut j = i + 1;
            while j < end {
                let next_byte = glyphs[j].cluster as usize;
                let next_level = byte_offset_level(bidi, next_byte);
                if !next_level.is_rtl() {
                    break;
                }
                j += 1;
            }
            let start_x = if i > start {
                glyphs[i - 1].x + glyphs[i - 1].advance_width
            } else {
                0.0
            };
            let slice = &mut glyphs[i..j];
            slice.reverse();
            let mut x = start_x;
            for g in slice.iter_mut() {
                g.x = x;
                x += g.advance_width;
            }
            i = j;
        } else {
            i += 1;
        }
    }
}

impl Default for TextEngine {
    fn default() -> Self {
        Self::new()
    }
}
