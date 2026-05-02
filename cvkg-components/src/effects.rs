use cvkg_core::{View, Rect, Renderer, Never};

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

/// NiflheimFrost - Thick refractive ice/glass effect (inspired by Glur)
pub struct NiflheimFrost<V: View> {
    pub content: V,
    pub frost_intensity: f32,
    pub blur_radius: f32,
}

impl<V: View> NiflheimFrost<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            frost_intensity: 0.8,
            blur_radius: 30.0,
        }
    }
}

impl<V: View> View for NiflheimFrost<V> {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // 1. Apply thick Bifrost (refraction + blur)
        renderer.bifrost(rect, self.blur_radius, 1.2, 0.95);
        
        // 2. Render frost 'crystal' overlay
        let t = renderer.elapsed_time();
        for i in 0..15 {
            let x_off = ((t + i as f32) * 0.5).sin() * rect.width * 0.4;
            let y_off = ((t + i as f32 * 1.5) * 0.4).cos() * rect.height * 0.4;
            renderer.draw_line(
                rect.x + rect.width / 2.0 + x_off,
                rect.y + rect.height / 2.0 + y_off,
                rect.x + rect.width / 2.0 + x_off + 10.0,
                rect.y + rect.height / 2.0 + y_off + 10.0,
                [1.0, 1.0, 1.0, 0.1 * self.frost_intensity],
                1.0,
            );
        }

        // 3. Render content
        self.content.render(renderer, rect);
    }
}

/// FutharkFlow - Animated runic power-lines connecting components (inspired by Arwes)
pub struct FutharkFlow {
    pub speed: f32,
    pub color: [f32; 4],
}

impl Default for FutharkFlow {
    fn default() -> Self {
        Self {
            speed: 3.0,
            color: [0.0, 1.0, 1.0, 0.6],
        }
    }
}

impl View for FutharkFlow {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        let runes = ['ᚠ', 'ᚢ', 'ᚦ', 'ᚨ', 'ᚱ', 'ᚲ', 'ᚷ', 'ᚹ'];
        
        let flow_pos = (t * self.speed).fract();
        let rune_idx = ((t * self.speed).floor() as usize) % runes.len();
        
        // Draw the 'power line'
        renderer.draw_line(rect.x, rect.y + rect.height / 2.0, rect.x + rect.width, rect.y + rect.height / 2.0, [0.0, 0.5, 0.8, 0.2], 1.0);
        
        // Draw the moving rune pulse
        let rx = rect.x + flow_pos * rect.width;
        let ry = rect.y + rect.height / 2.0;
        
        renderer.gungnir(Rect { x: rx - 10.0, y: ry - 10.0, width: 20.0, height: 20.0 }, self.color, 5.0, 0.8);
        renderer.draw_text(&runes[rune_idx].to_string(), rx - 5.0, ry + 5.0, 14.0, self.color);
    }
}
