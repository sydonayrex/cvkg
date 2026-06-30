//! # CVKG Icon System
//!
//! Provides an icon registry and `Icon` component for rendering named icons.
//! Icons are defined as SVG path data and rendered via the GPU path rasterizer
//! (draw_svg) with material_id=0 (Opaque) and draw_order=200 (above UI chrome,
//! below overlays).
//!
//! ## Usage
//!
//! ```no_run
//! use cvkg_icons::{IconRegistry, IconData};
//!
//! let mut registry = IconRegistry::with_defaults();
//! registry.register("close", IconData::Svg("M6 6 L18 18 M18 6 L6 18".into()));
//! ```

use cvkg_core::{AriaProperties, AriaRole, Never, Rect, Renderer, View};
use std::collections::HashMap;

// --- Icon Data ---

/// Source data for an icon.
#[derive(Debug, Clone)]
pub enum IconData {
    /// SVG path data (SVG path `d` attribute format).
    Svg(String),
    /// Icon font glyph index (for future icon font support).
    Glyph(u32),
}

// --- Icon Registry ---

/// Thread-safe icon registry mapping icon names to their data.
///
/// Icons are registered by name and looked up at render time.
/// The registry ships with a default set of 24 common icons.
#[derive(Debug, Clone)]
pub struct IconRegistry {
    icons: HashMap<String, IconData>,
}

impl Default for IconRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl IconRegistry {
    /// Creates an empty icon registry.
    pub fn new() -> Self {
        Self {
            icons: HashMap::new(),
        }
    }

    /// Creates a registry pre-populated with the default icon set.
    pub fn with_defaults() -> Self {
        let mut reg = Self::new();
        reg.load_defaults();
        reg
    }

    /// Registers an icon by name, replacing any previous entry.
    pub fn register(&mut self, name: &str, data: IconData) {
        self.icons.insert(name.to_string(), data);
    }

    /// Returns the icon data for a name, or None if not found.
    pub fn get(&self, name: &str) -> Option<&IconData> {
        self.icons.get(name)
    }

    /// Returns true if the registry contains the given icon name.
    pub fn contains(&self, name: &str) -> bool {
        self.icons.contains_key(name)
    }

    /// Returns the number of registered icons.
    pub fn len(&self) -> usize {
        self.icons.len()
    }

    /// Returns true if the registry has no icons.
    pub fn is_empty(&self) -> bool {
        self.icons.is_empty()
    }

    fn load_defaults(&mut self) {
        // Navigation
        self.register("close", IconData::Svg("M6 6 L18 18 M18 6 L6 18".into()));
        self.register(
            "menu",
            IconData::Svg("M3 6 L21 6 M3 12 L21 12 M3 18 L21 18".into()),
        );
        self.register("chevron-right", IconData::Svg("M9 6 L15 12 L9 18".into()));
        self.register("chevron-down", IconData::Svg("M6 9 L12 15 L18 9".into()));
        self.register("chevron-left", IconData::Svg("M15 6 L9 12 L15 18".into()));
        self.register("chevron-up", IconData::Svg("M6 15 L12 9 L18 15".into()));
        self.register(
            "arrow-right",
            IconData::Svg("M5 12 L19 12 M13 6 L19 12 L13 18".into()),
        );
        self.register(
            "arrow-left",
            IconData::Svg("M19 12 L5 12 M11 6 L5 12 L11 18".into()),
        );

        // Actions
        self.register("add", IconData::Svg("M12 5 L12 19 M5 12 L19 12".into()));
        self.register("remove", IconData::Svg("M5 12 L19 12".into()));
        self.register(
            "search",
            IconData::Svg("M11 4 A7 7 0 1 0 11 18 A7 7 0 1 0 11 4 M16 16 L21 21".into()),
        );
        self.register("settings", IconData::Svg("M12 8 A4 4 0 1 0 12 16 A4 4 0 1 0 12 8 M12 1 L12 4 M12 20 L12 23 M1 12 L4 12 M20 12 L23 12".into()));
        self.register(
            "edit",
            IconData::Svg("M3 21 L3 17 L17 7 L21 11 L7 21 Z".into()),
        );
        self.register("delete", IconData::Svg("M6 6 L18 18 M18 6 L6 18".into()));
        self.register(
            "copy",
            IconData::Svg("M8 4 L8 16 L20 16 L20 8 L16 4 Z M8 8 L16 8 L16 4".into()),
        );
        self.register("check", IconData::Svg("M4 12 L10 18 L20 6".into()));

        // Status
        self.register(
            "info",
            IconData::Svg(
                "M12 4 A8 8 0 1 0 12 20 A8 8 0 1 0 12 4 M12 11 L12 17 M12 8 L12 8.5".into(),
            ),
        );
        self.register(
            "warning",
            IconData::Svg("M12 3 L22 21 L2 21 Z M12 10 L12 15 M12 17 L12 17.5".into()),
        );
        self.register(
            "error",
            IconData::Svg(
                "M12 4 A8 8 0 1 0 12 20 A8 8 0 1 0 12 4 M12 8 L12 13 M12 16 L12 16.5".into(),
            ),
        );
        self.register(
            "success",
            IconData::Svg("M12 4 A8 8 0 1 0 12 20 A8 8 0 1 0 12 4 M8 12 L11 15 L16 9".into()),
        );

        // Media
        self.register("play", IconData::Svg("M8 5 L19 12 L8 19 Z".into()));
        self.register("pause", IconData::Svg("M6 5 L6 19 M18 5 L18 19".into()));
        self.register("stop", IconData::Svg("M5 5 L19 5 L19 19 L5 19 Z".into()));

        // Misc
        self.register(
            "home",
            IconData::Svg("M3 12 L12 3 L21 12 L21 21 L3 21 Z".into()),
        );
        self.register(
            "user",
            IconData::Svg("M12 4 A4 4 0 1 0 12 12 A4 4 0 1 0 12 4 M4 21 A8 8 0 1 1 20 21".into()),
        );
        self.register(
            "calendar",
            IconData::Svg("M4 4 L20 4 L20 20 L4 20 Z M4 10 L20 10 M8 4 L8 2 M16 4 L16 2".into()),
        );
        self.register(
            "mail",
            IconData::Svg("M3 5 L21 5 L21 19 L3 19 Z M3 5 L12 13 L21 5".into()),
        );
    }
}

// --- Icon Component ---

/// A view that renders a named icon from a registry.
///
/// The icon is drawn as an SVG path centered within the given rect.
/// If the icon name is not found in the registry, nothing is rendered.
///
/// # Accessibility
///
/// Always provide an `aria_label` for screen readers. Decorative icons
/// should use `aria_hidden: true`.
#[derive(Clone)]
pub struct Icon<'a> {
    registry: &'a IconRegistry,
    name: &'a str,
    color: [f32; 4],
    aria_label: Option<&'a str>,
    aria_hidden: bool,
}

impl<'a> Icon<'a> {
    /// Creates a new icon view referencing the given registry and icon name.
    pub fn new(registry: &'a IconRegistry, name: &'a str) -> Self {
        Self {
            registry,
            name,
            color: [1.0, 1.0, 1.0, 1.0],
            aria_label: None,
            aria_hidden: false,
        }
    }

    /// Sets the icon color (RGBA, 0.0-1.0 range).
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    /// Sets the accessibility label for screen readers.
    pub fn with_aria_label(mut self, label: &'a str) -> Self {
        self.aria_label = Some(label);
        self
    }

    /// Marks this icon as decorative (hidden from screen readers).
    pub fn decorative(mut self) -> Self {
        self.aria_hidden = true;
        self
    }
}

impl<'a> View for Icon<'a> {
    type Body = Never;

    fn body(self) -> Self::Body {
        panic!("Icon is a leaf component")
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let Some(data) = self.registry.get(self.name) else {
            return;
        };

        match data {
            IconData::Svg(path_data) => {
                // Build a minimal SVG string from the path data.
                // The SVG viewBox is 24x24 (standard icon grid).
                let svg = format!(
                    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="{}" height="{}"><path d="{}" fill="none" stroke="rgba({},{},{},{})" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
                    rect.width,
                    rect.height,
                    path_data,
                    self.color[0],
                    self.color[1],
                    self.color[2],
                    self.color[3],
                );
                renderer.draw_svg_with_offset(&svg, rect, 0.0);
            }
            IconData::Glyph(idx) => {
                // Render the icon font glyph by converting the glyph index
                // (which is a Unicode codepoint) to a character and drawing it
                // as text at the icon's position with the icon's color.
                if let Some(ch) = char::from_u32(*idx) {
                    let mut text_buf = [0u8; 4];
                    let text = ch.encode_utf8(&mut text_buf);
                    renderer.draw_text_raw(text, rect.x, rect.y, rect.height, self.color);
                }
            }
        }
    }

    fn aria_properties(&self) -> Option<AriaProperties> {
        if self.aria_hidden {
            return Some(AriaProperties::new(AriaRole::Presentation, ""));
        }
        self.aria_label
            .map(|label| AriaProperties::new(AriaRole::Img, label))
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_default_has_icons() {
        let reg = IconRegistry::with_defaults();
        assert!(reg.len() >= 20, "default registry should have 20+ icons");
    }

    #[test]
    fn registry_contains_defaults() {
        let reg = IconRegistry::with_defaults();
        assert!(reg.contains("close"));
        assert!(reg.contains("menu"));
        assert!(reg.contains("add"));
        assert!(reg.contains("search"));
        assert!(reg.contains("settings"));
        assert!(reg.contains("check"));
        assert!(reg.contains("warning"));
        assert!(reg.contains("error"));
        assert!(reg.contains("home"));
        assert!(reg.contains("user"));
    }

    #[test]
    fn registry_register_and_get() {
        let mut reg = IconRegistry::new();
        assert!(reg.is_empty());
        reg.register("test", IconData::Svg("M0 0 L1 1".into()));
        assert_eq!(reg.len(), 1);
        assert!(reg.contains("test"));
        assert!(matches!(reg.get("test"), Some(IconData::Svg(_))));
    }

    #[test]
    fn registry_replace() {
        let mut reg = IconRegistry::new();
        reg.register("a", IconData::Svg("M0 0".into()));
        reg.register("a", IconData::Svg("M1 1".into()));
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.get("a").unwrap().clone().svg_path(), Some("M1 1"));
    }

    #[test]
    fn icon_component_new() {
        let reg = IconRegistry::with_defaults();
        let icon = Icon::new(&reg, "close");
        assert_eq!(icon.name, "close");
        assert_eq!(icon.color, [1.0, 1.0, 1.0, 1.0]);
        assert!(!icon.aria_hidden);
    }

    #[test]
    fn icon_component_with_color() {
        let reg = IconRegistry::with_defaults();
        let icon = Icon::new(&reg, "close").with_color([0.5, 0.5, 0.5, 1.0]);
        assert_eq!(icon.color, [0.5, 0.5, 0.5, 1.0]);
    }

    #[test]
    fn icon_component_decorative() {
        let reg = IconRegistry::with_defaults();
        let icon = Icon::new(&reg, "close").decorative();
        assert!(icon.aria_hidden);
    }

    #[test]
    fn icon_component_with_aria_label() {
        let reg = IconRegistry::with_defaults();
        let icon = Icon::new(&reg, "close").with_aria_label("Close dialog");
        assert_eq!(icon.aria_label, Some("Close dialog"));
    }
}

#[cfg(test)]
mod smoke_tests {
    use super::*;

    #[test]
    fn icon_registry_default_constructs() {
        let reg = IconRegistry::default();
        assert!(!reg.is_empty());
        assert!(reg.len() >= 20);
    }

    #[test]
    fn icon_data_svg_constructs() {
        let data = IconData::Svg("M0 0 L1 1".into());
        assert_eq!(data.svg_path(), Some("M0 0 L1 1"));
    }

    #[test]
    fn icon_data_glyph_constructs() {
        let data = IconData::Glyph(65);
        assert_eq!(data.svg_path(), None);
    }
}

impl IconData {
    /// Returns the SVG path string if this is an Svg variant.
    pub fn svg_path(&self) -> Option<&str> {
        match self {
            IconData::Svg(s) => Some(s),
            IconData::Glyph(_) => None,
        }
    }
}
