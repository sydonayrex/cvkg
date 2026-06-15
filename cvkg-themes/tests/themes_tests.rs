use cvkg_themes::Theme;
#[test]
fn test_theme_creation() {
    let theme = Theme::dark();
    assert!(theme.is_dark());
}

#[test]
fn test_semantic_colors_exist() {
    let theme = Theme::dark();
    let colors = &theme.colors;

    assert!(colors.primary.a > 0.0);
    assert!(colors.secondary.a > 0.0);
    assert!(colors.accent.a > 0.0);
    assert!(colors.background.a > 0.0);
    assert!(colors.surface.a > 0.0);
    assert!(colors.error.a > 0.0);
    assert!(colors.warning.a > 0.0);
    assert!(colors.success.a > 0.0);
    assert!(colors.text.a > 0.0);
    assert!(colors.text_dim.a > 0.0);
}

#[test]
fn test_typography_scale_values() {
    let theme = Theme::dark();
    let typo = &theme.typography;

    assert!(typo.hero >= typo.h1);
    assert!(typo.h1 >= typo.h2);
    assert!(typo.h2 >= typo.body);
    assert!(typo.body >= typo.caption);
    assert!(typo.caption >= typo.code);
}

#[test]
fn test_spacing_scale_values() {
    let theme = Theme::dark();
    let spacing = &theme.spacing;

    assert!(spacing.xs <= spacing.s);
    assert!(spacing.s <= spacing.m);
    assert!(spacing.m <= spacing.l);
    assert!(spacing.l <= spacing.xl);
}

#[test]
fn test_motion_parameters() {
    let theme = Theme::dark();

    let _ = &theme.motion.snappy;
    let _ = &theme.motion.fluid;
    let _ = &theme.motion.heavy;
    let _ = &theme.motion.bouncy;
}

#[test]
fn test_color_contrast_ratio() {
    let theme = Theme::dark();
    let ratio = theme.colors.text.contrast_ratio(&theme.colors.background);
    assert!(ratio > 0.0);
}

#[test]
fn test_is_dark_method() {
    let dark_theme = Theme::dark();
    assert!(dark_theme.is_dark());
}

#[test]
fn test_accessibility_validation_dark() {
    let theme = Theme::dark();
    let results = theme.validate_accessibility();
    for result in &results {
        assert!(
            result.passes,
            "APCA FAIL on dark theme: {} Lc={:.1} (level={})",
            result.level,
            result.contrast,
            result.level
        );
    }
}

#[test]
fn test_accessibility_validation_light() {
    let theme = Theme::light();
    let results = theme.validate_accessibility();
    for result in &results {
        assert!(
            result.passes,
            "APCA FAIL on light theme: {} Lc={:.1} (level={})",
            result.level,
            result.contrast,
            result.level
        );
    }
}

#[test]
fn test_accessibility_validation_custom_seed() {
    use cvkg_themes::OklchColor;
    let seed = OklchColor::new(0.55, 0.12, 260.0, 1.0);
    let theme = Theme::from_seed(seed);
    let results = theme.validate_accessibility();
    for result in &results {
        assert!(
            result.passes,
            "APCA FAIL on seeded theme: {} Lc={:.1} (level={})",
            result.level,
            result.contrast,
            result.level
        );
    }
}

#[test]
fn test_theme_clone() {
    let theme = Theme::dark();
    let cloned = theme.clone();

    assert_eq!(theme.is_dark(), cloned.is_dark());
    assert_eq!(theme.colors.text.r, cloned.colors.text.r);
}

#[test]
fn test_color_alpha_values() {
    let theme = Theme::dark();

    assert_eq!(theme.colors.primary.a, 1.0);
    assert_eq!(theme.colors.background.a, 1.0);
    assert_eq!(theme.colors.text.a, 1.0);
}

#[test]
fn test_theme_usage_journey() {
    let theme = Theme::dark();
    assert!(theme.is_dark());

    let bg_color = theme.colors.background;
    let text_color = theme.colors.text;

    assert!(bg_color.r >= 0.0 && bg_color.r <= 1.0);
    assert!(text_color.r >= 0.0 && text_color.r <= 1.0);

    let _warnings = theme.validate_accessibility();
}

#[test]
fn test_theme_typography_journey() {
    let theme = Theme::dark();

    let hero_size = theme.typography.hero;
    let body_size = theme.typography.body;
    let caption_size = theme.typography.caption;

    assert!(hero_size > body_size);
    assert!(body_size > caption_size);
}

#[test]
fn test_theme_motion_journey() {
    let theme = Theme::dark();

    let snappy = theme.motion.snappy;
    let fluid = theme.motion.fluid;

    assert!(snappy.stiffness > 0.0);
    assert!(fluid.stiffness > 0.0);
}

#[test]
fn smoke_test_theme_compiles() {
    let _theme = Theme::dark();
}

#[test]
fn smoke_test_theme_values_are_sensible() {
    let theme = Theme::dark();

    assert!(theme.typography.hero > 0.0);
    assert!(theme.spacing.xl > theme.spacing.xs);
}
