use crate::theme;
use crate::{RADIUS_LG, RADIUS_SM};
use cvkg_core::{Never, Rect, Renderer, View};
use cvkg_runic_text::{FontAxisInfo, TextStyle, VariableAxis};
use std::sync::Arc;

/// Colors used by the font axis panel (resolved from theme tokens).
fn color_panel_bg() -> [f32; 4] {
    theme::surface()
}
fn color_panel_border() -> [f32; 4] {
    theme::border()
}
fn color_text_title() -> [f32; 4] {
    theme::text()
}
fn color_text_label() -> [f32; 4] {
    theme::text_muted()
}
fn color_slider_bg() -> [f32; 4] {
    theme::surface_elevated()
}
fn color_slider_fill() -> [f32; 4] {
    theme::with_alpha(theme::accent(), 0.85)
}
fn color_slider_border() -> [f32; 4] {
    theme::border()
}
fn color_toggle_on() -> [f32; 4] {
    theme::with_alpha(theme::accent(), 0.9)
}
fn color_toggle_off() -> [f32; 4] {
    theme::surface_elevated()
}
fn color_toggle_knob() -> [f32; 4] {
    theme::text()
}

/// A slider panel for controlling variable font axes.
///
/// Displays a labeled slider for each axis in the font's fvar table:
/// - Weight (wght): 100–900
/// - Width (wdth): 75–150
/// - Italic (ital): 0.0–1.0
/// - Slant (slnt): -15–0
/// - Plus any custom axes defined by the font.
///
/// # COLRv1 Support
///
/// When `colr_enabled` is true, the component provides a rendered sample using
/// the COLR/CPAL color table. The application should use
/// `style.render_mode = RenderMode::Color` when COLR is enabled.
///
/// # Performance
///
/// Axis values are `Copy` -- no allocations on change. The panel is designed
/// to be rendered once per frame and kept in application state.
#[derive(Clone)]
pub struct FontAxisPanel {
    /// Font family name to query for axes.
    pub family: String,
    /// The default font size for rendering previews.
    pub font_size: f32,
    /// Current axis values (axis info + current value).
    pub axis_values: Vec<(FontAxisInfo, f32)>,
    /// Whether COLRv1 color rendering is enabled.
    pub colr_enabled: bool,
    /// Callback fired when any axis value changes.
    pub on_axis_change: Option<Arc<dyn Fn(u32, f32) + Send + Sync>>,
    /// Callback fired when COLR toggle changes.
    pub on_colr_toggle: Option<Arc<dyn Fn(bool) + Send + Sync>>,
}

impl FontAxisPanel {
    /// Create a new font axis panel from the given axes.
    ///
    /// Initial values are set to each axis's default.
    pub fn new(family: impl Into<String>, font_size: f32, axes: Vec<FontAxisInfo>) -> Self {
        let axis_values: Vec<(FontAxisInfo, f32)> = axes
            .into_iter()
            .map(|axis| {
                let val = axis.default.clamp(axis.min, axis.max);
                (axis, val)
            })
            .collect();

        Self {
            family: family.into(),
            font_size,
            axis_values,
            colr_enabled: false,
            on_axis_change: None,
            on_colr_toggle: None,
        }
    }

    /// Set the callback for axis value changes.
    pub fn on_axis_change(mut self, cb: impl Fn(u32, f32) + Send + Sync + 'static) -> Self {
        self.on_axis_change = Some(Arc::new(cb));
        self
    }

    /// Set the callback for COLR toggle changes.
    pub fn on_colr_toggle(mut self, cb: impl Fn(bool) + Send + Sync + 'static) -> Self {
        self.on_colr_toggle = Some(Arc::new(cb));
        self
    }

    /// Apply the current axis values to a `TextStyle`.
    pub fn apply_to_style(&self, style: &mut TextStyle) {
        style.variable_axes = self
            .axis_values
            .iter()
            .map(|(info, value)| VariableAxis::new(info.tag.to_be_bytes(), *value))
            .collect();
    }

    /// Set the value for a specific axis by tag.
    pub fn set_axis(&mut self, tag: u32, value: f32) {
        for (info, val) in &mut self.axis_values {
            if info.tag == tag {
                *val = value.clamp(info.min, info.max);
            }
        }
    }

    /// Get the current value for a specific axis by tag.
    pub fn get_axis(&self, tag: u32) -> Option<f32> {
        self.axis_values
            .iter()
            .find(|(info, _)| info.tag == tag)
            .map(|(_, val)| *val)
    }

    /// Returns true if COLR rendering is enabled.
    pub fn is_colr_enabled(&self) -> bool {
        self.colr_enabled
    }

    /// Enable or disable COLRv1 color rendering.
    pub fn set_colr_enabled(&mut self, enabled: bool) {
        self.colr_enabled = enabled;
    }

    /// Get all current axis values as `VariableAxis` entries.
    pub fn to_variable_axes(&self) -> Vec<VariableAxis> {
        self.axis_values
            .iter()
            .map(|(info, value)| VariableAxis::new(info.tag.to_be_bytes(), *value))
            .collect()
    }
}

impl View for FontAxisPanel {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Panel background
        renderer.fill_rounded_rect(rect, RADIUS_LG, color_panel_bg());
        renderer.stroke_rounded_rect(rect, RADIUS_LG, color_panel_border(), 1.0);

        let padding = 12.0;
        let slider_height = 28.0;
        let label_height = 18.0;
        let row_gap = 8.0;
        let mut y = rect.y + padding;

        // Title
        let title = format!("{} — Variable Axes", self.family);
        renderer.draw_text(&title, rect.x + padding, y + 14.0, 14.0, color_text_title());
        y += label_height + row_gap + 4.0;

        // One slider per axis
        for (info, value) in &self.axis_values {
            let row_height = slider_height + label_height + 4.0;

            // Label: "Weight (wght): 400"
            let label_text = format!(
                "{} ({}): {:.0}",
                info.display_name(),
                info.tag_string,
                *value
            );
            renderer.draw_text(
                &label_text,
                rect.x + padding,
                y + 14.0,
                12.0,
                color_text_label(),
            );

            // Slider track
            let track_y = y + label_height + 2.0;
            let track_rect = Rect {
                x: rect.x + padding,
                y: track_y,
                width: rect.width - padding * 2.0,
                height: slider_height,
            };
            let range = info.max - info.min;
            let normalized = if range > 0.0 {
                (*value - info.min) / range
            } else {
                0.5
            };
            let fill_w = track_rect.width * normalized.clamp(0.0, 1.0);

            // Track background
            renderer.fill_rounded_rect(track_rect, RADIUS_SM, color_slider_bg());
            renderer.stroke_rounded_rect(track_rect, RADIUS_SM, color_slider_border(), 1.0);

            // Track fill
            if fill_w > 0.0 {
                let fill_rect = Rect {
                    x: track_rect.x,
                    y: track_rect.y,
                    width: fill_w.max(8.0),
                    height: track_rect.height,
                };
                renderer.fill_rounded_rect(fill_rect, RADIUS_SM, color_slider_fill());
            }

            y += row_height + row_gap;
        }

        // COLRv1 toggle row
        y += 4.0;
        let toggle_row_h = 28.0;
        renderer.draw_text(
            "COLRv1 Color Rendering",
            rect.x + padding,
            y + 14.0,
            12.0,
            color_text_label(),
        );

        // Toggle switch
        let toggle_x = rect.x + rect.width - padding - 48.0;
        let toggle_w = 44.0;
        let toggle_h = 22.0;
        let toggle_rect = Rect {
            x: toggle_x,
            y: y + (toggle_row_h - toggle_h) * 0.5,
            width: toggle_w,
            height: toggle_h,
        };

        let toggle_bg = if self.colr_enabled {
            color_toggle_on()
        } else {
            color_toggle_off()
        };
        renderer.fill_rounded_rect(toggle_rect, toggle_h * 0.5, toggle_bg);
        renderer.stroke_rounded_rect(toggle_rect, toggle_h * 0.5, color_slider_border(), 1.0);

        // Toggle knob
        let knob_r = toggle_h * 0.38;
        let knob_x = if self.colr_enabled {
            toggle_rect.x + toggle_rect.width - knob_r * 2.0 - 3.0
        } else {
            toggle_rect.x + 3.0
        };
        let knob_y = toggle_rect.y + toggle_rect.height * 0.5;
        let knob_rect = Rect {
            x: knob_x - knob_r,
            y: knob_y - knob_r,
            width: knob_r * 2.0,
            height: knob_r * 2.0,
        };
        renderer.fill_ellipse(knob_rect, color_toggle_knob());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_axis_panel_creation() {
        let axes = vec![FontAxisInfo {
            tag: u32::from_be_bytes(*b"wght"),
            tag_string: "wght".to_string(),
            min: 100.0,
            max: 900.0,
            default: 400.0,
            is_standard: true,
        }];
        let panel = FontAxisPanel::new("TestFont", 16.0, axes);
        assert_eq!(panel.family, "TestFont");
        assert_eq!(panel.axis_values.len(), 1);
        assert_eq!(panel.get_axis(u32::from_be_bytes(*b"wght")), Some(400.0));
    }

    #[test]
    fn test_font_axis_panel_set_axis() {
        let axes = vec![FontAxisInfo {
            tag: u32::from_be_bytes(*b"wght"),
            tag_string: "wght".to_string(),
            min: 100.0,
            max: 900.0,
            default: 400.0,
            is_standard: true,
        }];
        let mut panel = FontAxisPanel::new("TestFont", 16.0, axes);
        panel.set_axis(u32::from_be_bytes(*b"wght"), 700.0);
        assert_eq!(panel.get_axis(u32::from_be_bytes(*b"wght")), Some(700.0));

        // Clamping: out-of-range values get clamped
        panel.set_axis(u32::from_be_bytes(*b"wght"), 9999.0);
        assert_eq!(panel.get_axis(u32::from_be_bytes(*b"wght")), Some(900.0));
    }

    #[test]
    fn test_font_axis_panel_to_variable_axes() {
        let axes = vec![
            FontAxisInfo {
                tag: u32::from_be_bytes(*b"wght"),
                tag_string: "wght".to_string(),
                min: 100.0,
                max: 900.0,
                default: 400.0,
                is_standard: true,
            },
            FontAxisInfo {
                tag: u32::from_be_bytes(*b"wdth"),
                tag_string: "wdth".to_string(),
                min: 75.0,
                max: 150.0,
                default: 100.0,
                is_standard: true,
            },
        ];
        let mut panel = FontAxisPanel::new("TestFont", 16.0, axes);
        panel.set_axis(u32::from_be_bytes(*b"wght"), 600.0);

        let va = panel.to_variable_axes();
        assert_eq!(va.len(), 2);
        assert_eq!(va[0].value, 600.0);
        assert_eq!(va[1].value, 100.0);
    }

    #[test]
    fn test_apply_to_style() {
        let axes = vec![FontAxisInfo {
            tag: u32::from_be_bytes(*b"wght"),
            tag_string: "wght".to_string(),
            min: 100.0,
            max: 900.0,
            default: 400.0,
            is_standard: true,
        }];
        let mut panel = FontAxisPanel::new("TestFont", 16.0, axes);
        panel.set_axis(u32::from_be_bytes(*b"wght"), 700.0);

        let mut style = TextStyle::new("TestFont", 16.0);
        panel.apply_to_style(&mut style);
        assert_eq!(style.variable_axes.len(), 1);
        assert_eq!(style.variable_axes[0].value, 700.0);
    }
}
