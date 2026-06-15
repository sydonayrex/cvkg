//! Theme helpers for CVKG components.
//!
//! Provides convenient access to themed colors via `StyleResolver::color_array()`.
//! Every color used by a component should be resolved through this module,
//! never hardcoded as raw `[f32; 4]` arrays.
//!
//! Token keys match the semantic color tokens defined in cvkg-core's `default_tokens()`:
//!   - "background", "surface", "surface_elevated", "surface_overlay"
//!   - "primary", "secondary", "accent", "accent_hover"
//!   - "text", "text_muted", "text_dim"
//!   - "border", "border_strong"
//!   - "hover", "active", "disabled", "disabled_text"
//!   - "success", "warning", "error", "info"
//!   - "focus_ring", "shadow", "code_bg"

use cvkg_core::StyleResolver;

/// Resolve a themed color by token key. Returns `[f32; 4]` RGBA.
#[inline]
pub fn color(key: &str) -> [f32; 4] {
    StyleResolver::color_array(key)
}

// === Convenience wrappers for hot-path code ===

/// Background color (root canvas). Adaptive: black in dark, white in light.
#[inline]
pub fn bg() -> [f32; 4] {
    color("background")
}

/// Surface color (cards, panels). Adaptive.
#[inline]
pub fn surface() -> [f32; 4] {
    color("surface")
}

/// Elevated surface (dialogs, popovers). Adaptive.
#[inline]
pub fn surface_elevated() -> [f32; 4] {
    color("surface_elevated")
}

/// Overlay surface (modals, sheets). Adaptive.
#[inline]
pub fn surface_overlay() -> [f32; 4] {
    color("surface_overlay")
}

/// Primary text color. Adaptive.
#[inline]
pub fn text() -> [f32; 4] {
    color("text")
}

/// Muted text color (secondary text, placeholders). Adaptive.
#[inline]
pub fn text_muted() -> [f32; 4] {
    color("text_muted")
}

/// Dim text color (tertiary text, disabled hints). Adaptive.
#[inline]
pub fn text_dim() -> [f32; 4] {
    color("text_dim")
}

/// Border color. Adaptive.
#[inline]
pub fn border() -> [f32; 4] {
    color("border")
}

/// Strong border color (dividers, separators). Adaptive.
#[inline]
pub fn border_strong() -> [f32; 4] {
    color("border_strong")
}

/// Primary accent color (NiflCyan #00FFFF).
#[inline]
pub fn accent() -> [f32; 4] {
    color("accent")
}

/// Accent hover color (#33FFFF).
#[inline]
pub fn accent_hover() -> [f32; 4] {
    color("accent_hover")
}

/// Hover state background. Adaptive.
#[inline]
pub fn hover() -> [f32; 4] {
    color("hover")
}

/// Active/pressed state background. Adaptive.
#[inline]
pub fn active_color() -> [f32; 4] {
    color("active")
}

/// Disabled state background. Adaptive.
#[inline]
pub fn disabled() -> [f32; 4] {
    color("disabled")
}

/// Disabled text color. Adaptive.
#[inline]
pub fn disabled_text() -> [f32; 4] {
    color("disabled_text")
}

/// Success color (#00E676).
#[inline]
pub fn success() -> [f32; 4] {
    color("success")
}

/// Warning color (#FFB300).
#[inline]
pub fn warning() -> [f32; 4] {
    color("warning")
}

/// Error color (#FF5252).
#[inline]
pub fn error_color() -> [f32; 4] {
    color("error")
}

/// Info color (#448AFF).
#[inline]
pub fn info() -> [f32; 4] {
    color("info")
}

/// Focus ring color (NiflCyan #00FFFF).
#[inline]
pub fn focus_ring() -> [f32; 4] {
    color("focus_ring")
}

/// Override the alpha channel of any color.
#[inline]
pub fn with_alpha(c: [f32; 4], a: f32) -> [f32; 4] {
    [c[0], c[1], c[2], a]
}

/// Shadow color. Adaptive.
#[inline]
pub fn shadow() -> [f32; 4] {
    color("shadow")
}

/// Code block background. Adaptive.
#[inline]
pub fn code_bg() -> [f32; 4] {
    color("code_bg")
}

// === Derived helpers ===

/// Primary brand color (NiflCyan #00FFFF).
#[inline]
pub fn primary() -> [f32; 4] {
    color("primary")
}

/// Secondary brand color (MuspelMagenta #FF00FF).
#[inline]
pub fn secondary() -> [f32; 4] {
    color("secondary")
}

/// Button background for the "primary" variant (uses accent color).
#[inline]
pub fn button_primary_bg() -> [f32; 4] {
    accent()
}

/// Button background for the "secondary" variant.
#[inline]
pub fn button_secondary_bg() -> [f32; 4] {
    surface_elevated()
}

/// Button background for the "danger" variant.
#[inline]
pub fn button_danger_bg() -> [f32; 4] {
    error_color()
}

/// Button background for the "ghost" variant (transparent).
#[inline]
pub fn button_ghost_bg() -> [f32; 4] {
    [0.0, 0.0, 0.0, 0.0]
}

/// Input field background.
#[inline]
pub fn input_bg() -> [f32; 4] {
    surface()
}

/// Input field border (focused state).
#[inline]
pub fn input_border_focus() -> [f32; 4] {
    accent()
}

/// Input field border (error state).
#[inline]
pub fn input_border_error() -> [f32; 4] {
    error_color()
}

/// Input field border (success state).
#[inline]
pub fn input_border_success() -> [f32; 4] {
    success()
}

/// Toggle/switch active (on) background.
#[inline]
pub fn toggle_active() -> [f32; 4] {
    accent()
}

/// Toggle/switch inactive (off) background. Adaptive.
#[inline]
pub fn toggle_inactive() -> [f32; 4] {
    surface_elevated()
}

/// Slider track filled portion.
#[inline]
pub fn slider_track_filled() -> [f32; 4] {
    accent()
}

/// Slider track unfilled portion. Adaptive.
#[inline]
pub fn slider_track_unfilled() -> [f32; 4] {
    surface_elevated()
}

/// Checkbox/radio checked background.
#[inline]
pub fn checkbox_checked() -> [f32; 4] {
    accent()
}

/// Checkbox/radio unchecked background.
#[inline]
pub fn checkbox_unchecked() -> [f32; 4] {
    surface()
}

/// SkollProgress bar fill color.
#[inline]
pub fn progress_fill() -> [f32; 4] {
    accent()
}

/// SkollProgress bar track color.
#[inline]
pub fn progress_track() -> [f32; 4] {
    surface_elevated()
}

/// HatiSpinner color.
#[inline]
pub fn spinner_color() -> [f32; 4] {
    accent()
}

/// Skeleton shimmer base color. Adaptive.
#[inline]
pub fn skeleton_base() -> [f32; 4] {
    color("hover")
}

/// Skeleton shimmer highlight color. Adaptive.
#[inline]
pub fn skeleton_highlight() -> [f32; 4] {
    surface_elevated()
}

/// Tab active background. Adaptive.
#[inline]
pub fn tab_active_bg() -> [f32; 4] {
    surface_elevated()
}

/// Tab inactive background.
#[inline]
pub fn tab_inactive_bg() -> [f32; 4] {
    [0.0, 0.0, 0.0, 0.0]
}

/// Tab hover background.
#[inline]
pub fn tab_hover_bg() -> [f32; 4] {
    hover()
}

/// Table row selected background.
#[inline]
pub fn table_row_selected() -> [f32; 4] {
    hover()
}

/// Table row hover background.
#[inline]
pub fn table_row_hover() -> [f32; 4] {
    hover()
}

/// Table header background.
#[inline]
pub fn table_header_bg() -> [f32; 4] {
    surface_elevated()
}

/// List item hover background.
#[inline]
pub fn list_item_hover() -> [f32; 4] {
    hover()
}

/// List item selected background.
#[inline]
pub fn list_item_selected() -> [f32; 4] {
    hover()
}

/// Chat bubble background (user).
#[inline]
pub fn chat_bubble_user() -> [f32; 4] {
    accent()
}

/// Chat bubble background (assistant).
#[inline]
pub fn chat_bubble_assistant() -> [f32; 4] {
    surface_elevated()
}

/// Chat bubble text color (user).
#[inline]
pub fn chat_text_user() -> [f32; 4] {
    [0.0, 0.0, 0.0, 1.0]
}

/// Chat bubble text color (assistant).
#[inline]
pub fn chat_text_assistant() -> [f32; 4] {
    text()
}

/// Tooltip background.
#[inline]
pub fn tooltip_bg() -> [f32; 4] {
    surface_overlay()
}

/// Toast success accent.
#[inline]
pub fn toast_success() -> [f32; 4] {
    success()
}

/// Toast error accent.
#[inline]
pub fn toast_error() -> [f32; 4] {
    error_color()
}

/// Toast warning accent.
#[inline]
pub fn toast_warning() -> [f32; 4] {
    warning()
}

/// Toast info accent.
#[inline]
pub fn toast_info() -> [f32; 4] {
    info()
}

// =============================================================================
// TYPOGRAPHY SCALE
// =============================================================================

/// Hero font size (48.0)
#[inline]
pub fn font_hero() -> f32 {
    48.0
}

/// Heading 1 font size (32.0)
#[inline]
pub fn font_h1() -> f32 {
    32.0
}

/// Heading 2 font size (24.0)
#[inline]
pub fn font_h2() -> f32 {
    24.0
}

/// Heading 3 font size (20.0)
#[inline]
pub fn font_h3() -> f32 {
    20.0
}

/// Standard body font size (16.0)
#[inline]
pub fn font_body() -> f32 {
    crate::FONT_BASE
}

/// Caption/small font size (12.0)
#[inline]
pub fn font_caption() -> f32 {
    12.0
}

/// Monospaced/code font size (14.0)
#[inline]
pub fn font_code() -> f32 {
    14.0
}

// === Additional color helpers for components ===

/// Viking Gold accent color (#FFD700 / [1.0, 0.84, 0.0, 1.0])
#[inline]
pub fn viking_gold() -> [f32; 4] {
    [1.0, 0.84, 0.0, 1.0]
}

/// Magenta Liquid accent color (#FF00FF / [1.0, 0.0, 1.0, 1.0])
#[inline]
pub fn magenta_liquid() -> [f32; 4] {
    [1.0, 0.0, 1.0, 1.0]
}

/// Critical red color for error/danger states ([1.0, 0.2, 0.2, 1.0])
#[inline]
pub fn critical_red() -> [f32; 4] {
    [1.0, 0.2, 0.2, 1.0]
}

/// Warning orange color ([1.0, 0.5, 0.0, 1.0])
#[inline]
pub fn warning_orange() -> [f32; 4] {
    [1.0, 0.5, 0.0, 1.0]
}

/// Hazard/danger border color (orange-red [1.0, 0.2, 0.0, 1.0])
#[inline]
pub fn hazard_orange() -> [f32; 4] {
    [1.0, 0.2, 0.0, 1.0]
}

// === Workflow / Agent status tokens ===

/// Running/active status color (uses accent).
#[inline]
pub fn status_running() -> [f32; 4] {
    accent()
}

/// Completed/success status color.
#[inline]
pub fn status_completed() -> [f32; 4] {
    success()
}

/// Failed/error status color.
#[inline]
pub fn status_failed() -> [f32; 4] {
    error_color()
}

/// Waiting/idle status color.
#[inline]
pub fn status_waiting() -> [f32; 4] {
    text_muted()
}

// === Inspector / debug panel tokens ===

/// Inspector panel background.
#[inline]
pub fn inspector_bg() -> [f32; 4] {
    surface()
}

/// Inspector panel border.
#[inline]
pub fn inspector_border() -> [f32; 4] {
    border()
}

/// Inspector accent highlight.
#[inline]
pub fn inspector_accent() -> [f32; 4] {
    accent()
}

/// Inspector warning highlight.
#[inline]
pub fn inspector_warning() -> [f32; 4] {
    warning()
}

// === Collaboration status tokens ===

/// Online indicator color.
#[inline]
pub fn collab_online() -> [f32; 4] {
    success()
}

/// Away indicator color.
#[inline]
pub fn collab_away() -> [f32; 4] {
    warning()
}

/// Offline indicator color.
#[inline]
pub fn collab_offline() -> [f32; 4] {
    text_muted()
}

// === Node type tokens (for workflow / graph nodes) ===

/// Concept node color (blue).
#[inline]
pub fn node_concept() -> [f32; 4] {
    [0.2, 0.6, 0.9, 1.0]
}

/// Entity node color (green).
#[inline]
pub fn node_entity() -> [f32; 4] {
    [0.4, 0.8, 0.4, 1.0]
}

/// Relation node color (orange).
#[inline]
pub fn node_relation() -> [f32; 4] {
    [0.9, 0.6, 0.2, 1.0]
}

/// Context node color (purple).
#[inline]
pub fn node_context() -> [f32; 4] {
    [0.8, 0.4, 0.8, 1.0]
}

// === Editor / canvas tokens ===

/// Editor/drawing canvas background.
#[inline]
pub fn editor_bg() -> [f32; 4] {
    surface()
}

/// Editor grid/axis line color.
#[inline]
pub fn editor_grid() -> [f32; 4] {
    border()
}

// === QR code tokens ===

/// QR code dark module color (adaptive).
#[inline]
pub fn qr_dark() -> [f32; 4] {
    text()
}

/// QR code light module color (adaptive).
#[inline]
pub fn qr_light() -> [f32; 4] {
    bg()
}
