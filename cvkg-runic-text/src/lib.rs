// CVKG Runic Text — Production text shaping, layout, and rasterization engine
//
// Features:
//   - Font discovery via fontdb (system fonts + user fonts)
//   - Text shaping via rustybuzz (OpenType shaping, ligatures, kerning)
//   - BiDi support via unicode-bidi
//   - Font fallback with glyph-level resolution
//   - LRU shape cache with deterministic keys
//   - Word wrapping, text alignment, line height modes
//   - Selection rects, hit testing, cursor positioning
//   - Text overflow modes (clip, ellipsis, visible, word-wrap)
//   - OpenType features and variable font axes
//   - TextStyle with weight, stretch, style, color, spacing, decorations
#![allow(
    clippy::too_many_arguments,
    clippy::needless_range_loop,
    clippy::ptr_arg
)]

use std::collections::HashMap;
use std::sync::Arc;

/// Shared test engine that loads only bundled fonts (no system fonts).
/// Uses Arc for thread-safe sharing across parallel tests.
#[allow(dead_code)]
static TEST_ENGINE: std::sync::OnceLock<Arc<RunicTextEngine>> = std::sync::OnceLock::new();

/// Get or create the shared test engine.
pub fn test_engine() -> &'static Arc<RunicTextEngine> {
    TEST_ENGINE.get_or_init(|| {
        let mut engine = RunicTextEngine::new_light();
        // Load bundled Jupiteroid font for tests
        engine.load_font_data(include_bytes!("../Fonts/Jupiteroid.ttf").to_vec());
        Arc::new(engine)
    })
}

use fontdb::{Database, Family, Query, Source, Stretch, Style, Weight};
use rustybuzz::{Direction, Feature, UnicodeBuffer};
use swash::FontRef;
use swash::scale::{Render, ScaleContext, Source as SwashSource};
use unicode_bidi::BidiInfo;
use unicode_segmentation::UnicodeSegmentation;

// ── Constants ──────────────────────────────────────────────────────────────

/// Default font size in pixels.
pub const DEFAULT_FONT_SIZE: f32 = 16.0;

/// Default line height multiplier.
pub const DEFAULT_LINE_HEIGHT: f32 = 1.2;

/// Maximum number of entries in the shape cache.
const MAX_CACHE_SIZE: usize = 1024;

// ── Error type ──────────────────────────────────────────────────────────────

/// Errors that can occur during text shaping and layout.
#[derive(Debug, Clone, PartialEq)]
pub enum ShapingError {
    /// No font could be found for the given text/style.
    NoFontFound(String),
    /// The font database returned an invalid font ID.
    InvalidFontId,
    /// Shaping produced no glyphs for non-empty input.
    EmptyShape(String),
    /// An embedded font data reference was invalid.
    InvalidFontData,
}

impl std::fmt::Display for ShapingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShapingError::NoFontFound(s) => write!(f, "No font found for: {}", s),
            ShapingError::InvalidFontId => write!(f, "Invalid font ID"),
            ShapingError::EmptyShape(s) => write!(f, "Empty shaping result for: {}", s),
            ShapingError::InvalidFontData => write!(f, "Invalid font data"),
        }
    }
}

impl std::error::Error for ShapingError {}

// ── FontAxisInfo ─────────────────────────────────────────────────────────────

/// Describes a single variable font axis.
#[derive(Debug, Clone, PartialEq)]
pub struct FontAxisInfo {
    /// The 4-byte axis tag (e.g. `b"wght"`, `b"wdth"`, `b"ital"`).
    pub tag: u32,
    /// The axis tag as a human-readable string.
    pub tag_string: String,
    /// Minimum value for this axis.
    pub min: f32,
    /// Maximum value for this axis.
    pub max: f32,
    /// Default value for this axis.
    pub default: f32,
    /// Whether this axis is a standard registered axis.
    pub is_standard: bool,
}

impl FontAxisInfo {
    /// Get the standard name for known axes, or the raw tag string for custom axes.
    pub fn display_name(&self) -> &str {
        match &self.tag_string[..] {
            "wght" => "Weight",
            "wdth" => "Width",
            "ital" => "Italic",
            "slnt" => "Slant",
            "opsz" => "Optical Size",
            "GRAD" => "Grade",
            "XTRA" => "X Tra Bold",
            "XOPQ" => "X Opacity",
            "YOPQ" => "Y Opacity",
            "YTLC" => "Y Tall Cap Height",
            "YTUC" => "Y Uppercase Height",
            "YTAS" => "Y Tall Ascender",
            "YTDE" => "Y Tall Descender",
            "YTFI" => "Y Tall Figure Height",
            _ => &self.tag_string,
        }
    }
}

// ── TextStyle ────────────────────────────────────────────────────────────────

/// Text decoration flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextDecorations {
    /// Underline.
    pub underline: bool,
    /// Strikethrough.
    pub strikethrough: bool,
    /// Overline.
    pub overline: bool,
}

/// How line height is computed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineHeight {
    /// Multiple of font size (e.g. 1.2 = 120% of font size).
    Multiple(f32),
    /// Fixed pixel height.
    Fixed(f32),
}

impl Default for LineHeight {
    fn default() -> Self {
        LineHeight::Multiple(DEFAULT_LINE_HEIGHT)
    }
}

impl LineHeight {
    /// Compute the line height in pixels for a given font size.
    pub fn to_pixels(self, font_size: f32) -> f32 {
        match self {
            LineHeight::Multiple(m) => font_size * m,
            LineHeight::Fixed(px) => px,
        }
    }
}

/// Text overflow handling mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextOverflow {
    /// Clip text at the boundary.
    Clip,
    /// Show ellipsis when text overflows.
    Ellipsis,
    /// Let text overflow visibly.
    Visible,
    /// Wrap words that exceed the width.
    #[default]
    WordWrap,
}

/// Text alignment within a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    /// Align to the start (left in LTR, right in RTL).
    #[default]
    Start,
    /// Align to the end.
    End,
    /// Center within the available width.
    Center,
    /// Justify text (stretch to fill width - basic implementation).
    Justify,
}

/// Glyph rasterization mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderMode {
    /// Standard grayscale anti-aliased rendering.
    Grayscale,
    /// LCD subpixel anti-aliased rendering (3-channel horizontal mask).
    #[default]
    Subpixel,
    /// Color emoji / layered vector font rendering (COLR/CPAL, SVG, sbix).
    Color,
    /// Multi-channel signed distance field rendering (resolution-independent).
    Msdf,
}

/// A variable font axis setting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VariableAxis {
    /// The OpenType axis tag (e.g. `wght`, `wdth`, `ital`).
    pub tag: u32,
    /// The axis value.
    pub value: f32,
}

impl VariableAxis {
    /// Create a new variable axis setting from a 4-byte tag.
    pub fn new(tag_bytes: [u8; 4], value: f32) -> Self {
        let tag = u32::from_be_bytes(tag_bytes);
        VariableAxis { tag, value }
    }

    /// Weight axis (100-900).
    pub fn weight(value: f32) -> Self {
        VariableAxis::new(*b"wght", value)
    }

    /// Width axis.
    pub fn width(value: f32) -> Self {
        VariableAxis::new(*b"wdth", value)
    }

    /// Italic axis (0.0 or 1.0).
    pub fn italic(value: f32) -> Self {
        VariableAxis::new(*b"ital", value)
    }

    /// Slant axis.
    pub fn slant(value: f32) -> Self {
        VariableAxis::new(*b"slnt", value)
    }
}

/// A Bezier spline path for positioning and rotating glyphs along arbitrary curves.
///
/// # Contract
/// The path is constructed from control points. The `sample` method interpolates
/// along the path at normalized parameter `t` (0.0 to 1.0) and returns the 2D position
/// and the tangent rotation angle in radians for orienting characters correctly.
#[derive(Debug, Clone, PartialEq)]
pub struct TextPath {
    /// Control points for the Bezier spline segments.
    pub control_points: Vec<(f32, f32)>,
}

impl TextPath {
    /// Create a new text path from control points.
    pub fn new(control_points: Vec<(f32, f32)>) -> Self {
        TextPath { control_points }
    }

    /// Sample the position and tangent rotation angle (radians) at normalized parameter `t` (0.0..=1.0).
    pub fn sample(&self, t: f32) -> ((f32, f32), f32) {
        if self.control_points.is_empty() {
            return ((0.0, 0.0), 0.0);
        }
        let n = self.control_points.len();
        if n == 1 {
            return (self.control_points[0], 0.0);
        }
        if n == 3 {
            // Quadratic Bezier interpolation
            let p0 = self.control_points[0];
            let p1 = self.control_points[1];
            let p2 = self.control_points[2];
            let u = 1.0 - t;
            let tt = t * t;
            let uu = u * u;
            let x = uu * p0.0 + 2.0 * u * t * p1.0 + tt * p2.0;
            let y = uu * p0.1 + 2.0 * u * t * p1.1 + tt * p2.1;
            let tx = 2.0 * u * (p1.0 - p0.0) + 2.0 * t * (p2.0 - p1.0);
            let ty = 2.0 * u * (p1.1 - p0.1) + 2.0 * t * (p2.1 - p1.1);
            let angle = ty.atan2(tx);
            ((x, y), angle)
        } else if n == 4 {
            // Cubic Bezier interpolation
            let p0 = self.control_points[0];
            let p1 = self.control_points[1];
            let p2 = self.control_points[2];
            let p3 = self.control_points[3];
            let u = 1.0 - t;
            let tt = t * t;
            let uu = u * u;
            let uuu = uu * u;
            let ttt = tt * t;
            let x = uuu * p0.0 + 3.0 * uu * t * p1.0 + 3.0 * u * tt * p2.0 + ttt * p3.0;
            let y = uuu * p0.1 + 3.0 * uu * t * p1.1 + 3.0 * u * tt * p2.1 + ttt * p3.1;
            let tx =
                3.0 * uu * (p1.0 - p0.0) + 6.0 * u * t * (p2.0 - p1.0) + 3.0 * tt * (p3.0 - p2.0);
            let ty =
                3.0 * uu * (p1.1 - p0.1) + 6.0 * u * t * (p2.1 - p1.1) + 3.0 * tt * (p3.1 - p2.1);
            let angle = ty.atan2(tx);
            ((x, y), angle)
        } else {
            // Fallback: Linear polyline interpolation
            let segments = n - 1;
            let scaled_t = t * segments as f32;
            let idx = (scaled_t.floor() as usize).min(segments - 1);
            let local_t = scaled_t - idx as f32;
            let p0 = self.control_points[idx];
            let p1 = self.control_points[idx + 1];
            let x = p0.0 + (p1.0 - p0.0) * local_t;
            let y = p0.1 + (p1.1 - p0.1) * local_t;
            let tx = p1.0 - p0.0;
            let ty = p1.1 - p0.1;
            let angle = ty.atan2(tx);
            ((x, y), angle)
        }
    }
}

/// Boundary shapes used for non-rectangular text wrapping.
///
/// # Contract
/// Represents geometric limits within which text flows are allowed or clipped.
/// The layouter checks collision with boundaries during the line reflow calculations.
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutBoundary {
    /// Circular boundary: center x, center y, radius.
    Circle {
        /// Center X coordinate.
        cx: f32,
        /// Center Y coordinate.
        cy: f32,
        /// Radius of boundary circle.
        r: f32,
    },
    /// Convex polygon boundary defined by a set of clockwise vertices.
    Polygon {
        /// Vertices (x, y) defining the polygon boundary.
        vertices: Vec<(f32, f32)>,
    },
}

impl LayoutBoundary {
    /// Compute the allowed horizontal span `[x_min, x_max]` at a vertical coordinate `y`.
    ///
    /// # Contract
    /// Checks intersection of a horizontal line at `y` with the boundary shape.
    /// Returns `Some((x_min, x_max))` if the line intersects the boundary, otherwise `None`.
    pub fn allowed_span(&self, y: f32) -> Option<(f32, f32)> {
        match self {
            LayoutBoundary::Circle { cx, cy, r } => {
                let dy = y - cy;
                if dy.abs() < *r {
                    let dx = (r * r - dy * dy).sqrt();
                    Some((cx - dx, cx + dx))
                } else {
                    None
                }
            }
            LayoutBoundary::Polygon { vertices } => {
                if vertices.len() < 3 {
                    return None;
                }
                let mut intersections = Vec::new();
                for i in 0..vertices.len() {
                    let p0 = vertices[i];
                    let p1 = vertices[(i + 1) % vertices.len()];
                    let y_min = p0.1.min(p1.1);
                    let y_max = p0.1.max(p1.1);
                    if y >= y_min && y <= y_max && (p1.1 - p0.1).abs() > 1e-5 {
                        let t = (y - p0.1) / (p1.1 - p0.1);
                        let x = p0.0 + t * (p1.0 - p0.0);
                        intersections.push(x);
                    }
                }
                if intersections.len() >= 2 {
                    intersections.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    Some((intersections[0], intersections[intersections.len() - 1]))
                } else {
                    None
                }
            }
        }
    }
}

/// An OpenType feature to enable during shaping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenTypeFeature {
    /// The feature tag (4-byte identifier).
    pub tag: u32,
    /// The feature value (0 = disable, 1 = enable, higher = alternate index).
    pub value: u32,
}

impl OpenTypeFeature {
    /// Create a new OpenType feature from a 4-byte tag.
    pub fn new(tag_bytes: [u8; 4], value: u32) -> Self {
        let tag = u32::from_be_bytes(tag_bytes);
        OpenTypeFeature { tag, value }
    }

    /// Enable standard ligatures.
    pub fn liga() -> Self {
        OpenTypeFeature::new(*b"liga", 1)
    }

    /// Enable kerning.
    pub fn kern() -> Self {
        OpenTypeFeature::new(*b"kern", 1)
    }

    /// Enable contextual alternates.
    pub fn calt() -> Self {
        OpenTypeFeature::new(*b"calt", 1)
    }

    /// Enable discretionary ligatures.
    pub fn dlig() -> Self {
        OpenTypeFeature::new(*b"dlig", 1)
    }
}

/// Complete text styling for a span of text.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    /// Font family name (primary).
    pub family: String,
    /// Fallback font family names.
    pub fallback_families: Vec<String>,
    /// Font size in pixels.
    pub font_size: f32,
    /// Font weight (100-900).
    pub weight: Weight,
    /// Font stretch.
    pub stretch: Stretch,
    /// Font style (normal, italic, oblique).
    pub style: Style,
    /// Text color as RGBA.
    pub color: [u8; 4],
    /// Letter spacing in pixels (added to each glyph advance).
    pub letter_spacing: f32,
    /// Word spacing in pixels (added to space glyph advance).
    pub word_spacing: f32,
    /// Line height mode.
    pub line_height: LineHeight,
    /// Text decorations.
    pub decorations: TextDecorations,
    /// OpenType features to enable (after liga/kern/calt which are always on).
    pub extra_features: Vec<OpenTypeFeature>,
    /// Variable font axis settings.
    pub variable_axes: Vec<VariableAxis>,
    /// Whether to synthesize bold/italic when the variant font is missing.
    pub synthesize_styles: bool,
    /// Rendering mode for glyph rasterization.
    pub render_mode: RenderMode,
    /// Whether to render glyphs as resolution-independent vector outlines.
    pub outline_rendering: bool,
    /// Unique identifier for dynamic material and visual rendering effects.
    pub material_effect_id: u32,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            family: "Jupiteroid".to_string(),
            fallback_families: vec![
                "Operation Napalm".to_string(),
                "OSerif".to_string(),
                "Lanix Ox".to_string(),
            ],
            font_size: DEFAULT_FONT_SIZE,
            weight: Weight::NORMAL,
            stretch: Stretch::Normal,
            style: Style::Normal,
            color: [255, 255, 255, 255],
            letter_spacing: 0.0,
            word_spacing: 0.0,
            line_height: LineHeight::default(),
            decorations: TextDecorations::default(),
            extra_features: vec![],
            variable_axes: vec![],
            synthesize_styles: false,
            render_mode: RenderMode::default(),
            outline_rendering: false,
            material_effect_id: 0,
        }
    }
}

impl TextStyle {
    /// Create a new text style with the given family and size.
    pub fn new(family: &str, font_size: f32) -> Self {
        TextStyle {
            family: family.to_string(),
            font_size,
            ..Default::default()
        }
    }

    /// Set the font weight.
    pub fn with_weight(mut self, weight: u16) -> Self {
        self.weight = Weight(weight);
        self
    }

    /// Set italic style.
    pub fn italic(mut self) -> Self {
        self.style = Style::Italic;
        self
    }

    /// Set the text color.
    pub fn with_color(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.color = [r, g, b, a];
        self
    }

    /// Set letter spacing.
    pub fn with_letter_spacing(mut self, spacing: f32) -> Self {
        self.letter_spacing = spacing;
        self
    }

    /// Set word spacing.
    pub fn with_word_spacing(mut self, spacing: f32) -> Self {
        self.word_spacing = spacing;
        self
    }

    /// Set line height as a multiple of font size.
    pub fn with_line_height_multiple(mut self, multiple: f32) -> Self {
        self.line_height = LineHeight::Multiple(multiple);
        self
    }

    /// Set a fixed line height in pixels.
    pub fn with_line_height_fixed(mut self, pixels: f32) -> Self {
        self.line_height = LineHeight::Fixed(pixels);
        self
    }

    /// Add an OpenType feature.
    pub fn with_feature(mut self, feature: OpenTypeFeature) -> Self {
        self.extra_features.push(feature);
        self
    }

    /// Add a variable font axis.
    pub fn with_axis(mut self, axis: VariableAxis) -> Self {
        self.variable_axes.push(axis);
        self
    }

    /// Enable underline decoration.
    pub fn with_underline(mut self) -> Self {
        self.decorations.underline = true;
        self
    }

    /// Enable strikethrough decoration.
    pub fn with_strikethrough(mut self) -> Self {
        self.decorations.strikethrough = true;
        self
    }

    /// Set whether outline vector path rendering is enabled.
    pub fn with_outline_rendering(mut self, enabled: bool) -> Self {
        self.outline_rendering = enabled;
        self
    }

    /// Set the material effect ID for dynamic visual rendering.
    pub fn with_material_effect(mut self, effect_id: u32) -> Self {
        self.material_effect_id = effect_id;
        self
    }
}

// ── TextSpan ─────────────────────────────────────────────────────────────────

/// Vertical alignment strategies for inline UI portals within a text line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PortalAlignment {
    /// Align the bottom of the portal box to the text baseline.
    #[default]
    Baseline,
    /// Align the top of the portal box to the top of the line height.
    Top,
    /// Center the portal box vertically within the line height.
    Center,
    /// Align the bottom of the portal box to the bottom of the line height.
    Bottom,
}

/// Identifies the layout behavior of a TextSpan (standard text vs inline portal).
#[derive(Debug, Clone, PartialEq, Default)]
pub enum TextSpanKind {
    /// Standard text flow.
    #[default]
    Text,
    /// An inline interactive widget box.
    Portal {
        /// Width of the portal box in pixels.
        width: f32,
        /// Height of the portal box in pixels.
        height: f32,
        /// Vertical alignment mode.
        alignment: PortalAlignment,
        /// Unique identifier for downstream portal instantiation.
        id: String,
    },
}

/// A span of text or an inline UI portal with associated styling.
#[derive(Debug, Clone, PartialEq)]
pub struct TextSpan {
    /// The text content (stores "\u{FFFC}" object placeholder for portals).
    pub text: String,
    /// The style to apply.
    pub style: TextStyle,
    /// Byte offset in the full text where this span starts.
    pub byte_offset: usize,
    /// Layout category of the span.
    pub kind: TextSpanKind,
}

impl TextSpan {
    /// Create a new text span.
    pub fn new(text: &str, style: TextStyle) -> Self {
        TextSpan {
            text: text.to_string(),
            style,
            byte_offset: 0,
            kind: TextSpanKind::Text,
        }
    }

    /// Create a new text span at a specific byte offset.
    pub fn at(text: &str, style: TextStyle, byte_offset: usize) -> Self {
        TextSpan {
            text: text.to_string(),
            style,
            byte_offset,
            kind: TextSpanKind::Text,
        }
    }

    /// Create a new inline UI portal span.
    pub fn portal(
        width: f32,
        height: f32,
        alignment: PortalAlignment,
        id: &str,
        style: TextStyle,
    ) -> Self {
        TextSpan {
            text: "\u{FFFC}".to_string(),
            style,
            byte_offset: 0,
            kind: TextSpanKind::Portal {
                width,
                height,
                alignment,
                id: id.to_string(),
            },
        }
    }

    /// Create a new inline UI portal span at a specific byte offset.
    pub fn portal_at(
        width: f32,
        height: f32,
        alignment: PortalAlignment,
        id: &str,
        style: TextStyle,
        byte_offset: usize,
    ) -> Self {
        TextSpan {
            text: "\u{FFFC}".to_string(),
            style,
            byte_offset,
            kind: TextSpanKind::Portal {
                width,
                height,
                alignment,
                id: id.to_string(),
            },
        }
    }
}

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
        direction: Direction,
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

// ── Glyph types ──────────────────────────────────────────────────────────────

/// A positioned glyph ready for rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphInstance {
    /// The glyph ID.
    pub glyph_id: u16,
    /// X position (pixels from origin).
    pub x: f32,
    /// Y position (pixels from origin, baseline-relative).
    pub y: f32,
    /// Rotation angle in radians (used when rendering text along curves).
    pub angle: f32,
    /// Advance width in pixels.
    pub advance_width: f32,
    /// Advance height in pixels.
    pub advance_height: f32,
    /// The cluster index this glyph belongs to.
    pub cluster: u32,
    /// Whether this glyph is from a RTL run.
    pub is_rtl: bool,
    /// Unique composite cache key for rasterization lookup, incorporating font identity, size, styling, and glyph ID.
    pub cache_key: u64,
    /// Linear index of this glyph in the paragraph (used for animation cascades).
    pub glyph_index: usize,
    /// Time offset applied to this glyph for kinetic typography.
    pub time_offset: f32,
}

/// A segment in a glyph vector outline path.
///
/// Exposes raw quadratic and cubic Bezier control points to be processed
/// and evaluated directly by GPU shaders for resolution-independent rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RunicPathSegment {
    /// Move the pen to the specified point. Starts a new subpath.
    MoveTo {
        /// X coordinate of destination point.
        x: f32,
        /// Y coordinate of destination point.
        y: f32,
    },
    /// Draw a straight line segment to the specified point.
    LineTo {
        /// X coordinate of destination point.
        x: f32,
        /// Y coordinate of destination point.
        y: f32,
    },
    /// Draw a quadratic Bezier curve to the specified point using one control point.
    QuadTo {
        /// X coordinate of the Bezier control point.
        cx: f32,
        /// Y coordinate of the Bezier control point.
        cy: f32,
        /// X coordinate of destination point.
        x: f32,
        /// Y coordinate of destination point.
        y: f32,
    },
    /// Draw a cubic Bezier curve to the specified point using two control points.
    CubicTo {
        /// X coordinate of the first Bezier control point.
        cx1: f32,
        /// Y coordinate of the first Bezier control point.
        cy1: f32,
        /// X coordinate of the second Bezier control point.
        cx2: f32,
        /// Y coordinate of the second Bezier control point.
        cy2: f32,
        /// X coordinate of destination point.
        x: f32,
        /// Y coordinate of destination point.
        y: f32,
    },
    /// Close the current subpath by drawing a straight line back to the start.
    Close,
}

/// A rasterized glyph image.
#[derive(Debug, Clone, PartialEq)]
pub struct GlyphImage {
    /// The glyph ID.
    pub glyph_id: u16,
    /// Width of the image in pixels.
    pub width: u32,
    /// Height of the image in pixels.
    pub height: u32,
    /// Pixel data (RGBA, premultiplied alpha).
    pub data: Vec<u8>,
    /// X offset from the cursor position.
    pub x_offset: f32,
    /// Y offset from the cursor position (positive = up).
    pub y_offset: f32,
    /// Cache key for the swash cache.
    pub cache_key: u64,
}

// ── LineInfo ─────────────────────────────────────────────────────────────────

/// Information about a single line of laid-out text.
#[derive(Debug, Clone, PartialEq)]
pub struct LineInfo {
    /// Index of the first glyph in this line.
    pub glyph_start: usize,
    /// Index past the last glyph in this line.
    pub glyph_end: usize,
    /// Y position of the line baseline.
    pub baseline_y: f32,
    /// Height of this line.
    pub height: f32,
    /// Width of the text content in this line.
    pub width: f32,
    /// X offset for alignment (0 for left-aligned).
    pub x_offset: f32,
    /// Byte offset in the full text where this line starts.
    pub byte_offset: usize,
    /// The text content of this line.
    pub text: String,
}

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

        let mut best_glyph = 0u32;
        let mut best_dist = u64::MAX;

        // Find the cluster whose byte range contains byte_index
        for glyph in &self.glyphs {
            let cluster_byte = self.byte_pos_for_cluster(glyph.cluster);
            let dist = if cluster_byte > byte_index {
                (cluster_byte - byte_index) as u64
            } else {
                (byte_index - cluster_byte) as u64
            };
            if dist < best_dist {
                best_dist = dist;
                best_glyph = glyph.cluster;
            }
        }

        // Find the glyph index for this cluster
        for (i, glyph) in self.glyphs.iter().enumerate() {
            if glyph.cluster == best_glyph {
                return (i, best_glyph);
            }
        }

        (0, 0)
    }

    /// Get the cursor position (x, line_index) for a byte index.
    pub fn cursor_position(&self, byte_index: usize) -> (f32, usize) {
        if self.glyphs.is_empty() {
            return (0.0, 0);
        }

        let (glyph_idx, _cluster) = self.hit_test(byte_index);

        // Find which line this glyph is on
        let mut line_idx = 0;
        for (li, line) in self.lines.iter().enumerate() {
            if glyph_idx >= line.glyph_start && glyph_idx < line.glyph_end {
                line_idx = li;
                break;
            }
        }

        // x is the left edge of the glyph, adjusted for alignment
        let glyph = &self.glyphs[glyph_idx];
        let line = &self.lines[line_idx];
        let x = line.x_offset + glyph.x;

        (x, line_idx)
    }

    /// Get selection rectangles for a byte range [start, end).
    pub fn selection_rects(&self, start: usize, end: usize) -> Vec<[f32; 4]> {
        if self.glyphs.is_empty() || start >= end {
            return vec![];
        }

        let mut rects = Vec::new();
        let mut current_rect: Option<[f32; 4]> = None;

        for glyph in &self.glyphs {
            let cluster_start = self.byte_pos_for_cluster(glyph.cluster);
            let cluster_end = if glyph.cluster + 1 < self.total_clusters() {
                self.byte_pos_for_cluster(glyph.cluster + 1)
            } else {
                self.text.len()
            };

            // Check if this glyph's cluster overlaps with the selection
            if cluster_start < end && cluster_end > start {
                // Find the line for y/height
                let mut line_top = 0.0f32;
                let mut line_h = self.height;
                for line in &self.lines {
                    if glyph.cluster >= self.glyphs[line.glyph_start].cluster
                        && (line.glyph_end == self.glyphs.len()
                            || glyph.cluster < self.glyphs[line.glyph_end].cluster)
                    {
                        line_top = line.baseline_y - self.ascent;
                        line_h = line.height;
                        break;
                    }
                }

                let x = glyph.x;
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

    /// Get the byte position for a cluster index.
    fn byte_pos_for_cluster(&self, cluster: u32) -> usize {
        self.grapheme_boundaries
            .get(cluster as usize)
            .copied()
            .unwrap_or(self.text.len())
    }

    /// Total number of clusters in the text.
    fn total_clusters(&self) -> u32 {
        self.grapheme_boundaries.len() as u32
    }
}

// ── FontData ─────────────────────────────────────────────────────────────────

/// Owning wrapper for font data that can be shared.
#[derive(Clone)]
struct FontData {
    data: std::sync::Arc<Vec<u8>>,
    index: u32,
}

impl FontData {
    fn new(data: Vec<u8>, index: u32) -> Self {
        FontData {
            data: std::sync::Arc::new(data),
            index,
        }
    }

    fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    fn font_ref(&self) -> Option<FontRef<'_>> {
        FontRef::from_index(&self.data, self.index as usize)
    }

    fn face(&self) -> Option<rustybuzz::Face<'_>> {
        rustybuzz::Face::from_slice(&self.data, self.index)
    }
}

// ── ResolvedFont ─────────────────────────────────────────────────────────────

/// A resolved font with its faces and metadata.
struct ResolvedFont {
    primary: FontData,
    fallbacks: Vec<FontData>,
    cache_key: u64,
    units_per_em: u16,
    ascent: f32,
    descent: f32,
    line_gap: f32,
    x_height: f32,
    cap_height: f32,
    has_colr: bool,
}

impl ResolvedFont {
    fn from_data(data: FontData) -> Option<Self> {
        let font_ref = data.font_ref()?;
        let _face_ref = font_ref; // FontRef derefs to provide table data

        // Get metrics from the font
        let _metrics = swash::scale::image::Image::new(); // placeholder
        // We'll get metrics via font_ref's internal data
        // Use swash's metrics method through the shape module
        let cache_key = font_ref.key.value();

        // Read metrics directly from the font data using ttf-parser
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

    fn metrics_pixels(&self, font_size: f32) -> (f32, f32, f32) {
        let scale = font_size / self.units_per_em as f32;
        (
            self.ascent * scale,
            self.descent * scale,
            self.line_gap * scale,
        )
    }
}

// ── RunicTextEngine ──────────────────────────────────────────────────────────

/// The main text shaping and layout engine.
pub struct RunicTextEngine {
    /// Font database.
    db: Database,
    /// Font data cache: fontdb::ID -> FontData.
    font_data: HashMap<fontdb::ID, FontData>,
    /// Shape cache.
    cache: HashMap<CacheKey, Vec<GlyphInstance>>,
    /// Cache access order for LRU eviction.
    cache_order: Vec<CacheKey>,
    /// Scale context for rasterization.
    scale_context: ScaleContext,
}

impl RunicTextEngine {
    /// Create a new text engine with system fonts and user fonts.
    ///
    /// # Contract
    /// Guaranteed to successfully instantiate a usable text engine. Loads all standard
    /// system and user fonts first, and then embeds Jupiteroid.ttf as an absolute last-resort
    /// fallback so text rendering cannot fail even on zero-font systems.
    pub fn new() -> Self {
        let mut db = Database::new();
        db.load_system_fonts();

        // Load user fonts from standard directories
        let home = std::env::var("HOME").unwrap_or_default();
        for dir in &[
            format!("{}/.local/share/fonts", home),
            format!("{}/.fonts", home),
            "/usr/share/fonts".to_string(),
            "/usr/local/share/fonts".to_string(),
        ] {
            db.load_fonts_dir(dir);
        }

        let mut engine = RunicTextEngine {
            db,
            font_data: HashMap::new(),
            cache: HashMap::new(),
            cache_order: Vec::new(),
            scale_context: ScaleContext::new(),
        };

        // Load Jupiteroid.ttf as a built-in last-resort fallback font
        engine.load_font_data(include_bytes!("../Fonts/Jupiteroid.ttf").to_vec());

        engine
    }

    /// Create a light text engine for testing — no system/user font loading.
    /// Only bundled fonts (loaded via `load_font_data()`) are available.
    pub fn new_light() -> Self {
        RunicTextEngine {
            db: Database::new(),
            font_data: HashMap::new(),
            cache: HashMap::new(),
            cache_order: Vec::new(),
            scale_context: ScaleContext::new(),
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
        self.db.load_font_data(data.clone());
        for face in self.db.faces() {
            let id = face.id;
            self.font_data.entry(id).or_insert_with(|| {
                let face_index = face.index;
                FontData::new(data.clone(), face_index)
            });
        }
    }
    /// Get or load FontData for a fontdb ID.
    fn get_font_data(&mut self, id: fontdb::ID) -> Option<FontData> {
        if let Some(data) = self.font_data.get(&id) {
            return Some(data.clone());
        }

        // Load from the database
        let (source, face_index) = self.db.face_source(id)?;
        let data = match source {
            Source::Binary(arc_data) => {
                // arc_data is Arc<dyn AsRef<[u8]> + Sync + Send>
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
    fn resolve_font(&mut self, style: &TextStyle) -> Result<ResolvedFont, ShapingError> {
        // Try primary family
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
                // Load fallbacks - collect IDs first to avoid borrow issues
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

        // Last resort: any font
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

    /// Build rustybuzz Features from a TextStyle.
    fn build_features(style: &TextStyle) -> Vec<Feature> {
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
    fn calculate_glyph_cache_key(
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
    fn shape_run(
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

        // Check cache
        if let Some(glyphs) = self.cache.get(&cache_key) {
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

        // Apply font fallback for missing glyphs
        self.apply_fallbacks(&mut glyphs, text, style, &resolved, &features);

        // Update cache
        self.insert_cache(cache_key, glyphs.clone());

        Ok(glyphs)
    }

    /// Check if a cluster represents a space character.
    fn is_space_cluster(text: &str, cluster: u32) -> bool {
        text.chars()
            .nth(cluster as usize)
            .is_some_and(|c| c.is_ascii_whitespace())
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
                let c = text
                    .chars()
                    .nth(glyph_cluster as usize)
                    .unwrap_or('\u{FFFD}');

                // Try each fallback font
                for fallback in &resolved.fallbacks {
                    if let Some(face) = fallback.face() {
                        let mut buf = UnicodeBuffer::new();
                        buf.add(c, glyph_cluster);
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

                            let fallback_key = fallback
                                .font_ref()
                                .map(|r| r.key.value())
                                .unwrap_or(resolved.cache_key);
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

    /// Insert into cache with LRU eviction.
    fn insert_cache(&mut self, key: CacheKey, value: Vec<GlyphInstance>) {
        if self.cache.len() >= MAX_CACHE_SIZE
            && let Some(oldest) = self.cache_order.first().cloned()
        {
            self.cache.remove(&oldest);
            self.cache_order.remove(0);
        }

        self.cache.insert(key, value);
        self.cache_order.push(key);
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

        // Shape each span
        for span in spans {
            // Determine direction from BiDi analysis
            let direction = if let Some(para_info) = bidi.paragraphs.first() {
                let mut dir = Direction::LeftToRight;
                for bi in para_info.range.clone() {
                    if bi < bidi.levels.len() {
                        if bidi.levels[bi].is_rtl() {
                            dir = Direction::RightToLeft;
                            has_rtl = true;
                        }
                        break;
                    }
                }
                dir
            } else {
                Direction::LeftToRight
            };

            let mut run_glyphs = match &span.kind {
                TextSpanKind::Text => self.shape_run(&span.text, &span.style, direction)?,
                TextSpanKind::Portal { width, height, .. } => {
                    vec![GlyphInstance {
                        glyph_id: 0xFFFF,
                        x: 0.0,
                        y: 0.0,
                        angle: 0.0,
                        advance_width: *width,
                        advance_height: *height,
                        cluster: span.byte_offset as u32,
                        is_rtl: false,
                        cache_key: 0,
                        glyph_index: 0,
                        time_offset: 0.0,
                    }]
                }
            };

            // Offset glyph x positions by accumulated width
            let span_offset_x = all_glyphs
                .last()
                .map(|g| g.x + g.advance_width)
                .unwrap_or(0.0);
            for glyph in &mut run_glyphs {
                glyph.x += span_offset_x;
            }

            // Track primary font metrics from the first span
            if all_glyphs.is_empty() {
                primary_metrics = (
                    span.style.font_size * 0.8, // ascent estimate
                    span.style.font_size * 0.2, // descent estimate
                    span.style.font_size * 0.2, // line gap estimate
                );
                if let Ok(resolved) = self.resolve_font(&span.style) {
                    primary_metrics = resolved.metrics_pixels(span.style.font_size);
                }
                primary_line_height_px = span.style.line_height.to_pixels(span.style.font_size);
            }

            for mut glyph in run_glyphs {
                glyph.glyph_index = global_glyph_index;
                glyph.time_offset = global_glyph_index as f32 * 0.05; // Base 50ms stagger
                global_glyph_index += 1;
                all_glyphs.push(glyph);
            }
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
                    // Compute byte position after this cluster
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
                    // Need to break
                    let break_glyph = if last_word_break_glyph > line_start_glyph {
                        last_word_break_glyph
                    } else {
                        i
                    };
                    let break_byte = if last_word_break_byte > line_start_byte {
                        last_word_break_byte
                    } else {
                        // Compute byte offset for cluster at break point
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

                    // Compute line width
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

                let mut x = x_offset;
                for g in &mut glyphs[line_start_glyph..] {
                    g.x = x;
                    if g.glyph_id == 0xFFFF {
                        // Locate matching portal span configuration by matching byte offset cluster index
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
                        // Adjust Y offset depending on portal alignment relative to baseline/line height
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
                    // Locate matching portal span configuration by matching byte offset cluster index
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
                    // Adjust Y offset depending on portal alignment relative to baseline/line height
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
                    reorder_line_rtl(glyphs, line.glyph_start, line.glyph_end);
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
                    // Find how many glyphs fit
                    let mut trunc_width = 0.0f32;
                    let mut trunc_glyph_end = line.glyph_start;
                    // Approximate ellipsis width
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
                    // Offset perpendicularly by the baseline relative coordinate
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
        }

        if style.synthesize_styles && style.weight >= Weight(700) {
            render.embolden(0.04);
        }

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

        // Try fallback fonts
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

        // Helper closure to map Outline points and verbs directly into RunicPathSegment vector
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

        // Use swash's outline scaler to retrieve raw curves
        if let Some(outline) = scaler.scale_outline(glyph_id) {
            return Ok(map_outline_to_segments(outline));
        }

        // Try fallbacks
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
        self.cache.clear();
        self.cache_order.clear();
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), MAX_CACHE_SIZE)
    }

    /// Get the number of faces in the database.
    pub fn font_count(&self) -> usize {
        self.db.faces().count()
    }

    /// Query the variable font axes available for a given font family.
    ///
    /// Returns `Ok(None)` if the font is not variable.
    /// Returns `Err` if the font cannot be found.
    ///
    /// # Arguments
    /// * `family` — Font family name.
    /// * `font_size` — Font size for resolving the face.
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

        // Use ttf-parser to read the fvar table
        let ttf_face = rustybuzz::ttf_parser::Face::parse(data.as_bytes(), data.index)
            .map_err(|_| ShapingError::InvalidFontData)?;

        // Check if this is a variable font
        let fvar_data = match ttf_face
            .raw_face()
            .table(rustybuzz::ttf_parser::Tag(u32::from_be_bytes(*b"fvar")))
        {
            Some(d) => d,
            None => return Ok(None), // Not a variable font
        };

        // Parse the fvar table manually
        // fvar table format: version(4), offsetToData(2), reserved(2), axisCount(2), axisSize(2), instanceCount(2), instanceSize(2)
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

            // fvar axis record: tag(4), minValue(4), defaultValue(4), maxValue(4), flags(2), nameID(2)
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
            let _name_id = u16::from_be_bytes([axis_data[18], axis_data[19]]);

            let tag_bytes = tag.to_be_bytes();
            let tag_string = String::from_utf8_lossy(&tag_bytes).trim().to_string();

            // Standard axes: wght, wdth, ital, slnt, opsz, plus many more
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

    // ── Backward-compatible API for cvkg-render-gpu ──────────────────────────

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
                grapheme_boundaries: vec![],
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
        let mut found: Option<(CacheKey, GlyphInstance)> = None;
        for (ck, glyphs) in &self.cache {
            if let Some(g) = glyphs.iter().find(|g| g.cache_key == cache_key) {
                found = Some((*ck, *g));
                break;
            }
        }
        let (ck, glyph) = found?;

        // Reconstruct font family from the database matching the font_cache_key
        let mut family = "sans-serif".to_string();
        let face_ids: Vec<fontdb::ID> = self.db.faces().map(|f| f.id).collect();
        for id in face_ids {
            if let Some(font_data) = self.get_font_data(id)
                && let Some(font_ref) = font_data.font_ref()
                && font_ref.key.value() == ck.font_cache_key
            {
                if let Some(face) = self.db.face(id)
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

fn byte_offset_level(bidi: &BidiInfo, byte_offset: usize) -> unicode_bidi::Level {
    if let Some(para) = bidi.paragraphs.first() {
        let relative = byte_offset.saturating_sub(para.range.start);
        if relative < bidi.levels.len() {
            return bidi.levels[relative];
        }
    }
    unicode_bidi::Level::ltr()
}

fn line_bidi_level(bidi: &BidiInfo, byte_offset: usize) -> unicode_bidi::Level {
    byte_offset_level(bidi, byte_offset)
}

fn reorder_line_rtl(glyphs: &mut [GlyphInstance], start: usize, end: usize) {
    if end <= start {
        return;
    }
    let slice = &mut glyphs[start..end];
    slice.reverse();
    let mut x = 0.0f32;
    for g in slice.iter_mut() {
        g.x = x;
        x += g.advance_width;
    }
}

impl Default for RunicTextEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ── FontMetrics ──────────────────────────────────────────────────────────────

/// Font metrics for a given style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontMetrics {
    /// Ascent above baseline.
    pub ascent: f32,
    /// Descent below baseline (positive value).
    pub descent: f32,
    /// Recommended line gap.
    pub line_gap: f32,
    /// Units per em.
    pub units_per_em: u16,
    /// X-height.
    pub x_height: f32,
    /// Cap height.
    pub cap_height: f32,
}

// ── Tests ────────────────────────────────────────────────────────────────────

// ── MSDF Glyph Rendering ────────────────────────────────────────────────────

pub mod msdf;

// ── Knuth-Plass Line Breaking ───────────────────────────────────────────────

pub mod knuth_plass;

// ── Color Emoji Atlas ───────────────────────────────────────────────────────

pub mod emoji;

// ── Subpixel LCD Positioning ────────────────────────────────────────────────

pub mod subpixel;

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_shaping() {
        let mut engine = RunicTextEngine::new_test();
        let style = TextStyle::new("Jupiteroid", 16.0);
        let glyphs = engine
            .shape_run("Hello", &style, Direction::LeftToRight)
            .unwrap();
        assert!(!glyphs.is_empty(), "Should produce glyphs for 'Hello'");
    }

    #[test]
    fn test_hit_test() {
        let mut engine = RunicTextEngine::new_test();
        let style = TextStyle::new("Jupiteroid", 16.0);
        let spans = vec![TextSpan::new("Hello", style.clone())];
        let shaped = engine
            .shape_layout(&spans, None, TextAlign::Start, TextOverflow::WordWrap)
            .unwrap();
        let (glyph_idx, cluster) = shaped.hit_test(0);
        assert!(glyph_idx < shaped.glyphs.len());
        assert_eq!(cluster, 0);
    }

    #[test]
    fn test_word_wrapping() {
        let mut engine = RunicTextEngine::new_test();
        let style = TextStyle::new("Jupiteroid", 16.0);
        let spans = vec![TextSpan::new("Hello World This Is A Test", style.clone())];
        let shaped = engine
            .shape_layout(&spans, Some(80.0), TextAlign::Start, TextOverflow::WordWrap)
            .unwrap();
        assert!(
            shaped.lines.len() > 1,
            "Should wrap into multiple lines, got {}",
            shaped.lines.len()
        );
    }

    #[test]
    fn test_text_style_defaults() {
        let style = TextStyle::default();
        assert_eq!(style.family, "Jupiteroid");
        assert_eq!(style.font_size, DEFAULT_FONT_SIZE);
        assert_eq!(style.weight, Weight::NORMAL);
        assert_eq!(style.color, [255, 255, 255, 255]);
        assert!(!style.fallback_families.is_empty());
    }

    #[test]
    fn test_text_style_builder() {
        let style = TextStyle::new("Jupiteroid", 24.0)
            .with_weight(700)
            .italic()
            .with_color(255, 0, 0, 255)
            .with_letter_spacing(1.5)
            .with_underline();

        assert_eq!(style.font_size, 24.0);
        assert_eq!(style.weight, Weight(700));
        assert_eq!(style.style, Style::Italic);
        assert_eq!(style.color, [255, 0, 0, 255]);
        assert_eq!(style.letter_spacing, 1.5);
        assert!(style.decorations.underline);
    }

    #[test]
    fn test_line_height() {
        let multiple = LineHeight::Multiple(1.5);
        assert_eq!(multiple.to_pixels(16.0), 24.0);

        let fixed = LineHeight::Fixed(20.0);
        assert_eq!(fixed.to_pixels(16.0), 20.0);
    }

    #[test]
    fn test_cache_key_deterministic() {
        let key1 = CacheKey::new(
            "Hello",
            12345,
            16.0,
            Weight::NORMAL,
            Stretch::Normal,
            Style::Normal,
            Direction::LeftToRight,
            0.0,
            0.0,
        );
        let key2 = CacheKey::new(
            "Hello",
            12345,
            16.0,
            Weight::NORMAL,
            Stretch::Normal,
            Style::Normal,
            Direction::LeftToRight,
            0.0,
            0.0,
        );
        assert_eq!(key1, key2);

        let key3 = CacheKey::new(
            "World",
            12345,
            16.0,
            Weight::NORMAL,
            Stretch::Normal,
            Style::Normal,
            Direction::LeftToRight,
            0.0,
            0.0,
        );
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_cursor_position() {
        let mut engine = RunicTextEngine::new_test();
        let style = TextStyle::new("Jupiteroid", 16.0);
        let spans = vec![TextSpan::new("Hello", style.clone())];
        let shaped = engine
            .shape_layout(&spans, None, TextAlign::Start, TextOverflow::WordWrap)
            .unwrap();
        let (x, line) = shaped.cursor_position(0);
        assert_eq!(line, 0);
        assert!(x >= 0.0);
    }

    #[test]
    fn test_selection_rects() {
        let mut engine = RunicTextEngine::new_test();
        let style = TextStyle::new("Jupiteroid", 16.0);
        let spans = vec![TextSpan::new("Hello World", style.clone())];
        let shaped = engine
            .shape_layout(&spans, None, TextAlign::Start, TextOverflow::WordWrap)
            .unwrap();
        let rects = shaped.selection_rects(0, 5);
        assert!(
            !rects.is_empty(),
            "Should produce selection rects for 'Hello'"
        );
    }

    #[test]
    fn test_open_type_features() {
        let liga = OpenTypeFeature::liga();
        assert_eq!(liga.tag, u32::from_be_bytes(*b"liga"));
        assert_eq!(liga.value, 1);

        let kern = OpenTypeFeature::kern();
        assert_eq!(kern.tag, u32::from_be_bytes(*b"kern"));
    }

    #[test]
    fn test_variable_axes() {
        let weight = VariableAxis::weight(700.0);
        assert_eq!(weight.tag, u32::from_be_bytes(*b"wght"));
        assert_eq!(weight.value, 700.0);

        let italic = VariableAxis::italic(1.0);
        assert_eq!(italic.tag, u32::from_be_bytes(*b"ital"));
    }

    #[test]
    fn test_font_metrics() {
        let mut engine = RunicTextEngine::new_test();
        let style = TextStyle::new("Jupiteroid", 16.0);
        let metrics = engine.font_metrics(&style).unwrap();
        assert!(metrics.ascent > 0.0);
        assert!(metrics.descent > 0.0);
        assert!(metrics.units_per_em > 0);
    }

    #[test]
    fn test_empty_input() {
        let mut engine = RunicTextEngine::new_test();
        let style = TextStyle::new("Jupiteroid", 16.0);
        let spans = vec![TextSpan::new("", style.clone())];
        let shaped = engine
            .shape_layout(&spans, None, TextAlign::Start, TextOverflow::WordWrap)
            .unwrap();
        assert!(shaped.glyphs.is_empty());
    }

    #[test]
    fn test_multi_span_layout() {
        let mut engine = RunicTextEngine::new_test();
        let style1 = TextStyle::new("Jupiteroid", 16.0);
        let style2 = TextStyle::new("Jupiteroid", 24.0).with_color(255, 0, 0, 255);
        let spans = vec![
            TextSpan::at("Hello ", style1, 0),
            TextSpan::at("World", style2, 6),
        ];
        let shaped = engine
            .shape_layout(&spans, None, TextAlign::Start, TextOverflow::WordWrap)
            .unwrap();
        assert!(!shaped.glyphs.is_empty());
        assert_eq!(shaped.text, "Hello World");
    }

    #[test]
    fn test_text_align_center() {
        let mut engine = RunicTextEngine::new_test();
        let style = TextStyle::new("Jupiteroid", 16.0);
        let spans = vec![TextSpan::new("Hi", style.clone())];
        let shaped = engine
            .shape_layout(
                &spans,
                Some(200.0),
                TextAlign::Center,
                TextOverflow::WordWrap,
            )
            .unwrap();
        assert!(!shaped.lines.is_empty());
        let line = &shaped.lines[0];
        assert!(
            line.x_offset > 0.0,
            "Center-aligned line should have positive x_offset, got {}",
            line.x_offset
        );
    }

    #[test]
    fn test_text_overflow_ellipsis() {
        let mut engine = RunicTextEngine::new_test();
        let style = TextStyle::new("Jupiteroid", 16.0);
        let spans = vec![TextSpan::new("Hello World This Is Long", style.clone())];
        let shaped = engine
            .shape_layout(&spans, Some(50.0), TextAlign::Start, TextOverflow::Ellipsis)
            .unwrap();
        assert!(!shaped.lines.is_empty());
    }

    #[test]
    fn test_decorations() {
        let decorations = TextDecorations {
            underline: true,
            strikethrough: true,
            overline: false,
        };
        assert!(decorations.underline);
        assert!(decorations.strikethrough);
        assert!(!decorations.overline);
    }

    #[test]
    fn test_cache_eviction() {
        let mut engine = RunicTextEngine::new_test();
        let style = TextStyle::new("Jupiteroid", 16.0);

        let _ = engine.shape_run("Test", &style, Direction::LeftToRight);

        let (size, max) = engine.cache_stats();
        assert!(size > 0, "Cache should have entries after shaping");
        assert_eq!(max, MAX_CACHE_SIZE);

        engine.clear_cache();
        let (size, _) = engine.cache_stats();
        assert_eq!(size, 0);
    }

    #[test]
    fn test_font_count() {
        let engine = RunicTextEngine::new_test();
        let count = engine.font_count();
        assert!(count > 0, "Should find at least one font, got {}", count);
    }

    #[test]
    fn test_jupiteroid_font_available() {
        let engine = RunicTextEngine::new_test();
        assert!(engine.font_count() > 0, "Should have fonts loaded");
    }

    #[test]
    fn test_extract_glyph_path() {
        let mut engine = RunicTextEngine::new_test();
        let style = TextStyle::new("Jupiteroid", 16.0);

        // Shape a character to get a guaranteed valid glyph ID in the test font
        let glyphs = engine
            .shape_run("A", &style, Direction::LeftToRight)
            .unwrap();
        assert!(!glyphs.is_empty(), "Shaping 'A' should yield a glyph");
        let glyph_id = glyphs[0].glyph_id;

        // Extract the outline vector path for this glyph
        let path = engine.extract_glyph_path(glyph_id, 16.0, &style).unwrap();

        // Verify that the outline path is not empty and starts with MoveTo, containing at least one Close
        assert!(!path.is_empty(), "Glyph path for 'A' should not be empty");
        match path[0] {
            RunicPathSegment::MoveTo { x, y } => {
                assert!(x.is_finite());
                assert!(y.is_finite());
            }
            _ => panic!("Expected first segment to be a MoveTo, got {:?}", path[0]),
        }

        let has_close = path
            .iter()
            .any(|seg| matches!(seg, RunicPathSegment::Close));
        assert!(
            has_close,
            "Expected glyph path to contain at least one Close command"
        );

        // Assert all segment coordinates are finite values
        for segment in &path {
            match *segment {
                RunicPathSegment::MoveTo { x, y } => {
                    assert!(x.is_finite());
                    assert!(y.is_finite());
                }
                RunicPathSegment::LineTo { x, y } => {
                    assert!(x.is_finite());
                    assert!(y.is_finite());
                }
                RunicPathSegment::QuadTo { cx, cy, x, y } => {
                    assert!(cx.is_finite());
                    assert!(cy.is_finite());
                    assert!(x.is_finite());
                    assert!(y.is_finite());
                }
                RunicPathSegment::CubicTo {
                    cx1,
                    cy1,
                    cx2,
                    cy2,
                    x,
                    y,
                } => {
                    assert!(cx1.is_finite());
                    assert!(cy1.is_finite());
                    assert!(cx2.is_finite());
                    assert!(cy2.is_finite());
                    assert!(x.is_finite());
                    assert!(y.is_finite());
                }
                RunicPathSegment::Close => {}
            }
        }
    }

    #[test]
    fn test_new_text_style_fields() {
        let style = TextStyle::new("Jupiteroid", 16.0)
            .with_outline_rendering(true)
            .with_material_effect(42);

        assert!(style.outline_rendering);
        assert_eq!(style.material_effect_id, 42);
    }

    #[test]
    fn test_text_path_sampling() {
        // Curve: (0,0) -> (100, 100) -> (200, 0)
        let tp = TextPath::new(vec![(0.0, 0.0), (100.0, 100.0), (200.0, 0.0)]);
        let ((x_start, y_start), angle_start) = tp.sample(0.0);
        let ((x_mid, y_mid), angle_mid) = tp.sample(0.5);

        assert_eq!(x_start, 0.0);
        assert_eq!(y_start, 0.0);
        assert!(angle_start > 0.0);

        assert_eq!(x_mid, 100.0);
        assert_eq!(y_mid, 50.0);
        assert!(angle_mid.abs() < 1e-4); // peak tangent is horizontal (angle=0)
    }

    #[test]
    fn test_layout_boundary_circle() {
        let boundary = LayoutBoundary::Circle {
            cx: 100.0,
            cy: 100.0,
            r: 50.0,
        };
        // At y = 100 (center of circle), allowed span should be [50.0, 150.0]
        let span = boundary.allowed_span(100.0).unwrap();
        assert_eq!(span.0, 50.0);
        assert_eq!(span.1, 150.0);

        // At y = 150 (edge), dy = 50 -> dx = 0 -> allowed span [100.0, 100.0]
        let span_edge = boundary.allowed_span(150.0);
        assert!(span_edge.is_none() || span_edge.unwrap().0 >= 100.0);
    }

    #[test]
    fn test_shape_layout_with_path_and_boundary() {
        let mut engine = RunicTextEngine::new_test();
        let style = TextStyle::new("Jupiteroid", 16.0);
        let spans = vec![TextSpan::new(
            "Hello World Curved Layout Test String",
            style,
        )];

        // Test with curve path
        let tp = TextPath::new(vec![(0.0, 0.0), (100.0, 50.0), (200.0, 0.0)]);
        let shaped_path = engine
            .shape_layout_ex(
                &spans,
                None,
                TextAlign::Start,
                TextOverflow::WordWrap,
                Some(tp),
                None,
            )
            .unwrap();
        assert!(!shaped_path.glyphs.is_empty());
        // Verify glyph angles are non-zero due to curve tangent mapping
        let has_angles = shaped_path.glyphs.iter().any(|g| g.angle != 0.0);
        assert!(has_angles);

        // Test with boundary circle
        let boundary = LayoutBoundary::Circle {
            cx: 100.0,
            cy: 100.0,
            r: 50.0,
        };
        let shaped_boundary = engine
            .shape_layout_ex(
                &spans,
                None,
                TextAlign::Start,
                TextOverflow::WordWrap,
                None,
                Some(boundary),
            )
            .unwrap();
        assert!(!shaped_boundary.glyphs.is_empty());
    }

    #[test]
    fn test_portal_alignment() {
        let mut engine = RunicTextEngine::new_test();
        let style = TextStyle::new("Jupiteroid", 16.0);

        // Construct portal spans with different vertical alignment modes using correct byte offsets
        let spans = vec![
            TextSpan::at("Txt ", style.clone(), 0),
            TextSpan::portal_at(
                30.0,
                20.0,
                PortalAlignment::Baseline,
                "p_base",
                style.clone(),
                4,
            ),
            TextSpan::portal_at(30.0, 20.0, PortalAlignment::Top, "p_top", style.clone(), 7),
            TextSpan::portal_at(
                30.0,
                20.0,
                PortalAlignment::Center,
                "p_center",
                style.clone(),
                10,
            ),
            TextSpan::portal_at(
                30.0,
                20.0,
                PortalAlignment::Bottom,
                "p_bottom",
                style.clone(),
                13,
            ),
        ];

        // 1. Verify single-line layout (no wrapping)
        let shaped_single = engine
            .shape_layout(&spans, None, TextAlign::Start, TextOverflow::WordWrap)
            .unwrap();

        let portals_s: Vec<_> = shaped_single
            .glyphs
            .iter()
            .filter(|g| g.glyph_id == 0xFFFF)
            .collect();
        assert_eq!(portals_s.len(), 4);

        let baseline_y = shaped_single.lines[0].baseline_y;
        let ascent = shaped_single.ascent;
        let line_height_px = shaped_single.lines[0].height;

        // Baseline alignment -> y = baseline_y
        assert_eq!(portals_s[0].y, baseline_y);

        // Top alignment -> y = baseline_y - ascent
        assert_eq!(portals_s[1].y, baseline_y - ascent);

        // Center alignment -> y = baseline_y - ascent + (line_height - portal_h) / 2
        assert_eq!(
            portals_s[2].y,
            baseline_y - ascent + (line_height_px - 20.0) / 2.0
        );

        // Bottom alignment -> y = baseline_y - ascent + line_height - portal_h
        assert_eq!(portals_s[3].y, baseline_y - ascent + line_height_px - 20.0);

        // 2. Verify wrapped line layouts
        let shaped_wrapped = engine
            .shape_layout(&spans, Some(50.0), TextAlign::Start, TextOverflow::WordWrap)
            .unwrap();

        let portals_w: Vec<_> = shaped_wrapped
            .glyphs
            .iter()
            .filter(|g| g.glyph_id == 0xFFFF)
            .collect();
        assert_eq!(portals_w.len(), 4);
    }
}
