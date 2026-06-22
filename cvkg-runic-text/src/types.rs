use std::collections::HashMap;
// No imports needed here

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

// ── VariableAxis ─────────────────────────────────────────────────────────────

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

// =============================================================================
// P1-52: Typography Capability Model
// =============================================================================
//
// Exposes supported text features at runtime so applications can query
// and adapt to the text engine's capabilities.

/// Describes the capabilities of the text rendering engine.
/// Applications can query this to determine which features are available.
#[derive(Clone, Debug, Default)]
pub struct TextCapabilities {
    /// Whether variable fonts are supported.
    pub variable_fonts: bool,
    /// Whether color fonts (COLR/CPAL, SVG-in-OpenType, bitmap) are supported.
    pub color_fonts: bool,
    /// Whether OpenType features (ligatures, kerning, stylistic sets) are supported.
    pub open_type_features: bool,
    /// Whether subpixel positioning is used.
    pub subpixel_positioning: bool,
    /// Whether bidirectional text (Arabic, Hebrew) is supported.
    pub bidi: bool,
    /// Whether vertical text layout (Japanese, Chinese) is supported.
    pub vertical_text: bool,
    /// Whether font fallback chains are supported.
    pub font_fallback: bool,
    /// Whether hinting is applied at small sizes.
    pub hinting: bool,
    /// Whether shaping cache is enabled.
    pub shaping_cache: bool,
    /// Whether multi-atlas glyph management is used.
    pub multi_atlas: bool,
    /// Maximum number of glyph atlases.
    pub max_atlases: usize,
    /// Whether atlas defragmentation is supported.
    pub atlas_defragmentation: bool,
}

impl TextCapabilities {
    /// Return the default capabilities for the current engine build.
    pub fn default_capabilities() -> Self {
        Self {
            variable_fonts: true,
            color_fonts: false,  // Not yet implemented
            open_type_features: true,
            subpixel_positioning: true,
            bidi: true,
            vertical_text: false,  // Not yet implemented (P1-62)
            font_fallback: true,
            hinting: true,
            shaping_cache: true,
            multi_atlas: false,  // Not yet implemented (P1-60)
            max_atlases: 1,
            atlas_defragmentation: false,  // Not yet implemented (P1-59)
        }
    }

    /// Returns true if all common features are supported.
    pub fn is_fully_featured(&self) -> bool {
        self.variable_fonts
            && self.open_type_features
            && self.subpixel_positioning
            && self.bidi
            && self.font_fallback
            && self.hinting
            && self.shaping_cache
    }
}

// =============================================================================
// P1-54: Font Fallback Chain
// =============================================================================

/// A font fallback chain defines the order in which fonts are tried
/// when a glyph is not found in the primary font.
#[derive(Clone, Debug)]
pub struct FontFallbackChain {
    /// Ordered list of font family names to try.
    pub families: Vec<String>,
    /// Per-script fallback overrides (key is script name like "CJK", "Arabic").
    pub script_overrides: HashMap<&'static str, Vec<String>>,
}

impl Default for FontFallbackChain {
    fn default() -> Self {
        let mut script_overrides: HashMap<&'static str, Vec<String>> = HashMap::new();
        // CJK fallback
        script_overrides.insert(
            "CJK",
            vec![
                "Noto Sans CJK SC".to_string(),
                "Noto Sans CJK JP".to_string(),
                "Noto Sans CJK KR".to_string(),
            ],
        );
        // Arabic fallback
        script_overrides.insert(
            "Arabic",
            vec!["Noto Sans Arabic".to_string()],
        );
        // Emoji fallback
        script_overrides.insert(
            "Emoji",
            vec!["Noto Color Emoji".to_string()],
        );
        Self {
            families: vec![
                "system-ui".to_string(),
                "sans-serif".to_string(),
            ],
            script_overrides,
        }
    }
}

impl FontFallbackChain {
    /// Get the fallback chain for a specific script name.
    pub fn for_script(&self, script: &str) -> &[String] {
        self.script_overrides
            .get(script)
            .map(|v| v.as_slice())
            .unwrap_or(&self.families)
    }
}

// =============================================================================
// P1-55: Font Matching Strategy
// =============================================================================

/// Font matching strategy following CSS font-matching algorithm.
/// Matches family name, weight, stretch, and style.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FontMatchStrategy {
    /// Match by family name only (fastest).
    FamilyName,
    /// Match by family name, weight, and style (CSS-like).
    CssLike,
    /// Match by family name, weight, stretch, and style (full).
    Full,
}

impl Default for FontMatchStrategy {
    fn default() -> Self {
        FontMatchStrategy::CssLike
    }
}

// =============================================================================
// P1-56: Subpixel Positioning
// =============================================================================

/// Subpixel positioning mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubpixelMode {
    /// No subpixel positioning (integer pixel positions).
    None,
    /// Fractional pixel positions (1/64 pixel precision).
    Fractional,
}

impl Default for SubpixelMode {
    fn default() -> Self {
        SubpixelMode::Fractional
    }
}

// =============================================================================
// P1-57: Hinting Strategy
// =============================================================================

/// Hinting strategy for small text sizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HintingStrategy {
    /// No hinting (best for large sizes, screens with high DPI).
    None,
    /// Autohinting (algorithmic, works with any font).
    Auto,
    /// TrueType hinting (follows font instructions, best quality at small sizes).
    TrueType,
    /// Autohinting for small sizes, none for large sizes.
    AutoIfSmall,
}

impl Default for HintingStrategy {
    fn default() -> Self {
        HintingStrategy::AutoIfSmall
    }
}

impl HintingStrategy {
    /// Determine the effective hinting for a given font size.
    pub fn for_size(&self, size_pt: f32) -> HintingStrategy {
        match self {
            HintingStrategy::AutoIfSmall => {
                if size_pt <= 14.0 {
                    HintingStrategy::Auto
                } else {
                    HintingStrategy::None
                }
            }
            other => *other,
        }
    }
}

// =============================================================================
// P1-59: Atlas Defragmentation
// =============================================================================

/// Controls when atlas defragmentation is triggered.
#[derive(Clone, Copy, Debug)]
pub struct AtlasDefragConfig {
    /// Fragmentation ratio threshold (0.0-1.0). Defrag when wasted space
    /// exceeds this fraction of total atlas size.
    pub fragmentation_threshold: f32,
    /// Minimum time between defragmentation passes (seconds).
    pub min_interval_secs: f32,
}

impl Default for AtlasDefragConfig {
    fn default() -> Self {
        Self {
            fragmentation_threshold: 0.3,
            min_interval_secs: 5.0,
        }
    }
}

// =============================================================================
// P1-60: Multi-Atlas Scaling
// =============================================================================

/// Configuration for multi-atlas glyph management.
#[derive(Clone, Copy, Debug)]
pub struct MultiAtlasConfig {
    /// Maximum number of glyph atlases.
    pub max_atlases: usize,
    /// Size of each atlas in pixels (width and height).
    pub atlas_size: u32,
    /// Whether to enable LRU eviction across atlases.
    pub lru_eviction: bool,
}

impl Default for MultiAtlasConfig {
    fn default() -> Self {
        Self {
            max_atlases: 4,
            atlas_size: 4096,
            lru_eviction: true,
        }
    }
}

// =============================================================================
// P1-61: Shaping Cache Strategy
// =============================================================================

/// Configuration for the shaping cache.
#[derive(Clone, Copy, Debug)]
pub struct ShapingCacheConfig {
    /// Maximum number of cached shaping results.
    pub max_entries: usize,
    /// Whether to track hit/miss statistics.
    pub track_stats: bool,
}

impl Default for ShapingCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 4096,
            track_stats: true,
        }
    }
}

// =============================================================================
// P1-62: Vertical Text Support
// =============================================================================

/// Vertical text layout mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalTextMode {
    /// Horizontal text layout (default).
    Horizontal,
    /// Vertical text layout, right-to-left (traditional Japanese).
    VerticalRl,
    /// Vertical text layout, left-to-right (modern Chinese).
    VerticalLr,
}

impl Default for VerticalTextMode {
    fn default() -> Self {
        VerticalTextMode::Horizontal
    }
}
