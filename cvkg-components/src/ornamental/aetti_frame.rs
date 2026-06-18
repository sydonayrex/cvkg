//! ÆttiFrame -- Runic ornamental border system.
//!
//! Named after the Ættir, the three groups of eight runes in the Elder Futhark.
//! Each `RunicStyle` renders a distinct decorative border pattern using the
//! existing renderer primitives (`draw_line`, `fill_rounded_rect`, `stroke_rect`, etc.).

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// The five runic ornamental border styles.
///
/// Each style evokes a different Norse-inspired visual language:
/// - `CarvedStone`: Elder Futhark characters carved into weathered rock.
/// - `Knotwork`: Interlocking Celtic-Norse knot patterns along edges.
/// - `HammeredMetal`: Bronze/iron plate with rivets at the corners.
/// - `DragonScale`: Overlapping scale tessellation reminiscent of Fáfnir.
/// - `IceCrystal`: Fractal ice formations inspired by Niflheim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunicStyle {
    /// Elder Futhark characters carved into stone.
    CarvedStone,
    /// Interlocking knotwork pattern.
    Knotwork,
    /// Hammered metal with rivets at corners.
    HammeredMetal,
    /// Dragon-scale tessellation.
    DragonScale,
    /// Ice crystal formations.
    IceCrystal,
}

/// An ornamental frame that decorates a rectangular region with runic borders.
///
/// `ÆttiFrame` implements `View` so it can be composed directly into any
/// render tree. The `intensity` field (0.0–1.0) controls the opacity of
/// the ornamental elements.
///
/// # Contract
/// - `style` selects the pattern family.
/// - `intensity` maps to alpha multiplier; 0.0 is invisible, 1.0 is fully opaque.
/// - When `animate` is true, the renderer's `elapsed_time` drives subtle motion.
pub struct ÆttiFrame {
    /// Selects which border pattern to render.
    pub style: RunicStyle,
    /// Alpha multiplier for all ornamental elements (0.0–1.0).
    pub intensity: f32,
    /// Whether to animate with the renderer's elapsed time.
    pub animate: bool,
}

impl Default for ÆttiFrame {
    /// Creates a default ÆttiFrame with `CarvedStone` style.
    ///
    /// # Contract
    /// - intensity defaults to 0.8.
    /// - animate defaults to true.
    fn default() -> Self {
        Self::new(RunicStyle::CarvedStone)
    }
}

impl ÆttiFrame {
    /// Creates a new ÆttiFrame with the given `style`.
    ///
    /// # Contract
    /// - intensity defaults to 0.8.
    /// - animate defaults to true.
    pub fn new(style: RunicStyle) -> Self {
        Self {
            style,
            intensity: 0.8,
            animate: true,
        }
    }

    /// Sets the intensity (alpha multiplier) and returns `self`.
    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    /// Enables or disables animation and returns `self`.
    pub fn with_animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }
}

// ── Rendering ──────────────────────────────────────────────────────────────

impl View for ÆttiFrame {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    /// Renders the ornamental border pattern within the given `rect`.
    ///
    /// # Contract
    /// - All four edges are decorated according to `self.style`.
    /// - The interior of `rect` is left untouched (border-only).
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let bw = 4.0_f32;

        match self.style {
            RunicStyle::CarvedStone => self.render_carved_stone(renderer, rect, bw),
            RunicStyle::Knotwork => self.render_knotwork(renderer, rect, bw),
            RunicStyle::HammeredMetal => self.render_hammered_metal(renderer, rect, bw),
            RunicStyle::DragonScale => self.render_dragon_scale(renderer, rect, bw),
            RunicStyle::IceCrystal => self.render_ice_crystal(renderer, rect, bw),
        }
    }
}

impl ÆttiFrame {
    // ── CarvedStone ────────────────────────────────────────────────────

    /// Draws warm amber rune-carved border lines on all four edges,
    /// with a darker shadow line offset below each edge for depth.
    fn render_carved_stone(&self, renderer: &mut dyn Renderer, rect: Rect, bw: f32) {
        let tint = self.intensity;
        let rune_gold = theme::with_alpha(theme::viking_gold(), tint * 0.8);
        let shadow = theme::with_alpha(theme::shadow(), tint * 0.4);
        let glow = theme::with_alpha(theme::viking_gold(), tint * 0.3);

        // Top edge + glow + shadow
        renderer.draw_line(rect.x, rect.y, rect.x + rect.width, rect.y, glow, bw + 2.0);
        renderer.draw_line(rect.x, rect.y, rect.x + rect.width, rect.y, rune_gold, bw);
        renderer.draw_line(
            rect.x,
            rect.y + bw,
            rect.x + rect.width,
            rect.y + bw,
            shadow,
            1.0,
        );

        // Bottom edge + glow + shadow
        renderer.draw_line(
            rect.x,
            rect.y + rect.height,
            rect.x + rect.width,
            rect.y + rect.height,
            glow,
            bw + 2.0,
        );
        renderer.draw_line(
            rect.x,
            rect.y + rect.height,
            rect.x + rect.width,
            rect.y + rect.height,
            rune_gold,
            bw,
        );
        renderer.draw_line(
            rect.x,
            rect.y + rect.height - bw,
            rect.x + rect.width,
            rect.y + rect.height - bw,
            shadow,
            1.0,
        );

        // Left edge + glow + shadow
        renderer.draw_line(rect.x, rect.y, rect.x, rect.y + rect.height, glow, bw + 2.0);
        renderer.draw_line(rect.x, rect.y, rect.x, rect.y + rect.height, rune_gold, bw);
        renderer.draw_line(
            rect.x + bw,
            rect.y,
            rect.x + bw,
            rect.y + rect.height,
            shadow,
            1.0,
        );

        // Right edge + glow + shadow
        renderer.draw_line(
            rect.x + rect.width,
            rect.y,
            rect.x + rect.width,
            rect.y + rect.height,
            glow,
            bw + 2.0,
        );
        renderer.draw_line(
            rect.x + rect.width,
            rect.y,
            rect.x + rect.width,
            rect.y + rect.height,
            rune_gold,
            bw,
        );
        renderer.draw_line(
            rect.x + rect.width - bw,
            rect.y,
            rect.x + rect.width - bw,
            rect.y + rect.height,
            shadow,
            1.0,
        );

        // Corner rune accents: small filled squares at each corner
        let cr = bw * 1.5;
        let corner_color = theme::with_alpha(theme::viking_gold(), tint * 0.9);
        for &(cx, cy) in &[
            (rect.x, rect.y),
            (rect.x + rect.width, rect.y),
            (rect.x, rect.y + rect.height),
            (rect.x + rect.width, rect.y + rect.height),
        ] {
            renderer.fill_rounded_rect(
                Rect {
                    x: cx - cr / 2.0,
                    y: cy - cr / 2.0,
                    width: cr,
                    height: cr,
                },
                1.0,
                corner_color,
            );
        }
    }

    // ── Knotwork ───────────────────────────────────────────────────────

    /// Draws interlocking knotwork diamonds along each edge.
    fn render_knotwork(&self, renderer: &mut dyn Renderer, rect: Rect, bw: f32) {
        let tint = self.intensity;
        let line_color = theme::accent();
        let line_color = [line_color[0], line_color[1], line_color[2], tint * 0.9];

        let spacing = 24.0_f32;
        // Number of diamonds on the longer edges
        let count_h = ((rect.width / spacing).floor() as usize).max(2);
        let count_v = ((rect.height / spacing).floor() as usize).max(2);
        let step_h = rect.width / count_h as f32;
        let step_v = rect.height / count_v as f32;
        let half = bw * 1.5;

        // Top edge diamonds
        for i in 0..count_h {
            let cx = rect.x + step_h * (i as f32 + 0.5);
            let cy = rect.y;
            // Diamond: four lines
            renderer.draw_line(cx - half, cy, cx, cy - half, line_color, 1.5);
            renderer.draw_line(cx, cy - half, cx + half, cy, line_color, 1.5);
            renderer.draw_line(cx + half, cy, cx, cy + half, line_color, 1.5);
            renderer.draw_line(cx, cy + half, cx - half, cy, line_color, 1.5);
        }

        // Bottom edge diamonds
        for i in 0..count_h {
            let cx = rect.x + step_h * (i as f32 + 0.5);
            let cy = rect.y + rect.height;
            renderer.draw_line(cx - half, cy, cx, cy - half, line_color, 1.5);
            renderer.draw_line(cx, cy - half, cx + half, cy, line_color, 1.5);
            renderer.draw_line(cx + half, cy, cx, cy + half, line_color, 1.5);
            renderer.draw_line(cx, cy + half, cx - half, cy, line_color, 1.5);
        }

        // Left edge diamonds
        for i in 0..count_v {
            let cx = rect.x;
            let cy = rect.y + step_v * (i as f32 + 0.5);
            renderer.draw_line(cx - half, cy, cx, cy - half, line_color, 1.5);
            renderer.draw_line(cx, cy - half, cx + half, cy, line_color, 1.5);
            renderer.draw_line(cx + half, cy, cx, cy + half, line_color, 1.5);
            renderer.draw_line(cx, cy + half, cx - half, cy, line_color, 1.5);
        }

        // Right edge diamonds
        for i in 0..count_v {
            let cx = rect.x + rect.width;
            let cy = rect.y + step_v * (i as f32 + 0.5);
            renderer.draw_line(cx - half, cy, cx, cy - half, line_color, 1.5);
            renderer.draw_line(cx, cy - half, cx + half, cy, line_color, 1.5);
            renderer.draw_line(cx + half, cy, cx, cy + half, line_color, 1.5);
            renderer.draw_line(cx, cy + half, cx - half, cy, line_color, 1.5);
        }

        // Connecting border lines between diamonds
        renderer.draw_line(rect.x, rect.y, rect.x + rect.width, rect.y, line_color, bw);
        renderer.draw_line(
            rect.x,
            rect.y + rect.height,
            rect.x + rect.width,
            rect.y + rect.height,
            line_color,
            bw,
        );
        renderer.draw_line(rect.x, rect.y, rect.x, rect.y + rect.height, line_color, bw);
        renderer.draw_line(
            rect.x + rect.width,
            rect.y,
            rect.x + rect.width,
            rect.y + rect.height,
            line_color,
            bw,
        );
    }

    // ── HammeredMetal ──────────────────────────────────────────────────

    /// Draws dark metallic border plates with rivets at the corners.
    fn render_hammered_metal(&self, renderer: &mut dyn Renderer, rect: Rect, bw: f32) {
        let tint = self.intensity;
        let plate = theme::with_alpha(theme::surface(), tint * 0.95);
        let highlight = theme::with_alpha(theme::text_dim(), tint * 0.6);
        let rivet_color = theme::with_alpha(theme::viking_gold(), tint);
        let plate_bw = bw * 2.0;

        // Top plate
        renderer.draw_line(rect.x, rect.y, rect.x + rect.width, rect.y, plate, plate_bw);
        renderer.draw_line(
            rect.x,
            rect.y + 1.0,
            rect.x + rect.width,
            rect.y + 1.0,
            highlight,
            1.0,
        );

        // Bottom plate
        renderer.draw_line(
            rect.x,
            rect.y + rect.height,
            rect.x + rect.width,
            rect.y + rect.height,
            plate,
            plate_bw,
        );

        // Left plate
        renderer.draw_line(
            rect.x,
            rect.y,
            rect.x,
            rect.y + rect.height,
            plate,
            plate_bw,
        );
        renderer.draw_line(
            rect.x + 1.0,
            rect.y,
            rect.x + 1.0,
            rect.y + rect.height,
            highlight,
            1.0,
        );

        // Right plate
        renderer.draw_line(
            rect.x + rect.width,
            rect.y,
            rect.x + rect.width,
            rect.y + rect.height,
            plate,
            plate_bw,
        );

        // Rivets: small filled circles at corners and midpoints
        let rivet_r = bw * 0.8;
        let rivet_positions = [
            (rect.x, rect.y),
            (rect.x + rect.width, rect.y),
            (rect.x, rect.y + rect.height),
            (rect.x + rect.width, rect.y + rect.height),
            (rect.x + rect.width / 2.0, rect.y),
            (rect.x + rect.width / 2.0, rect.y + rect.height),
            (rect.x, rect.y + rect.height / 2.0),
            (rect.x + rect.width, rect.y + rect.height / 2.0),
        ];

        for &(rx, ry) in &rivet_positions {
            renderer.fill_ellipse(
                Rect {
                    x: rx - rivet_r,
                    y: ry - rivet_r,
                    width: rivet_r * 2.0,
                    height: rivet_r * 2.0,
                },
                rivet_color,
            );
        }
    }

    // ── DragonScale ────────────────────────────────────────────────────

    /// Draws overlapping scale/half-circle tessellation along edges.
    fn render_dragon_scale(&self, renderer: &mut dyn Renderer, rect: Rect, bw: f32) {
        let tint = self.intensity;
        let scale_color = theme::with_alpha(theme::success(), tint * 0.85);
        let outline_color = theme::with_alpha(theme::border(), tint * 0.9);

        let scale_w = 16.0_f32;
        let scale_h = bw * 3.0;
        let count_h = ((rect.width / scale_w).floor() as usize).max(3);
        let count_v = ((rect.height / scale_w).floor() as usize).max(3);

        // Top edge scales (triangular ray representations)
        let step_h = rect.width / count_h as f32;
        for i in 0..count_h {
            let cx = rect.x + step_h * (i as f32 + 0.5);
            let cy = rect.y;
            // Upward-pointing triangle for top edge
            let vertices = [
                [cx - step_h * 0.4, cy],
                [cx, cy - scale_h],
                [cx + step_h * 0.4, cy],
            ];
            renderer.fill_polygon(&vertices, scale_color);
            renderer.stroke_polygon(&vertices, outline_color, 1.0);
        }

        // Bottom edge scales (downward-pointing triangles)
        for i in 0..count_h {
            let cx = rect.x + step_h * (i as f32 + 0.5);
            let cy = rect.y + rect.height;
            let vertices = [
                [cx - step_h * 0.4, cy],
                [cx, cy + scale_h],
                [cx + step_h * 0.4, cy],
            ];
            renderer.fill_polygon(&vertices, scale_color);
            renderer.stroke_polygon(&vertices, outline_color, 1.0);
        }

        // Left edge scales (leftward-pointing triangles)
        let step_v = rect.height / count_v as f32;
        for i in 0..count_v {
            let cx = rect.x;
            let cy = rect.y + step_v * (i as f32 + 0.5);
            let vertices = [
                [cx, cy - step_v * 0.4],
                [cx - scale_h, cy],
                [cx, cy + step_v * 0.4],
            ];
            renderer.fill_polygon(&vertices, scale_color);
            renderer.stroke_polygon(&vertices, outline_color, 1.0);
        }

        // Right edge scales (rightward-pointing triangles)
        for i in 0..count_v {
            let cx = rect.x + rect.width;
            let cy = rect.y + step_v * (i as f32 + 0.5);
            let vertices = [
                [cx, cy - step_v * 0.4],
                [cx + scale_h, cy],
                [cx, cy + step_v * 0.4],
            ];
            renderer.fill_polygon(&vertices, scale_color);
            renderer.stroke_polygon(&vertices, outline_color, 1.0);
        }
    }

    // ── IceCrystal ─────────────────────────────────────────────────────

    /// Draws fractal ice crystal formations along edges.
    fn render_ice_crystal(&self, renderer: &mut dyn Renderer, rect: Rect, bw: f32) {
        let tint = self.intensity;
        let ice_color = theme::with_alpha(theme::info(), tint * 0.7);
        let ice_bright = theme::with_alpha(theme::accent(), tint * 0.9);
        let t = renderer.elapsed_time();

        let spacing = 20.0_f32;
        let count_h = ((rect.width / spacing).floor() as usize).max(2);
        let count_v = ((rect.height / spacing).floor() as usize).max(2);
        let step_h = rect.width / count_h as f32;
        let step_v = rect.height / count_v as f32;

        // Animate crystal height slightly
        let crystal_h = if self.animate {
            bw * 3.0 + (t * 3.0).sin() * bw * 0.5
        } else {
            bw * 3.0
        };

        // Top edge crystals
        for i in 0..count_h {
            let cx = rect.x + step_h * (i as f32 + 0.5);
            let cy = rect.y;
            // Vertical spike
            renderer.draw_line(cx, cy, cx, cy - crystal_h, ice_color, 1.5);
            // Branch lines
            let branch = crystal_h * 0.4;
            renderer.draw_line(
                cx,
                cy - crystal_h * 0.6,
                cx - branch,
                cy - crystal_h * 0.3,
                ice_bright,
                1.0,
            );
            renderer.draw_line(
                cx,
                cy - crystal_h * 0.6,
                cx + branch,
                cy - crystal_h * 0.3,
                ice_bright,
                1.0,
            );
        }

        // Bottom edge crystals
        for i in 0..count_h {
            let cx = rect.x + step_h * (i as f32 + 0.5);
            let cy = rect.y + rect.height;
            renderer.draw_line(cx, cy, cx, cy + crystal_h, ice_color, 1.5);
            let branch = crystal_h * 0.4;
            renderer.draw_line(
                cx,
                cy + crystal_h * 0.6,
                cx - branch,
                cy + crystal_h * 0.3,
                ice_bright,
                1.0,
            );
            renderer.draw_line(
                cx,
                cy + crystal_h * 0.6,
                cx + branch,
                cy + crystal_h * 0.3,
                ice_bright,
                1.0,
            );
        }

        // Left edge crystals
        for i in 0..count_v {
            let cx = rect.x;
            let cy = rect.y + step_v * (i as f32 + 0.5);
            renderer.draw_line(cx, cy, cx - crystal_h, cy, ice_color, 1.5);
            let branch = crystal_h * 0.4;
            renderer.draw_line(
                cx - crystal_h * 0.6,
                cy,
                cx - crystal_h * 0.3,
                cy - branch,
                ice_bright,
                1.0,
            );
            renderer.draw_line(
                cx - crystal_h * 0.6,
                cy,
                cx - crystal_h * 0.3,
                cy + branch,
                ice_bright,
                1.0,
            );
        }

        // Right edge crystals
        for i in 0..count_v {
            let cx = rect.x + rect.width;
            let cy = rect.y + step_v * (i as f32 + 0.5);
            renderer.draw_line(cx, cy, cx + crystal_h, cy, ice_color, 1.5);
            let branch = crystal_h * 0.4;
            renderer.draw_line(
                cx + crystal_h * 0.6,
                cy,
                cx + crystal_h * 0.3,
                cy - branch,
                ice_bright,
                1.0,
            );
            renderer.draw_line(
                cx + crystal_h * 0.6,
                cy,
                cx + crystal_h * 0.3,
                cy + branch,
                ice_bright,
                1.0,
            );
        }

        // Base border line
        renderer.draw_line(rect.x, rect.y, rect.x + rect.width, rect.y, ice_bright, bw);
        renderer.draw_line(
            rect.x,
            rect.y + rect.height,
            rect.x + rect.width,
            rect.y + rect.height,
            ice_bright,
            bw,
        );
        renderer.draw_line(rect.x, rect.y, rect.x, rect.y + rect.height, ice_bright, bw);
        renderer.draw_line(
            rect.x + rect.width,
            rect.y,
            rect.x + rect.width,
            rect.y + rect.height,
            ice_bright,
            bw,
        );
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Unit Tests
// ════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use cvkg_core::{ElapsedTime, Renderer};

    /// A mock renderer that records every draw command as a string.
    struct MockRenderer {
        commands: Vec<String>,
    }

    impl MockRenderer {
        fn new() -> Self {
            Self {
                commands: Vec::new(),
            }
        }
    }

    impl ElapsedTime for MockRenderer {
        fn elapsed_time(&self) -> f32 {
            0.0
        }
        fn delta_time(&self) -> f32 {
            1.0 / 60.0
        }
    }

    impl Renderer for MockRenderer {
        fn fill_rect(&mut self, _rect: Rect, _color: [f32; 4]) {}
        fn fill_rounded_rect(&mut self, rect: Rect, _radius: f32, _color: [f32; 4]) {
            self.commands
                .push(format!("FillRoundedRect({:.1},{:.1})", rect.x, rect.y));
        }
        fn fill_ellipse(&mut self, rect: Rect, _color: [f32; 4]) {
            self.commands
                .push(format!("FillEllipse({:.1},{:.1})", rect.x, rect.y));
        }
        fn stroke_rect(&mut self, _rect: Rect, _color: [f32; 4], _w: f32) {}
        fn stroke_rounded_rect(&mut self, _rect: Rect, _radius: f32, _color: [f32; 4], _w: f32) {}
        fn stroke_ellipse(&mut self, _rect: Rect, _color: [f32; 4], _w: f32) {}
        fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, _color: [f32; 4], _w: f32) {
            self.commands.push(format!(
                "DrawLine({:.1},{:.1}->{:.1},{:.1})",
                x1, y1, x2, y2
            ));
        }
        fn fill_polygon(&mut self, vertices: &[[f32; 2]], _color: [f32; 4]) {
            self.commands
                .push(format!("FillPolygon(verts={})", vertices.len()));
        }
        fn stroke_polygon(&mut self, vertices: &[[f32; 2]], _color: [f32; 4], _w: f32) {
            self.commands
                .push(format!("StrokePolygon(verts={})", vertices.len()));
        }
        fn draw_text(&mut self, _text: &str, _x: f32, _y: f32, _size: f32, _color: [f32; 4]) {}
        fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
            (text.len() as f32 * size * 0.6, size)
        }
        fn push_vnode(&mut self, _rect: Rect, _name: &'static str) {}
        fn pop_vnode(&mut self) {}
        fn set_key(&mut self, _key: &str) {}
        fn set_aria_role(&mut self, _role: &str) {}
        fn set_aria_label(&mut self, _label: &str) {}
        fn memoize(&mut self, _id: u64, _data_hash: u64, render_fn: &dyn Fn(&mut dyn Renderer)) {
            render_fn(self);
        }
    }

    fn test_rect() -> Rect {
        Rect {
            x: 10.0,
            y: 20.0,
            width: 200.0,
            height: 100.0,
        }
    }

    // ── Test 1: CarvedStone renders corner accents and border lines ─────

    #[test]
    fn test_carved_stone_renders_corner_accents_and_borders() {
        let mut renderer = MockRenderer::new();
        let frame = ÆttiFrame::new(RunicStyle::CarvedStone);
        let rect = test_rect();

        frame.render(&mut renderer, rect);

        // Should have corner accent FillRoundedRect calls (4 corners)
        let corner_count = renderer
            .commands
            .iter()
            .filter(|c| c.starts_with("FillRoundedRect"))
            .count();
        assert!(
            corner_count >= 4,
            "CarvedStone should render at least 4 corner accents, got {}",
            corner_count
        );

        // Should have DrawLine calls for the border edges
        let line_count = renderer
            .commands
            .iter()
            .filter(|c| c.starts_with("DrawLine"))
            .count();
        assert!(
            line_count >= 4,
            "CarvedStone should render at least 4 border lines, got {}",
            line_count
        );
    }

    // ── Test 2: Knotwork renders diamond patterns ───────────────────────

    #[test]
    fn test_knotwork_renders_diamond_patterns() {
        let mut renderer = MockRenderer::new();
        let frame = ÆttiFrame::new(RunicStyle::Knotwork);
        let rect = test_rect();

        frame.render(&mut renderer, rect);

        // Knotwork draws 4 lines per diamond, plus 4 border lines.
        // With a 200x100 rect and 24px spacing: ~8 diamonds on top/bottom, ~4 on left/right
        let line_count = renderer
            .commands
            .iter()
            .filter(|c| c.starts_with("DrawLine"))
            .count();
        assert!(
            line_count > 20,
            "Knotwork should render many diamond lines, got {}",
            line_count
        );
    }

    // ── Test 3: HammeredMetal renders rivets (ellipses) ─────────────────

    #[test]
    fn test_hammered_metal_renders_rivets() {
        let mut renderer = MockRenderer::new();
        let frame = ÆttiFrame::new(RunicStyle::HammeredMetal);
        let rect = test_rect();

        frame.render(&mut renderer, rect);

        // HammeredMetal renders 8 rivets as filled ellipses
        let ellipse_count = renderer
            .commands
            .iter()
            .filter(|c| c.starts_with("FillEllipse"))
            .count();
        assert!(
            ellipse_count >= 8,
            "HammeredMetal should render at least 8 rivets, got {}",
            ellipse_count
        );
    }

    // ── Test 4: DragonScale renders triangular polygons ─────────────────

    #[test]
    fn test_dragon_scale_renders_triangular_polygons() {
        let mut renderer = MockRenderer::new();
        let frame = ÆttiFrame::new(RunicStyle::DragonScale);
        let rect = test_rect();

        frame.render(&mut renderer, rect);

        // DragonScale renders filled triangles (3-vert polygons) for scales
        let fill_poly_count = renderer
            .commands
            .iter()
            .filter(|c| c.starts_with("FillPolygon"))
            .count();
        assert!(
            fill_poly_count > 0,
            "DragonScale should render filled polygon scales"
        );

        // Each scale is a triangle (3 vertices)
        let tri_count = renderer
            .commands
            .iter()
            .filter(|c| c.contains("verts=3"))
            .count();
        assert!(
            tri_count > 0,
            "DragonScale should render triangular scales (3 verts each)"
        );
    }

    // ── Test 5: IceCrystal renders branching lines ──────────────────────

    #[test]
    fn test_ice_crystal_renders_branching_lines() {
        let mut renderer = MockRenderer::new();
        let frame = ÆttiFrame::new(RunicStyle::IceCrystal);
        let rect = test_rect();

        frame.render(&mut renderer, rect);

        // IceCrystal renders many lines: base border + crystal spikes + branches
        let line_count = renderer
            .commands
            .iter()
            .filter(|c| c.starts_with("DrawLine"))
            .count();
        assert!(
            line_count > 10,
            "IceCrystal should render many crystal lines, got {}",
            line_count
        );
    }

    // ── Test 6: Default and builder methods ─────────────────────────────

    #[test]
    fn test_default_and_builder_methods() {
        let frame = ÆttiFrame::default();
        assert_eq!(frame.style, RunicStyle::CarvedStone);
        assert!((frame.intensity - 0.8).abs() < f32::EPSILON);
        assert!(frame.animate);

        let frame = ÆttiFrame::new(RunicStyle::IceCrystal)
            .with_intensity(0.5)
            .with_animate(false);
        assert_eq!(frame.style, RunicStyle::IceCrystal);
        assert!((frame.intensity - 0.5).abs() < f32::EPSILON);
        assert!(!frame.animate);
    }

    // ── Test 7: All styles produce some output ──────────────────────────

    #[test]
    fn test_all_styles_produce_output() {
        let styles = [
            RunicStyle::CarvedStone,
            RunicStyle::Knotwork,
            RunicStyle::HammeredMetal,
            RunicStyle::DragonScale,
            RunicStyle::IceCrystal,
        ];

        for style in &styles {
            let mut renderer = MockRenderer::new();
            let frame = ÆttiFrame::new(*style);
            let rect = test_rect();

            frame.render(&mut renderer, rect);

            assert!(
                !renderer.commands.is_empty(),
                "Style {:?} should produce at least one render command",
                style
            );
        }
    }
}
