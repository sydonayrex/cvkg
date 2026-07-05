//! PricingTable component for landing pages.
//!
//! A grid of PricingCards for displaying pricing options.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// Pricing card for displaying a single pricing option.
#[derive(Clone)]
pub struct PricingCard {
    title: String,
    price: String,
    features: Vec<String>,
    cta_text: String,
}

impl PricingCard {
    /// Create a new PricingCard.
    pub fn new(title: impl Into<String>, price: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            price: price.into(),
            features: Vec::new(),
            cta_text: "Choose Plan".to_string(),
        }
    }

    /// Add a feature to the card.
    pub fn feature(mut self, feature: impl Into<String>) -> Self {
        self.features.push(feature.into());
        self
    }

    /// Set the CTA button text.
    pub fn cta_text(mut self, text: impl Into<String>) -> Self {
        self.cta_text = text.into();
        self
    }
}

/// PricingTable for displaying multiple pricing options.
///
/// ## Accessibility
/// - Role: `table` with `aria-label="Pricing options"`
/// - Keyboard: Tab to focus CTA buttons
/// - Focus: children receive focus as normal
/// - ARIA: Each card uses `role="row"` or `role="gridcell"`
pub struct PricingTable {
    cards: Vec<PricingCard>,
}

impl PricingTable {
    /// Create a new PricingTable.
    pub fn new() -> Self {
        Self { cards: Vec::new() }
    }

    /// Add a pricing card.
    pub fn card(mut self, card: PricingCard) -> Self {
        self.cards.push(card);
        self
    }
}

impl Default for PricingTable {
    fn default() -> Self {
        Self::new()
    }
}

impl View for PricingTable {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let card_width = rect.width / self.cards.len().max(1) as f32 - 20.0;
        for (i, card) in self.cards.iter().enumerate() {
            let card_x = rect.x + 10.0 + i as f32 * (card_width + 20.0);
            let card_rect = Rect {
                x: card_x,
                y: rect.y + 20.0,
                width: card_width,
                height: rect.height - 40.0,
            };
            renderer.fill_rounded_rect(card_rect, 8.0, theme::surface_elevated());
            renderer.stroke_rounded_rect(card_rect, 8.0, theme::border(), 1.0);

            // Render title
            let (_, title_h) = renderer.measure_text(&card.title, 18.0);
            renderer.draw_text_raw(
                &card.title,
                card_x + 20.0,
                card_rect.y + 20.0 + title_h / 2.0,
                18.0,
                theme::text(),
            );

            // Render price
            let (_, price_h) = renderer.measure_text(&card.price, 24.0);
            renderer.draw_text_raw(
                &card.price,
                card_x + 20.0,
                card_rect.y + 40.0 + price_h / 2.0,
                24.0,
                theme::accent(),
            );
        }
    }
}
