use crate::theme;
use crate::theme::glassmorphism_enabled;
use cvkg_core::{Never, Rect, Renderer, View};

/// HatiCarousel - A horizontal scrolling carousel component.
#[doc(alias = "Carousel")]
pub struct HatiCarousel {
    pub items: Vec<Box<dyn Fn(&mut dyn Renderer, Rect) + Send + Sync>>,
    pub item_width: f32,
    pub item_height: f32,
    pub spacing: f32,
    pub scroll_offset: f32,
}

impl HatiCarousel {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            item_width: 200.0,
            item_height: 150.0,
            spacing: 16.0,
            scroll_offset: 0.0,
        }
    }

    pub fn item_width(mut self, w: f32) -> Self {
        self.item_width = w;
        self
    }

    pub fn item_height(mut self, h: f32) -> Self {
        self.item_height = h;
        self
    }
}

impl View for HatiCarousel {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!("Primitive view has no body")
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if glassmorphism_enabled() {
            renderer.bifrost(rect, 15.0, 1.2, 0.85);
        }
        renderer.fill_rounded_rect(rect, 8.0, theme::surface());

        let mut x = rect.x - self.scroll_offset;
        for item in &self.items {
            let item_rect = Rect {
                x,
                y: rect.y,
                width: self.item_width,
                height: self.item_height,
            };
            item(renderer, item_rect);
            x += self.item_width + self.spacing;
        }
    }
}
