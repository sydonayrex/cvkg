//! HoverCard component for hover-triggered floating content cards.
//!
//! Displays a floating card when the user hovers over a trigger element,
//! with a configurable delay before showing and hiding.

use crate::theme;
use crate::{FONT_SM, RADIUS_LG, SPACE_MD};
use cvkg_core::{Never, Rect, Renderer, View};
use std::cell::Cell;

/// HoverCard - A floating card that appears on hover with a delay.
///
/// The trigger content is rendered inline. When the pointer hovers over
/// the trigger area, the card content is displayed in a floating panel
/// after the specified delay.
///
/// # Example
/// ```
/// use cvkg_components::hover_card::HoverCard;
/// let card = HoverCard::new(
///     "Hover over me",
///     "This is the card content that appears on hover.",
/// )
/// .delay_ms(300);
/// ```
#[derive(Clone)]
pub struct HoverCard {
    /// The trigger text that the user hovers over.
    trigger: String,
    /// The content text displayed inside the floating card.
    content: String,
    /// Delay in milliseconds before showing the card.
    delay_ms: u64,
    /// Preferred position of the card relative to the trigger.
    position: HoverCardPosition,
    /// When the pointer entered the trigger area. None = not hovering.
    hover_start: Cell<Option<std::time::Instant>>,
}

/// Position options for the hover card relative to its trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HoverCardPosition {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

impl HoverCard {
    /// Create a new HoverCard with trigger and content text.
    pub fn new(trigger: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            trigger: trigger.into(),
            content: content.into(),
            delay_ms: 500,
            position: HoverCardPosition::Bottom,
            hover_start: Cell::new(None),
        }
    }

    /// Set the hover delay in milliseconds.
    pub fn delay_ms(mut self, ms: u64) -> Self {
        self.delay_ms = ms;
        self
    }

    /// Set the position of the card relative to the trigger.
    pub fn position(mut self, pos: HoverCardPosition) -> Self {
        self.position = pos;
        self
    }
}

impl View for HoverCard {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "HoverCard");

        let trigger_rect = rect;

        // Render trigger text
        renderer.push_vnode(trigger_rect, "HoverCardTrigger");
        renderer.draw_text(
            &self.trigger,
            rect.x,
            rect.y + (rect.height - FONT_SM) / 2.0,
            FONT_SM,
            theme::accent(),
        );
        renderer.pop_vnode();

        // Check hover state via pointer position
        let [px, py] = renderer.get_pointer_position();
        let is_hovering = trigger_rect.contains(px, py);
        // Update hover_start timer
        let hover_start = if is_hovering {
            if self.hover_start.get().is_none() {
                self.hover_start.set(Some(std::time::Instant::now()));
            }
            self.hover_start.get()
        } else {
            self.hover_start.set(None);
            None
        };

        // Only show card after delay has elapsed
        let show_card = hover_start.map_or(false, |start| {
            start.elapsed().as_millis() as u64 >= self.delay_ms
        });

        // Compute card dimensions
        let (tw, _) = renderer.measure_text(&self.content, FONT_SM);
        let card_w = (tw + SPACE_MD * 2.0).max(180.0).min(320.0);
        let card_h = 80.0f32;
        let gap = 8.0f32;

        let card_rect = match self.position {
            HoverCardPosition::Top => Rect {
                x: rect.x,
                y: rect.y - card_h - gap,
                width: card_w,
                height: card_h,
            },
            HoverCardPosition::Bottom => Rect {
                x: rect.x,
                y: rect.y + rect.height + gap,
                width: card_w,
                height: card_h,
            },
            HoverCardPosition::Left => Rect {
                x: rect.x - card_w - gap,
                y: rect.y,
                width: card_w,
                height: card_h,
            },
            HoverCardPosition::Right => Rect {
                x: rect.x + rect.width + gap,
                y: rect.y,
                width: card_w,
                height: card_h,
            },
        };

        if show_card {
            renderer.push_vnode(card_rect, "HoverCardContent");

            // Glass background
            renderer.bifrost(card_rect, 15.0, 1.5, 0.95);
            renderer.fill_rounded_rect(
                card_rect,
                RADIUS_LG,
                theme::with_alpha(theme::surface(), 0.92),
            );
            renderer.stroke_rounded_rect(card_rect, RADIUS_LG, theme::border(), 1.0);

            // Content text
            renderer.draw_text(
                &self.content,
                card_rect.x + SPACE_MD,
                card_rect.y + (card_rect.height - FONT_SM) / 2.0,
                FONT_SM,
                theme::text(),
            );

            renderer.pop_vnode();
        }

        renderer.pop_vnode();
    }
}
