use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// EmptyState - A placeholder component for empty content areas.
///
/// # Examples
/// ```
/// use cvkg_components::EmptyState;
/// let empty = EmptyState::new("No items found", "Try adding your first item to get started.")
///     .icon("📦")
///     .action("Add Item", || {});
/// ```
#[derive(Clone)]
pub struct EmptyState {
    pub title: String,
    pub description: String,
    pub icon: String,
    pub action_label: Option<String>,
}

impl EmptyState {
    pub fn new(title: &str, description: &str) -> Self {
        Self {
            title: title.to_string(),
            description: description.to_string(),
            icon: String::new(),
            action_label: None,
        }
    }

    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = icon.to_string();
        self
    }

    pub fn action(mut self, label: &str, _callback: impl Fn() + 'static) -> Self {
        self.action_label = Some(label.to_string());
        self
    }
}

impl View for EmptyState {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!("Primitive view has no body")
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let cx = rect.x + rect.width / 2.0;
        let mut cy = rect.y + rect.height / 2.0 - 20.0;

        if !self.icon.is_empty() {
            let (tw, th) = renderer.measure_text(&self.icon, 32.0);
            renderer.draw_text(&self.icon, cx - tw / 2.0, cy - th / 2.0, 32.0, theme::text_dim());
            cy += 24.0;
        }

        let (tw, th) = renderer.measure_text(&self.title, 16.0);
        renderer.draw_text(&self.title, cx - tw / 2.0, cy, 16.0, theme::text());
        cy += th + 8.0;

        let (dw, dh) = renderer.measure_text(&self.description, 12.0);
        renderer.draw_text(&self.description, cx - dw / 2.0, cy, 12.0, theme::text_dim());
        cy += dh + 16.0;

        if let Some(ref action) = self.action_label {
            let (aw, ah) = renderer.measure_text(action, 14.0);
            let btn_rect = Rect {
                x: cx - aw / 2.0 - 12.0,
                y: cy - ah / 2.0 - 6.0,
                width: aw + 24.0,
                height: ah + 12.0,
            };
            renderer.fill_rounded_rect(btn_rect, 6.0, theme::accent());
            renderer.draw_text(action, btn_rect.x + 12.0, btn_rect.y + 6.0, 14.0, theme::text());
        }
    }
}
