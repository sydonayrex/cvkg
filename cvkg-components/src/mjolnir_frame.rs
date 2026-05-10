use cvkg_core::{Never, Rect, Renderer, View};

/// MjolnirFrame - A geometric, non-rectangular UI frame with chromatic aberration.
/// Section 4.5: "Mjolnir's Edge — Geometric slicing and destructive visual feedback."
pub struct MjolnirFrame {
    pub border_color: [f32; 4],
    pub border_width: f32,
    pub bevel_size: f32,
    pub glitch_intensity: f32,
}

impl Default for MjolnirFrame {
    fn default() -> Self {
        Self::new()
    }
}

impl MjolnirFrame {
    pub fn new() -> Self {
        Self {
            border_color: [0.0, 1.0, 1.0, 0.8], // Cyan Default
            border_width: 1.5,
            bevel_size: 20.0,
            glitch_intensity: 0.1,
        }
    }

    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.border_color = color;
        self
    }

    pub fn with_glitch(mut self, intensity: f32) -> Self {
        self.glitch_intensity = intensity;
        self
    }
}

impl View for MjolnirFrame {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        let bevel = self.bevel_size;
        
        // 1. Calculate path points for a beveled rectangle
        // TL, TR, BR, BL with slices
        let points = [
            (rect.x + bevel, rect.y),
            (rect.x + rect.width - bevel, rect.y),
            (rect.x + rect.width, rect.y + bevel),
            (rect.x + rect.width, rect.y + rect.height - bevel),
            (rect.x + rect.width - bevel, rect.y + rect.height),
            (rect.x + bevel, rect.y + rect.height),
            (rect.x, rect.y + rect.height - bevel),
            (rect.x, rect.y + bevel),
        ];

        // 2. Main Border
        self.draw_beveled_path(renderer, &points, self.border_color, self.border_width);

        // 3. Chromatic Aberration (Glitch)
        if self.glitch_intensity > 0.0 {
            let offset = (t * 10.0).sin() * self.glitch_intensity * 2.0;
            
            // Red Shift
            let mut red_points = points;
            for p in &mut red_points { p.0 += offset; }
            self.draw_beveled_path(renderer, &red_points, [1.0, 0.0, 0.0, 0.4], 1.0);

            // Blue Shift
            let mut blue_points = points;
            for p in &mut blue_points { p.0 -= offset; }
            self.draw_beveled_path(renderer, &blue_points, [0.0, 0.0, 1.0, 0.4], 1.0);
        }

        // 4. Inner "Scanline" Glow
        let alpha = (t * 2.0).sin().abs() * 0.1 + 0.05;
        renderer.fill_rect(rect, [self.border_color[0], self.border_color[1], self.border_color[2], alpha]);
    }
}

impl MjolnirFrame {
    fn draw_beveled_path(&self, renderer: &mut dyn Renderer, points: &[(f32, f32); 8], color: [f32; 4], width: f32) {
        for i in 0..8 {
            let p1 = points[i];
            let p2 = points[(i + 1) % 8];
            renderer.draw_line(p1.0, p1.1, p2.0, p2.1, color, width);
        }
    }
}
