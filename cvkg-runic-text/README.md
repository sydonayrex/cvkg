# cvkg-runic-text

**cvkg-runic-text** is the natively integrated Cyber Viking text shaping and layout engine for CVKG. It provides a stateless, high-performance typography pipeline that replaces generic text libraries with a solution optimized for the CVKG execution model.

## Features

*   **Stateless Shaping**: Uses `rustybuzz` for high-fidelity text shaping.
*   **Native Layout**: Implements word wrapping and line breaking using Unicode standards (`unicode-segmentation`, `unicode-linebreak`).
*   **Global Font Fallback**: Automatically resolves missing glyphs (emojis, foreign characters) by scanning system fonts and splicing font runs.
*   **Bidirectional Support (BiDi)**: Correctly handles mixed RTL (Arabic/Hebrew) and LTR text directionality.
*   **Interactive Metrics**: Provides `hit_test` (Position-to-Index) and `cursor_position` (Index-to-Position) for building text editors and selectable labels.
*   **Performance Optimized**: Includes an LRU cache for shaped text and supports subpixel positioning for maximum clarity.

## Core API

### `RunicTextEngine`
The main engine instance that manages `fontdb` and the shaping cache.
*   `new()`: Initializes the engine with system fonts.
*   `shape(text, font, size)`: Simple single-line shaping.
*   `shape_layout(spans, max_width)`: Advanced multi-span layout with wrapping.
*   `rasterize(cache_key)`: Generates a bitmap for a specific glyph.

### `ShapedText`
The result of a layout operation.
*   `hit_test(x, y)`: Maps visual coordinates to string byte offsets.
*   `cursor_position(index)`: Maps string offsets to visual coordinates.
