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

pub trait SvgInterceptor {
    /// Provide global styles to be injected into a `<style>` block at the top of the SVG.
    fn global_styles(&self) -> Option<String> {
        None
    }

    /// Inject custom attributes (e.g., `data-*`) into the element with the given `id`.
    fn inject_attributes(&self, _id: &str) -> Vec<(&'static str, String)> {
        Vec::new()
    }

    /// Inject raw child XML payloads (e.g., HTML inside `<foreignObject>`) into the element with the given `id`.
    fn inject_children(&self, _id: &str) -> Option<String> {
        None
    }
}

/// Serializes a `usvg::Tree` to valid SVG XML.
///
/// This uses usvg's built-in writer but performs a post-processing pass using `quick-xml`
/// to inject UI metadata, animations, and custom payloads.
pub struct SvgSerializer<'a> {
    config: SerializerConfig,
    stats: SerializationStats,
    interceptor: Option<Box<dyn SvgInterceptor + 'a>>,
}

impl<'a> SvgSerializer<'a> {
    /// Create a new serializer with default configuration.
    pub fn new() -> Self {
        Self {
            config: SerializerConfig::default(),
            stats: SerializationStats::new(),
            interceptor: None,
        }
    }

    /// Create a new serializer with the given configuration.
    pub fn with_config(config: SerializerConfig) -> Self {
        Self {
            config,
            stats: SerializationStats::new(),
            interceptor: None,
        }
    }

    /// Set the interceptor for injecting UI metadata.
    pub fn with_interceptor(mut self, interceptor: Box<dyn SvgInterceptor + 'a>) -> Self {
        self.interceptor = Some(interceptor);
        self
    }

    /// Serialize a `usvg::Tree` to an SVG XML string.
    pub fn serialize(&mut self, tree: &usvg::Tree) -> Result<String, SvgSerializeError> {
        use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
        let mut writer =
            quick_xml::Writer::new_with_indent(Vec::new(), b' ', self.config.indent as usize);

        // 1. Write SVG declaration
        if self.config.write_svg_declaration {
            writer
                .write_event(Event::Decl(quick_xml::events::BytesDecl::new(
                    "1.0",
                    Some("UTF-8"),
                    None,
                )))
                .map_err(|e| SvgSerializeError::WriteError(e.to_string()))?;
        }

        // 2. Write <svg> root element
        let mut root_elem = BytesStart::new("svg");
        root_elem.push_attribute(("xmlns", "http://www.w3.org/2000/svg"));
        for (prefix, uri) in &self.config.custom_namespaces {
            root_elem.push_attribute((format!("xmlns:{}", prefix).as_str(), uri.as_str()));
        }

        let width_str = format_svg_float(tree.size().width(), self.config.decimal_places);
        let height_str = format_svg_float(tree.size().height(), self.config.decimal_places);
        root_elem.push_attribute(("width", width_str.as_str()));
        root_elem.push_attribute(("height", height_str.as_str()));
        root_elem.push_attribute((
            "viewBox",
            format!("0 0 {} {}", width_str, height_str).as_str(),
        ));

        writer
            .write_event(Event::Start(root_elem))
            .map_err(|e| SvgSerializeError::WriteError(e.to_string()))?;

        // 3. Global Styles
        if let Some(interceptor) = &self.interceptor
            && let Some(styles) = interceptor.global_styles()
        {
            writer
                .write_event(Event::Start(BytesStart::new("style")))
                .map_err(|e| SvgSerializeError::WriteError(e.to_string()))?;
            writer
                .write_event(Event::Text(BytesText::new(&styles)))
                .map_err(|e| SvgSerializeError::WriteError(e.to_string()))?;
            writer
                .write_event(Event::End(BytesEnd::new("style")))
                .map_err(|e| SvgSerializeError::WriteError(e.to_string()))?;
        }

        // 4. Recursive Traversal
        self.serialize_node(
            usvg::Node::Group(Box::new(tree.root().clone())),
            &mut writer,
        )?;

        // 5. Close <svg>
        writer
            .write_event(Event::End(BytesEnd::new("svg")))
            .map_err(|e| SvgSerializeError::WriteError(e.to_string()))?;

        let svg_string = String::from_utf8(writer.into_inner()).unwrap();

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

    fn serialize_node<W: std::io::Write>(
        &self,
        node: usvg::Node,
        writer: &mut quick_xml::Writer<W>,
    ) -> Result<(), SvgSerializeError> {
        use quick_xml::events::{BytesEnd, BytesStart, Event};

        let tag_name = match &node {
            usvg::Node::Group(_) => "g",
            usvg::Node::Path(_) => "path",
            usvg::Node::Image(_) => "image",
            usvg::Node::Text(_) => "text",
        };

        let mut elem = BytesStart::new(tag_name);

        let mut node_id = String::new();
        for (k, v) in node.to_svg_attrs() {
            if k == "id" {
                node_id = v.clone();
                // apply id prefix
                if let Some(prefix) = &self.config.id_prefix {
                    node_id = format!("{}{}", prefix, node_id);
                    elem.push_attribute((k, node_id.as_str()));
                    continue;
                }
            }
            elem.push_attribute((k, v.as_str()));
        }

        let injected_attrs = if let Some(interceptor) = &self.interceptor {
            if !node_id.is_empty() {
                interceptor.inject_attributes(&node_id)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        for (k, v) in injected_attrs {
            elem.push_attribute((k, v.as_str()));
        }

        let child_payload = if let Some(interceptor) = &self.interceptor {
            if !node_id.is_empty() {
                interceptor.inject_children(&node_id)
            } else {
                None
            }
        } else {
            None
        };

        let children = node.children();
        let has_children = children.is_some_and(|c| !c.is_empty());
        let has_payload = child_payload.is_some();

        if !has_children && !has_payload {
            writer
                .write_event(Event::Empty(elem))
                .map_err(|e| SvgSerializeError::WriteError(e.to_string()))?;
        } else {
            writer
                .write_event(Event::Start(elem))
                .map_err(|e| SvgSerializeError::WriteError(e.to_string()))?;

            if let Some(payload) = child_payload {
                writer
                    .get_mut()
                    .write_all(payload.as_bytes())
                    .map_err(SvgSerializeError::Io)?;
            }

            if let Some(children_nodes) = children {
                for child in children_nodes {
                    self.serialize_node(child.clone(), writer)?;
                }
            }

            writer
                .write_event(Event::End(BytesEnd::new(tag_name)))
                .map_err(|e| SvgSerializeError::WriteError(e.to_string()))?;
        }

        Ok(())
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

impl<'a> Default for SvgSerializer<'a> {
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
    if !(SMALL_THRESHOLD..=LARGE_THRESHOLD).contains(&abs) {
        format!("{value:.1e}")
    } else {
        let formatted = format!("{:.*}", decimal_places, value);
        if formatted.contains('.') {
            formatted
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
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
                if let Some(fill) = p.fill() {
                    attrs.extend(fill_to_svg_attrs(fill));
                }
                if let Some(stroke) = p.stroke() {
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
    #[allow(clippy::approx_constant)]
    fn test_format_svg_float_decimal() {
        assert_eq!(format_svg_float(3.14159_f32, 3), "3.142");
    }

    #[test]
    fn test_format_svg_float_trailing_zeros() {
        assert_eq!(format_svg_float(1.500, 3), "1.5");
    }

    #[test]
    fn test_format_svg_float_scientific_small() {
        let result = format_svg_float(0.0000001, 3);
        assert!(
            result.contains("e"),
            "Expected scientific notation, got: {result}"
        );
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
        let tree =
            usvg::Tree::from_str(svg_input, &usvg::Options::default()).expect("parse failed");
        let result = serialize_svg(&tree);
        assert!(result.is_ok());
        let xml = result.unwrap();
        assert!(
            xml.contains("#ff0000"),
            "Expected red fill in output: {xml}"
        );
    }

    #[test]
    fn test_serialize_with_config() {
        let config = SerializerConfig {
            indent: 0,
            decimal_places: 1,
            ..Default::default()
        };

        let svg_input = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100" fill="blue"/></svg>"#;
        let tree =
            usvg::Tree::from_str(svg_input, &usvg::Options::default()).expect("parse failed");
        let result = serialize_svg_with_config(&tree, config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_serialize_empty_svg() {
        let svg_input = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"></svg>"#;
        let tree =
            usvg::Tree::from_str(svg_input, &usvg::Options::default()).expect("parse failed");
        let result = serialize_svg(&tree);
        assert!(result.is_ok());
    }

    #[test]
    fn test_serialize_nested_groups() {
        let svg_input = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><g id="a"><g id="b"><rect width="50" height="50"/></g></g></svg>"#;
        let tree =
            usvg::Tree::from_str(svg_input, &usvg::Options::default()).expect("parse failed");
        let result = serialize_svg(&tree);
        assert!(result.is_ok());
    }

    #[test]
    fn test_serialization_stats() {
        let svg_input = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100"/></svg>"#;
        let tree =
            usvg::Tree::from_str(svg_input, &usvg::Options::default()).expect("parse failed");
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

    struct TestInterceptor;
    impl SvgInterceptor for TestInterceptor {
        fn global_styles(&self) -> Option<String> {
            Some(".my-anim { animation: pulse 1s; }".to_string())
        }

        fn inject_attributes(&self, id: &str) -> Vec<(&'static str, String)> {
            if id == "test_node" {
                vec![("data-state", "hover".to_string())]
            } else {
                Vec::new()
            }
        }

        fn inject_children(&self, id: &str) -> Option<String> {
            if id == "test_node" {
                Some("<foreignObject><div>Hello</div></foreignObject>".to_string())
            } else {
                None
            }
        }
    }

    #[test]
    fn test_svg_interceptor_hydration() {
        let svg_input = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><g id="test_node"><rect width="50" height="50"/></g><rect id="empty_node" width="10" height="10"/></svg>"#;
        let tree =
            usvg::Tree::from_str(svg_input, &usvg::Options::default()).expect("parse failed");

        let mut serializer = SvgSerializer::new().with_interceptor(Box::new(TestInterceptor));
        let xml = serializer.serialize(&tree).expect("serialize failed");

        assert!(
            xml.contains("<style>.my-anim { animation: pulse 1s; }</style>"),
            "Missing global styles: {}",
            xml
        );
        assert!(
            xml.contains("data-state=\"hover\""),
            "Missing injected attribute: {}",
            xml
        );
        assert!(
            xml.contains("<foreignObject><div>Hello</div></foreignObject>"),
            "Missing injected children: {}",
            xml
        );

        // Also verify empty_node converted properly if it matched
        assert!(xml.contains(r#"id="empty_node""#));
    }
}

#[cfg(test)]
mod smoke_tests {
    use super::*;

    #[test]
    fn serializer_config_default() {
        let config = SerializerConfig::default();
        assert_eq!(config.indent, 2);
        assert_eq!(config.decimal_places, 3);
    }

    #[test]
    fn serialization_stats_default() {
        let stats = SerializationStats::default();
        assert_eq!(stats.element_count, 0);
        assert_eq!(stats.xml_size_bytes, 0);
    }

    #[test]
    fn id_tracker_constructs() {
        let tracker = IdTracker::new();
        assert_eq!(tracker.register("test"), "test");
    }

    #[test]
    fn svg_serialize_error_display() {
        let err = SvgSerializeError::WriteError("fail".into());
        assert!(err.to_string().contains("fail"));
    }
}
