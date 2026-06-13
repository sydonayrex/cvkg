use crate::{Button, ButtonVariant, FONT_BASE, RADIUS_MD};
use cvkg_core::{Never, Rect, Renderer, View};

/// ButtonGroup -- a segmented container for buttons.
/// Renders buttons joined together with shared borders, no gap between them.
#[derive(Clone)]
pub struct ButtonGroup {
    pub(crate) children: Vec<Button>,
    pub(crate) variant: ButtonVariant,
    pub(crate) spacing: f32,
}

impl ButtonGroup {
    pub fn new(children: Vec<Button>) -> Self {
        Self {
            children,
            variant: ButtonVariant::Secondary,
            spacing: 0.0,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl View for ButtonGroup {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.children.is_empty() {
            return;
        }

        let count = self.children.len() as f32;
        let total_spacing = (count - 1.0) * self.spacing;
        let item_width = (rect.width - total_spacing) / count;
        let item_height = rect.height;

        for (i, button) in self.children.iter().enumerate() {
            let x = rect.x + i as f32 * (item_width + self.spacing);
            let y = rect.y;

            // Draw background for each segment
            let bg_color = match self.variant {
                ButtonVariant::Default => [0.15, 0.15, 0.2, 1.0],
                ButtonVariant::Secondary => [0.1, 0.1, 0.15, 1.0],
                ButtonVariant::Ghost => [0.0, 0.0, 0.0, 0.0],
                ButtonVariant::Destructive => [0.6, 0.1, 0.1, 1.0],
                ButtonVariant::Link => [0.0, 0.0, 0.0, 0.0],
                ButtonVariant::Glass => [0.15, 0.15, 0.2, 0.6],
                ButtonVariant::TintedGlass => [0.2, 0.15, 0.25, 0.6],
                ButtonVariant::Capsule => [0.15, 0.15, 0.2, 1.0],
            };

            let segment_rect = Rect {
                x,
                y,
                width: item_width,
                height: item_height,
            };

            // Draw rounded rect with appropriate corner radius
            let is_first = i == 0;
            let is_last = i == self.children.len() - 1;

            if self.spacing == 0.0 {
                // Joined buttons: only round outer corners
                let radius = RADIUS_MD;
                if is_first && is_last {
                    renderer.fill_rounded_rect(segment_rect, radius, bg_color);
                } else if is_first {
                    // Round left corners only -- approximate with full radius
                    renderer.fill_rounded_rect(segment_rect, radius, bg_color);
                } else if is_last {
                    renderer.fill_rounded_rect(segment_rect, radius, bg_color);
                } else {
                    renderer.fill_rect(segment_rect, bg_color);
                }

                // Draw separator between buttons
                if !is_last {
                    let sep_color = [0.3, 0.3, 0.35, 1.0];
                    let sep_rect = Rect {
                        x: x + item_width - 0.5,
                        y: y + 4.0,
                        width: 1.0,
                        height: item_height - 8.0,
                    };
                    renderer.fill_rect(sep_rect, sep_color);
                }
            } else {
                renderer.fill_rounded_rect(segment_rect, RADIUS_MD, bg_color);
            }

            // Draw button label
            let label = button.label.clone();
            let label_color = [0.9, 0.9, 0.95, 1.0];
            let (text_w, text_h) = renderer.measure_text(&label, FONT_BASE);
            let text_x = x + (item_width - text_w) / 2.0;
            let text_y = y + (item_height - text_h) / 2.0;
            renderer.draw_text(&label, text_x, text_y, FONT_BASE, label_color);
        }
    }
}
