use cvkg_core::{Never, Rect, Renderer, View};

/// A spinning runic activity indicator.
/// Section 4.5: "Kinetic runic pulses for background processing."
#[derive(Clone)]
pub struct ValkyrieIndicator {
    pub size: f32,
    pub color: [f32; 4],
}

impl ValkyrieIndicator {
    pub fn new(size: f32) -> Self {
        Self {
            size,
            color: [0.0, 1.0, 1.0, 1.0],
        }
    }
}

impl View for ValkyrieIndicator {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;
        let radius = self.size / 2.0;

        // 1. Spinning Runic Ring
        let runes = ['ᚠ', 'ᚢ', 'ᚦ', 'ᚨ', 'ᚱ', 'ᚲ', 'ᚷ', 'ᚹ'];
        for (i, rune) in runes.iter().enumerate() {
            let angle = t * 3.0 + (i as f32 * std::f32::consts::PI * 2.0 / runes.len() as f32);
            let rx = center_x + angle.cos() * radius;
            let ry = center_y + angle.sin() * radius;

            let alpha = 0.2 + (angle.sin() * 0.5 + 0.5) * 0.8;
            let mut c = self.color;
            c[3] *= alpha;

            renderer.draw_text(&rune.to_string(), rx - 5.0, ry + 5.0, 12.0, c);
        }

        // 2. Central Pulse
        let pulse = (t * 5.0).sin() * 0.2 + 0.8;
        renderer.gungnir(
            Rect {
                x: center_x - radius * 0.5,
                y: center_y - radius * 0.5,
                width: radius,
                height: radius,
            },
            self.color,
            10.0 * pulse,
            0.5,
        );
    }
}
