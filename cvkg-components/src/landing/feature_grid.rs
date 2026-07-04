//! FeatureGrid component for landing pages.
//!
//! A 3-column grid of FeatureCards for displaying product features.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// Feature item for the grid.
#[derive(Clone)]
pub struct FeatureItem {
    icon: String,
    title: String,
    description: String,
}

impl FeatureItem {
    /// Create a new FeatureItem.
    pub fn new(icon: impl Into<String>, title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            title: title.into(),
            description: description.into(),
        }
    }
}

/// FeatureGrid for displaying features in a grid layout.
///
/// ## Accessibility
/// - Role: `region` with `aria-label="Features"`
/// - Keyboard: Tab to focus interactive elements within features
/// - Focus: children receive focus as normal
/// - ARIA: Each feature uses `role="region"` with `aria-labelledby`
pub struct FeatureGrid {
    items: Vec<FeatureItem>,
    columns: usize,
}

impl FeatureGrid {
    /// Create a new FeatureGrid.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            columns: 3,
        }
    }

    /// Add a feature item.
    pub fn item(mut self, item: FeatureItem) -> Self {
        self.items.push(item);
        self
    }

    /// Set the number of columns.
    pub fn columns(mut self, cols: usize) -> Self {
        self.columns = cols.max(1);
        self
    }
}

impl Default for FeatureGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl View for FeatureGrid {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let cols = self.columns as f32;
        let item_width = (rect.width - (cols - 1.0) * 20.0) / cols;
        let item_height = 100.0;

        for (i, item) in self.items.iter().enumerate() {
            let col = i % self.columns;
            let row = i / self.columns;
            let x = rect.x + col as f32 * (item_width + 20.0);
            let y = rect.y + row as f32 * (item_height + 20.0);
            let item_rect = Rect {
                x,
                y,
                width: item_width,
                height: item_height,
            };

            renderer.fill_rounded_rect(item_rect, 8.0, theme::surface_elevated());
            renderer.stroke_rounded_rect(item_rect, 8.0, theme::border(), 1.0);

            // Render icon
            renderer.draw_text_raw(
                &item.icon,
                x + 16.0,
                y + 16.0,
                20.0,
                theme::accent(),
            );

            // Render title (height is unused)
            let _ = renderer.measure_text(&item.title, 16.0);
            renderer.draw_text_raw(
                &item.title,
                x + 16.0,
                y + 45.0,
                16.0,
                theme::text(),
            );

            // Render description
            renderer.draw_text_raw(
                &item.description,
                x + 16.0,
                y + 65.0,
                14.0,
                theme::text_muted(),
            );
        }
    }
}