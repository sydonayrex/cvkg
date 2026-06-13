use crate::{Color, FONT_SM, SPACE_XS};
use cvkg_core::{Never, Rect, Renderer, View};

/// Breadcrumb item representing a single navigation segment.
#[derive(Clone)]
pub struct BreadcrumbItem {
    pub label: String,
    pub href: Option<String>,
    pub is_current: bool,
}

impl BreadcrumbItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: None,
            is_current: false,
        }
    }

    pub fn href(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }

    pub fn current(mut self, is_current: bool) -> Self {
        self.is_current = is_current;
        self
    }
}

/// Breadcrumb navigation component.
/// Displays a path of navigation links separated by a delimiter.
#[derive(Clone)]
pub struct Breadcrumb {
    pub(crate) items: Vec<BreadcrumbItem>,
    pub(crate) separator: String,
    pub(crate) color: Color,
    pub(crate) current_color: Color,
    pub(crate) font_size: f32,
}

impl Breadcrumb {
    pub fn new(items: Vec<BreadcrumbItem>) -> Self {
        Self {
            items,
            separator: "/".to_string(),
            color: Color::GRAY,
            current_color: Color::WHITE,
            font_size: FONT_SM,
        }
    }

    pub fn separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn current_color(mut self, color: Color) -> Self {
        self.current_color = color;
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
}

impl View for Breadcrumb {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let mut x = rect.x;
        let y = rect.y;
        let total = self.items.len();

        for (i, item) in self.items.iter().enumerate() {
            let is_last = i == total - 1;
            let color = if item.is_current || is_last {
                self.current_color
            } else {
                self.color
            };

            let label_color = color.as_array();
            renderer.draw_text(&item.label, x, y, self.font_size, label_color);

            // Measure text width for positioning
            let (w, _) = renderer.measure_text(&item.label, self.font_size);
            x += w;

            // Draw separator between items
            if !is_last {
                let sep_color = self.color.as_array();
                renderer.draw_text(&self.separator, x, y, self.font_size, sep_color);
                let (sep_w, _) = renderer.measure_text(&self.separator, self.font_size);
                x += sep_w + SPACE_XS;
            }
        }
    }
}
