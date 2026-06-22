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

pub mod global_cache;
pub mod emoji;
pub mod knuth_plass;
pub mod msdf;
pub mod subpixel;

// Sub-modules
pub mod types;
pub mod style;
pub mod path;
pub mod span;
pub mod layout;
pub mod engine;

// Re-exports
pub use types::{
    ShapingError, FontAxisInfo, VariableAxis, GlyphInstance, RunicPathSegment,
    GlyphImage, LineInfo, FontMetrics, TextCapabilities, FontFallbackChain,
    FontMatchStrategy, SubpixelMode, HintingStrategy, AtlasDefragConfig,
    MultiAtlasConfig, ShapingCacheConfig, VerticalTextMode,
};
pub use style::{
    DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT, TextDecorations, LineHeight,
    TextOverflow, TextAlign, RenderMode, OpenTypeFeature, TextStyle,
};
pub use path::{TextPath, LayoutBoundary};
pub use span::{
    PortalAlignment, TextSpanKind, TextSpan, TextRun, SemanticKind,
    SemanticRange, Paragraph,
};
pub use layout::ShapedText;
pub use engine::{TextEngine, CacheKey};

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
