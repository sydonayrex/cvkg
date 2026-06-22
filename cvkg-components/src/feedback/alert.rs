//! Alert — Non-modal inline alert component.
//!
//! Displays an inline alert message with an icon, title, description,
//! and optional action button. Unlike AlertDialog, this is non-modal
//! and can be embedded inline in layouts.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// An inline alert component.
///
/// Displays a styled alert box with icon, title, description, and optional action.
///
/// # Examples
/// ```
/// use cvkg_components::Alert;
/// let alert = Alert::new("Changes saved")
///         .description("Your changes have been saved successfully.")
///     .variant(AlertVariant::Success);
/// ```
#[derive(Clone)]
pub struct Alert {
    /// The alert title.
    pub title: String,
    /// Optional description text.
    pub description: String,
    /// Visual variant.
    pub variant: AlertVariant,
    /// Optional action button text.
    pub action: Option<String>,
}

/// Alert visual variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertVariant {
    /// Informational alert (blue).
    Info,
    /// Success alert (green).
    Success,
    /// Warning alert (amber).
    Warning,
    /// Error/danger alert (red).
    Error,
}

impl Alert {
    /// Create a new Alert with the given title.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            description: String::new(),
            variant: AlertVariant::Info,
            action: None,
        }
    }

    /// Set the description text.
    pub fn description(mut self, text: &str) -> Self {
        self.description = text.to_string();
        self
    }

    /// Set the visual variant.
    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the action button text.
    pub fn action(mut self, text: &str) -> Self {
        self.action = Some(text.to_string());
        self
    }
}

impl View for Alert {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!("Primitive view has no body")
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let (icon_color, bg_color) = match self.variant {
            AlertVariant::Info => (theme::info(), [0.05, 0.1, 0.2, 0.8]),
            AlertVariant::Success => (theme::success(), [0.02, 0.15, 0.05, 0.8]),
            AlertVariant::Warning => (theme::warning(), [0.15, 0.1, 0.02, 0.8]),
            AlertVariant::Error => (theme::error(), [0.15, 0.02, 0.02, 0.8]),
        };

        let padding = 12.0;
        let corner_radius = 8.0;

        // Background
        renderer.fill_rounded_rect(rect, corner_radius, bg_color);

        // Left accent bar
        let bar_rect = Rect {
            x: rect.x,
            y: rect.y + padding,
            width: 4.0,
            height: rect.height - padding * 2.0,
        };
        renderer.fill_rect(bar_rect, icon_color);

        // Icon area (simplified colored square)
        let icon_size = 20.0;
        let icon_rect = Rect {
            x: rect.x + padding + 8.0,
            y: rect.y + padding,
            width: icon_size,
            height: icon_size,
        };
        renderer.fill_rounded_rect(icon_rect, 4.0, icon_color);

        // Title text
        let text_x = icon_rect.x + icon_size + padding;
        let mut text_y = rect.y + padding;
        let (tw, th) = renderer.measure_text(&self.title, 14.0);
        renderer.draw_text(&self.title, text_x, text_y, 14.0, theme::text());

        // Description text
        if !self.description.is_empty() {
            text_y += th + 4.0;
            let (dw, _dh) = renderer.measure_text(&self.description, 12.0);
            renderer.draw_text(&self.description, text_x, text_y, 12.0, theme::text_dim());
        }

        // Action button (if present)
        if let Some(ref action_text) = self.action {
            let action_x = rect.x + rect.width - padding - 80.0;
            let action_y = rect.y + padding;
            renderer.fill_rounded_rect(
                Rect {
                    x: action_x,
                    y: action_y,
                    width: 80.0,
                    height: 24.0,
                },
                4.0,
                icon_color,
            );
            let (aw, ah) = renderer.measure_text(action_text, 12.0);
            renderer.draw_text(
                action_text,
                action_x + (80.0 - aw) / 2.0,
                action_y + (24.0 - ah) / 2.0,
                12.0,
                theme::text(),
            );
        }
    }
}
