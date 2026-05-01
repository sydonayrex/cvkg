use cvkg_core::{View, Rect, Renderer, Never, ElapsedTime};

/// Seiðr - Holographic projection effect with scanline animation (Norse magic)
pub struct Seiðr {
    pub base_color: [f32; 4],
    pub scanline_speed: f32,
    pub flicker_intensity: f32,
}

impl Default for Seiðr {
    fn default() -> Self {
        Self {
            base_color: [0.0, 0.8, 1.0, 0.3],
            scanline_speed: 2.0,
            flicker_intensity: 0.1,
        }
    }
}

impl View for Seiðr {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }
    
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        
        let flicker = 1.0 + (t * 13.0).sin() * self.flicker_intensity;
        let color = [
            self.base_color[0] * flicker,
            self.base_color[1] * flicker,
            self.base_color[2] * flicker,
            self.base_color[3],
        ];
        
        renderer.fill_rounded_rect(rect, 8.0, color);
        
        let scan_y = (t * self.scanline_speed).fract() * rect.height;
        for i in 0..5 {
            let y = rect.y + (scan_y + i as f32 * 20.0) % rect.height;
            renderer.draw_line(
                rect.x, y,
                rect.x + rect.width, y,
                [0.5, 1.0, 0.8, 0.4],
                1.0,
            );
        }
    }
}

/// LokiGlitch - Digital distortion text effect (Norse trickster)
pub struct LokiGlitch {
    pub content: String,
    pub font_size: f32,
    pub base_color: [f32; 4],
    pub glitch_intensity: f32,
}

impl LokiGlitch {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            font_size: 16.0,
            base_color: [1.0, 1.0, 1.0, 1.0],
            glitch_intensity: 5.0,
        }
    }
}

impl View for LokiGlitch {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }
    
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        
        // Base text
        renderer.draw_text(&self.content, rect.x, rect.y, self.font_size, self.base_color);
        
        // Red glitch offset
        if (t * 10.0).sin().abs() > 0.8 {
            renderer.draw_text(
                &self.content,
                rect.x + (t * 15.0).sin() * self.glitch_intensity,
                rect.y,
                self.font_size,
                [1.0, 0.0, 0.3, 0.8],
            );
        }
        
        // Blue glitch offset
        if (t * 7.0).cos().abs() > 0.85 {
            renderer.draw_text(
                &self.content,
                rect.x - (t * 12.0).cos() * self.glitch_intensity,
                rect.y,
                self.font_size,
                [0.3, 0.7, 1.0, 0.8],
            );
        }
    }
}

/// MidgardLines - A standalone scanline overlay effect
pub struct MidgardLines {
    pub speed: f32,
    pub density: f32,
    pub color: [f32; 4],
}

impl Default for MidgardLines {
    fn default() -> Self {
        Self {
            speed: 1.0,
            density: 20.0,
            color: [0.0, 1.0, 1.0, 0.2],
        }
    }
}

impl View for MidgardLines {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }
    
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        let scan_y = (t * self.speed).fract() * rect.height;
        
        let mut y = rect.y + scan_y % self.density;
        while y < rect.y + rect.height {
            renderer.draw_line(
                rect.x, y,
                rect.x + rect.width, y,
                self.color,
                1.0,
            );
            y += self.density;
        }
    }
}
