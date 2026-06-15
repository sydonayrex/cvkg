//! Freyr Inspector - Property inspection and editing component
//!
//! The Vanir god Freyr governs prosperity and possessions - this inspector
//! manages properties and attributes of UI components.

use crate::theme;
use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};

/// Property value types
#[derive(Debug, Clone)]
pub enum PropertyValue {
    Text(String),
    Number(f64),
    Boolean(bool),
    Color([f32; 4]),
    Enum(String, Vec<String>),
}

/// Property definition for inspection
pub struct Property {
    pub name: String,
    pub value: PropertyValue,
    pub description: String,
}

/// Freyr Inspector for property editing
pub struct FreyrInspector {
    pub properties: Vec<Property>,
    pub title: String,
}

impl FreyrInspector {
    pub fn new(title: &str) -> Self {
        Self {
            properties: Vec::new(),
            title: title.to_string(),
        }
    }

    pub fn property(mut self, name: &str, value: PropertyValue, desc: &str) -> Self {
        self.properties.push(Property {
            name: name.to_string(),
            value,
            description: desc.to_string(),
        });
        self
    }

    pub fn text_prop(self, name: &str, value: &str, desc: &str) -> Self {
        self.property(name, PropertyValue::Text(value.to_string()), desc)
    }

    pub fn number_prop(self, name: &str, value: f64, desc: &str) -> Self {
        self.property(name, PropertyValue::Number(value), desc)
    }

    pub fn bool_prop(self, name: &str, value: bool, desc: &str) -> Self {
        self.property(name, PropertyValue::Boolean(value), desc)
    }
}

impl View for FreyrInspector {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rect(
            Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: 28.0,
            },
            theme::inspector_bg(),
        );
        renderer.draw_text(
            &self.title,
            rect.x + 10.0,
            rect.y + 9.0,
            13.0,
            theme::inspector_accent(),
        );

        let row_h = 28.0;
        let mut current_y = rect.y + 32.0;

        for prop in &self.properties {
            let prop_rect = Rect {
                x: rect.x,
                y: current_y,
                width: rect.width,
                height: row_h,
            };
            renderer.fill_rect(prop_rect, theme::inspector_border());
            renderer.draw_text(
                &prop.name,
                prop_rect.x + 8.0,
                prop_rect.y + 8.0,
                11.0,
                theme::text(),
            );

            let value_str = match &prop.value {
                PropertyValue::Text(s) => s.clone(),
                PropertyValue::Number(n) => format!("{:.2}", n),
                PropertyValue::Boolean(b) => {
                    if *b {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                }
                PropertyValue::Color([r, g, b, a]) => format!(
                    "#{:02X}{:02X}{:02X}{:02X}",
                    (*r * 255.0) as u8,
                    (*g * 255.0) as u8,
                    (*b * 255.0) as u8,
                    (*a * 255.0) as u8
                ),
                PropertyValue::Enum(v, _) => v.clone(),
            };
            renderer.draw_text(
                &value_str,
                prop_rect.x + 100.0,
                prop_rect.y + 8.0,
                11.0,
                theme::info(),
            );
            current_y += row_h;
        }
    }
}

impl LayoutView for FreyrInspector {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 280.0,
            height: 36.0 + self.properties.len() as f32 * 28.0,
        }
    }
    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}
