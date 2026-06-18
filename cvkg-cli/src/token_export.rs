//! Design Token Export -- Item 13
//!
//! Exports CVKG design tokens to various formats: Figma JSON, CSS variables,
//! Swift constants, and plain JSON.
//!
//! # OS-agnostic
//! Pure file I/O. No platform-specific APIs.

use std::collections::HashMap;

/// Token export engine.
#[derive(Default)]
pub struct TokenExport {
    /// Color tokens: name → [r, g, b, a]
    colors: HashMap<String, [f32; 4]>,
    /// Spacing tokens: name → value in px
    spacing: HashMap<String, f32>,
    /// Border radius tokens: name → value in px
    radius: HashMap<String, f32>,
    /// Typography tokens: name → font size in px
    typography: HashMap<String, f32>,
}

impl TokenExport {
    /// Create a new token export with CVKG's default design tokens.
    pub fn new() -> Self {
        let mut colors = HashMap::new();
        // Light theme colors
        colors.insert("background".to_string(), [1.0, 1.0, 1.0, 1.0]);
        colors.insert("surface".to_string(), [0.98, 0.98, 0.98, 1.0]);
        colors.insert("surface_elevated".to_string(), [0.95, 0.95, 0.95, 1.0]);
        colors.insert("surface_overlay".to_string(), [0.92, 0.92, 0.94, 1.0]);
        colors.insert("text".to_string(), [0.1, 0.1, 0.12, 1.0]);
        colors.insert("text_muted".to_string(), [0.4, 0.4, 0.45, 1.0]);
        colors.insert("text_dim".to_string(), [0.55, 0.55, 0.6, 1.0]);
        colors.insert("primary".to_string(), [0.2, 0.2, 0.25, 1.0]);
        colors.insert("secondary".to_string(), [0.5, 0.5, 0.55, 1.0]);
        colors.insert("accent".to_string(), [0.0, 0.8, 1.0, 1.0]);
        colors.insert("accent_hover".to_string(), [0.2, 0.85, 1.0, 1.0]);
        colors.insert("border".to_string(), [0.85, 0.85, 0.88, 1.0]);
        colors.insert("border_strong".to_string(), [0.7, 0.7, 0.75, 1.0]);
        colors.insert("hover".to_string(), [0.95, 0.95, 0.97, 1.0]);
        colors.insert("active".to_string(), [0.9, 0.9, 0.93, 1.0]);
        colors.insert("disabled".to_string(), [0.92, 0.92, 0.94, 1.0]);
        colors.insert("disabled_text".to_string(), [0.65, 0.65, 0.7, 1.0]);
        colors.insert("success".to_string(), [0.2, 0.8, 0.4, 1.0]);
        colors.insert("warning".to_string(), [1.0, 0.7, 0.0, 1.0]);
        colors.insert("error".to_string(), [0.95, 0.2, 0.3, 1.0]);
        colors.insert("info".to_string(), [0.2, 0.6, 1.0, 1.0]);
        colors.insert("focus_ring".to_string(), [0.0, 0.8, 1.0, 0.8]);
        colors.insert("shadow".to_string(), [0.0, 0.0, 0.0, 0.15]);
        colors.insert("code_bg".to_string(), [0.96, 0.96, 0.98, 1.0]);

        // Dark theme overrides
        let mut dark_colors = HashMap::new();
        dark_colors.insert("background".to_string(), [0.05, 0.05, 0.08, 1.0]);
        dark_colors.insert("surface".to_string(), [0.1, 0.1, 0.14, 1.0]);
        dark_colors.insert("surface_elevated".to_string(), [0.15, 0.15, 0.2, 1.0]);
        dark_colors.insert("surface_overlay".to_string(), [0.18, 0.18, 0.24, 1.0]);
        dark_colors.insert("text".to_string(), [0.95, 0.95, 0.97, 1.0]);
        dark_colors.insert("text_muted".to_string(), [0.6, 0.6, 0.65, 1.0]);
        dark_colors.insert("text_dim".to_string(), [0.45, 0.45, 0.5, 1.0]);
        dark_colors.insert("border".to_string(), [0.25, 0.25, 0.3, 1.0]);
        dark_colors.insert("border_strong".to_string(), [0.35, 0.35, 0.4, 1.0]);
        dark_colors.insert("hover".to_string(), [0.15, 0.15, 0.2, 1.0]);
        dark_colors.insert("active".to_string(), [0.2, 0.2, 0.28, 1.0]);
        dark_colors.insert("disabled".to_string(), [0.12, 0.12, 0.16, 1.0]);
        dark_colors.insert("disabled_text".to_string(), [0.35, 0.35, 0.4, 1.0]);
        dark_colors.insert("shadow".to_string(), [0.0, 0.0, 0.0, 0.4]);

        let mut spacing = HashMap::new();
        spacing.insert("xs".to_string(), 4.0);
        spacing.insert("sm".to_string(), 8.0);
        spacing.insert("md".to_string(), 16.0);
        spacing.insert("lg".to_string(), 24.0);
        spacing.insert("xl".to_string(), 32.0);
        spacing.insert("2xl".to_string(), 48.0);
        spacing.insert("3xl".to_string(), 64.0);

        let mut radius = HashMap::new();
        radius.insert("xs".to_string(), 2.0);
        radius.insert("sm".to_string(), 4.0);
        radius.insert("md".to_string(), 6.0);
        radius.insert("lg".to_string(), 8.0);
        radius.insert("xl".to_string(), 12.0);
        radius.insert("2xl".to_string(), 16.0);
        radius.insert("full".to_string(), 9999.0);

        let mut typography = HashMap::new();
        typography.insert("footnote".to_string(), 10.0);
        typography.insert("caption".to_string(), 12.0);
        typography.insert("body".to_string(), 14.0);
        typography.insert("body_large".to_string(), 16.0);
        typography.insert("heading3".to_string(), 20.0);
        typography.insert("heading2".to_string(), 24.0);
        typography.insert("heading1".to_string(), 32.0);
        typography.insert("display".to_string(), 48.0);

        Self {
            colors,
            spacing,
            radius,
            typography,
        }
    }

    /// Generate token output in the specified format.
    pub fn generate(&self, format: &str) -> Result<String, String> {
        match format {
            "figma" => Ok(self.generate_figma()),
            "css" => Ok(self.generate_css()),
            "swift" => Ok(self.generate_swift()),
            "json" => Ok(self.generate_json()),
            other => Err(format!("Unknown format: {}", other)),
        }
    }

    /// Generate Figma Tokens JSON format.
    fn generate_figma(&self) -> String {
        let mut parts = vec!["{".to_string()];

        // Colors
        parts.push("  \"colors\": {".to_string());
        let color_entries: Vec<String> = self
            .colors
            .iter()
            .map(|(name, rgba)| {
                let r = (rgba[0] * 255.0).round() as u32;
                let g = (rgba[1] * 255.0).round() as u32;
                let b = (rgba[2] * 255.0).round() as u32;
                format!(
                    "    \"{}\": {{\"r\": {}, \"g\": {}, \"b\": {}, \"a\": {:.2}}}",
                    name, r, g, b, rgba[3]
                )
            })
            .collect();
        parts.push(color_entries.join(",\n"));
        parts.push("  },".to_string());

        // Spacing
        parts.push("  \"spacing\": {".to_string());
        let spacing_entries: Vec<String> = self
            .spacing
            .iter()
            .map(|(name, val)| format!("    \"{}\": {}", name, val))
            .collect();
        parts.push(spacing_entries.join(",\n"));
        parts.push("  },".to_string());

        // Radius
        parts.push("  \"radius\": {".to_string());
        let radius_entries: Vec<String> = self
            .radius
            .iter()
            .map(|(name, val)| format!("    \"{}\": {}", name, val))
            .collect();
        parts.push(radius_entries.join(",\n"));
        parts.push("  },".to_string());

        // Typography
        parts.push("  \"typography\": {".to_string());
        let type_entries: Vec<String> = self
            .typography
            .iter()
            .map(|(name, val)| format!("    \"{}\": {}", name, val))
            .collect();
        parts.push(type_entries.join(",\n"));
        parts.push("  }".to_string());

        parts.push("}".to_string());
        parts.join("\n")
    }

    /// Generate CSS custom properties.
    fn generate_css(&self) -> String {
        let mut lines = vec![":root {".to_string()];

        // Colors
        for (name, rgba) in &self.colors {
            let css_name = format!("--color-{}", name.replace('_', "-"));
            let r = (rgba[0] * 255.0).round() as u32;
            let g = (rgba[1] * 255.0).round() as u32;
            let b = (rgba[2] * 255.0).round() as u32;
            if rgba[3] < 1.0 {
                lines.push(format!(
                    "  {}: rgba({}, {}, {}, {:.2});",
                    css_name, r, g, b, rgba[3]
                ));
            } else {
                lines.push(format!("  {}: rgb({}, {}, {});", css_name, r, g, b));
            }
        }

        // Spacing
        for (name, val) in &self.spacing {
            lines.push(format!(
                "  --spacing-{}: {}px;",
                name.replace('_', "-"),
                val
            ));
        }

        // Radius
        for (name, val) in &self.radius {
            lines.push(format!("  --radius-{}: {}px;", name.replace('_', "-"), val));
        }

        // Typography
        for (name, val) in &self.typography {
            lines.push(format!(
                "  --font-size-{}: {}px;",
                name.replace('_', "-"),
                val
            ));
        }

        lines.push("}".to_string());
        lines.join("\n")
    }

    /// Generate SwiftUI-compatible constants.
    fn generate_swift(&self) -> String {
        let mut lines = vec![
            "// CVKG Design Tokens — SwiftUI".to_string(),
            "// Auto-generated by cvkg tokens export --format swift".to_string(),
            "".to_string(),
            "import SwiftUI".to_string(),
            "".to_string(),
            "struct CVKGTheme {".to_string(),
        ];

        // Colors
        lines.append(&mut vec![
            "    // MARK: - Colors".to_string(),
            "    struct Colors {".to_string(),
        ]);
        for (name, rgba) in &self.colors {
            let swift_name = Self::to_camel_case(name);
            lines.push(format!(
                "        static let {} = Color(red: {:.3}, green: {:.3}, blue: {:.3}, opacity: {:.2})",
                swift_name, rgba[0], rgba[1], rgba[2], rgba[3]
            ));
        }
        lines.push("    }".to_string());
        lines.push("".to_string());

        // Spacing
        lines.append(&mut vec![
            "    // MARK: - Spacing".to_string(),
            "    struct Spacing {".to_string(),
        ]);
        for (name, val) in &self.spacing {
            let swift_name = Self::to_camel_case(name);
            lines.push(format!(
                "        static let {}: CGFloat = {}",
                swift_name, val
            ));
        }
        lines.push("    }".to_string());
        lines.push("".to_string());

        // Radius
        lines.append(&mut vec![
            "    // MARK: - Corner Radius".to_string(),
            "    struct Radius {".to_string(),
        ]);
        for (name, val) in &self.radius {
            let swift_name = Self::to_camel_case(name);
            lines.push(format!(
                "        static let {}: CGFloat = {}",
                swift_name, val
            ));
        }
        lines.push("    }".to_string());
        lines.push("".to_string());

        // Typography
        lines.append(&mut vec![
            "    // MARK: - Typography".to_string(),
            "    struct Typography {".to_string(),
        ]);
        for (name, val) in &self.typography {
            let swift_name = Self::to_camel_case(name);
            lines.push(format!(
                "        static let {}: CGFloat = {}",
                swift_name, val
            ));
        }
        lines.push("    }".to_string());

        lines.push("}".to_string());
        lines.join("\n")
    }

    /// Generate plain JSON format.
    fn generate_json(&self) -> String {
        let mut parts = vec!["{".to_string()];

        parts.push("  \"colors\": {".to_string());
        let color_entries: Vec<String> = self
            .colors
            .iter()
            .map(|(name, rgba)| {
                format!(
                    "    \"{}\": [{:.3}, {:.3}, {:.3}, {:.2}]",
                    name, rgba[0], rgba[1], rgba[2], rgba[3]
                )
            })
            .collect();
        parts.push(color_entries.join(",\n"));
        parts.push("  },".to_string());

        parts.push("  \"spacing\": {".to_string());
        let spacing_entries: Vec<String> = self
            .spacing
            .iter()
            .map(|(name, val)| format!("    \"{}\": {}", name, val))
            .collect();
        parts.push(spacing_entries.join(",\n"));
        parts.push("  },".to_string());

        parts.push("  \"radius\": {".to_string());
        let radius_entries: Vec<String> = self
            .radius
            .iter()
            .map(|(name, val)| format!("    \"{}\": {}", name, val))
            .collect();
        parts.push(radius_entries.join(",\n"));
        parts.push("  },".to_string());

        parts.push("  \"typography\": {".to_string());
        let type_entries: Vec<String> = self
            .typography
            .iter()
            .map(|(name, val)| format!("    \"{}\": {}", name, val))
            .collect();
        parts.push(type_entries.join(",\n"));
        parts.push("  }".to_string());

        parts.push("}".to_string());
        parts.join("\n")
    }

    /// Convert snake_case to camelCase for Swift.
    fn to_camel_case(s: &str) -> String {
        let mut result = String::new();
        let mut capitalize = false;
        for c in s.chars() {
            if c == '_' {
                capitalize = true;
            } else if capitalize {
                result.push(c.to_ascii_uppercase());
                capitalize = false;
            } else {
                result.push(c);
            }
        }
        // Capitalize first letter
        if let Some(first) = result.chars().next() {
            result = first.to_ascii_uppercase().to_string() + &result[1..];
        }
        result
    }
}
