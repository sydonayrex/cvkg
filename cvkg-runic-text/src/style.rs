use fontdb::{Stretch, Style, Weight};
use crate::types::{VariableAxis};

// ── Constants ──────────────────────────────────────────────────────────────

/// Default font size in pixels.
pub const DEFAULT_FONT_SIZE: f32 = 16.0;

/// Default line height multiplier.
pub const DEFAULT_LINE_HEIGHT: f32 = 1.2;

// ── TextDecorations ──────────────────────────────────────────────────────────

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

// ── LineHeight ───────────────────────────────────────────────────────────────

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

// ── TextOverflow ─────────────────────────────────────────────────────────────

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

// ── TextAlign ────────────────────────────────────────────────────────────────

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

// ── RenderMode ───────────────────────────────────────────────────────────────

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

// ── OpenTypeFeature ──────────────────────────────────────────────────────────

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

// ── TextStyle ────────────────────────────────────────────────────────────────

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
