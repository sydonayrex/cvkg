use cvkg_runic_text::{RunicTextEngine, TextSpan, TextStyle, TextAlign, TextOverflow};

#[test]
fn test_rustybuzz_clusters() {
    let mut engine = RunicTextEngine::new_test();
    let text = "A👨‍👩‍👧‍👦B"; // A (1 byte), Emoji (25 bytes), B (1 byte)
    let style = TextStyle::new("Jupiteroid", 16.0);
    let spans = vec![TextSpan::new(text, style)];
    
    let shaped = engine.shape_layout(&spans, None, TextAlign::Start, TextOverflow::Clip).unwrap();
    
    for g in &shaped.glyphs {
        println!("Glyph id: {}, cluster: {}", g.glyph_id, g.cluster);
    }
}
