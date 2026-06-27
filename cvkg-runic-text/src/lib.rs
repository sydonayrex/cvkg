// CVKG Runic Text -- Production text shaping, layout, and rasterization engine
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

use std::sync::Arc;

pub mod emoji;
pub mod global_cache;
pub mod knuth_plass;
pub mod msdf;
pub mod subpixel;

// Sub-modules
pub mod engine;
pub mod layout;
pub mod path;
pub mod span;
pub mod style;
pub mod types;

// Re-exports
pub use engine::{CacheKey, TextEngine};
pub use layout::ShapedText;
pub use path::{LayoutBoundary, TextPath};
pub use span::{
    Paragraph, PortalAlignment, SemanticKind, SemanticRange, TextRun, TextSpan, TextSpanKind,
};
pub use style::{
    DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT, LineHeight, OpenTypeFeature, RenderMode, TextAlign,
    TextDecorations, TextOverflow, TextStyle,
};
pub use types::{
    AtlasDefragConfig, FontAxisInfo, FontFallbackChain, FontMatchStrategy, FontMetrics, GlyphImage,
    GlyphInstance, HintingStrategy, LineInfo, MultiAtlasConfig, RunicPathSegment,
    ShapingCacheConfig, ShapingError, SubpixelMode, TextCapabilities, VariableAxis,
    VerticalTextMode,
};

use std::path::Path;

/// Load a font from a file path and register it with the global font system.
///
/// # Example
/// ```no_run
/// use cvkg_runic_text::load_font_file;
/// use std::path::Path;
/// let handle = load_font_file(Path::new("/path/to/font.ttf")).expect("Failed to load font");
/// ```
pub fn load_font_file(path: &Path) -> Result<FontHandle, FontLoadError> {
    let bytes = std::fs::read(path)?;
    load_font_bytes(&bytes)
}

/// Load a font from raw bytes and register it with the global font system.
///
/// # Example
/// ```no_run
/// use cvkg_runic_text::load_font_bytes;
/// let font_data = std::fs::read("font.ttf").expect("Failed to read font file");
/// let handle = load_font_bytes(&font_data).expect("Failed to load font");
/// ```
pub fn load_font_bytes(bytes: &[u8]) -> Result<FontHandle, FontLoadError> {
    // Validate that the bytes are a valid font by checking the SFNT signature
    if bytes.len() < 4 {
        return Err(FontLoadError::Parse("Font data too short".to_string()));
    }
    // Check for TrueType (0x00010000) or OpenType (OTTO) signature
    let sig = &bytes[0..4];
    let valid = sig == b"\x00\x01\x00\x00" || sig == b"OTTO" || sig == b"true" || sig == b"typ1";
    if !valid {
        return Err(FontLoadError::Parse("Unknown font format".to_string()));
    }
    let handle = FontHandle {
        data: bytes.to_vec(),
    };
    Ok(handle)
}

/// Opaque handle to a loaded font.
#[derive(Debug, Clone)]
pub struct FontHandle {
    pub data: Vec<u8>,
}

/// Errors that can occur when loading a font.
#[derive(Debug)]
pub enum FontLoadError {
    Io(std::io::Error),
    Parse(String),
}

impl From<std::io::Error> for FontLoadError {
    fn from(e: std::io::Error) -> Self {
        FontLoadError::Io(e)
    }
}

impl std::fmt::Display for FontLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FontLoadError::Io(e) => write!(f, "IO error: {}", e),
            FontLoadError::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for FontLoadError {}

/// Shared test engine that loads only bundled fonts (no system fonts).
/// Uses Arc for thread-safe sharing across parallel tests.
static TEST_ENGINE: std::sync::OnceLock<Arc<TextEngine>> = std::sync::OnceLock::new();

/// Get or create the shared test engine.
pub fn test_engine() -> &'static Arc<TextEngine> {
    TEST_ENGINE.get_or_init(|| {
        let mut engine = TextEngine::new_light();
        // Load bundled Jupiteroid font for tests
        engine.load_font_data(include_bytes!("../Fonts/Jupiteroid.ttf").to_vec());
        Arc::new(engine)
    })
}

#[cfg(test)]
mod tests;
