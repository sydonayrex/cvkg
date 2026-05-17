// ── MSDF Glyph Rendering ────────────────────────────────────────────────────
//
// Multi-channel Signed Distance Field glyph atlas for resolution-independent
// text rendering. Generates SDF from rasterized bitmaps and packs them into
// a shelf-packed atlas.


/// A single glyph in the MSDF atlas.
#[derive(Debug, Clone, PartialEq)]
pub struct MsdfGlyph {
    /// Glyph ID from the font.
    pub glyph_id: u32,
    /// X position in the atlas (pixels).
    pub atlas_x: u32,
    /// Y position in the atlas (pixels).
    pub atlas_y: u32,
    /// Width of the glyph cell in the atlas (pixels).
    pub atlas_w: u32,
    /// Height of the glyph cell in the atlas (pixels).
    pub atlas_h: u32,
    /// SDF distance values, row-major, one byte per pixel.
    /// Values < 128 are inside the glyph, > 128 are outside.
    pub sdf_data: Vec<u8>,
}

impl MsdfGlyph {
    /// Creates a new MSDF glyph entry.
    pub fn new(
        glyph_id: u32,
        atlas_x: u32,
        atlas_y: u32,
        w: u32,
        h: u32,
        sdf_data: Vec<u8>,
    ) -> Self {
        Self {
            glyph_id,
            atlas_x,
            atlas_y,
            atlas_w: w,
            atlas_h: h,
            sdf_data,
        }
    }

    /// Returns true if the given UV coordinate (0..1) falls inside the glyph shape.
    pub fn contains_uv(&self, u: f32, v: f32) -> bool {
        let px = (u * self.atlas_w as f32).clamp(0.0, self.atlas_w as f32 - 1.0) as usize;
        let py = (v * self.atlas_h as f32).clamp(0.0, self.atlas_h as f32 - 1.0) as usize;
        let idx = py * self.atlas_w as usize + px;
        self.sdf_data.get(idx).is_some_and(|&d| d >= 128)
    }
}

/// A shelf-packed atlas of MSDF glyphs.
#[derive(Debug, Clone, PartialEq)]
pub struct MsdfAtlas {
    /// Atlas width in pixels.
    pub width: u32,
    /// Atlas height in pixels.
    pub height: u32,
    /// Glyphs stored in this atlas.
    pub glyphs: Vec<MsdfGlyph>,
    /// Raw SDF buffer (all glyph data concatenated).
    pub sdf_buffer: Vec<u8>,
}

impl MsdfAtlas {
    /// Creates an empty atlas with the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            glyphs: Vec::new(),
            sdf_buffer: Vec::new(),
        }
    }

    /// Inserts a glyph into the atlas. Returns true if it was added.
    pub fn insert(&mut self, glyph: MsdfGlyph) -> bool {
        self.glyphs.push(glyph);
        true
    }

    /// Looks up a glyph by ID.
    pub fn get_glyph(&self, glyph_id: u32) -> Option<&MsdfGlyph> {
        self.glyphs.iter().find(|g| g.glyph_id == glyph_id)
    }

    /// Returns the number of glyphs in the atlas.
    pub fn len(&self) -> usize {
        self.glyphs.len()
    }

    /// Returns true if the atlas has no glyphs.
    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }
}

impl Default for MsdfAtlas {
    fn default() -> Self {
        Self::new(1024, 1024)
    }
}

/// Generates a signed distance field from a 1-bit bitmap.
///
/// For each pixel in the output, computes the minimum Euclidean distance
/// to the nearest edge (transition from 0 to 1 or vice versa) within
/// `spread` pixels. The result is normalized to 0..255 where 128 means
/// exactly on the edge, < 128 is inside, > 128 is outside.
///
/// # Arguments
/// * `bitmap` - 1-bit image data (0 = background, nonzero = foreground), row-major.
/// * `width` - Bitmap width in pixels.
/// * `height` - Bitmap height in pixels.
/// * `spread` - Maximum search radius in pixels for distance computation.
///
/// # Returns
/// A `Vec<u8>` of length `width * height` with SDF values.
pub fn generate_sdf(bitmap: &[u8], width: usize, height: usize, spread: f32) -> Vec<u8> {
    if width == 0 || height == 0 {
        return Vec::new();
    }

    let spread_i = spread.ceil() as isize;
    let _spread_sq = spread * spread;
    let mut output = vec![0u8; width * height];

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let inside = bitmap[idx] != 0;
            let mut min_dist_sq = f32::INFINITY;

            // Search within spread radius for nearest edge
            let y_min = (y as isize - spread_i).max(0) as usize;
            let y_max = ((y as isize + spread_i) as usize).min(height - 1);
            let x_min = (x as isize - spread_i).max(0) as usize;
            let x_max = ((x as isize + spread_i) as usize).min(width - 1);

            for sy in y_min..=y_max {
                for sx in x_min..=x_max {
                    let s_idx = sy * width + sx;
                    let s_inside = bitmap[s_idx] != 0;
                    if s_inside != inside {
                        let dx = sx as f32 - x as f32;
                        let dy = sy as f32 - y as f32;
                        let d_sq = dx * dx + dy * dy;
                        if d_sq < min_dist_sq {
                            min_dist_sq = d_sq;
                        }
                    }
                }
            }

            let dist = min_dist_sq.sqrt();
            // Normalize: 0 distance from edge = 128, spread distance = 0 or 255
            let normalized = if inside {
                128.0 + (dist / spread) * 127.0
            } else {
                128.0 - (dist / spread) * 128.0
            };
            output[idx] = normalized.clamp(0.0, 255.0) as u8;
        }
    }

    output
}

/// Packs glyphs into an atlas using shelf packing.
///
/// Sorts glyphs by height descending, then packs them into horizontal shelves.
/// Each shelf has a fixed height equal to the tallest glyph in it.
///
/// # Arguments
/// * `glyphs` - Mutable slice of MsdfGlyph entries (atlas_x/atlas_y will be set).
/// * `max_size` - Maximum atlas dimension (width and height).
///
/// # Returns
/// `Ok((width, height))` with the final atlas dimensions, or `Err` if a glyph
/// exceeds `max_size`.
pub fn pack_atlas(glyphs: &mut [MsdfGlyph], max_size: u32) -> Result<(u32, u32), String> {
    if glyphs.is_empty() {
        return Ok((0, 0));
    }

    // Sort by height descending for better shelf utilization
    glyphs.sort_by_key(|b| std::cmp::Reverse(b.atlas_h));

    let mut current_x: u32 = 0;
    let mut current_y: u32 = 0;
    let mut shelf_height: u32 = 0;
    let mut max_x: u32 = 0;

    for glyph in glyphs.iter_mut() {
        if glyph.atlas_w > max_size || glyph.atlas_h > max_size {
            return Err(format!(
                "Glyph {} ({}x{}) exceeds max atlas size {}",
                glyph.glyph_id, glyph.atlas_w, glyph.atlas_h, max_size
            ));
        }

        // Start a new shelf if this glyph doesn't fit on the current one
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
mod msdf_tests {
    use super::*;

    #[test]
    fn test_generate_sdf_empty() {
        let result = generate_sdf(&[], 0, 0, 8.0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_generate_sdf_single_pixel() {
        // A single foreground pixel in a 3x3 grid
        let mut bitmap = vec![0u8; 9];
        bitmap[4] = 1; // center pixel
        let sdf = generate_sdf(&bitmap, 3, 3, 2.0);
        assert_eq!(sdf.len(), 9);
        // Center pixel should be inside (>= 128)
        assert!(sdf[4] >= 128, "center pixel should be inside");
        // Corners should be outside (< 128)
        assert!(sdf[0] < 128, "corner should be outside");
    }

    #[test]
    fn test_generate_sdf_solid_block() {
        // 4x4 solid block
        let bitmap = vec![1u8; 16];
        let sdf = generate_sdf(&bitmap, 4, 4, 4.0);
        assert_eq!(sdf.len(), 16);
        // All pixels are inside (no edges within the block)
        for &v in &sdf {
            assert!(v >= 128, "solid block pixels should be inside");
        }
    }

    #[test]
    fn test_pack_atlas_empty() {
        let mut glyphs: Vec<MsdfGlyph> = vec![];
        let result = pack_atlas(&mut glyphs, 1024);
        assert_eq!(result, Ok((0, 0)));
    }

    #[test]
    fn test_pack_atlas_single() {
        let sdf = vec![128u8; 16]; // 4x4
        let mut glyphs = vec![MsdfGlyph::new(1, 0, 0, 4, 4, sdf)];
        let (w, h) = pack_atlas(&mut glyphs, 1024).unwrap();
        assert_eq!(w, 4);
        assert_eq!(h, 4);
        assert_eq!(glyphs[0].atlas_x, 0);
        assert_eq!(glyphs[0].atlas_y, 0);
    }

    #[test]
    fn test_pack_atlas_wraps_shelf() {
        // Two 64x64 glyphs in a 100-wide atlas should go on separate shelves
        let sdf = vec![128u8; 64 * 64];
        let mut glyphs = vec![
            MsdfGlyph::new(1, 0, 0, 64, 64, sdf.clone()),
            MsdfGlyph::new(2, 0, 0, 64, 64, sdf),
        ];
        let (w, h) = pack_atlas(&mut glyphs, 100).unwrap();
        assert_eq!(w, 64);
        assert_eq!(h, 128); // two shelves of 64
        assert_eq!(glyphs[0].atlas_y, 0);
        assert_eq!(glyphs[1].atlas_y, 64);
    }

    #[test]
    fn test_pack_atlas_oversized() {
        let sdf = vec![128u8; 100];
        let mut glyphs = vec![MsdfGlyph::new(1, 0, 0, 200, 200, sdf)];
        let result = pack_atlas(&mut glyphs, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_msdf_glyph_contains_uv() {
        // 2x2 glyph: top-left and bottom-right are inside
        let sdf = vec![200u8, 50, 50, 200];
        let glyph = MsdfGlyph::new(1, 0, 0, 2, 2, sdf);
        assert!(glyph.contains_uv(0.25, 0.25)); // top-left
        assert!(!glyph.contains_uv(0.75, 0.25)); // top-right
        assert!(!glyph.contains_uv(0.25, 0.75)); // bottom-left
        assert!(glyph.contains_uv(0.75, 0.75)); // bottom-right
    }

    #[test]
    fn test_msdf_atlas_lookup() {
        let mut atlas = MsdfAtlas::new(512, 512);
        assert!(atlas.is_empty());
        atlas.insert(MsdfGlyph::new(42, 0, 0, 16, 16, vec![128u8; 256]));
        assert_eq!(atlas.len(), 1);
        assert!(atlas.get_glyph(42).is_some());
        assert!(atlas.get_glyph(99).is_none());
    }
}
