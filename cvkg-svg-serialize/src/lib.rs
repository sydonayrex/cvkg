//! # CVKG SVG Serializer
//!
//! Provides SVG XML serialization for `usvg::Tree` with configurable output options.
//! Wraps usvg's built-in `WriteOptions` with CVKG's configuration types and integrates
//! into the CVKG `Renderer` trait.

use std::collections::HashMap;
use thiserror::Error;

// ── Error Type ──────────────────────────────────────────────────────────────

/// Errors that can occur during SVG serialization.
#[derive(Error, Debug)]
pub enum SvgSerializeError {
    /// The usvg tree could not be serialized.
    #[error("SVG serialization failed: {0}")]
    WriteError(String),
    /// An I/O error occurred during writing.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// The output exceeded the maximum allowed size.
    #[error("Output exceeded maximum size of {max} bytes (got {actual})")]
    Oversize { max: usize, actual: usize },
}

// ── Configuration ────────────────────────────────────────────────────────────

/// Controls how SVG XML is generated.
#[derive(Clone, Debug)]
pub struct SerializerConfig {
    /// Pretty-print indentation level (number of spaces). 0 = no indentation.
    pub indent: u32,
    /// If true, serialize presentation attributes as `style=""`.
    pub inline_style: bool,
    /// Float precision for coordinate values (number of decimal places).
    pub decimal_places: usize,
    /// If true, include the `<svg>` root element attributes.
    pub write_svg_declaration: bool,
    /// Custom namespace prefix -> URI pairs.
    pub custom_namespaces: HashMap<String, String>,
    /// If true, emit `<!-- -->` comments.
    pub comments: bool,
    /// Maximum output size in bytes (0 = unlimited).
    pub max_output_size: usize,
    /// ID prefix to add to all element IDs.
    pub id_prefix: Option<String>,
    /// If true, single quotes are used for attribute values instead of double quotes.
    pub use_single_quote: bool,
}

impl Default for SerializerConfig {
    fn default() -> Self {
        Self {
            indent: 2,
            inline_style: false,
            decimal_places: 3,
            write_svg_declaration: true,
            custom_namespaces: HashMap::new(),
            comments: false,
            max_output_size: 0,
            id_prefix: None,
            use_single_quote: false,
        }
    }
}

impl SerializerConfig {
    /// Convert to usvg `WriteOptions` for the built-in serializer.
    pub(crate) fn to_write_options(&self) -> usvg::WriteOptions {
        let indent = if self.indent == 0 {
            usvg::Indent::None
        } else {
            usvg::Indent::Spaces(self.indent as u8)
        };

        let coords_precision = self.decimal_places.min(15) as u8;

        usvg::WriteOptions {
            id_prefix: self.id_prefix.clone(),
            preserve_text: false,
            coordinates_precision: coords_precision,
            transforms_precision: coords_precision,
            use_single_quote: self.use_single_quote,
            indent,
            attributes_indent: usvg::Indent::None,
        }
    }
}

// ── Serialization Stats ──────────────────────────────────────────────────────

/// Statistics about a completed serialization pass.
#[derive(Clone, Debug, Default)]
pub struct SerializationStats {
    /// Number of elements written.
    pub element_count: usize,
    /// Number of attributes written.
    pub attribute_count: usize,
    /// Number of IDs that were auto-generated to resolve collisions.
    pub generated_ids: usize,
    /// Total size of the XML output in bytes.
    pub xml_size_bytes: usize,
}

impl SerializationStats {
    /// Create a new, empty stats record.
    pub fn new() -> Self {
        Self::default()
    }
}

impl std::fmt::Display for SerializationStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "elements={} attrs={} generated_ids={} size={}B",
            self.element_count, self.attribute_count, self.generated_ids, self.xml_size_bytes
        )
    }
}

// ── SVG Serializer ───────────────────────────────────────────────────────────

/// Serializes a `usvg::Tree` to valid SVG XML.
///
/// This is the primary type for SVG serialization. It walks the usvg DOM tree
/// and emits XML via usvg's built-in writer with CVKG's configuration mapped
/// to usvg's `WriteOptions`.
pub struct SvgSerializer {
    config: SerializerConfig,
    stats: SerializationStats,
}

impl SvgSerializer {
    /// Create a new serializer with default configuration.
    pub fn new() -> Self {
        Self {
            config: SerializerConfig::default(),
            stats: SerializationStats::new(),
        }
    }

    /// Create a new serializer with the given configuration.
    pub fn with_config(config: SerializerConfig) -> Self {
        Self {
            config,
            stats: SerializationStats::new(),
        }
    }

    /// Serialize a `usvg::Tree` to an SVG XML string.
    pub fn serialize(&mut self, tree: &usvg::Tree) -> Result<String, SvgSerializeError> {
        let opt = self.config.to_write_options();
        let svg_string = tree.to_string(&opt);

        // Enforce output size limit if configured.
        if self.config.max_output_size > 0 && svg_string.len() > self.config.max_output_size {
            return Err(SvgSerializeError::Oversize {
                max: self.config.max_output_size,
                actual: svg_string.len(),
            });
        }

        // Update stats.
        self.stats.xml_size_bytes = svg_string.len();
        self.stats.element_count = count_xml_elements(&svg_string);

        Ok(svg_string)
    }

    /// Serialize a `usvg::Tree` to SVG XML and write it to the given writer.
    pub fn serialize_to<W: std::io::Write>(
        &mut self,
        tree: &usvg::Tree,
        writer: &mut W,
    ) -> Result<(), SvgSerializeError> {
        let svg_string = self.serialize(tree)?;
        writer.write_all(svg_string.as_bytes())?;
        Ok(())
    }

    /// Serialize to a `Vec<u8>` (convenience method for clipboard/file export).
    pub fn serialize_to_vec(&mut self, tree: &usvg::Tree) -> Result<Vec<u8>, SvgSerializeError> {
        let svg_string = self.serialize(tree)?;
        Ok(svg_string.into_bytes())
    }

    /// Returns the statistics from the last serialization.
    pub fn stats(&self) -> &SerializationStats {
        &self.stats
    }

    /// Returns the configuration.
    pub fn config(&self) -> &SerializerConfig {
        &self.config
    }

    /// Returns the mutable configuration.
    pub fn config_mut(&mut self) -> &mut SerializerConfig {
        &mut self.config
    }
}

impl Default for SvgSerializer {
    fn default() -> Self {
        Self::new()
    }
}

// ── Free Functions ───────────────────────────────────────────────────────────

/// Serialize a `usvg::Tree` to an SVG XML string with default options.
///
/// This is a convenience function for one-shot serialization.
pub fn serialize_svg(tree: &usvg::Tree) -> Result<String, SvgSerializeError> {
    let mut serializer = SvgSerializer::new();
    serializer.serialize(tree)
}

/// Serialize a `usvg::Tree` to an SVG XML string with custom configuration.
pub fn serialize_svg_with_config(
    tree: &usvg::Tree,
    config: SerializerConfig,
) -> Result<String, SvgSerializeError> {
    let mut serializer = SvgSerializer::with_config(config);
    serializer.serialize(tree)
}

/// Serialize a `usvg::Tree` directly to a file path.
pub fn serialize_svg_to_file(
    tree: &usvg::Tree,
    path: &std::path::Path,
) -> Result<(), SvgSerializeError> {
    let svg_string = serialize_svg(tree)?;
    std::fs::write(path, svg_string)?;
    Ok(())
}

// ── Float Formatting Utilities ──────────────────────────────────────────────

/// Format an f32 for SVG output with the given number of decimal places.
///
/// Removes trailing zeros and uses scientific notation for very small/large numbers.
pub fn format_svg_float(value: f32, decimal_places: usize) -> String {
    const SMALL_THRESHOLD: f32 = 1e-6;
    const LARGE_THRESHOLD: f32 = 1e6;

    if value == 0.0 {
        return "0".to_string();
    }

    let abs = value.abs();
    if abs < SMALL_THRESHOLD || abs > LARGE_THRESHOLD {
        format!("{value:.1e}")
    } else {
        let formatted = format!("{:.*}", decimal_places, value);
        if formatted.contains('.') {
            let trimmed = formatted.trim_end_matches('0');
            if trimmed.ends_with('.') {
                trimmed[..trimmed.len() - 1].to_string()
            } else {
                trimmed.to_string()
            }
        } else {
            formatted
        }
    }
}

/// Format an SVG color as `#rrggbb` or `#rrggbbaa`.
pub fn format_svg_color(color: &usvg::Color, opacity: usvg::Opacity) -> String {
    let alpha = (opacity.get() * 255.0).round() as u8;
    if alpha == 255 {
        format!("#{:02x}{:02x}{:02x}", color.red, color.green, color.blue)
    } else {
        format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            color.red, color.green, color.blue, alpha
        )
    }
}

/// Format an SVG `Transform` attribute string.
pub fn format_svg_transform(transform: &usvg::Transform) -> String {
    if transform.is_identity() {
        String::new()
    } else {
        format!(
            "matrix({} {} {} {} {} {})",
            format_svg_float(transform.sx, 3),
            format_svg_float(transform.ky, 3),
            format_svg_float(transform.kx, 3),
            format_svg_float(transform.sy, 3),
            format_svg_float(transform.tx, 3),
            format_svg_float(transform.ty, 3),
        )
    }
}

// ── Internal Helpers ────────────────────────────────────────────────────────

/// Count the number of XML element close tags in a string (rough estimate for stats).
fn count_xml_elements(xml: &str) -> usize {
    xml.matches("</").count()
}

// ── ID Management ────────────────────────────────────────────────────────────

/// Tracks IDs during serialization to detect and resolve collisions.
#[derive(Debug, Default)]
pub struct IdTracker {
    seen: HashMap<String, u32>,
}

impl IdTracker {
    /// Create a new, empty ID tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an ID, returning a unique variant if there's a collision.
    ///
    /// If the ID has been seen before, appends `_1`, `_2`, etc. until unique.
    pub fn register(&mut self, id: &str) -> String {
        if id.is_empty() {
            return String::new();
        }

        match self.seen.get(id) {
            None => {
                self.seen.insert(id.to_string(), 0);
                id.to_string()
            }
            Some(&count) => {
                let new_count = count + 1;
                self.seen.insert(id.to_string(), new_count);
                let candidate = format!("{id}_{new_count}");
                self.register(&candidate)
            }
        }
    }

    /// Reset all tracked IDs.
    pub fn clear(&mut self) {
        self.seen.clear();
    }
}

// ── SerializableNode trait ──────────────────────────────────────────────────

/// Trait for third-party code to extend serialization of specific node types.
pub trait SerializableNode {
    /// Return the SVG attribute key-value pairs for this node.
    fn to_svg_attrs(&self) -> Vec<(&str, String)>;
    /// Return the children of this node, if any.
    fn children(&self) -> Option<&[usvg::Node]>;
}

impl SerializableNode for usvg::Node {
    fn to_svg_attrs(&self) -> Vec<(&str, String)> {
        let mut attrs = Vec::new();
        match self {
            usvg::Node::Group(g) => {
                if !g.id().is_empty() {
                    attrs.push(("id", g.id().to_string()));
                }
                let transform = g.abs_transform();
                if !transform.is_identity() {
                    attrs.push(("transform", format_svg_transform(&transform)));
                }
                let opacity = g.opacity().get();
                if (opacity - 1.0).abs() > f32::EPSILON {
                    attrs.push(("opacity", format_svg_float(opacity, 3)));
                }
                if g.blend_mode() != usvg::BlendMode::Normal {
                    attrs.push(("mix-blend-mode", blend_mode_to_svg(g.blend_mode())));
                }
            }
            usvg::Node::Path(p) => {
                if !p.id().is_empty() {
                    attrs.push(("id", p.id().to_string()));
                }
                if let Some(ref fill) = p.fill() {
                    attrs.extend(fill_to_svg_attrs(fill));
                }
                if let Some(ref stroke) = p.stroke() {
                    attrs.extend(stroke_to_svg_attrs(stroke));
                }
            }
            usvg::Node::Image(img) => {
                if !img.id().is_empty() {
                    attrs.push(("id", img.id().to_string()));
                }
            }
            usvg::Node::Text(t) => {
                if !t.id().is_empty() {
                    attrs.push(("id", t.id().to_string()));
                }
            }
        }
        attrs
    }

    fn children(&self) -> Option<&[usvg::Node]> {
        match self {
            usvg::Node::Group(g) => Some(g.children()),
            // Text nodes are flat (chunks/spans), not a child tree.
            _ => None,
        }
    }
}

// ── Attribute Helper Functions ───────────────────────────────────────────────

/// Convert a `usvg::Fill` to SVG attribute pairs.
pub fn fill_to_svg_attrs(fill: &usvg::Fill) -> Vec<(&'static str, String)> {
    let mut attrs = Vec::new();

    match fill.paint() {
        usvg::Paint::Color(c) => {
            attrs.push(("fill", format_svg_color(c, fill.opacity())));
        }
        usvg::Paint::LinearGradient(lg) => {
            attrs.push(("fill", format!("url(#{})", lg.id())));
        }
        usvg::Paint::RadialGradient(rg) => {
            attrs.push(("fill", format!("url(#{})", rg.id())));
        }
        usvg::Paint::Pattern(pat) => {
            attrs.push(("fill", format!("url(#{})", pat.id())));
        }
    }

    let opacity = fill.opacity().get();
    if (opacity - 1.0).abs() > f32::EPSILON {
        attrs.push(("fill-opacity", format_svg_float(opacity, 3)));
    }

    if fill.rule() != usvg::FillRule::NonZero {
        attrs.push(("fill-rule", "evenodd".to_string()));
    }

    attrs
}

/// Convert a `usvg::Stroke` to SVG attribute pairs.
pub fn stroke_to_svg_attrs(stroke: &usvg::Stroke) -> Vec<(&'static str, String)> {
    let mut attrs = Vec::new();

    match stroke.paint() {
        usvg::Paint::Color(c) => {
            attrs.push(("stroke", format_svg_color(c, stroke.opacity())));
        }
        usvg::Paint::LinearGradient(lg) => {
            attrs.push(("stroke", format!("url(#{})", lg.id())));
        }
        usvg::Paint::RadialGradient(rg) => {
            attrs.push(("stroke", format!("url(#{})", rg.id())));
        }
        usvg::Paint::Pattern(pat) => {
            attrs.push(("stroke", format!("url(#{})", pat.id())));
        }
    }

    let opacity = stroke.opacity().get();
    if (opacity - 1.0).abs() > f32::EPSILON {
        attrs.push(("stroke-opacity", format_svg_float(opacity, 3)));
    }

    let width = stroke.width().get();
    if (width - 1.0).abs() > f32::EPSILON {
        attrs.push(("stroke-width", format_svg_float(width, 3)));
    }

    match stroke.linecap() {
        usvg::LineCap::Butt => {} // default
        usvg::LineCap::Round => attrs.push(("stroke-linecap", "round".to_string())),
        usvg::LineCap::Square => attrs.push(("stroke-linecap", "square".to_string())),
    }

    match stroke.linejoin() {
        usvg::LineJoin::Miter => {} // default
        usvg::LineJoin::Round => attrs.push(("stroke-linejoin", "round".to_string())),
        usvg::LineJoin::Bevel => attrs.push(("stroke-linejoin", "bevel".to_string())),
        usvg::LineJoin::MiterClip => attrs.push(("stroke-linejoin", "miter-clip".to_string())),
    }

    let miter = stroke.miterlimit().get();
    if (miter - 4.0).abs() > f32::EPSILON {
        attrs.push(("stroke-miterlimit", format_svg_float(miter, 3)));
    }

    if let Some(dashes) = stroke.dasharray() {
        let dash_str: Vec<String> = dashes.iter().map(|d| format_svg_float(*d, 3)).collect();
        attrs.push(("stroke-dasharray", dash_str.join(" ")));
        let offset = stroke.dashoffset();
        if offset.abs() > f32::EPSILON {
            attrs.push(("stroke-dashoffset", format_svg_float(offset, 3)));
        }
    }

    attrs
}

/// Convert a `usvg::BlendMode` to its SVG string.
pub fn blend_mode_to_svg(mode: usvg::BlendMode) -> String {
    match mode {
        usvg::BlendMode::Normal => "normal",
        usvg::BlendMode::Multiply => "multiply",
        usvg::BlendMode::Screen => "screen",
        usvg::BlendMode::Overlay => "overlay",
        usvg::BlendMode::Darken => "darken",
        usvg::BlendMode::Lighten => "lighten",
        usvg::BlendMode::ColorDodge => "color-dodge",
        usvg::BlendMode::ColorBurn => "color-burn",
        usvg::BlendMode::HardLight => "hard-light",
        usvg::BlendMode::SoftLight => "soft-light",
        usvg::BlendMode::Difference => "difference",
        usvg::BlendMode::Exclusion => "exclusion",
        usvg::BlendMode::Hue => "hue",
        usvg::BlendMode::Saturation => "saturation",
        usvg::BlendMode::Color => "color",
        usvg::BlendMode::Luminosity => "luminosity",
    }
    .to_string()
}

/// Convert a `usvg::ClipPath` to SVG attribute pairs.
pub fn clip_path_to_svg_attrs(clip: &usvg::ClipPath) -> Vec<(&'static str, String)> {
    if clip.id().is_empty() {
        Vec::new()
    } else {
        vec![("clip-path", format!("url(#{})", clip.id()))]
    }
}

/// Convert a `usvg::Mask` to SVG attribute pairs.
pub fn mask_to_svg_attrs(mask: &usvg::Mask) -> Vec<(&'static str, String)> {
    if mask.id().is_empty() {
        Vec::new()
    } else {
        let mut attrs = vec![("mask", format!("url(#{})", mask.id()))];
        if mask.kind() != usvg::MaskType::Luminance {
            attrs.push(("mask-type", "alpha".to_string()));
        }
        attrs
    }
}

/// Current version of the cvkg-svg-serialize crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_svg_float_zero() {
        assert_eq!(format_svg_float(0.0, 3), "0");
    }

    #[test]
    fn test_format_svg_float_whole() {
        assert_eq!(format_svg_float(42.0, 3), "42");
    }

    #[test]
    fn test_format_svg_float_decimal() {
        assert_eq!(format_svg_float(3.14159, 3), "3.142");
    }

    #[test]
    fn test_format_svg_float_trailing_zeros() {
        assert_eq!(format_svg_float(1.500, 3), "1.5");
    }

    #[test]
    fn test_format_svg_float_scientific_small() {
        let result = format_svg_float(0.0000001, 3);
        assert!(result.contains("e"), "Expected scientific notation, got: {result}");
    }

    #[test]
    fn test_format_svg_color_opaque() {
        let c = usvg::Color::new_rgb(255, 128, 0);
        assert_eq!(format_svg_color(&c, usvg::Opacity::ONE), "#ff8000");
    }

    #[test]
    fn test_format_svg_color_transparent() {
        let c = usvg::Color::new_rgb(255, 128, 0);
        let alpha = usvg::Opacity::new(0.5).unwrap();
        let result = format_svg_color(&c, alpha);
        assert!(result.starts_with("#ff8000"), "Got: {result}");
    }

    #[test]
    fn test_id_tracker_unique() {
        let mut t = IdTracker::new();
        assert_eq!(t.register("foo"), "foo");
        assert_eq!(t.register("bar"), "bar");
    }

    #[test]
    fn test_id_tracker_collision() {
        let mut t = IdTracker::new();
        assert_eq!(t.register("foo"), "foo");
        assert_eq!(t.register("foo"), "foo_1");
        assert_eq!(t.register("foo"), "foo_2");
    }

    #[test]
    fn test_serialize_basic_rect() {
        let svg_input = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100" fill="red"/></svg>"#;
        let tree = usvg::Tree::from_str(svg_input, &usvg::Options::default()).expect("parse failed");
        let result = serialize_svg(&tree);
        assert!(result.is_ok());
        let xml = result.unwrap();
        assert!(xml.contains("#ff0000"), "Expected red fill in output: {xml}");
        
    }

    #[test]
    fn test_serialize_with_config() {
        let mut config = SerializerConfig::default();
        config.indent = 0;
        config.decimal_places = 1;

        let svg_input = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100" fill="blue"/></svg>"#;
        let tree = usvg::Tree::from_str(svg_input, &usvg::Options::default()).expect("parse failed");
        let result = serialize_svg_with_config(&tree, config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_serialize_empty_svg() {
        let svg_input = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"></svg>"#;
        let tree = usvg::Tree::from_str(svg_input, &usvg::Options::default()).expect("parse failed");
        let result = serialize_svg(&tree);
        assert!(result.is_ok());
    }

    #[test]
    fn test_serialize_nested_groups() {
        let svg_input = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><g id="a"><g id="b"><rect width="50" height="50"/></g></g></svg>"#;
        let tree = usvg::Tree::from_str(svg_input, &usvg::Options::default()).expect("parse failed");
        let result = serialize_svg(&tree);
        assert!(result.is_ok());
    }

    #[test]
    fn test_serialization_stats() {
        let svg_input = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100"/></svg>"#;
        let tree = usvg::Tree::from_str(svg_input, &usvg::Options::default()).expect("parse failed");
        let mut serializer = SvgSerializer::new();
        let _ = serializer.serialize(&tree).unwrap();
        let stats = serializer.stats();
        assert!(stats.xml_size_bytes > 0);
        assert!(stats.element_count > 0);
    }

    #[test]
    fn test_blend_mode_normal() {
        assert_eq!(blend_mode_to_svg(usvg::BlendMode::Normal), "normal");
    }

    #[test]
    fn test_blend_mode_multiply() {
        assert_eq!(blend_mode_to_svg(usvg::BlendMode::Multiply), "multiply");
    }

    #[test]
    fn test_format_svg_transform_identity() {
        let identity = usvg::Transform::identity();
        assert_eq!(format_svg_transform(&identity), "");
    }

    #[test]
    fn test_format_svg_transform_translate() {
        let t = usvg::Transform::from_translate(10.0, 20.0);
        let result = format_svg_transform(&t);
        assert!(result.starts_with("matrix("), "Got: {result}");
        assert!(result.contains("10"), "Expected 10 in: {result}");
        assert!(result.contains("20"), "Expected 20 in: {result}");
    }
}

