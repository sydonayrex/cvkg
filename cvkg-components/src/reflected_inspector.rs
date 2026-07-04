//! ReflectedInspector — a runtime property inspector for reflected types.
//!
//! Displays a title bar and a list of field rows, each showing the field name
//! and its current value. Field values are rendered according to their JSON
//! type (bool → ON/OFF, number → numeric, string → quoted, array → [...]).
//!
//! The inspector takes a `&'static TypeMeta` reference (obtained from the
//! concrete type's `Reflected::type_meta()` or from `ReflectRegistry`) and a
//! `dyn Reflected` instance. This keeps the design object-safe.
//!
//! # Usage
//!
//! ```ignore
//! use cvkg_reflect::{Reflected, TypeMeta};
//! use cvkg_components::reflected_inspector::ReflectedInspector;
//!
//! let instance = Rc::new(RefCell::new(ColorStop::new(0.5, "mid")));
//!
//! // Option A: known concrete type
//! let inspector = ReflectedInspector::new(
//!     "Color Stop",
//!     ColorStop::type_meta(),
//!     instance,
//! );
//!
//! // Option B: looked up from registry
//! let meta = REGISTRY.lock().unwrap().get("ColorStop").copied().unwrap();
//! let inspector = ReflectedInspector::new("Properties", meta, instance);
//! ```

use std::sync::{Arc, Mutex};

use cvkg_core::{Never, Rect, Renderer, TextHAlign, TextVAlign, View};
use cvkg_reflect::{Reflected, TypeMeta};

use crate::theme;

/// A runtime inspector panel for a single reflected value.
pub struct ReflectedInspector {
    /// Panel title shown in the title bar.
    pub title: String,
    /// Static type schema — field names, kinds, docs.
    pub type_meta: &'static TypeMeta,
    /// The reflected instance being inspected.
    pub instance: Arc<Mutex<dyn Reflected + Send>>,
}

impl ReflectedInspector {
    /// Create a new inspector for a reflected type instance.
    ///
    /// - `title` — panel header text
    /// - `type_meta` — the static schema from `SomeType::type_meta()`
    /// - `instance` — a shared reference to the live value
    pub fn new(
        title: &str,
        type_meta: &'static TypeMeta,
        instance: Arc<Mutex<dyn Reflected + Send>>,
    ) -> Self {
        Self {
            title: title.to_string(),
            type_meta,
            instance,
        }
    }
}

impl View for ReflectedInspector {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let instance = self.instance.lock().unwrap();

        // ── Panel background ─────────────────────────────────────────
        renderer.fill_rect(rect, theme::surface_elevated());
        renderer.stroke_rect(rect, theme::border(), 1.0);

        let margin = 8.0;
        let mut y = rect.y + margin;

        // ── Title bar ─────────────────────────────────────────────────
        let title_h = 28.0;
        let title_rect = Rect {
            x: rect.x + margin,
            y,
            width: rect.width - margin * 2.0,
            height: title_h,
        };
        renderer.draw_text(
            &self.title,
            &title_rect,
            14.0,
            theme::accent(),
            TextHAlign::Left,
            TextVAlign::Middle,
        );
        y += title_h;

        // ── Separator ─────────────────────────────────────────────────
        let sep_rect = Rect {
            x: rect.x + margin,
            y,
            width: rect.width - margin * 2.0,
            height: 1.0,
        };
        renderer.fill_rect(sep_rect, theme::border());
        y += 6.0;

        // ── Field rows ────────────────────────────────────────────────
        let row_h = 22.0;
        let label_w = (rect.width - margin * 3.0) * 0.35;
        let value_x = rect.x + margin + label_w + margin;
        let value_w = rect.width - label_w - margin * 3.0;

        for (i, field) in self.type_meta.fields.iter().enumerate() {
            let row_rect = Rect {
                x: rect.x + margin,
                y,
                width: rect.width - margin * 2.0,
                height: row_h,
            };

            // Alternating row background
            if i % 2 == 0 {
                renderer.fill_rect(row_rect, theme::with_alpha(theme::surface(), 0.3));
            }

            // Field name label
            let label_rect = Rect {
                x: row_rect.x,
                y: row_rect.y,
                width: label_w,
                height: row_h,
            };
            renderer.draw_text(
                field.name,
                &label_rect,
                12.0,
                if field.read_only {
                    theme::text_muted()
                } else {
                    theme::text()
                },
                TextHAlign::Left,
                TextVAlign::Middle,
            );

            // Field value via `get_field` (object-safe)
            let value_text = match instance.get_field(field.name) {
                Some(v) => format_value(&v),
                None => "<error>".to_string(),
            };

            let value_rect = Rect {
                x: value_x,
                y: row_rect.y,
                width: value_w,
                height: row_h,
            };
            // Muted color for read-only fields
            let value_color = if field.read_only {
                theme::text_muted()
            } else {
                theme::text()
            };
            renderer.draw_text(
                &value_text,
                &value_rect,
                12.0,
                value_color,
                TextHAlign::Left,
                TextVAlign::Middle,
            );

            y += row_h;
        }

        // ── Overflow indicator ───────────────────────────────────────
        let consumed = y - rect.y;
        if consumed > rect.height {
            let overflow_rect = Rect {
                x: rect.x + rect.width - 6.0,
                y: rect.y + 2.0,
                width: 4.0,
                height: rect.height - 4.0,
            };
            renderer.fill_rounded_rect(overflow_rect, 2.0, theme::warning());
        }
    }
}

/// Format a `serde_json::Value` into a compact display string.
fn format_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => {
            if *b {
                "ON".to_string()
            } else {
                "OFF".to_string()
            }
        }
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if f.fract() == 0.0 && f.is_finite() && f >= i64::MIN as f64 && f <= i64::MAX as f64
                {
                    format!("{}", n.as_i64().unwrap_or(f as i64))
                } else {
                    format!("{:.4}", f)
                }
            } else {
                n.to_string()
            }
        }
        serde_json::Value::String(s) => format!("\"{s}\""),
        serde_json::Value::Array(arr) => {
            let parts: Vec<String> = arr.iter().map(format_value).collect();
            format!("[{}]", parts.join(", "))
        }
        serde_json::Value::Object(obj) => {
            // Detect common CVKG shapes: Vec2, Vec3, Color, Rect
            if obj.len() == 2 && obj.contains_key("x") && obj.contains_key("y") {
                format!(
                    "({}, {})",
                    fmt_val(obj.get("x")),
                    fmt_val(obj.get("y")),
                )
            } else if obj.len() == 3 && obj.contains_key("x") && obj.contains_key("y")
                && obj.contains_key("z")
            {
                format!(
                    "({}, {}, {})",
                    fmt_val(obj.get("x")),
                    fmt_val(obj.get("y")),
                    fmt_val(obj.get("z")),
                )
            } else if obj.contains_key("width") && obj.contains_key("height") {
                format!(
                    "({}, {}) {} x {}",
                    fmt_val(obj.get("x")),
                    fmt_val(obj.get("y")),
                    fmt_val(obj.get("width")),
                    fmt_val(obj.get("height")),
                )
            } else {
                let parts: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("{k}: {}", format_value(v)))
                    .collect();
                format!("{{{}}}", parts.join(", "))
            }
        }
    }
}

/// Extract a number from an optional `Value` reference.
fn fmt_val(v: Option<&serde_json::Value>) -> String {
    match v {
        Some(serde_json::Value::Number(n)) => {
            if let Some(f) = n.as_f64() {
                format!("{:.2}", f)
            } else {
                n.to_string()
            }
        }
        Some(other) => format_value(other),
        None => "?".to_string(),
    }
}
