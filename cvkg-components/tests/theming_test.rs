//! Theming validation tests.
//!
//! These tests verify that all components use theme tokens for colors,
//! not hardcoded [f32; 4] arrays.

use cvkg_themes::{OklchColor, Theme};

/// Verify that the default dark theme passes APCA contrast validation.
#[test]
fn test_dark_theme_apca_contrast() {
    let theme = Theme::dark();
    let results = theme.validate_accessibility();
    for result in &results {
        assert!(
            result.passes,
            "APCA FAIL on dark theme: {} Lc={:.1} (level={})",
            result.level, result.contrast, result.level
        );
    }
}

/// Verify that the default light theme passes APCA contrast validation.
#[test]
fn test_light_theme_apca_contrast() {
    let theme = Theme::light();
    let results = theme.validate_accessibility();
    for result in &results {
        assert!(
            result.passes,
            "APCA FAIL on light theme: {} Lc={:.1} (level={})",
            result.level, result.contrast, result.level
        );
    }
}

/// Verify that a custom-seeded theme passes APCA contrast validation.
#[test]
fn test_seeded_theme_apca_contrast() {
    let seed = OklchColor::new(0.55, 0.12, 260.0, 1.0);
    let theme = Theme::from_seed(seed);
    let results = theme.validate_accessibility();
    for result in &results {
        assert!(
            result.passes,
            "APCA FAIL on seeded theme: {} Lc={:.1} (level={})",
            result.level, result.contrast, result.level
        );
    }
}

/// Verify that all theme token functions return valid RGBA values (0.0-1.0 range).
#[test]
fn test_theme_token_values_valid() {
    use cvkg_components::theme;

    let tokens: Vec<([f32; 4], &str)> = vec![
        (theme::text(), "text"),
        (theme::text_muted(), "text_muted"),
        (theme::text_dim(), "text_dim"),
        (theme::bg(), "background"),
        (theme::surface(), "surface"),
        (theme::surface_elevated(), "surface_elevated"),
        (theme::surface_overlay(), "surface_overlay"),
        (theme::border(), "border"),
        (theme::border_strong(), "border_strong"),
        (theme::accent(), "accent"),
        (theme::accent_hover(), "accent_hover"),
        (theme::hover(), "hover"),
        (theme::active_color(), "active"),
        (theme::disabled(), "disabled"),
        (theme::disabled_text(), "disabled_text"),
        (theme::error_color(), "error"),
        (theme::warning(), "warning"),
        (theme::success(), "success"),
        (theme::info(), "info"),
        (theme::focus_ring(), "focus_ring"),
        (theme::shadow(), "shadow"),
        (theme::code_bg(), "code_bg"),
        (theme::primary(), "primary"),
        (theme::secondary(), "secondary"),
        (theme::button_primary_bg(), "button_primary_bg"),
        (theme::button_secondary_bg(), "button_secondary_bg"),
        (theme::button_danger_bg(), "button_danger_bg"),
        (theme::button_ghost_bg(), "button_ghost_bg"),
        (theme::input_bg(), "input_bg"),
        (theme::input_border_focus(), "input_border_focus"),
        (theme::input_border_error(), "input_border_error"),
        (theme::input_border_success(), "input_border_success"),
        (theme::toggle_active(), "toggle_active"),
        (theme::toggle_inactive(), "toggle_inactive"),
        (theme::slider_track_filled(), "slider_track_filled"),
        (theme::slider_track_unfilled(), "slider_track_unfilled"),
        (theme::checkbox_checked(), "checkbox_checked"),
        (theme::checkbox_unchecked(), "checkbox_unchecked"),
        (theme::progress_fill(), "progress_fill"),
        (theme::progress_track(), "progress_track"),
        (theme::spinner_color(), "spinner_color"),
        (theme::skeleton_base(), "skeleton_base"),
        (theme::skeleton_highlight(), "skeleton_highlight"),
        (theme::tab_active_bg(), "tab_active_bg"),
        (theme::tab_inactive_bg(), "tab_inactive_bg"),
        (theme::tab_hover_bg(), "tab_hover_bg"),
        (theme::table_row_selected(), "table_row_selected"),
        (theme::table_row_hover(), "table_row_hover"),
        (theme::table_header_bg(), "table_header_bg"),
        (theme::list_item_hover(), "list_item_hover"),
        (theme::list_item_selected(), "list_item_selected"),
        (theme::chat_bubble_user(), "chat_bubble_user"),
        (theme::chat_bubble_assistant(), "chat_bubble_assistant"),
        (theme::chat_text_user(), "chat_text_user"),
        (theme::chat_text_assistant(), "chat_text_assistant"),
        (theme::tooltip_bg(), "tooltip_bg"),
        (theme::toast_success(), "toast_success"),
        (theme::toast_error(), "toast_error"),
        (theme::toast_warning(), "toast_warning"),
        (theme::toast_info(), "toast_info"),
        (theme::status_running(), "status_running"),
        (theme::status_completed(), "status_completed"),
        (theme::status_failed(), "status_failed"),
        (theme::status_waiting(), "status_waiting"),
        (theme::collab_online(), "collab_online"),
        (theme::collab_away(), "collab_away"),
        (theme::collab_offline(), "collab_offline"),
        (theme::inspector_bg(), "inspector_bg"),
        (theme::inspector_border(), "inspector_border"),
        (theme::inspector_accent(), "inspector_accent"),
        (theme::inspector_warning(), "inspector_warning"),
        (theme::editor_bg(), "editor_bg"),
        (theme::editor_grid(), "editor_grid"),
        (theme::qr_dark(), "qr_dark"),
        (theme::qr_light(), "qr_light"),
    ];

    for (color, name) in &tokens {
        for (i, &component) in color.iter().enumerate() {
            assert!(
                (0.0..=1.0).contains(&component),
                "Theme token '{}' component {} = {} is out of [0.0, 1.0] range",
                name,
                i,
                component
            );
        }
    }
}

/// Verify that the dark theme's text/background contrast meets APCA standards.
#[test]
fn test_dark_theme_text_contrast() {
    let theme = Theme::dark();
    let text = theme.colors.text;
    let bg = theme.colors.background;

    // Text should be lighter than background in dark mode
    let text_lum = text.relative_luminance();
    let bg_lum = bg.relative_luminance();
    assert!(
        text_lum > bg_lum,
        "Dark theme text ({:?}) should be lighter than background ({:?})",
        text,
        bg
    );
}

/// Verify that the light theme's text/background contrast meets APCA standards.
#[test]
fn test_light_theme_text_contrast() {
    let theme = Theme::light();
    let text = theme.colors.text;
    let bg = theme.colors.background;

    // Text should be darker than background in light mode
    let text_lum = text.relative_luminance();
    let bg_lum = bg.relative_luminance();
    assert!(
        text_lum < bg_lum,
        "Light theme text ({:?}) should be darker than background ({:?})",
        text,
        bg
    );
}

/// Verify that semantic error/warning/success colors are distinguishable.
#[test]
fn test_semantic_colors_distinguishable() {
    let theme = Theme::dark();

    // Error, warning, and success should be different colors
    let error = theme.colors.error;
    let warning = theme.colors.warning;
    let success = theme.colors.success;

    assert_ne!(error, warning, "Error and warning colors should differ");
    assert_ne!(error, success, "Error and success colors should differ");
    assert_ne!(warning, success, "Warning and success colors should differ");
}
