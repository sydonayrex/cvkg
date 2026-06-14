//! SVG parsing helpers and free functions.
use crate::types::SvgAnimation;

pub fn parse_svg_animations(data: &[u8]) -> Vec<SvgAnimation> {
    let mut parsed_animations = Vec::new();
    if let Ok(xml_doc) = roxmltree::Document::parse(std::str::from_utf8(data).unwrap_or("")) {
        for node in xml_doc.descendants() {
            if node.tag_name().name() == "animateTransform" || node.tag_name().name() == "animate" {
                let target_id = node
                    .attribute("href")
                    .or_else(|| node.attribute(("http://www.w3.org/1999/xlink", "href")))
                    .or_else(|| node.attribute("xlink:href"))
                    .or_else(|| node.parent_element().and_then(|p| p.attribute("id")))
                    .unwrap_or("")
                    .trim_start_matches('#')
                    .to_string();

                if !target_id.is_empty() {
                    let dur_str = node.attribute("dur").unwrap_or("1s");
                    let duration = if dur_str.ends_with("ms") {
                        dur_str
                            .trim_end_matches("ms")
                            .parse::<f32>()
                            .unwrap_or(1000.0)
                            / 1000.0
                    } else {
                        dur_str.trim_end_matches('s').parse::<f32>().unwrap_or(1.0)
                    };

                    let attr = node
                        .attribute("attributeName")
                        .unwrap_or("transform")
                        .to_string();

                    let (from_val, to_val) = if let Some(values) = node.attribute("values") {
                        let parts: Vec<&str> = values.split(';').collect();
                        if parts.len() >= 2 {
                            let f = parts[0].trim().parse::<f32>().unwrap_or(0.0);
                            let t = parts[parts.len() - 1].trim().parse::<f32>().unwrap_or(0.0);
                            (f, t)
                        } else {
                            (0.0, 360.0) // Fallback defaults
                        }
                    } else {
                        let f = node
                            .attribute("from")
                            .unwrap_or(if attr == "stroke-dashoffset" { "1" } else { "0" })
                            .parse::<f32>()
                            .unwrap_or(0.0);
                        let t = node
                            .attribute("to")
                            .unwrap_or(if attr == "stroke-dashoffset" { "0" } else { "360" })
                            .parse::<f32>()
                            .unwrap_or(if attr == "stroke-dashoffset" { 0.0 } else { 360.0 });
                        (f, t)
                    };

                    parsed_animations.push(SvgAnimation {
                        target_id,
                        attribute_name: attr,
                        from_val,
                        to_val,
                        duration,
                        vertex_range: 0..0, // Will be filled during tessellation
                    });
                }
            }
        }
    }
    parsed_animations
}

// --- SVG Helpers ---

pub(crate) fn usvg_to_lyon(path: &usvg::Path, transform: usvg::Transform) -> lyon::path::Path {
    let mut builder = lyon::path::Path::builder();
    let mut is_open = false;
    
    // Helper to transform a point
    let tx = |p: usvg::tiny_skia_path::Point| -> lyon::math::Point {
        let nx = transform.sx * p.x + transform.kx * p.y + transform.tx;
        let ny = transform.ky * p.x + transform.sy * p.y + transform.ty;
        lyon::math::point(nx, ny)
    };

    for segment in path.data().segments() {
        match segment {
            usvg::tiny_skia_path::PathSegment::MoveTo(p) => {
                if is_open {
                    builder.end(false);
                }
                builder.begin(tx(p));
                is_open = true;
            }
            usvg::tiny_skia_path::PathSegment::LineTo(p) => {
                builder.line_to(tx(p));
            }
            usvg::tiny_skia_path::PathSegment::QuadTo(p1, p) => {
                builder.quadratic_bezier_to(tx(p1), tx(p));
            }
            usvg::tiny_skia_path::PathSegment::CubicTo(p1, p2, p) => {
                builder.cubic_bezier_to(tx(p1), tx(p2), tx(p));
            }
            usvg::tiny_skia_path::PathSegment::Close => {
                if is_open {
                    builder.end(true);
                    is_open = false;
                }
            }
        }
    }
    if is_open {
        builder.end(false);
    }
    builder.build()
}
