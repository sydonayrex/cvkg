//! Semantic style values shared by components.
//!
//! A `Style` carries meaning, not resolved RGBA. Each backend resolves a
//! semantic token against its active theme. ``merge`` applies state overlays
//! so five widget states share one definition.

/// Semantic color roles used by UI state overlays.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticColor {
    Primary,
    Secondary,
    Muted,
    Border,
    Success,
    Danger,
    Warning,
    Text,
    Surface,
}

/// A resolvable component style.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style {
    pub background: Option<SemanticColor>,
    pub foreground: Option<SemanticColor>,
    pub border: Option<SemanticColor>,
    pub radius: Option<f32>,
    pub disabled_background: Option<SemanticColor>,
    pub reduced_motion: bool,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn background(mut self, token: SemanticColor) -> Self {
        self.background = Some(token);
        self
    }

    pub fn disabled_background(mut self, token: SemanticColor) -> Self {
        self.disabled_background = Some(token);
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius);
        self
    }

    /// Overlay `other` on `self`. Explicit values in `other` win.
    pub fn merge(self, other: &Style) -> Self {
        Self {
            background: other.background.or(self.background),
            foreground: other.foreground.or(self.foreground),
            border: other.border.or(self.border),
            radius: other.radius.or(self.radius),
            disabled_background: other.disabled_background.or(self.disabled_background),
            reduced_motion: self.reduced_motion || other.reduced_motion,
        }
    }

    pub fn resolve_disabled_background(&self) -> Option<SemanticColor> {
        self.disabled_background
    }
}
