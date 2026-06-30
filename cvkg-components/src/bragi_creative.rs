//! Bragi Creative Suite - Rich content editing components
//!
//! Bragi the skaldic god governs poetry and creative expression - this suite
//! provides rich text, markdown, and vector editing capabilities.

use crate::theme;
use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};

/// Rich text editor state
#[derive(Debug, Clone)]
pub struct RichTextState {
    pub content: String,
    pub cursor_pos: usize,
    pub selection: Option<(usize, usize)>,
}

/// Markdown editor with live preview
#[derive(Debug, Clone)]
pub struct MarkdownEditor {
    pub source: String,
    pub preview_mode: bool,
}

/// SVG canvas for vector editing
#[derive(Debug, Clone)]
pub struct SvgCanvas {
    pub elements: Vec<SvgElement>,
}

/// SVG element definition
#[derive(Debug, Clone)]
pub struct SvgElement {
    pub id: String,
    pub element_type: SvgType,
    pub properties: Vec<(String, String)>,
}

#[derive(Debug, Clone, Copy)]
pub enum SvgType {
    Rect,
    Circle,
    Path,
    Text,
    Group,
}

/// Bragi Creative Suite for rich content editing
#[doc(alias = "CreativeTools")]
pub struct BragiCreative {
    pub components: Vec<CreativeComponent>,
    pub active_editor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreativeComponent {
    pub name: String,
    pub component_type: CreativeType,
    pub data: String,
}

#[derive(Debug, Clone, Copy)]
pub enum CreativeType {
    RichText,
    Markdown,
    Svg,
    Texture,
}

impl Default for BragiCreative {
    fn default() -> Self {
        Self::new()
    }
}

impl BragiCreative {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            active_editor: None,
        }
    }

    pub fn rich_text(mut self, name: &str, content: &str) -> Self {
        self.components.push(CreativeComponent {
            name: name.to_string(),
            component_type: CreativeType::RichText,
            data: content.to_string(),
        });
        self
    }

    pub fn markdown(mut self, name: &str, source: &str) -> Self {
        self.components.push(CreativeComponent {
            name: name.to_string(),
            component_type: CreativeType::Markdown,
            data: source.to_string(),
        });
        self
    }

    pub fn svg(mut self, name: &str) -> Self {
        self.components.push(CreativeComponent {
            name: name.to_string(),
            component_type: CreativeType::Svg,
            data: String::new(),
        });
        self
    }

    pub fn active(mut self, name: &str) -> Self {
        self.active_editor = Some(name.to_string());
        self
    }
}

impl View for BragiCreative {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rect(rect, theme::surface_elevated());
        renderer.draw_text_raw(
            "Bragi Creative Suite",
            rect.x + 10.0,
            rect.y + 20.0,
            14.0,
            [0.8, 0.7, 1.0, 1.0],
        );

        let mut y = rect.y + 45.0;
        for comp in &self.components {
            let bg = if self.active_editor.as_deref() == Some(&comp.name) {
                [0.1, 0.12, 0.15, 1.0]
            } else {
                [0.08, 0.08, 0.1, 1.0]
            };
            renderer.fill_rect(
                Rect {
                    x: rect.x + 10.0,
                    y,
                    width: rect.width - 20.0,
                    height: 30.0,
                },
                bg,
            );

            let icon = match comp.component_type {
                CreativeType::RichText => "📝",
                CreativeType::Markdown => "MD",
                CreativeType::Svg => "⚡",
                CreativeType::Texture => "🎨",
            };
            renderer.draw_text_raw(
                &format!("{} {}", icon, comp.name),
                rect.x + 15.0,
                y + 8.0,
                11.0,
                [0.8, 0.9, 1.0, 1.0],
            );
            y += 35.0;
        }
    }
}

impl LayoutView for BragiCreative {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 280.0,
            height: 70.0 + self.components.len() as f32 * 35.0,
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
