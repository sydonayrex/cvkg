/// Probe quick-xml's escaping behavior to verify security properties.
/// Run with: cargo test --test escaping_probe -- --nocapture
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;

#[test]
fn escaping_probe() {
    // === 1. BytesText escaping for CSS content (the <style> block) ===
    let mut w = Writer::new_with_indent(Vec::new(), b' ', 2);
    w.write_event(Event::Start(BytesStart::new("style"))).unwrap();
    let css = ".a > .b { color: red; }\n.c::before { content: \"<\"; }";
    w.write_event(Event::Text(BytesText::new(css))).unwrap();
    w.write_event(Event::End(BytesEnd::new("style"))).unwrap();
    let out = String::from_utf8(w.into_inner()).unwrap();
    println!("=== BytesText CSS output ===");
    println!("{}", out);
    println!("Contains '&gt;' (BAD for CSS): {}", out.contains("&gt;"));
    println!("Contains '&lt;' (BAD for CSS): {}", out.contains("&lt;"));
    println!();

    // === 2. push_attribute value escaping ===
    let mut w2 = Writer::new_with_indent(Vec::new(), b' ', 2);
    let mut elem = BytesStart::new("div");
    let malicious_id = r#"foo" onmouseover="alert(1)" x="y"#;
    elem.push_attribute(("id", malicious_id));
    w2.write_event(Event::Empty(elem)).unwrap();
    let out2 = String::from_utf8(w2.into_inner()).unwrap();
    println!("=== Attribute value injection test ===");
    println!("{}", out2);
    println!("Injection succeeded: {}", out2.contains(r#"onmouseover"#));
    println!();

    // === 3. Attribute name injection via xmlns: prefix ===
    let mut w3 = Writer::new_with_indent(Vec::new(), b' ', 2);
    let mut root = BytesStart::new("svg");
    let malicious_prefix = r#"a" xmlns:evil="true""#;
    root.push_attribute((format!("xmlns:{}", malicious_prefix).as_str(), "http://example.com"));
    w3.write_event(Event::Empty(root)).unwrap();
    let out3 = String::from_utf8(w3.into_inner()).unwrap();
    println!("=== xmlns prefix injection test ===");
    println!("{}", out3);
    println!("Injection succeeded (has 'evil'): {}", out3.contains("evil"));
    println!();

    // === 4. Control characters in attribute values ===
    let mut w4 = Writer::new_with_indent(Vec::new(), b' ', 2);
    let mut elem4 = BytesStart::new("g");
    let ctrl_id = "test\x00null\x01\x02";
    elem4.push_attribute(("id", ctrl_id));
    w4.write_event(Event::Empty(elem4)).unwrap();
    let out4 = w4.into_inner();
    println!("=== Control char test ===");
    println!("Bytes: {:02x?}", &out4[..out4.len().min(100)]);
    println!("Is valid UTF-8: {}", String::from_utf8(out4.clone()).is_ok());
    println!();

    // === 5. Single quote vs double quote ===
    let mut w5 = Writer::new_with_indent(Vec::new(), b' ', 0);
    let mut elem5 = BytesStart::new("path");
    elem5.push_attribute(("d", "M10 10"));
    w5.write_event(Event::Empty(elem5)).unwrap();
    let out5 = String::from_utf8(w5.into_inner()).unwrap();
    println!("=== Quote style (always double) ===");
    println!("{}", out5);
    println!();
}
