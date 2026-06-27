// =============================================================================
// THEME CONTEXT -- Thread-local theme access for components
// =============================================================================
//
// Components call `use_theme()` to get the current SemanticColors.
// The native renderer sets this via `set_current_theme()` before each frame.
// Falls back to dark theme defaults if no theme has been set.
//
// We store SemanticColors directly (not the full Theme) to avoid depending
// on cvkg-themes from cvkg-core. The colors are cloned into thread-local storage.

use crate::SemanticColors;
use std::cell::RefCell;

thread_local! {
    /// Thread-local theme context for the current frame.
    static THEME_CONTEXT: RefCell<Option<ThemeContext>> = const { RefCell::new(None) };
}

/// Theme context available to components during render.
/// Includes both semantic colors and visual effect flags.
#[derive(Debug, Clone)]
pub struct ThemeContext {
    /// Semantic colors for the current theme.
    pub colors: SemanticColors,
    /// If true, components may use glassmorphic effects (frosted glass, blur).
    /// If false, components should render with solid backgrounds.
    pub glassmorphism_enabled: bool,
}

impl ThemeContext {
    /// Create a dark theme context with glassmorphism enabled.
    pub fn dark() -> Self {
        Self {
            colors: SemanticColors::dark(),
            glassmorphism_enabled: true,
        }
    }

    /// Create a light theme context with glassmorphism disabled.
    pub fn light() -> Self {
        Self {
            colors: SemanticColors::light(),
            glassmorphism_enabled: false,
        }
    }
}

/// Set the current theme context for this thread.
/// Called by the native renderer before each frame.
pub fn set_current_theme(colors: SemanticColors) {
    THEME_CONTEXT.with(|cell| {
        let is_light =
            (colors.background.r + colors.background.g + colors.background.b) / 3.0 > 0.5;
        let glassmorphism = !is_light; // light themes default to no glassmorphism
        *cell.borrow_mut() = Some(ThemeContext {
            colors,
            glassmorphism_enabled: glassmorphism,
        });
    });
}

/// Set the full theme context (including glassmorphism flag).
pub fn set_theme_context(ctx: ThemeContext) {
    THEME_CONTEXT.with(|cell| {
        *cell.borrow_mut() = Some(ctx);
    });
}

/// Clear the current theme. Called after each frame.
pub fn clear_current_theme() {
    THEME_CONTEXT.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

/// Access the current semantic colors from within a component's `render()` method.
///
/// Returns the colors set by the most recent `set_current_theme()` call.
/// Falls back to dark theme defaults if no theme has been set.
///
/// # Example
/// ```no_run
/// use cvkg_core::{use_theme, Renderer, Rect};
///
/// fn render_button(renderer: &mut dyn Renderer, rect: Rect) {
///     let colors = use_theme();
///     renderer.fill_rounded_rect(rect, 8.0, [colors.accent.r, colors.accent.g, colors.accent.b, colors.accent.a]);
/// }
/// ```
pub fn use_theme() -> SemanticColors {
    THEME_CONTEXT.with(|cell| {
        cell.borrow()
            .clone()
            .map(|ctx| ctx.colors)
            .unwrap_or_else(SemanticColors::dark)
    })
}

/// Access the full theme context from within a component's `render()` method.
///
/// Returns the current `ThemeContext` including both colors and effect flags.
/// Falls back to dark theme defaults if no theme has been set.
pub fn use_theme_context() -> ThemeContext {
    THEME_CONTEXT.with(|cell| cell.borrow().clone().unwrap_or_else(ThemeContext::dark))
}

/// Returns true if glassmorphic effects are enabled in the current theme.
/// Components should check this before calling `renderer.bifrost()`.
pub fn glassmorphism_enabled() -> bool {
    THEME_CONTEXT.with(|cell| {
        cell.borrow()
            .as_ref()
            .map(|ctx| ctx.glassmorphism_enabled)
            .unwrap_or(true) // default: glassmorphism on (dark theme)
    })
}
