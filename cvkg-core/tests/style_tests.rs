use cvkg_core::{SemanticColor, Style};

#[test]
fn state_overlay_merges_over_base_style() {
    let base = Style::new()
        .background(SemanticColor::Surface)
        .disabled_background(SemanticColor::Muted)
        .radius(6.0);
    let overlay = Style::new().background(SemanticColor::Primary).radius(8.0);

    let merged = base.merge(&overlay);

    assert_eq!(merged.background, Some(SemanticColor::Primary));
    assert_eq!(merged.disabled_background, Some(SemanticColor::Muted));
    assert_eq!(merged.radius, Some(8.0));
}
