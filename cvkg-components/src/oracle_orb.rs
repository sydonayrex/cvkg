use cvkg_core::{Never, Rect, Renderer, View};

/// The AI's visual presence. A pulsating runic orb (inspired by Orb).
/// Section 4.2: "Animated artifacts for AI-assisted interfaces."
pub struct OracleOrb {
    pub size: f32,
    pub color: [f32; 4],
    pub activity: f32, // 0.0 to 1.0
}

impl OracleOrb {
    pub fn new(size: f32) -> Self {
        Self {
            size,
            color: [0.0, 1.0, 1.0, 1.0], // Cyan
            activity: 0.5,
        }
    }

    pub fn activity(mut self, val: f32) -> Self {
        self.activity = val;
        self
    }
}

impl View for OracleOrb {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;
        let base_radius = self.size / 2.0;

        // 1. Core Pulsation
        let pulse = 1.0 + (t * (2.0 + self.activity * 5.0)).sin() * 0.1 * self.activity;
        let radius = base_radius * pulse;

        // 2. Gungnir Glow Layers
        for i in 1..4 {
            let layer_radius = radius * (1.0 + i as f32 * 0.2);
            let alpha = 0.4 / i as f32;
            let mut c = self.color;
            c[3] *= alpha;
            renderer.gungnir(
                Rect {
                    x: center_x - layer_radius,
                    y: center_y - layer_radius,
                    width: layer_radius * 2.0,
                    height: layer_radius * 2.0,
                },
                c,
                15.0 * self.activity,
                0.8,
            );
        }

        // 3. Central Runic Eye
        let runes = ['ᚦ', 'ᚢ', 'ᚱ', 'ᚲ'];
        let rune_idx = ((t * 2.0).floor() as usize) % runes.len();
        renderer.draw_text(
            &runes[rune_idx].to_string(),
            center_x - 10.0,
            center_y + 10.0,
            24.0,
            self.color,
        );

        // 4. Orbiting Shards
        for i in 0..6 {
            let angle = t * 2.0 + (i as f32 * std::f32::consts::PI * 2.0 / 6.0);
            let dist = radius * 1.5;
            let sx = center_x + angle.cos() * dist;
            let sy = center_y + angle.sin() * dist;
            
            renderer.fill_rect(
                Rect { x: sx - 2.0, y: sy - 2.0, width: 4.0, height: 4.0 },
                self.color,
            );
        }
    }
}
