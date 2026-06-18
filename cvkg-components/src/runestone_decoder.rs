use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};
use std::time::Duration;

/// RunestoneDecoder - A text component that "deciphers" ancient runes into digital text.
/// Section 4.12: "Reading the Runes -- Temporal data reconstruction."
pub struct RunestoneDecoder {
    pub text: String,
    pub duration: Duration,
    pub start_time: f32,
}

impl RunestoneDecoder {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            duration: Duration::from_secs(2),
            start_time: 0.0,
        }
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

impl View for RunestoneDecoder {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        let progress = ((t - self.start_time) / self.duration.as_secs_f32()).clamp(0.0, 1.0);

        let runes = [
            'ᚠ', 'ᚢ', 'ᚦ', 'ᚨ', 'ᚱ', 'ᚲ', 'ᚷ', 'ᚹ', 'ᚺ', 'ᚻ', 'ᚼ', 'ᛁ', 'ᛃ', 'ᛇ', 'ᛈ', 'ᛉ', 'ᛊ',
            'ᛏ', 'ᛒ', 'ᛖ', 'ᛗ', 'ᛚ', 'ᛜ', 'ᛟ', 'ᛞ',
        ];

        let mut display_text = String::new();
        let chars: Vec<char> = self.text.chars().collect();

        for (i, &c) in chars.iter().enumerate() {
            let char_progress = (progress * chars.len() as f32) - i as f32;
            let char_progress = char_progress.clamp(0.0, 1.0);

            if char_progress >= 1.0 {
                display_text.push(c);
            } else if char_progress > 0.0 {
                // Cycle runes based on time
                let rune_idx = ((t * 20.0 + i as f32) as usize) % runes.len();
                display_text.push(runes[rune_idx]);
            } else {
                // Hidden or first rune
                display_text.push(' ');
            }
        }

        let color = theme::accent(); // Cyan deciphering glow
        renderer.draw_text(
            &display_text,
            rect.x,
            rect.y + rect.height * 0.8,
            rect.height * 0.8,
            color,
        );
    }
}
