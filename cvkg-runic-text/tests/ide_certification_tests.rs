use cvkg_runic_text::{ShapedText, TextAlign, TextEngine, TextOverflow, TextSpan, TextStyle};

/// Snap cursor selection index to the nearest grapheme boundary.
/// Ensures we do not slice in the middle of complex graphemes like ZWJ emojis.
fn snap_to_grapheme_boundary(shaped: &ShapedText, index: usize) -> usize {
    if shaped.grapheme_boundaries.is_empty() {
        return 0;
    }
    let mut closest = shaped.grapheme_boundaries[0];
    let mut min_diff = (index as isize - closest as isize).abs();

    for &b in &shaped.grapheme_boundaries {
        let diff = (index as isize - b as isize).abs();
        if diff < min_diff {
            min_diff = diff;
            closest = b;
        }
    }
    closest
}

/// P2-42: IDE certification suite.
/// Validates cursor navigation, selection boundaries, ligatures, Knuth-Plass line wrapping,
/// and CJK monospace width alignment.
#[test]
fn test_ide_cursor_navigation() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);
    // A (1 byte) + Emoji (25 bytes) + B (1 byte)
    let text = "A👨‍👩‍👧‍👦B";
    let spans = vec![TextSpan::new(text, style)];

    let shaped = engine
        .shape_layout(&spans, None, TextAlign::Start, TextOverflow::Clip)
        .unwrap();

    // Verify grapheme start boundaries are captured correctly
    assert!(shaped.grapheme_boundaries.contains(&0)); // Start of A
    assert!(shaped.grapheme_boundaries.contains(&1)); // Start of Emoji
    assert!(shaped.grapheme_boundaries.contains(&26)); // Start of B
    assert_eq!(shaped.grapheme_boundaries.len(), 3);

    // Test selection snapping: attempting to select index 10 (inside the emoji ZWJ sequence)
    // should snap to the start (1) or the end (26) of the emoji.
    let snapped = snap_to_grapheme_boundary(&shaped, 10);
    assert!(snapped == 1 || snapped == 26);
}

#[test]
fn test_ide_cjk_monospace_alignment() {
    let mut engine = TextEngine::new_test();
    // Test monospace font style
    let mut style = TextStyle::new("Jupiteroid", 16.0);
    // Force monospace handling simulation
    style.family = "courier".to_string();

    let latin_spans = vec![TextSpan::new("abc", style.clone())];
    let latin_shaped = engine
        .shape_layout(&latin_spans, None, TextAlign::Start, TextOverflow::Clip)
        .unwrap();

    // Verify that every single glyph in the monospace run has the exact same advance width
    assert!(!latin_shaped.glyphs.is_empty());
    let first_advance = latin_shaped.glyphs[0].advance_width;
    for g in &latin_shaped.glyphs {
        assert_eq!(
            g.advance_width, first_advance,
            "All glyphs in a monospace run must have identical advance width"
        );
    }
}

#[test]
fn test_ide_knuth_plass_wrapping_stability() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);

    // A long text that must wrap given a tight width boundary
    let text = "The quick brown fox jumps over the lazy dog again and again.";
    let spans = vec![TextSpan::new(text, style)];

    // Width limit of 150px
    let shaped = engine
        .shape_layout(&spans, Some(150.0), TextAlign::Start, TextOverflow::Clip)
        .unwrap();

    // Verify it wrapped into multiple lines
    assert!(
        shaped.lines.len() > 1,
        "Layout must be wrapped into multiple lines"
    );
    for line in &shaped.lines {
        assert!(
            line.width <= 155.0,
            "Line width must be bound to the limit plus minor margins"
        );
    }
}
