//! Confidence visualization for AI-generated content.
//!
//! `TrustMark` wraps any view with a visual indicator of model confidence,
//! helping readers calibrate their trust in generated output. Research shows
//! that surfacing uncertainty reduces over-reliance on AI systems.
//!
//! The component uses a left-side border rather than a full border because
//! full borders are visually ambiguous with form validation errors. Colors
//! are chosen to be deuteranopia-safe (no red/green confusion).

use crate::SPACE_XS;
use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// Width of the left trust indicator border in logical pixels.
const TRUST_BORDER_WIDTH: f32 = 3.0;

/// Radius of the indicator icon drawn at the top-right corner.
const INDICATOR_RADIUS: f32 = 4.0;

/// Confidence bands for AI-generated content.
///
/// Discrete bands are used instead of continuous percentages because
/// fine-grained precision is misleading — models cannot self-assess
/// their own accuracy with that resolution. The bands map to qualitative
/// judgments that are more actionable for end users.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustLevel {
    /// High confidence: the model is likely correct (>85%).
    /// Shown as a filled teal indicator.
    High,
    /// Medium confidence: treat with moderate skepticism (60-85%).
    /// Shown as a filled yellow indicator.
    Medium,
    /// Low confidence: significant uncertainty, verify independently (30-60%).
    /// Shown as a hollow orange indicator.
    Low,
    /// Very low confidence: essentially a guess (<30%).
    /// Shown as a hollow red indicator.
    VeryLow,
    /// Unknown: confidence could not be determined.
    /// Shown as a "?" mark.
    Unknown,
}

impl TrustLevel {
    /// Returns the deuteranopia-safe display color for this trust level.
    ///
    /// Colors avoid the red-green axis and instead use teal, yellow,
    /// orange, and red at different luminance levels, which remain
    /// distinguishable under deuteranopia (the most common color vision
    /// deficiency, affecting ~6% of males).
    fn color(self) -> [f32; 4] {
        match self {
            TrustLevel::High => theme::success(),         // teal #00E676
            TrustLevel::Medium => theme::warning(),       // yellow #FFB300
            TrustLevel::Low => theme::warning_orange(),   // orange #FF8000
            TrustLevel::VeryLow => theme::critical_red(), // red #FF3333
            TrustLevel::Unknown => theme::text_dim(),     // gray, adaptive
        }
    }

    /// Returns the indicator style for this trust level.
    fn indicator_style(self) -> IndicatorStyle {
        match self {
            TrustLevel::High => IndicatorStyle::Filled,
            TrustLevel::Medium => IndicatorStyle::Filled,
            TrustLevel::Low => IndicatorStyle::Hollow,
            TrustLevel::VeryLow => IndicatorStyle::Hollow,
            TrustLevel::Unknown => IndicatorStyle::QuestionMark,
        }
    }
}

/// Visual style of the top-right indicator icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IndicatorStyle {
    /// Filled circle — signals presence of information.
    Filled,
    /// Hollow (stroked) circle — signals partial or degraded confidence.
    Hollow,
    /// Question mark — signals that confidence is not available.
    QuestionMark,
}

/// A wrapper view that adds a trust confidence indicator around any content.
///
/// `TrustMark` renders a thin colored border on the left edge of its child
/// and a small indicator icon at the top-right corner. The child view
/// receives the full allocated rect; the trust decorations are drawn as
/// overlays so they do not affect layout.
///
/// # Why a left border only
///
/// A full border around content is easily confused with form field
/// validation states (error/success). A left border is a conventional
/// pattern for "annotation" or "status" that does not clash with form UI.
///
/// # Example
/// ```
/// use cvkg_components::{TrustMark, TrustLevel, Text};
/// let view = TrustMark::new(Text::new("Generated answer…"), TrustLevel::Medium);
/// ```
#[derive(Clone)]
pub struct TrustMark<V: View> {
    /// The wrapped child view that displays the actual content.
    content: V,
    /// The confidence level that determines border and indicator styling.
    level: TrustLevel,
}

impl<V: View> TrustMark<V> {
    /// Create a new `TrustMark` wrapping `content` with the given `level`.
    ///
    /// # Contract
    /// - The child view is rendered at the full allocated rect.
    /// - The trust border and indicator are drawn on top without shrinking content area.
    pub fn new(content: V, level: TrustLevel) -> Self {
        Self { content, level }
    }
}

impl<V: View> View for TrustMark<V> {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let color = self.level.color();

        // Render the child content at the full rect first (underlay).
        self.content.render(renderer, rect);

        // Draw the left trust border as a thin filled rectangle.
        let border_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: TRUST_BORDER_WIDTH,
            height: rect.height,
        };
        renderer.fill_rect(border_rect, color);

        // Draw the indicator icon at the top-right corner.
        self.draw_indicator(renderer, rect, color);
    }
}

impl<V: View> TrustMark<V> {
    /// Draws the indicator icon at the top-right corner of the content rect.
    fn draw_indicator(&self, renderer: &mut dyn Renderer, rect: Rect, color: [f32; 4]) {
        let cx = rect.x + rect.width - SPACE_XS - INDICATOR_RADIUS;
        let cy = rect.y + SPACE_XS + INDICATOR_RADIUS;

        match self.level.indicator_style() {
            IndicatorStyle::Filled => {
                renderer.fill_ellipse(
                    Rect {
                        x: cx - INDICATOR_RADIUS,
                        y: cy - INDICATOR_RADIUS,
                        width: INDICATOR_RADIUS * 2.0,
                        height: INDICATOR_RADIUS * 2.0,
                    },
                    color,
                );
            }
            IndicatorStyle::Hollow => {
                renderer.stroke_ellipse(
                    Rect {
                        x: cx - INDICATOR_RADIUS,
                        y: cy - INDICATOR_RADIUS,
                        width: INDICATOR_RADIUS * 2.0,
                        height: INDICATOR_RADIUS * 2.0,
                    },
                    color,
                    1.5,
                );
            }
            IndicatorStyle::QuestionMark => {
                renderer.draw_text("?", cx - 3.0, cy + 4.0, 10.0, color);
            }
        }
    }
}

/// Extension trait that adds `.trustmark(level)` to any `View`.
///
/// This provides a fluent, chainable API consistent with the crate's
/// other extension traits (e.g., `ViewExt::sheet()`). Implementing it
/// as a blanket trait means any view can be wrapped with a trust
/// indicator without modifying the original view type.
///
/// # Example
/// ```
/// use cvkg_components::{TrustExt, TrustLevel, Text};
/// let view = Text::new("AI-generated summary").trustmark(TrustLevel::High);
/// ```
pub trait TrustExt: View + Sized {
    /// Wrap this view with a confidence indicator at the given trust level.
    fn trustmark(self, level: TrustLevel) -> TrustMark<Self> {
        TrustMark::new(self, level)
    }
}

// Blanket implementation: any View can be given a trustmark.
impl<T: View + Sized> TrustExt for T {}
