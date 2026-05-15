# cvkg-runic-text

![CVKG Hero HUD](../docs/images/cvkg_hero.png)

`cvkg-runic-text` is the authoritative text shaping and layout engine for CVKG, providing high-performance, stateless processing of multi-lingual and rich text.

## Boundaries and Responsibilities

This crate manages the transition from strings to glyphs. It does NOT handle GPU rendering (delegated to `cvkg-render-gpu`). Its responsibilities include:
- Shaping text using `rustybuzz` (HarfBuzz) for complex scripts and ligatures.
- Managing font discovery and fallback using `fontdb`.
- Implementing bidirectional (BiDi) text layout for LTR and RTL support.
- Performing word wrapping and multi-line layout based on viewport constraints.
- Rasterizing glyphs into bitmaps for the GPU atlas via `swash`.
- Providing high-accuracy hit-testing for cursor placement and text selection.

## Public API Overview

### Core Engine
- `RunicTextEngine`: The central manager for font resources, shaping caches, and rasterization.
- `TextSpan`: Represents a styled segment of text within a larger document.

### Layout Types
- `ShapedText`: The result of a layout pass, containing positioned glyph instances and document bounds.
- `GlyphInstance`: A single shaped glyph with its absolute position and logical cluster mapping.
- `CacheKey`: Uniquely identifies a glyph/size/font combination for atlas caching.

### Key Methods
- `RunicTextEngine::shape_layout()`: The primary entry point for multi-line, multi-style text layout.
- `ShapedText::hit_test(x, y)`: Maps a visual coordinate to a logical text index.
- `ShapedText::cursor_position(index)`: Maps a logical index to a visual coordinate.

## Known Limitations
- Font fallback relies on system-available fonts; embed critical fonts as assets for cross-platform consistency.
- Large-scale text layout (e.g., whole books) should be chunked to avoid shaping cache exhaustion.
- Subpixel positioning is supported, but glyph hinting is disabled by default to maintain rhythmic consistency.