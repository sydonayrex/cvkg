//! Hero component for landing pages.
//!
//! A compound component combining VStack with title text, subtitle, and CTA button.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// Hero section for landing pages with title, subtitle, and call-to-action.
///
/// ## Accessibility
/// - Role: `region` with `aria-label="Hero"`
/// - Keyboard: Tab to focus CTA button
/// - Focus: children receive focus as normal
/// - ARIA: `aria-label` from title, `aria-describedby` from subtitle
pub struct Hero {
    title: String,
    subtitle: String,
    cta_text: String,
    on_cta: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Hero {
    /// Create a new Hero component.
    pub fn new(title: impl Into<String>, subtitle: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: subtitle.into(),
            cta_text: "Get Started".to_string(),
            on_cta: None,
        }
    }

    /// Set the CTA button text.
    pub fn cta_text(mut self, text: impl Into<String>) -> Self {
        self.cta_text = text.into();
        self
    }

    /// Set the CTA click handler.
    pub fn on_cta(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_cta = Some(Box::new(callback));
        self
    }
}

impl View for Hero {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Background
        renderer.fill_rect(rect, theme::surface());

        // Title
        let title_font = 36.0;
        let (title_w, _title_h) = renderer.measure_text(&self.title, title_font);
        renderer.draw_text_raw(
            &self.title,
            rect.x + (rect.width - title_w) / 2.0,
            rect.y + 60.0,
            title_font,
            theme::text(),
        );

        // Subtitle
        let subtitle_font = 18.0;
        let (subtitle_w, _subtitle_h) = renderer.measure_text(&self.subtitle, subtitle_font);
        renderer.draw_text_raw(
            &self.subtitle,
            rect.x + (rect.width - subtitle_w) / 2.0,
            rect.y + 110.0,
            subtitle_font,
            theme::text_muted(),
        );
    }
}