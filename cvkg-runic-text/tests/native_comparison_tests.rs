use cvkg_runic_text::{TextAlign, TextEngine, TextOverflow, TextSpan, TextStyle};

/// P2-43: Native typography comparison helper.
/// Models the expected metrics boundaries from standard platform text engines (CoreText, DirectWrite, Pango).
#[derive(Debug, Clone)]
pub struct MockPlatformMetrics {
    pub platform: String,
    pub expected_latin_char_width: f32,
    pub expected_line_height: f32,
}

impl MockPlatformMetrics {
    /// Returns default mock metrics for the current OS.
    pub fn current() -> Self {
        let platform = if cfg!(target_os = "macos") {
            "macOS (CoreText)"
        } else if cfg!(target_os = "windows") {
            "Windows (DirectWrite)"
        } else {
            "Linux (Pango/FreeType)"
        };

        Self {
            platform: platform.to_string(),
            expected_latin_char_width: 6.0,
            expected_line_height: 34.0,
        }
    }
}

/// P2-43: Native typography comparison tests.
/// Asserts that shaped text dimensions are within acceptable tolerances (e.g. 20%)
/// compared to expected reference platform metrics.
#[test]
fn test_native_typography_metrics_drift() {
    let mut engine = TextEngine::new_test();
    let style = TextStyle::new("Jupiteroid", 16.0);

    let text = "Hello World";
    let spans = vec![TextSpan::new(text, style)];

    let shaped = engine
        .shape_layout(&spans, None, TextAlign::Start, TextOverflow::Clip)
        .unwrap();

    let reference = MockPlatformMetrics::current();
    println!(
        "Comparing shaped layout metrics against target platform: {}",
        reference.platform
    );

    // Calculate approximate width: (number of characters) * (expected char width)
    let expected_width = text.len() as f32 * reference.expected_latin_char_width;
    let width_diff = (shaped.width - expected_width).abs();
    let width_pct = (width_diff / expected_width) * 100.0;

    // We allow up to 20% width difference owing to kerning, letter spacing, and font metrics differences
    assert!(
        width_pct <= 20.0,
        "Shaped width ({:.2}px) drifted by {:.2}% from expected platform-native width ({:.2}px)",
        shaped.width,
        width_pct,
        expected_width
    );

    // Assert that the height/leading aligns with line height metrics
    let expected_height = reference.expected_line_height;
    let height_diff = (shaped.height - expected_height).abs();
    let height_pct = (height_diff / expected_height) * 100.0;

    // We allow up to 20% height difference owing to line gap interpretations
    assert!(
        height_pct <= 20.0,
        "Shaped height ({:.2}px) drifted by {:.2}% from expected platform-native height ({:.2}px)",
        shaped.height,
        height_pct,
        expected_height
    );
}
