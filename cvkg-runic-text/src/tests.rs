use super::*;
use fontdb::{Stretch, Style, Weight};
use rustybuzz::Direction;

const MAX_CACHE_SIZE: usize = 1024;

#[test]
fn test_text_measure_render_sync() {
    let mut engine1 = TextEngine::new_test();
    let mut engine2 = TextEngine::new_test();

    let text = "Hello, convergence!";
    let style = TextStyle::new("Jupiteroid", 16.0);
    let spans = vec![TextSpan::new(text, style)];

    let shaped1 = engine1
        .shape_layout(&spans, None, TextAlign::Start, TextOverflow::Clip)
        .unwrap();
    let shaped2 = engine2
        .shape_layout(&spans, None, TextAlign::Start, TextOverflow::Clip)
        .unwrap();

    assert_eq!(shaped1.width, shaped2.width, "Widths must match precisely");
    assert_eq!(
        shaped1.glyphs.len(),
        shaped2.glyphs.len(),
        "Glyph counts must match"
    );
    for (g1, g2) in shaped1.glyphs.iter().zip(shaped2.glyphs.iter()) {
        assert_eq!(g1.x, g2.x, "Glyph X positions must match");
        assert_eq!(
            g1.advance_width, g2.advance_width,
            "Glyph advances must match"
        );
    }
}

#[test]
fn test_basic_shaping() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);
    let glyphs = engine
        .shape_run("Hello", &style, Direction::LeftToRight)
        .unwrap();
    assert!(!glyphs.is_empty(), "Should produce glyphs for 'Hello'");
}

#[test]
fn test_hit_test() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);
    let spans = vec![TextSpan::new("Hello", style.clone())];
    let shaped = engine
        .shape_layout(&spans, None, TextAlign::Start, TextOverflow::WordWrap)
        .unwrap();
    let (glyph_idx, cluster) = shaped.hit_test(0);
    assert!(glyph_idx < shaped.glyphs.len());
    assert_eq!(cluster, 0);
}

#[test]
fn test_word_wrapping() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);
    let spans = vec![TextSpan::new("Hello World This Is A Test", style.clone())];
    let shaped = engine
        .shape_layout(&spans, Some(80.0), TextAlign::Start, TextOverflow::WordWrap)
        .unwrap();
    assert!(
        shaped.lines.len() > 1,
        "Should wrap into multiple lines, got {}",
        shaped.lines.len()
    );
}

#[test]
fn test_text_style_defaults() {
    let style = TextStyle::default();
    assert_eq!(style.family, "Jupiteroid");
    assert_eq!(style.font_size, DEFAULT_FONT_SIZE);
    assert_eq!(style.weight, Weight::NORMAL);
    assert_eq!(style.color, [255, 255, 255, 255]);
    assert!(!style.fallback_families.is_empty());
}

#[test]
fn test_text_style_builder() {
    let style = TextStyle::new("Jupiteroid", 24.0)
        .with_weight(700)
        .italic()
        .with_color(255, 0, 0, 255)
        .with_letter_spacing(1.5)
        .with_underline();

    assert_eq!(style.font_size, 24.0);
    assert_eq!(style.weight, Weight(700));
    assert_eq!(style.style, Style::Italic);
    assert_eq!(style.color, [255, 0, 0, 255]);
    assert_eq!(style.letter_spacing, 1.5);
    assert!(style.decorations.underline);
}

#[test]
fn test_line_height() {
    let multiple = LineHeight::Multiple(1.5);
    assert_eq!(multiple.to_pixels(16.0), 24.0);

    let fixed = LineHeight::Fixed(20.0);
    assert_eq!(fixed.to_pixels(16.0), 20.0);
}

#[test]
fn test_cache_key_deterministic() {
    let key1 = CacheKey::new(
        "Hello",
        12345,
        16.0,
        Weight::NORMAL,
        Stretch::Normal,
        Style::Normal,
        Direction::LeftToRight,
        0.0,
        0.0,
    );
    let key2 = CacheKey::new(
        "Hello",
        12345,
        16.0,
        Weight::NORMAL,
        Stretch::Normal,
        Style::Normal,
        Direction::LeftToRight,
        0.0,
        0.0,
    );
    assert_eq!(key1, key2);

    let key3 = CacheKey::new(
        "World",
        12345,
        16.0,
        Weight::NORMAL,
        Stretch::Normal,
        Style::Normal,
        Direction::LeftToRight,
        0.0,
        0.0,
    );
    assert_ne!(key1, key3);
}

#[test]
fn test_cursor_model() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);
    let text = "a👨‍👩‍👧‍👦b";
    let spans = vec![TextSpan::new(text, style.clone())];
    let shaped = engine
        .shape_layout(&spans, None, TextAlign::Start, TextOverflow::Clip)
        .unwrap();

    let (x_a, _) = shaped.cursor_position(0);
    let (x_emoji, _) = shaped.cursor_position(1);
    let (x_b, _) = shaped.cursor_position(26);
    let (x_end, _) = shaped.cursor_position(27);

    assert!(x_a < x_emoji);
    assert!(x_emoji < x_b);
    assert!(x_b < x_end);
}

#[test]
fn test_unicode_compliance_uax29() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);

    let text = "🏳️‍🌈";
    let spans = vec![TextSpan::new(text, style.clone())];
    let shaped = engine
        .shape_layout(&spans, None, TextAlign::Start, TextOverflow::Clip)
        .unwrap();

    let (x_start, _) = shaped.cursor_position(0);
    let (x_end, _) = shaped.cursor_position(text.len());
    assert!(x_start <= x_end);

    let (hit_idx, hit_cluster) = shaped.hit_test(text.len() / 2);
    assert_eq!(hit_idx, 0);
    assert_eq!(hit_cluster, 0);
}

#[test]
fn test_cursor_position() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);
    let spans = vec![TextSpan::new("Hello", style.clone())];
    let shaped = engine
        .shape_layout(&spans, None, TextAlign::Start, TextOverflow::WordWrap)
        .unwrap();
    let (x, line) = shaped.cursor_position(0);
    assert_eq!(line, 0);
    assert!(x >= 0.0);
}

#[test]
fn test_selection_rects() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);
    let spans = vec![TextSpan::new("Hello World", style.clone())];
    let shaped = engine
        .shape_layout(&spans, None, TextAlign::Start, TextOverflow::WordWrap)
        .unwrap();
    let rects = shaped.selection_rects(0, 5);
    assert!(
        !rects.is_empty(),
        "Should produce selection rects for 'Hello'"
    );
}

#[test]
fn test_open_type_features() {
    let liga = OpenTypeFeature::liga();
    assert_eq!(liga.tag, u32::from_be_bytes(*b"liga"));
    assert_eq!(liga.value, 1);

    let kern = OpenTypeFeature::kern();
    assert_eq!(kern.tag, u32::from_be_bytes(*b"kern"));
}

#[test]
fn test_variable_axes() {
    let weight = VariableAxis::weight(700.0);
    assert_eq!(weight.tag, u32::from_be_bytes(*b"wght"));
    assert_eq!(weight.value, 700.0);

    let italic = VariableAxis::italic(1.0);
    assert_eq!(italic.tag, u32::from_be_bytes(*b"ital"));
}

#[test]
fn test_font_metrics() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);
    let metrics = engine.font_metrics(&style).unwrap();
    assert!(metrics.ascent > 0.0);
    assert!(metrics.descent > 0.0);
    assert!(metrics.units_per_em > 0);
}

#[test]
fn test_empty_input() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);
    let spans = vec![TextSpan::new("", style.clone())];
    let shaped = engine
        .shape_layout(&spans, None, TextAlign::Start, TextOverflow::WordWrap)
        .unwrap();
    assert!(shaped.glyphs.is_empty());
}

#[test]
fn test_multi_span_layout() {
    let mut engine = TextEngine::new_test();
    let style1 = TextStyle::new("Jupiteroid", 16.0);
    let style2 = TextStyle::new("Jupiteroid", 24.0).with_color(255, 0, 0, 255);
    let spans = vec![
        TextSpan::at("Hello ", style1, 0),
        TextSpan::at("World", style2, 6),
    ];
    let shaped = engine
        .shape_layout(&spans, None, TextAlign::Start, TextOverflow::WordWrap)
        .unwrap();
    assert!(!shaped.glyphs.is_empty());
    assert_eq!(shaped.text, "Hello World");
}

#[test]
fn test_text_align_center() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);
    let spans = vec![TextSpan::new("Hi", style.clone())];
    let shaped = engine
        .shape_layout(
            &spans,
            Some(200.0),
            TextAlign::Center,
            TextOverflow::WordWrap,
        )
        .unwrap();
    assert!(!shaped.lines.is_empty());
    let line = &shaped.lines[0];
    assert!(
        line.x_offset > 0.0,
        "Center-aligned line should have positive x_offset, got {}",
        line.x_offset
    );
}

#[test]
fn test_text_overflow_ellipsis() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);
    let spans = vec![TextSpan::new("Hello World This Is Long", style.clone())];
    let shaped = engine
        .shape_layout(&spans, Some(50.0), TextAlign::Start, TextOverflow::Ellipsis)
        .unwrap();
    assert!(!shaped.lines.is_empty());
}

#[test]
fn test_decorations() {
    let decorations = TextDecorations {
        underline: true,
        strikethrough: true,
        overline: false,
    };
    assert!(decorations.underline);
    assert!(decorations.strikethrough);
    assert!(!decorations.overline);
}

#[test]
fn test_cache_eviction() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);

    let _ = engine.shape_run("Test", &style, Direction::LeftToRight);

    let (size, max) = engine.cache_stats();
    assert!(size > 0, "Cache should have entries after shaping");
    assert_eq!(max, MAX_CACHE_SIZE);

    engine.clear_cache();
    let (size, _) = engine.cache_stats();
    assert_eq!(size, 0);
}

#[test]
fn test_font_count() {
    let engine = TextEngine::new_test();
    let count = engine.font_count();
    assert!(count > 0, "Should find at least one font, got {}", count);
}

#[test]
fn test_jupiteroid_font_available() {
    let engine = TextEngine::new_test();
    assert!(engine.font_count() > 0, "Should have fonts loaded");
}

#[test]
fn test_extract_glyph_path() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);

    let glyphs = engine
        .shape_run("A", &style, Direction::LeftToRight)
        .unwrap();
    assert!(!glyphs.is_empty(), "Shaping 'A' should yield a glyph");
    let glyph_id = glyphs[0].glyph_id;

    let path = engine.extract_glyph_path(glyph_id, 16.0, &style).unwrap();

    assert!(!path.is_empty(), "Glyph path for 'A' should not be empty");
    match path[0] {
        RunicPathSegment::MoveTo { x, y } => {
            assert!(x.is_finite());
            assert!(y.is_finite());
        }
        _ => panic!("Expected first segment to be a MoveTo, got {:?}", path[0]),
    }

    let has_close = path
        .iter()
        .any(|seg| matches!(seg, RunicPathSegment::Close));
    assert!(
        has_close,
        "Expected glyph path to contain at least one Close command"
    );

    for segment in &path {
        match *segment {
            RunicPathSegment::MoveTo { x, y } => {
                assert!(x.is_finite());
                assert!(y.is_finite());
            }
            RunicPathSegment::LineTo { x, y } => {
                assert!(x.is_finite());
                assert!(y.is_finite());
            }
            RunicPathSegment::QuadTo { cx, cy, x, y } => {
                assert!(cx.is_finite());
                assert!(cy.is_finite());
                assert!(x.is_finite());
                assert!(y.is_finite());
            }
            RunicPathSegment::CubicTo {
                cx1,
                cy1,
                cx2,
                cy2,
                x,
                y,
            } => {
                assert!(cx1.is_finite());
                assert!(cy1.is_finite());
                assert!(cx2.is_finite());
                assert!(cy2.is_finite());
                assert!(x.is_finite());
                assert!(y.is_finite());
            }
            RunicPathSegment::Close => {}
        }
    }
}

#[test]
fn test_new_text_style_fields() {
    let style = TextStyle::new("Jupiteroid", 16.0)
        .with_outline_rendering(true)
        .with_material_effect(42);

    assert!(style.outline_rendering);
    assert_eq!(style.material_effect_id, 42);
}

#[test]
fn test_text_path_sampling() {
    let tp = TextPath::new(vec![(0.0, 0.0), (100.0, 100.0), (200.0, 0.0)]);
    let ((x_start, y_start), angle_start) = tp.sample(0.0);
    let ((x_mid, y_mid), angle_mid) = tp.sample(0.5);

    assert_eq!(x_start, 0.0);
    assert_eq!(y_start, 0.0);
    assert!(angle_start > 0.0);

    assert_eq!(x_mid, 100.0);
    assert_eq!(y_mid, 50.0);
    assert!(angle_mid.abs() < 1e-4);
}

#[test]
fn test_layout_boundary_circle() {
    let boundary = LayoutBoundary::Circle {
        cx: 100.0,
        cy: 100.0,
        r: 50.0,
    };
    let span = boundary.allowed_span(100.0).unwrap();
    assert_eq!(span.0, 50.0);
    assert_eq!(span.1, 150.0);

    let span_edge = boundary.allowed_span(150.0);
    assert!(span_edge.is_none() || span_edge.unwrap().0 >= 100.0);
}

#[test]
fn test_shape_layout_with_path_and_boundary() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);
    let spans = vec![TextSpan::new(
        "Hello World Curved Layout Test String",
        style,
    )];

    let tp = TextPath::new(vec![(0.0, 0.0), (100.0, 50.0), (200.0, 0.0)]);
    let shaped_path = engine
        .shape_layout_ex(
            &spans,
            None,
            TextAlign::Start,
            TextOverflow::WordWrap,
            Some(tp),
            None,
        )
        .unwrap();
    assert!(!shaped_path.glyphs.is_empty());
    let has_angles = shaped_path.glyphs.iter().any(|g| g.angle != 0.0);
    assert!(has_angles);

    let boundary = LayoutBoundary::Circle {
        cx: 100.0,
        cy: 100.0,
        r: 50.0,
    };
    let shaped_boundary = engine
        .shape_layout_ex(
            &spans,
            None,
            TextAlign::Start,
            TextOverflow::WordWrap,
            None,
            Some(boundary),
        )
        .unwrap();
    assert!(!shaped_boundary.glyphs.is_empty());
}

#[test]
fn test_portal_alignment() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);

    let spans = vec![
        TextSpan::at("Txt ", style.clone(), 0),
        TextSpan::portal_at(
            30.0,
            20.0,
            PortalAlignment::Baseline,
            "p_base",
            style.clone(),
            4,
        ),
        TextSpan::portal_at(30.0, 20.0, PortalAlignment::Top, "p_top", style.clone(), 7),
        TextSpan::portal_at(
            30.0,
            20.0,
            PortalAlignment::Center,
            "p_center",
            style.clone(),
            10,
        ),
        TextSpan::portal_at(
            30.0,
            20.0,
            PortalAlignment::Bottom,
            "p_bottom",
            style.clone(),
            13,
        ),
    ];

    let shaped_single = engine
        .shape_layout(&spans, None, TextAlign::Start, TextOverflow::WordWrap)
        .unwrap();

    let portals_s: Vec<_> = shaped_single
        .glyphs
        .iter()
        .filter(|g| g.glyph_id == 0xFFFF)
        .collect();
    assert_eq!(portals_s.len(), 4);

    let baseline_y = shaped_single.lines[0].baseline_y;
    let ascent = shaped_single.ascent;
    let line_height_px = shaped_single.lines[0].height;

    assert_eq!(portals_s[0].y, baseline_y);
    assert_eq!(portals_s[1].y, baseline_y - ascent);
    assert_eq!(
        portals_s[2].y,
        baseline_y - ascent + (line_height_px - 20.0) / 2.0
    );
    assert_eq!(portals_s[3].y, baseline_y - ascent + line_height_px - 20.0);

    let shaped_wrapped = engine
        .shape_layout(&spans, Some(50.0), TextAlign::Start, TextOverflow::WordWrap)
        .unwrap();

    let portals_w: Vec<_> = shaped_wrapped
        .glyphs
        .iter()
        .filter(|g| g.glyph_id == 0xFFFF)
        .collect();
    assert_eq!(portals_w.len(), 4);
}

#[test]
fn test_text_semantic_layer_and_virtualization() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);

    let mut paragraph = Paragraph::new("First paragraph with code element.");
    paragraph.add_run(TextRun::new(
        0,
        34,
        "First paragraph with code element.",
        style.clone(),
    ));
    paragraph.add_semantic_range(SemanticRange::new(21, 25, SemanticKind::Code, None));

    assert_eq!(paragraph.runs.len(), 1);
    assert_eq!(paragraph.semantic_ranges.len(), 1);
    assert_eq!(paragraph.semantic_ranges[0].kind, SemanticKind::Code);

    let mut paragraphs = Vec::new();
    for i in 0..100 {
        let mut p = Paragraph::new(&format!("Paragraph line index: {}", i));
        p.add_run(TextRun::new(0, p.text.len(), &p.text, style.clone()));
        paragraphs.push(p);
    }

    let line_h = 20.0;
    let virtual_shaped = engine
        .shape_layout_virtualized(&paragraphs, line_h, 100.0, 200.0, None, TextAlign::Start)
        .unwrap();

    assert!(!virtual_shaped.lines.is_empty());
    assert!(virtual_shaped.lines.len() < 100);
    assert_eq!(virtual_shaped.height, 100.0 * line_h);
}

#[test]
fn default_capabilities_are_sensible() {
    let caps = TextCapabilities::default_capabilities();
    assert!(caps.variable_fonts);
    assert!(caps.open_type_features);
    assert!(caps.subpixel_positioning);
    assert!(caps.bidi);
    assert!(caps.font_fallback);
    assert!(caps.hinting);
    assert!(caps.shaping_cache);
}

#[test]
fn fully_featured_requires_all() {
    let caps = TextCapabilities::default_capabilities();
    assert!(caps.is_fully_featured());
    assert!(!caps.color_fonts);
    assert!(!caps.vertical_text);
    assert!(!caps.multi_atlas);
}

#[test]
fn default_fallback_chain_has_defaults() {
    let chain = FontFallbackChain::default();
    assert!(!chain.families.is_empty());
    assert!(chain.script_overrides.contains_key("CJK"));
}

#[test]
fn script_override_takes_priority() {
    let chain = FontFallbackChain::default();
    let cjk = chain.for_script("CJK");
    assert!(cjk.iter().any(|f| f.contains("CJK")));
}

#[test]
fn default_strategy_is_css_like() {
    assert_eq!(FontMatchStrategy::default(), FontMatchStrategy::CssLike);
}

#[test]
fn default_subpixel_is_fractional() {
    assert_eq!(SubpixelMode::default(), SubpixelMode::Fractional);
}

#[test]
fn auto_if_small_hints_at_small_sizes() {
    let strategy = HintingStrategy::AutoIfSmall;
    assert_eq!(strategy.for_size(10.0), HintingStrategy::Auto);
    assert_eq!(strategy.for_size(14.0), HintingStrategy::Auto);
    assert_eq!(strategy.for_size(16.0), HintingStrategy::None);
    assert_eq!(strategy.for_size(24.0), HintingStrategy::None);
}

#[test]
fn explicit_strategies_unchanged() {
    assert_eq!(HintingStrategy::None.for_size(10.0), HintingStrategy::None);
    assert_eq!(HintingStrategy::Auto.for_size(24.0), HintingStrategy::Auto);
}
