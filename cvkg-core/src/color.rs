use crate::Color;

/// A complete set of semantic colors for UI components.
///
/// Each color serves a specific role in the UI. Components should reference
/// these semantic roles rather than hardcoding RGBA values.
///
/// # Example
/// ```no_run
/// use cvkg_core::{use_theme, Renderer, Rect};
///
/// fn render_button(renderer: &mut dyn Renderer, rect: Rect) {
///     let colors = use_theme();
///     // Use accent color for the button background
///     renderer.fill_rounded_rect(rect, 8.0,
///         [colors.accent.r, colors.accent.g, colors.accent.b, colors.accent.a]);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SemanticColors {
    /// Primary brand color -- used for key interactive elements.
    pub primary: Color,
    /// Secondary color -- used for less prominent interactive elements.
    pub secondary: Color,
    /// Accent color -- used for highlights, focus rings, CTAs.
    pub accent: Color,
    /// Page/window background color.
    pub background: Color,
    /// Surface color -- used for cards, panels, sheets.
    pub surface: Color,
    /// Error color -- used for destructive actions, error messages.
    pub error: Color,
    /// Warning color -- used for caution indicators.
    pub warning: Color,
    /// Success color -- used for positive feedback.
    pub success: Color,
    /// Primary text color.
    pub text: Color,
    /// Dimmed/disabled text color.
    pub text_dim: Color,
}

impl SemanticColors {
    /// Dark theme semantic colors (default fallback).
    pub fn dark() -> Self {
        Self {
            primary: Color::new(1.0, 0.84, 0.0, 1.0),      // Viking Gold
            secondary: Color::new(1.0, 0.0, 1.0, 1.0),     // Magenta Liquid
            accent: Color::new(1.0, 0.0, 0.4, 1.0),        // Crimson Flash
            background: Color::new(0.02, 0.02, 0.05, 1.0), // Deep Void
            surface: Color::new(0.05, 0.05, 0.07, 1.0),    // Tactical Obsidian
            error: Color::new(1.0, 0.2, 0.2, 1.0),         // Red
            warning: Color::new(1.0, 0.8, 0.0, 1.0),       // Yellow
            success: Color::new(0.0, 1.0, 0.5, 1.0),       // Green
            text: Color::new(0.95, 0.95, 1.0, 1.0),        // Near-white
            text_dim: Color::new(0.6, 0.6, 0.7, 1.0),      // Gray
        }
    }

    /// Light theme semantic colors.
    pub fn light() -> Self {
        Self {
            primary: Color::new(0.35, 0.30, 0.70, 1.0),
            secondary: Color::new(0.30, 0.50, 0.30, 1.0),
            accent: Color::new(0.30, 0.35, 0.75, 1.0),
            background: Color::new(0.97, 0.97, 0.98, 1.0),
            surface: Color::new(0.93, 0.93, 0.95, 1.0),
            error: Color::new(0.75, 0.15, 0.15, 1.0),
            warning: Color::new(0.80, 0.60, 0.0, 1.0),
            success: Color::new(0.15, 0.65, 0.30, 1.0),
            text: Color::new(0.08, 0.08, 0.10, 1.0),
            text_dim: Color::new(0.40, 0.40, 0.45, 1.0),
        }
    }

    /// Convert the accent color semantic color into interactive state colors.
    ///
    /// This provides hover/active/focus/disabled variants derived from the
    /// accent color, matching the pattern that `cvkg-themes::StateColors` uses.
    pub fn accent_states(&self) -> InteractiveColorStates {
        InteractiveColorStates::from_color(self.accent)
    }

    /// Convert the primary color into interactive state colors.
    pub fn primary_states(&self) -> InteractiveColorStates {
        InteractiveColorStates::from_color(self.primary)
    }

    /// Convert the error color into interactive state colors.
    pub fn error_states(&self) -> InteractiveColorStates {
        InteractiveColorStates::from_color(self.error)
    }

    /// Convert the success color into interactive state colors.
    pub fn success_states(&self) -> InteractiveColorStates {
        InteractiveColorStates::from_color(self.success)
    }
}

/// Interactive state colors derived from a single base color.
///
/// Provides hover/active/focus/disabled variants for any color,
/// derived via simple lightness adjustments in sRGB space.
#[derive(Debug, Clone)]
pub struct InteractiveColorStates {
    pub default: Color,
    pub hover: Color,
    pub active: Color,
    pub focus: Color,
    pub disabled: Color,
    pub focus_ring: Color,
}

impl InteractiveColorStates {
    /// Derive interactive state colors from a base sRGB color.
    ///
    /// Uses simple lightness adjustments:
    /// - Hover: +15% lightness
    /// - Active: -15% lightness
    /// - Focus: same as default
    /// - Disabled: 40% opacity
    /// - Focus ring: base color at 70% opacity
    pub fn from_color(base: Color) -> Self {
        Self {
            default: base,
            hover: base.lighten(0.15),
            active: base.darken(0.15),
            focus: base,
            disabled: Color::new(base.r, base.g, base.b, base.a * 0.4),
            focus_ring: Color::new(base.r, base.g, base.b, base.a * 0.7),
        }
    }

    /// Get the color for a specific interactive state.
    pub fn color_for(&self, state: InteractiveState) -> Color {
        match state {
            InteractiveState::Default => self.default,
            InteractiveState::Hover => self.hover,
            InteractiveState::Active => self.active,
            InteractiveState::Focus => self.focus,
            InteractiveState::Disabled => self.disabled,
        }
    }
}

/// Interactive state for a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractiveState {
    Default,
    Hover,
    Active,
    Focus,
    Disabled,
}
