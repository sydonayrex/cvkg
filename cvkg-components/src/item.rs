//! Item component for generic list items.
//!
//! A flexible list item with optional title, subtitle, leading/trailing
//! elements, click handler, and disabled state.

use crate::theme;
use crate::{FONT_BASE, FONT_SM, RADIUS_MD, SPACE_SM};
use cvkg_core::{Never, Rect, Renderer, View};
use std::sync::Arc;

/// Item - A generic list item component.
///
/// Renders a horizontal row with optional leading content (icon/avatar),
/// a title and optional subtitle in the middle, and optional trailing
/// content (badge, arrow, etc.). Supports click interaction and disabled state.
///
/// # Example
/// ```
/// use cvkg_components::item::Item;
/// let item = Item::new("Settings")
///     .subtitle("App preferences and configuration")
///     .leading("⚙")
///     .trailing("›");
/// ```
#[derive(Clone)]
pub struct Item {
    /// Primary title text.
    title: String,
    /// Optional subtitle text displayed below the title.
    subtitle: Option<String>,
    /// Optional leading element (icon, avatar, etc.).
    leading: Option<String>,
    /// Optional trailing element (badge, arrow, etc.).
    trailing: Option<String>,
    /// Optional click callback.
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Whether this item is disabled.
    disabled: bool,
}

impl Item {
    /// Create a new Item with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            leading: None,
            trailing: None,
            on_click: None,
            disabled: false,
        }
    }

    /// Set the subtitle text.
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set the leading element text.
    pub fn leading(mut self, leading: impl Into<String>) -> Self {
        self.leading = Some(leading.into());
        self
    }

    /// Set the trailing element text.
    pub fn trailing(mut self, trailing: impl Into<String>) -> Self {
        self.trailing = Some(trailing.into());
        self
    }

    /// Set the click callback.
    pub fn on_click(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(callback));
        self
    }

    /// Set whether this item is disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Compute the item height based on content.
    #[allow(dead_code)]
    fn item_height(&self) -> f32 {
        if self.subtitle.is_some() { 64.0 } else { 48.0 }
    }
}

impl View for Item {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Item");

        let bg_color = if self.disabled {
            theme::disabled()
        } else {
            theme::surface()
        };

        // Background
        renderer.fill_rounded_rect(rect, RADIUS_MD, bg_color);

        // Hover highlight (subtle)
        if !self.disabled {
            let [px, py] = renderer.get_pointer_position();
            if rect.contains(px, py) {
                renderer.fill_rounded_rect(rect, RADIUS_MD, theme::hover());
            }
        }

        let mut content_x = rect.x + SPACE_SM;
        let has_subtitle = self.subtitle.is_some();
        let text_y = if has_subtitle {
            rect.y + 12.0
        } else {
            rect.y + (rect.height - FONT_BASE) / 2.0
        };

        // Leading element
        if let Some(ref lead) = self.leading {
            renderer.draw_text_raw(lead, content_x, text_y, FONT_BASE, theme::text());
            let (lw, _) = renderer.measure_text(lead, FONT_BASE);
            content_x += lw + SPACE_SM;
        }

        // Title
        let title_color = if self.disabled {
            theme::disabled_text()
        } else {
            theme::text()
        };
        renderer.draw_text_raw(&self.title, content_x, text_y, FONT_BASE, title_color);

        // Subtitle
        if let Some(ref sub) = self.subtitle {
            renderer.draw_text_raw(
                sub,
                content_x,
                text_y + FONT_BASE + 4.0,
                FONT_SM,
                theme::text_muted(),
            );
        }

        // Trailing element (right-aligned)
        if let Some(ref trail) = self.trailing {
            let (tw, _) = renderer.measure_text(trail, FONT_BASE);
            renderer.draw_text_raw(
                trail,
                rect.x + rect.width - tw - SPACE_SM,
                text_y,
                FONT_BASE,
                theme::text_muted(),
            );
        }

        // Click handler
        if !self.disabled
            && let Some(ref cb) = self.on_click
        {
            let cb = cb.clone();
            renderer.register_handler(
                "pointerclick",
                Arc::new(move |_| {
                    (cb)();
                }),
            );
        }

        renderer.pop_vnode();
    }
}
