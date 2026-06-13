//! RuneInspector — Detachable floating inspector panel.
//! Named after the runic tablets used by Norse scholars.

use cvkg_core::{Never, Rect, Renderer, View};

pub struct RuneInspector {
    pub title: String,
    pub position: InspectorPosition,
    pub size: (f32, f32),
    pub is_expanded: bool,
}

pub enum InspectorPosition {
    TrailingAttached,
    Floating { x: f32, y: f32 },
}

impl RuneInspector {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            position: InspectorPosition::Floating { x: 100.0, y: 100.0 },
            size: (280.0, 400.0),
            is_expanded: true,
        }
    }
}

impl View for RuneInspector {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Glass background (heavier blur than toolbars)
        renderer.bifrost(rect, 30.0, 1.3, 0.75);
        renderer.fill_rounded_rect(rect, 12.0, [0.06, 0.06, 0.08, 0.9]);

        // Title bar
        let _title_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: 36.0,
        };
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: 12.0,
            },
            12.0,
            [0.08, 0.08, 0.1, 0.5],
        );
        renderer.draw_text(
            &self.title,
            rect.x + 12.0,
            rect.y + 10.0,
            13.0,
            [0.9, 0.9, 0.92, 1.0],
        );

        // Close button
        let close_rect = Rect {
            x: rect.x + rect.width - 28.0,
            y: rect.y + 8.0,
            width: 20.0,
            height: 20.0,
        };
        renderer.fill_ellipse(close_rect, [0.8, 0.2, 0.2, 0.9]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspector_new() {
        let inspector = RuneInspector::new("Properties");
        assert_eq!(inspector.title, "Properties");
        assert_eq!(inspector.size, (280.0, 400.0));
        assert!(inspector.is_expanded);
        assert!(matches!(
            inspector.position,
            InspectorPosition::Floating { .. }
        ));
    }
}
