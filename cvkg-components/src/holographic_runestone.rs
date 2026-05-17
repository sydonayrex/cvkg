use cvkg_core::{Never, Rect, Renderer, View};

/// A floating, semi-transparent 3D runic element.
/// Section 4.8: "Volumetric projections for ethereal data visualization."
pub struct HolographicRunestone {
    pub rune: char,
    pub size: f32,
}

impl HolographicRunestone {
    pub fn new(rune: char, size: f32) -> Self {
        Self { rune, size }
    }
}

impl View for HolographicRunestone {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;

        // 1. Floating Animation
        let float_y = (t * 2.0).sin() * 10.0;
        let scale = 1.0 + (t * 3.0).cos() * 0.05;

        // 2. Holographic Scanlines (Muspelheim effect)
        let mut color = [0.0, 1.0, 1.0, 0.4];
        let flicker = if (t * 20.0).sin() > 0.95 { 0.2 } else { 1.0 };
        color[3] *= flicker;

        // 3. Multi-layered Runic Projection
        for i in 0..3 {
            let offset = i as f32 * 2.0 * scale;
            let alpha = 0.6 / (i + 1) as f32;
            let mut c = color;
            c[3] *= alpha;

            renderer.draw_text(
                &self.rune.to_string(),
                center_x - (self.size / 2.0) + offset,
                center_y + (self.size / 2.0) + float_y + offset,
                self.size * scale,
                c,
            );
        }

        // 4. Projection Base (Glow)
        renderer.gungnir(
            Rect {
                x: center_x - self.size * 0.6,
                y: center_y + self.size * 0.6 + float_y,
                width: self.size * 1.2,
                height: 10.0,
            },
            color,
            5.0,
            0.3,
        );
    }
}
