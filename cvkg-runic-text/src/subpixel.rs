// ── Subpixel LCD Positioning ────────────────────────────────────────────────
///
/// Provides subpixel-aware glyph positioning for LCD screens. Splits each
/// pixel into R, G, B subpixels and computes independent coverage values
/// for sharper text rendering on standard-RGB LCD panels.
/// Subpixel layout order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SubpixelOrder {
    /// RGB horizontal stripe (most common).
    #[default]
    Rgb,
    /// BGR horizontal stripe (some laptops).
    Bgr,
    /// RGB vertical stripe (rare, some monitors).
    RgbVertical,
    /// No subpixel rendering (grayscale).
    None,
}

/// A glyph positioned at subpixel resolution.
#[derive(Debug, Clone, PartialEq)]
pub struct SubpixelGlyph {
    /// Glyph ID from the font.
    pub glyph_id: u32,
    /// X offset in sub-pixel units (1/3 pixel resolution).
    /// E.g., 1 = 1/3 pixel, 2 = 2/3 pixel, 3 = 1 full pixel.
    pub x_subpixel: i32,
    /// Y offset in whole pixels.
    pub y_pixel: i32,
    /// Coverage for the R subpixel column (0..255).
    pub coverage_r: u8,
    /// Coverage for the G subpixel column (0..255).
    pub coverage_g: u8,
    /// Coverage for the B subpixel column (0..255).
    pub coverage_b: u8,
    /// Glyph width in whole pixels.
    pub width: u32,
    /// Glyph height in whole pixels.
    pub height: u32,
}

impl SubpixelGlyph {
    /// Creates a new subpixel-positioned glyph.
    pub fn new(
        glyph_id: u32,
        x_subpixel: i32,
        y_pixel: i32,
        coverage_r: u8,
        coverage_g: u8,
        coverage_b: u32,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            glyph_id,
            x_subpixel,
            y_pixel,
            coverage_r,
            coverage_g,
            coverage_b: coverage_b as u8,
            width,
            height,
        }
    }

    /// Returns the whole-pixel x position (x_subpixel / 3).
    pub fn x_pixel(&self) -> i32 {
        self.x_subpixel / 3
    }

    /// Returns the fractional part (0, 1, or 2).
    pub fn x_fraction(&self) -> i32 {
        self.x_subpixel % 3
    }

    /// Returns true if the glyph is at a whole-pixel boundary.
    pub fn is_pixel_aligned(&self) -> bool {
        self.x_subpixel % 3 == 0
    }
}

/// Computes subpixel coverage for a glyph given a fractional x offset.
///
/// Uses a simple 3-tap box filter to distribute coverage across R, G, B
/// subpixel columns based on the fractional position.
///
/// # Arguments
/// * `fraction` - Fractional position (0 = aligned, 1 = 1/3 shift, 2 = 2/3 shift).
/// * `order` - Subpixel layout order.
///
/// # Returns
/// A `(r, g, b)` tuple with coverage values 0..255.
pub fn subpixel_coverage(fraction: i32, order: SubpixelOrder) -> (u8, u8, u8) {
    let f = fraction.rem_euclid(3);

    // Box filter weights for each subpixel at each fractional position
    // Position 0: [1.0, 0.0, 0.0] -> full R
    // Position 1: [0.33, 0.67, 0.0] -> blend R->G
    // Position 2: [0.0, 0.33, 0.67] -> blend G->B
    let weights: [f32; 3] = match f {
        0 => [1.0, 0.0, 0.0],
        1 => [0.33, 0.67, 0.0],
        2 => [0.0, 0.33, 0.67],
        _ => [0.33, 0.34, 0.33],
    };

    let to_u8 = |w: f32| (w * 255.0).round() as u8;

    match order {
        SubpixelOrder::Rgb => (to_u8(weights[0]), to_u8(weights[1]), to_u8(weights[2])),
        SubpixelOrder::Bgr => (to_u8(weights[2]), to_u8(weights[1]), to_u8(weights[0])),
        SubpixelOrder::RgbVertical | SubpixelOrder::None => {
            // Vertical or no subpixel: equal coverage
            let avg = to_u8((weights[0] + weights[1] + weights[2]) / 3.0);
            (avg, avg, avg)
        }
    }
}

/// Lays out glyphs with subpixel positioning.
///
/// Takes a sequence of glyph advances (in pixels, possibly fractional) and
/// computes subpixel-positioned glyphs. Each glyph's x position is snapped
/// to 1/3-pixel boundaries for LCD subpixel rendering.
///
/// # Arguments
/// * `glyph_ids` - Slice of glyph IDs.
/// * `advances` - Slice of advance widths in pixels (one per glyph).
/// * `dpi_scale` - DPI scaling factor (1.0 = 96 DPI).
/// * `order` - Subpixel layout order.
///
/// # Returns
/// A `Vec<SubpixelGlyph>` with computed subpixel positions.
pub fn layout_subpixel(
    glyph_ids: &[u32],
    advances: &[f32],
    dpi_scale: f32,
    order: SubpixelOrder,
) -> Vec<SubpixelGlyph> {
    if glyph_ids.is_empty() || advances.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    // Track position in 1/3-pixel units
    let mut x_accum: i32 = 0;

    for (i, (&glyph_id, &advance)) in glyph_ids.iter().zip(advances.iter()).enumerate() {
        // Convert advance to 1/3-pixel units
        let advance_subpx = (advance * dpi_scale * 3.0).round() as i32;
        let x_subpixel = x_accum;
        let fraction = x_subpixel.rem_euclid(3);
        let (r, g, b) = subpixel_coverage(fraction, order);

        // Estimate glyph dimensions from advance (rough approximation)
        let width = (advance * dpi_scale).max(1.0).ceil() as u32;
        let height = (dpi_scale * 16.0).max(1.0).ceil() as u32; // assume 16px em

        result.push(SubpixelGlyph {
            glyph_id,
            x_subpixel,
            y_pixel: 0,
            coverage_r: r,
            coverage_g: g,
            coverage_b: b,
            width,
            height,
        });

        x_accum += advance_subpx;

        // Safety: prevent infinite loops on zero advances
        if advance <= 0.0 && i < glyph_ids.len() - 1 {
            x_accum += 3; // minimum 1 pixel advance
        }
    }

    result
}

/// Renders subpixel glyphs into an RGBA framebuffer.
///
/// Each glyph is rendered as a colored rectangle weighted by its subpixel
/// coverage values. This is a simplified renderer -- a real implementation
/// would use the glyph's actual alpha mask.
///
/// # Arguments
/// * `framebuffer` - Mutable RGBA buffer (width * height * 4 bytes).
/// * `fb_width` - Framebuffer width in pixels.
/// * `fb_height` - Framebuffer height in pixels.
/// * `glyphs` - Slice of subpixel-positioned glyphs.
/// * `text_color` - Base text color as (r, g, b, a).
pub fn render_lcd(
    framebuffer: &mut [u8],
    fb_width: u32,
    fb_height: u32,
    glyphs: &[SubpixelGlyph],
    text_color: (u8, u8, u8, u8),
) {
    if framebuffer.is_empty() || fb_width == 0 || fb_height == 0 {
        return;
    }

    let (tr, tg, tb, ta) = text_color;

    for glyph in glyphs {
        let x_start = glyph.x_pixel().max(0) as u32;
        let y_start = glyph.y_pixel.max(0) as u32;
        let x_end = (glyph.x_pixel() + glyph.width as i32).min(fb_width as i32) as u32;
        let y_end = (glyph.y_pixel + glyph.height as i32).min(fb_height as i32) as u32;

        for y in y_start..y_end {
            for x in x_start..x_end {
                let idx = (y * fb_width + x) as usize * 4;
                if idx + 3 >= framebuffer.len() {
                    continue;
                }

                // Compute subpixel coverage based on x position within the glyph
                let local_x = (x - x_start) as f32;
                let subpixel_phase = (local_x * 3.0) as i32 % 3;
                let (sr, sg, sb) = subpixel_coverage(subpixel_phase, SubpixelOrder::Rgb);

                // Blend subpixel coverage with text color
                let blend = |text: u8, sub: u8| -> u8 {
                    let coverage = (sub as f32 / 255.0) * (ta as f32 / 255.0);
                    let val = text as f32 * coverage;
                    val.min(255.0) as u8
                };

                framebuffer[idx] = framebuffer[idx].saturating_add(blend(tr, sr));
                framebuffer[idx + 1] = framebuffer[idx + 1].saturating_add(blend(tg, sg));
                framebuffer[idx + 2] = framebuffer[idx + 2].saturating_add(blend(tb, sb));
                // Alpha channel stays as-is (pre-multiplied)
            }
        }
    }
}

/// Estimates the visual sharpness improvement from subpixel rendering.
///
/// Returns a ratio (1.0 = no improvement, ~3.0 = full 3x horizontal resolution).
pub fn subpixel_sharpness_factor(order: SubpixelOrder) -> f32 {
    match order {
        SubpixelOrder::Rgb | SubpixelOrder::Bgr => 3.0,
        SubpixelOrder::RgbVertical => 1.0, // No horizontal improvement
        SubpixelOrder::None => 1.0,
    }
}

#[cfg(test)]
mod subpixel_tests {
    use super::*;

    #[test]
    fn test_subpixel_coverage_aligned() {
        let (r, g, b) = subpixel_coverage(0, SubpixelOrder::Rgb);
        assert!(r > 200, "Aligned should have full R coverage");
        assert!(g < 50, "Aligned should have minimal G coverage");
        assert!(b < 50, "Aligned should have minimal B coverage");
    }

    #[test]
    fn test_subpixel_coverage_shifted() {
        let (r, g, b) = subpixel_coverage(1, SubpixelOrder::Rgb);
        assert!(r > 0 && r < 200, "1/3 shift should blend R");
        assert!(g > 50, "1/3 shift should have significant G");
        assert!(b < 50, "1/3 shift should have minimal B");
    }

    #[test]
    fn test_subpixel_coverage_bgr() {
        let rgb = subpixel_coverage(0, SubpixelOrder::Rgb);
        let bgr = subpixel_coverage(0, SubpixelOrder::Bgr);
        assert_eq!(rgb.0, bgr.2, "BGR should swap R and B");
        assert_eq!(rgb.2, bgr.0);
        assert_eq!(rgb.1, bgr.1, "G should be the same");
    }

    #[test]
    fn test_subpixel_coverage_none() {
        let (r, g, b) = subpixel_coverage(0, SubpixelOrder::None);
        assert_eq!(r, g, "None order should have equal coverage");
        assert_eq!(g, b);
    }

    #[test]
    fn test_layout_subpixel_empty() {
        let glyphs = layout_subpixel(&[], &[], 1.0, SubpixelOrder::Rgb);
        assert!(glyphs.is_empty());
    }

    #[test]
    fn test_layout_subpixel_single() {
        let glyphs = layout_subpixel(&[42], &[8.0], 1.0, SubpixelOrder::Rgb);
        assert_eq!(glyphs.len(), 1);
        assert_eq!(glyphs[0].glyph_id, 42);
        assert_eq!(glyphs[0].x_subpixel, 0);
        assert!(glyphs[0].is_pixel_aligned());
    }

    #[test]
    fn test_layout_subpixel_fractional() {
        // Advance of 5.5px at 1.0 scale = 16.5 subpixels -> rounds to 17
        let glyphs = layout_subpixel(&[1, 2], &[5.5, 5.5], 1.0, SubpixelOrder::Rgb);
        assert_eq!(glyphs.len(), 2);
        // First glyph at 0, second at 17 subpixels (5.67 pixels)
        assert_eq!(glyphs[0].x_subpixel, 0);
        assert_eq!(glyphs[1].x_subpixel, 17);
        assert!(!glyphs[1].is_pixel_aligned());
    }

    #[test]
    fn test_layout_subpixel_dpi_scale() {
        let glyphs_1x = layout_subpixel(&[1], &[8.0], 1.0, SubpixelOrder::Rgb);
        let glyphs_2x = layout_subpixel(&[1], &[8.0], 2.0, SubpixelOrder::Rgb);
        assert_eq!(glyphs_1x[0].x_subpixel, 0);
        assert_eq!(glyphs_2x[0].x_subpixel, 0);
        // Width should be doubled
        assert!(glyphs_2x[0].width >= glyphs_1x[0].width * 2);
    }

    #[test]
    fn test_subpixel_glyph_pixel_position() {
        let glyph = SubpixelGlyph::new(1, 7, 0, 255, 128, 64, 10, 16);
        assert_eq!(glyph.x_pixel(), 2); // 7 / 3 = 2
        assert_eq!(glyph.x_fraction(), 1); // 7 % 3 = 1
        assert!(!glyph.is_pixel_aligned());
    }

    #[test]
    fn test_subpixel_glyph_aligned() {
        let glyph = SubpixelGlyph::new(1, 9, 0, 255, 128, 64, 10, 16);
        assert_eq!(glyph.x_pixel(), 3);
        assert!(glyph.is_pixel_aligned());
    }

    #[test]
    fn test_sharpness_factor() {
        assert_eq!(subpixel_sharpness_factor(SubpixelOrder::Rgb), 3.0);
        assert_eq!(subpixel_sharpness_factor(SubpixelOrder::Bgr), 3.0);
        assert_eq!(subpixel_sharpness_factor(SubpixelOrder::RgbVertical), 1.0);
        assert_eq!(subpixel_sharpness_factor(SubpixelOrder::None), 1.0);
    }

    #[test]
    fn test_render_lcd_empty() {
        let mut fb = vec![0u8; 100 * 100 * 4];
        render_lcd(&mut fb, 100, 100, &[], (255, 255, 255, 255));
        // Should not panic, buffer unchanged
        assert!(fb.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_render_lcd_single_glyph() {
        let mut fb = vec![0u8; 100 * 100 * 4];
        let glyphs = vec![SubpixelGlyph::new(1, 0, 0, 255, 0, 0, 10, 16)];
        render_lcd(&mut fb, 100, 100, &glyphs, (255, 255, 255, 255));
        // Check that some pixels were written
        let has_nonzero = fb.iter().any(|&v| v > 0);
        assert!(has_nonzero, "LCD rendering should produce non-zero pixels");
    }

    #[test]
    fn test_render_lcd_zero_size() {
        let mut fb = vec![];
        let glyphs = vec![SubpixelGlyph::new(1, 0, 0, 255, 0, 0, 10, 16)];
        render_lcd(&mut fb, 0, 0, &glyphs, (255, 255, 255, 255));
        // Should not panic
    }
}
