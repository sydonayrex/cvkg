//! TestimonialCard component for landing pages.
//!
//! A card component displaying customer quotes with author and avatar.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// Testimonial quote item.
#[derive(Clone)]
pub struct TestimonialItem {
    quote: String,
    author: String,
    avatar: Option<String>, // Could be URL or placeholder
}

impl TestimonialItem {
    /// Create a new TestimonialItem.
    pub fn new(quote: impl Into<String>, author: impl Into<String>) -> Self {
        Self {
            quote: quote.into(),
            author: author.into(),
            avatar: None,
        }
    }

    /// Set the avatar (URL or placeholder).
    pub fn avatar(mut self, avatar: impl Into<String>) -> Self {
        self.avatar = Some(avatar.into());
        self
    }
}

/// TestimonialCard for displaying customer testimonials.
///
/// ## Accessibility
/// - Role: `figure` with `aria-label="Testimonial"`
/// - Keyboard: Tab to focus interactive elements
/// - Focus: children receive focus as normal
/// - ARIA: `aria-label` from author, `aria-describedby` for quote content
pub struct TestimonialCard {
    items: Vec<TestimonialItem>,
}

impl TestimonialCard {
    /// Create a new TestimonialCard.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add a testimonial item.
    pub fn item(mut self, item: TestimonialItem) -> Self {
        self.items.push(item);
        self
    }
}

impl Default for TestimonialCard {
    fn default() -> Self {
        Self::new()
    }
}

impl View for TestimonialCard {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Card background
        renderer.fill_rounded_rect(rect, 12.0, theme::surface_elevated());
        renderer.stroke_rounded_rect(rect, 12.0, theme::border(), 1.0);

        for (i, item) in self.items.iter().enumerate() {
            let item_y = rect.y + 20.0 + i as f32 * 80.0;

            // Quote mark
            renderer.draw_text_raw(
                &"\"".to_string(),
                rect.x + 20.0,
                item_y,
                24.0,
                theme::accent(),
            );

            // Quote text (truncated for display)
            let display_quote = if item.quote.len() > 60 {
                format!("{}...", &item.quote[..60])
            } else {
                item.quote.clone()
            };
            renderer.draw_text_raw(
                &display_quote,
                rect.x + 20.0,
                item_y + 20.0,
                14.0,
                theme::text_muted(),
            );

            // Author
            renderer.draw_text_raw(
                &format!("- {}", item.author),
                rect.x + 20.0,
                item_y + 50.0,
                12.0,
                theme::text(),
            );
        }
    }
}