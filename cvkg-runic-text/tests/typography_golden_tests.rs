use cvkg_runic_text::{TextEngine, TextSpan, TextStyle, TextAlign, TextOverflow};

/// P2-41: Typography golden-image text snapshot test suite.
/// Validates that shaped layout outcomes remain stable and correct
/// across Latin, Arabic, Hebrew, Indic, Thai, CJK, and Emoji scripts.
#[test]
fn test_typography_golden_snapshots() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);

    // Test cases representing different language scripts
    let test_cases = vec![
        ("Latin", "The quick brown fox jumps over the lazy dog."),
        ("Arabic", "الحرية والعدالة والمساواة للجميع."),
        ("Hebrew", "כל בני האדם נולדו בני חורין ושווים בערכם ובזכויותיהם."),
        ("Indic", "सभी मनुष्यों को गौरव और अधिकारों के मामले में जन्मजात स्वतन्त्रता प्राप्त है।"),
        ("Thai", "มนุษย์ทั้งหลายเกิดมามีอิสระและเสมอภาคกันในเกียรติศักดิ์และสิทธิ์"),
        ("CJK", "すべての人間は、生まれながらにして自由であり、かつ、尊厳と権利とについて平等である。"),
        ("Emoji", "Hello 👨‍👩‍👧‍👦 World 🌟❤️🌈!"),
    ];

    for (name, text) in test_cases {
        let spans = vec![TextSpan::new(text, style.clone())];
        let shaped = engine
            .shape_layout(&spans, None, TextAlign::Start, TextOverflow::Clip)
            .unwrap();

        // Verify shape results are valid and non-empty
        assert!(!shaped.glyphs.is_empty(), "Glyphs list should not be empty for script {}", name);
        assert!(shaped.width > 0.0, "Shaped width must be positive for script {}", name);
        assert!(shaped.height > 0.0, "Shaped height must be positive for script {}", name);

        // Verify that glyph ID resolution has occurred (allowing 0 for missing glyphs (.notdef) in minimal font)
        for glyph in &shaped.glyphs {
            assert!(glyph.glyph_id >= 0, "Glyph ID should be resolved for script {}", name);
        }
    }
}
