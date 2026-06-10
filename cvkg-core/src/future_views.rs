use crate::animation::{Animated, SpringConfig};
use crate::{Never, Rect, Renderer, Size, SizeProposal, View};

/// A futuristic text view designed for Agentic UIs.
/// Handles streaming token generation (LLMs) with typewriter reveal effects and glowing cursors.
#[derive(Clone)]
pub struct StreamingText {
    pub text: String,
    pub visible_chars: Animated<f32>,
    pub font_size: f32,
    pub color: [f32; 4],
    pub cursor_color: [f32; 4],
}

impl StreamingText {
    pub fn new(text: String, font_size: f32, color: [f32; 4], cursor_color: [f32; 4]) -> Self {
        let mut anim = Animated::new(0.0, SpringConfig::snappy());
        anim.set_target(text.len() as f32); // Target is full length
        Self {
            text,
            visible_chars: anim,
            font_size,
            color,
            cursor_color,
        }
    }
}

impl View for StreamingText {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let visible_count = self.visible_chars.value.floor() as usize;
        let safe_count = visible_count.min(self.text.len());
        let current_text = &self.text[0..safe_count];

        // Draw the streaming text
        // (Assuming renderer.draw_text handles bounds wrapping natively for now)
        renderer.draw_text(current_text, rect.x, rect.y, self.font_size, self.color);

        // Calculate cursor position by measuring the current text
        let (width, _height) = renderer.measure_text(current_text, self.font_size);

        // Draw glowing cursor block
        let cursor_rect = Rect::new(rect.x + width + 2.0, rect.y, 8.0, self.font_size);
        renderer.fill_rect(cursor_rect, self.cursor_color);

        // Use futuristic glow (Gungnir) around the cursor if the renderer supports it
        // Usually we would use modifier, but we are inside the primitive render
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        let (w, h) = renderer.measure_text(&self.text, self.font_size);
        Size::new(w + 10.0, h)
    }
}

/// A generic GPU Particle Emitter view for magical/fluid UI effects.
#[derive(Clone)]
pub struct ParticleEmitter {
    pub count: u32,
    pub effect_type: String,
    pub color: [f32; 4],
}

impl ParticleEmitter {
    pub fn new(count: u32, effect_type: impl Into<String>, color: [f32; 4]) -> Self {
        Self {
            count,
            effect_type: effect_type.into(),
            color,
        }
    }
}

impl View for ParticleEmitter {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Emit particles from the center of this view's rect
        let origin = [rect.x + rect.width / 2.0, rect.y + rect.height / 2.0];
        renderer.dispatch_particles(origin, self.count, &self.effect_type, self.color);
    }
}

/// A Volumetric Hologram projection view for the Asgard Realm.
#[derive(Clone)]
pub struct HologramView {
    pub hologram_id: String,
    pub time: f32,
}

impl HologramView {
    pub fn new(hologram_id: impl Into<String>, time: f32) -> Self {
        Self {
            hologram_id: hologram_id.into(),
            time,
        }
    }
}

impl View for HologramView {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.draw_hologram(rect, &self.hologram_id, self.time);
    }
}
