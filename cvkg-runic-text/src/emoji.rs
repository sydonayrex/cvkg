// ── Color Emoji Atlas ───────────────────────────────────────────────────────
///
/// Provides emoji glyph detection and atlas packing for color emoji rendering.
/// Uses Unicode emoji range detection and shelf-packing for atlas management.
/// A single color emoji glyph.
#[derive(Debug, Clone, PartialEq)]
pub struct EmojiGlyph {
    /// The Unicode codepoint for this emoji.
    pub codepoint: u32,
    /// X position in the atlas (pixels).
    pub atlas_x: u32,
    /// Y position in the atlas (pixels).
    pub atlas_y: u32,
    /// Width of the glyph cell in the atlas.
    pub atlas_w: u32,
    /// Height of the glyph cell in the atlas.
    pub atlas_h: u32,
    /// RGBA bitmap data for the emoji, row-major, 4 bytes per pixel.
    pub bitmap: Vec<u8>,
}

impl EmojiGlyph {
    /// Creates a new emoji glyph entry.
    pub fn new(
        codepoint: u32,
        atlas_x: u32,
        atlas_y: u32,
        w: u32,
        h: u32,
        bitmap: Vec<u8>,
    ) -> Self {
        Self {
            codepoint,
            atlas_x,
            atlas_y,
            atlas_w: w,
            atlas_h: h,
            bitmap,
        }
    }

    /// Returns the expected bitmap size in bytes for the given dimensions.
    pub fn expected_bitmap_size(w: u32, h: u32) -> usize {
        w as usize * h as usize * 4
    }

    /// Returns true if the bitmap has the correct size.
    pub fn bitmap_valid(&self) -> bool {
        self.bitmap.len() == Self::expected_bitmap_size(self.atlas_w, self.atlas_h)
    }
}

/// A shelf-packed atlas of color emoji glyphs.
#[derive(Debug, Clone, PartialEq)]
pub struct EmojiAtlas {
    /// Atlas width in pixels.
    pub width: u32,
    /// Atlas height in pixels.
    pub height: u32,
    /// Emoji glyphs in this atlas.
    pub glyphs: Vec<EmojiGlyph>,
}

impl EmojiAtlas {
    /// Creates an empty emoji atlas with the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            glyphs: Vec::new(),
        }
    }

    /// Inserts an emoji glyph into the atlas.
    pub fn insert(&mut self, glyph: EmojiGlyph) {
        self.glyphs.push(glyph);
    }

    /// Looks up an emoji glyph by codepoint.
    pub fn get_glyph(&self, codepoint: u32) -> Option<&EmojiGlyph> {
        self.glyphs.iter().find(|g| g.codepoint == codepoint)
    }

    /// Returns the number of glyphs.
    pub fn len(&self) -> usize {
        self.glyphs.len()
    }

    /// Returns true if the atlas is empty.
    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }

    /// Packs glyphs into this atlas using shelf packing.
    /// Sets atlas_x and atlas_y on each glyph.
    pub fn pack(&mut self) -> Result<(), String> {
        if self.glyphs.is_empty() {
            return Ok(());
        }

        // Sort by height descending
        self.glyphs.sort_by_key(|b| std::cmp::Reverse(b.atlas_h));

        let mut current_x: u32 = 0;
        let mut current_y: u32 = 0;
        let mut shelf_height: u32 = 0;

        for glyph in self.glyphs.iter_mut() {
            if glyph.atlas_w > self.width || glyph.atlas_h > self.height {
                return Err(format!(
                    "Emoji U+{:04X} ({}x{}) exceeds atlas size {}x{}",
                    glyph.codepoint, glyph.atlas_w, glyph.atlas_h, self.width, self.height
                ));
            }

            if current_x + glyph.atlas_w > self.width {
                current_y += shelf_height;
                current_x = 0;
                shelf_height = 0;
            }

            if current_y + glyph.atlas_h > self.height {
                return Err(format!(
                    "Atlas {}x{} full, cannot pack emoji U+{:04X}",
                    self.width, self.height, glyph.codepoint
                ));
            }

            glyph.atlas_x = current_x;
            glyph.atlas_y = current_y;
            current_x += glyph.atlas_w;
            shelf_height = shelf_height.max(glyph.atlas_h);
        }

        Ok(())
    }
}

impl Default for EmojiAtlas {
    fn default() -> Self {
        Self::new(2048, 2048)
    }
}

/// Returns true if the given Unicode codepoint is in an emoji range.
///
/// Covers the major emoji blocks:
/// - U+1F600..U+1F64F: Emoticons
/// - U+1F300..U+1F5FF: Misc Symbols and Pictographs
/// - U+1F680..U+1F6FF: Transport and Map
/// - U+1F1E0..U+1F1FF: Regional Indicators (flags)
/// - U+2600..U+26FF: Misc Symbols
/// - U+2700..U+27BF: Dingbats
/// - U+1F900..U+1F9FF: Supplemental Symbols
/// - U+1FA00..U+1FA6F: Chess Symbols
/// - U+1FA70..U+1FAFF: Symbols and Pictographs Extended-A
/// - U+FE0F: Variation Selector-16 (emoji presentation)
/// - U+200D: Zero Width Joiner (emoji sequences)
pub fn is_emoji(codepoint: u32) -> bool {
    matches!(codepoint,
        0x1F600..=0x1F64F | // Emoticons
        0x1F300..=0x1F5FF | // Misc Symbols and Pictographs
        0x1F680..=0x1F6FF | // Transport and Map
        0x1F1E0..=0x1F1FF | // Regional Indicators
        0x2600..=0x26FF |   // Misc Symbols
        0x2700..=0x27BF |   // Dingbats
        0x1F900..=0x1F9FF | // Supplemental Symbols
        0x1FA00..=0x1FA6F | // Chess Symbols
        0x1FA70..=0x1FAFF | // Extended-A
        0xFE0F |            // Variation Selector-16
        0x200D              // ZWJ
    )
}

/// Returns true if the codepoint is a base emoji (not a modifier or joiner).
pub fn is_emoji_base(codepoint: u32) -> bool {
    matches!(codepoint,
        0x1F600..=0x1F64F |
        0x1F300..=0x1F5FF |
        0x1F680..=0x1F6FF |
        0x1F1E0..=0x1F1FF |
        0x2600..=0x26FF |
        0x2700..=0x27BF |
        0x1F900..=0x1F9FF |
        0x1FA00..=0x1FA6F |
        0x1FA70..=0x1FAFF
    )
}

/// Renders an emoji codepoint to an RGBA bitmap.
///
/// This is a placeholder that generates a colored square with the emoji
/// codepoint rendered as text. For production use, this would load a color
/// emoji font (like Noto Color Emoji) and rasterize the glyph.
///
/// # Arguments
/// * `codepoint` - Unicode codepoint of the emoji.
/// * `size` - Output bitmap size (width = height = size).
///
/// # Returns
/// `Some(Vec<u8>)` with RGBA data, or `None` if the codepoint is not a valid emoji.
pub fn render_emoji(codepoint: u32, size: u32) -> Option<Vec<u8>> {
    if !is_emoji_base(codepoint) {
        return None;
    }

    let total_pixels = (size * size) as usize;
    let mut rgba = vec![0u8; total_pixels * 4];

    // Generate a deterministic color from the codepoint
    let r = ((codepoint >> 16) & 0xFF) as u8;
    let g = ((codepoint >> 8) & 0xFF) as u8;
    let b = (codepoint & 0xFF) as u8;

    // Fill the bitmap with a colored circle on a transparent background
    let center = size as f32 / 2.0;
    let radius = size as f32 * 0.4;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let dist_sq = dx * dx + dy * dy;
            let idx = (y * size + x) as usize * 4;

            if dist_sq <= radius * radius {
                // Inside the circle
                let dist = dist_sq.sqrt();
                let alpha = if dist > radius - 2.0 {
                    // Anti-aliased edge
                    ((radius - dist) / 2.0 * 255.0) as u8
                } else {
                    255
                };
                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
                rgba[idx + 3] = alpha;
            }
        }
    }

    Some(rgba)
}

/// Detects emoji runs in text and returns their byte ranges.
///
/// Returns a Vec of (start_byte, end_byte, codepoint) for each emoji found.
pub fn find_emoji_runs(text: &str) -> Vec<(usize, usize, u32)> {
    let mut runs = Vec::new();
    let mut current_start: Option<usize> = None;
    let mut current_codepoint: u32 = 0;

    for (byte_idx, ch) in text.char_indices() {
        let cp = ch as u32;
        if is_emoji_base(cp) {
            if current_start.is_none() {
                current_start = Some(byte_idx);
                current_codepoint = cp;
            }
        } else if is_emoji(cp) {
            // Modifier or joiner, continue the current run
        } else {
            // Not an emoji, close any open run
            if let Some(start) = current_start {
                runs.push((start, byte_idx, current_codepoint));
                current_start = None;
            }
        }
    }

    // Close any remaining run
    if let Some(start) = current_start {
        runs.push((start, text.len(), current_codepoint));
    }

    runs
}

/// Packs a set of emoji glyphs into an atlas.
///
/// # Arguments
/// * `glyphs` - Mutable slice of EmojiGlyph entries.
/// * `max_size` - Maximum atlas dimension.
///
/// # Returns
/// `Ok((width, height))` with final atlas dimensions.
pub fn pack_emoji_atlas(glyphs: &mut [EmojiGlyph], max_size: u32) -> Result<(u32, u32), String> {
    if glyphs.is_empty() {
        return Ok((0, 0));
    }

    // Sort by height descending
    glyphs.sort_by_key(|b| std::cmp::Reverse(b.atlas_h));

    let mut current_x: u32 = 0;
    let mut current_y: u32 = 0;
    let mut shelf_height: u32 = 0;
    let mut max_x: u32 = 0;

    for glyph in glyphs.iter_mut() {
        if glyph.atlas_w > max_size || glyph.atlas_h > max_size {
            return Err(format!(
                "Emoji U+{:04X} ({}x{}) exceeds max atlas size {}",
                glyph.codepoint, glyph.atlas_w, glyph.atlas_h, max_size
            ));
        }

        if current_x + glyph.atlas_w > max_size {
            current_y += shelf_height;
            current_x = 0;
            shelf_height = 0;
        }

        glyph.atlas_x = current_x;
        glyph.atlas_y = current_y;
        current_x += glyph.atlas_w;
        shelf_height = shelf_height.max(glyph.atlas_h);
        max_x = max_x.max(current_x);
    }

    let total_height = current_y + shelf_height;
    Ok((max_x, total_height))
}

#[cfg(test)]
mod emoji_tests {
    use super::*;

    #[test]
    fn test_is_emoji_emoticons() {
        assert!(is_emoji(0x1F600)); // 😀
        assert!(is_emoji(0x1F64F)); // 🙏
        assert!(!is_emoji(0x0041)); // 'A'
    }

    #[test]
    fn test_is_emoji_misc_symbols() {
        assert!(is_emoji(0x1F300)); // 🌀
        assert!(is_emoji(0x1F5FF)); // 🗿
    }

    #[test]
    fn test_is_emoji_transport() {
        assert!(is_emoji(0x1F680)); // 🚀
        assert!(is_emoji(0x1F6FF)); // 🛿
    }

    #[test]
    fn test_is_emoji_flags() {
        assert!(is_emoji(0x1F1FA)); // Regional indicator U
        assert!(is_emoji(0x1F1F8)); // Regional indicator S
    }

    #[test]
    fn test_is_emoji_dingbats() {
        assert!(is_emoji(0x2764)); // ❤
        assert!(is_emoji(0x2728)); // ✨
    }

    #[test]
    fn test_is_emoji_variation_selector() {
        assert!(is_emoji(0xFE0F));
    }

    #[test]
    fn test_is_emoji_zwj() {
        assert!(is_emoji(0x200D));
    }

    #[test]
    fn test_is_emoji_base() {
        assert!(is_emoji_base(0x1F600));
        assert!(!is_emoji_base(0xFE0F));
        assert!(!is_emoji_base(0x200D));
    }

    #[test]
    fn test_render_emoji_smiley() {
        let bitmap = render_emoji(0x1F600, 64);
        assert!(bitmap.is_some());
        let data = bitmap.unwrap();
        assert_eq!(data.len(), 64 * 64 * 4);
    }

    #[test]
    fn test_render_emoji_non_emoji() {
        assert!(render_emoji(0x0041, 64).is_none()); // 'A' is not an emoji
    }

    #[test]
    fn test_render_emoji_vs_is_none() {
        assert!(render_emoji(0xFE0F, 64).is_none()); // variation selector is not base
    }

    #[test]
    fn test_emoji_glyph_valid() {
        let bitmap = vec![0u8; 32 * 32 * 4];
        let glyph = EmojiGlyph::new(0x1F600, 0, 0, 32, 32, bitmap);
        assert!(glyph.bitmap_valid());
    }

    #[test]
    fn test_emoji_glyph_invalid() {
        let bitmap = vec![0u8; 100]; // Wrong size
        let glyph = EmojiGlyph::new(0x1F600, 0, 0, 32, 32, bitmap);
        assert!(!glyph.bitmap_valid());
    }

    #[test]
    fn test_emoji_atlas_insert() {
        let mut atlas = EmojiAtlas::new(512, 512);
        assert!(atlas.is_empty());
        atlas.insert(EmojiGlyph::new(
            0x1F600,
            0,
            0,
            64,
            64,
            vec![0u8; 64 * 64 * 4],
        ));
        assert_eq!(atlas.len(), 1);
        assert!(atlas.get_glyph(0x1F600).is_some());
        assert!(atlas.get_glyph(0x1F601).is_none());
    }

    #[test]
    fn test_find_emoji_runs() {
        let text = "hello 😀 world";
        let runs = find_emoji_runs(text);
        assert_eq!(runs.len(), 1);
        let (start, end, cp) = runs[0];
        assert_eq!(cp, 0x1F600);
        assert!(start > 0);
        assert_eq!(end - start, "😀".len());
    }

    #[test]
    fn test_find_multiple_emoji_runs() {
        let text = "😀 hello 😁 world 😂";
        let runs = find_emoji_runs(text);
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[0].2, 0x1F600);
        assert_eq!(runs[1].2, 0x1F601);
        assert_eq!(runs[2].2, 0x1F602);
    }

    #[test]
    fn test_find_no_emoji() {
        let text = "hello world";
        let runs = find_emoji_runs(text);
        assert!(runs.is_empty());
    }

    #[test]
    fn test_pack_emoji_atlas_empty() {
        let mut glyphs: Vec<EmojiGlyph> = vec![];
        let result = pack_emoji_atlas(&mut glyphs, 1024);
        assert_eq!(result, Ok((0, 0)));
    }

    #[test]
    fn test_pack_emoji_atlas_single() {
        let bitmap = vec![0u8; 64 * 64 * 4];
        let mut glyphs = vec![EmojiGlyph::new(0x1F600, 0, 0, 64, 64, bitmap)];
        let (w, h) = pack_emoji_atlas(&mut glyphs, 1024).unwrap();
        assert_eq!(w, 64);
        assert_eq!(h, 64);
        assert_eq!(glyphs[0].atlas_x, 0);
        assert_eq!(glyphs[0].atlas_y, 0);
    }

    #[test]
    fn test_atlas_pack_method() {
        let bitmap = vec![0u8; 64 * 64 * 4];
        let mut atlas = EmojiAtlas::new(128, 128);
        atlas.insert(EmojiGlyph::new(0x1F600, 0, 0, 64, 64, bitmap.clone()));
        atlas.insert(EmojiGlyph::new(0x1F601, 0, 0, 64, 64, bitmap));
        assert!(atlas.pack().is_ok());
        assert_eq!(atlas.get_glyph(0x1F600).unwrap().atlas_x, 0);
        assert_eq!(atlas.get_glyph(0x1F601).unwrap().atlas_x, 64);
    }

    #[test]
    fn test_atlas_pack_overflow() {
        let bitmap = vec![0u8; 64 * 64 * 4];
        let mut atlas = EmojiAtlas::new(64, 64);
        // Two 64x64 glyphs can't fit in 64x64 after the first (shelf is full height)
        atlas.insert(EmojiGlyph::new(0x1F600, 0, 0, 64, 64, bitmap.clone()));
        atlas.insert(EmojiGlyph::new(0x1F601, 0, 0, 64, 64, bitmap));
        // Actually they fit on separate shelves: shelf 1 at y=0, shelf 2 at y=64
        // But atlas height is only 64, so shelf 2 overflows
        assert!(atlas.pack().is_err());
    }
}
