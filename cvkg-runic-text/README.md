# cvkg-runic-text

**cvkg-runic-text** provides text shaping, layout, and font fallback for CVKG applications.

## What This Crate Does

- Provides text shaping via rustybuzz (HarfBuzz subset)
- Provides font loading and glyph cache management
- Supports Global Font Fallback for missing glyphs
- Provides BiDi (bidirectional text) support

## What This Crate Does NOT Do

- Does not provide rendering (see cvkg-render-gpu)
- Does not provide layout calculations (see cvkg-layout)
- Does not provide font file discovery

## Public API Overview

### Shaper

```rust
/// Text shaper for converting Unicode text to positioned glyphs
pub struct Shaper {
    // private fields
}
impl Shaper {
    /// Create a new shaper with default configuration
    pub fn new() -> Self;
    
    /// Shape text using the given font and size
    pub fn shape(&mut self, text: &str, font_id: u32, size: f32) -> Vec<GlyphInfo>;
    
    /// Register a font from data
    pub fn register_font(&mut self, font_data: &[u8]) -> Result<u32, ShaperError>;
}
```

### GlyphInfo

```rust
/// Positioned glyph from text shaping
pub struct GlyphInfo {
    pub glyph_id: u32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub x_advance: f32,
    pub y_advance: f32,
}```

### Error Types

```rust
pub enum ShaperError {
    InvalidFontData,
    FontNotFound(u32),
    OutOfMemory,
}```

## Usage Example

```rust
use cvkg_runic_text::{Shaper, GlyphInfo};

let mut shaper = Shaper::new();
let font_id = shaper.register_font(include_bytes!("font.ttf")).unwrap();
glyphs = shaper.shape("Hello, World!", font_id, 16.0);
```

## Known Limitations

- Font loading requires manual registration; no system font discovery
- BiDi support is basic; complex scripts may have issues
- Glyph cache is in-memory only; no persistence