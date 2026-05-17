use cvkg_core::{Never, Rect, Renderer, View};

/// A top/bottom notification system.
/// Represents 'Thought' (Huginn) and 'Memory' (Muninn).
/// Section 4.4: "Avian messaging protocols for high-priority alerts."
pub struct RavenMessenger {
    pub message: String,
    pub is_huginn: bool, // Top
    pub duration: f32,
}

impl RavenMessenger {
    pub fn top(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            is_huginn: true,
            duration: 3.0,
        }
    }

    pub fn bottom(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            is_huginn: false,
            duration: 3.0,
        }
    }
}

impl View for RavenMessenger {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();

        // 1. Sliding Animation
        let offset = (t * 2.0).min(1.0).sin(); // Slide in
        let y = if self.is_huginn {
            rect.y - 100.0 + offset * 120.0
        } else {
            rect.y + rect.height + 100.0 - offset * 120.0
        };

        let msg_rect = Rect {
            x: rect.x + 20.0,
            y,
            width: rect.width - 40.0,
            height: 60.0,
        };

        // 2. Raven Wing Background (Feathered Edge)
        renderer.fill_rounded_rect(msg_rect, 8.0, [0.05, 0.05, 0.08, 0.95]);
        renderer.stroke_rounded_rect(msg_rect, 8.0, [0.2, 0.2, 0.3, 0.8], 1.0);

        // 3. Runic Pulse icon
        let icon = if self.is_huginn { 'ᚻ' } else { 'ᛗ' }; // H for Huginn, M for Muninn
        let pulse = 0.5 + (t * 4.0).sin() * 0.5;
        renderer.draw_text(
            &icon.to_string(),
            msg_rect.x + 15.0,
            msg_rect.y + 35.0,
            24.0,
            [0.0, 1.0, 1.0, pulse],
        );

        // 4. Message Text
        renderer.draw_text(
            &self.message,
            msg_rect.x + 50.0,
            msg_rect.y + 35.0,
            16.0,
            [1.0, 1.0, 1.0, 0.9],
        );
    }
}
