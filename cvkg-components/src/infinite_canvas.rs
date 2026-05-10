use cvkg_core::{layout::{LayoutCache, LayoutView, SizeProposal}, Rect, Renderer, Size, View, Never, AnyView};

/// An infinite canvas with pan/zoom viewport controls.
pub struct InfiniteCanvas {
    pub(crate) children: Vec<CanvasItem>,
    pub(crate) zoom: f32,
    pub(crate) pan_x: f32,
    pub(crate) pan_y: f32,
}

pub struct CanvasItem {
    pub id: String,
    pub content: AnyView,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl InfiniteCanvas {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
        }
    }

    pub fn zoom(mut self, factor: f32) -> Self {
        self.zoom = factor.clamp(0.1, 5.0);
        self
    }

    pub fn pan(mut self, x: f32, y: f32) -> Self {
        self.pan_x = x;
        self.pan_y = y;
        self
    }

    pub fn item(mut self, id: &str, content: impl View + Clone + 'static, x: f32, y: f32, width: f32, height: f32) -> Self {
        self.children.push(CanvasItem {
            id: id.to_string(),
            content: content.erase(),
            x, y, width, height,
        });
        self
    }
}

impl View for InfiniteCanvas {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Draw grid background
        let grid_size = 40.0 * self.zoom;
        let start_x = (rect.x - self.pan_x) / self.zoom;
        let start_y = (rect.y - self.pan_y) / self.zoom;

        let grid_color = [0.05, 0.05, 0.08, 1.0];
        let mut x = start_x;
        while x < rect.width {
            let line_x = (x * self.zoom + self.pan_x).max(rect.x);
            renderer.draw_line(line_x, rect.y, line_x, rect.y + rect.height, grid_color, 0.5);
            x += grid_size.max(1.0);
        }
        
        let mut y = start_y;
        while y < rect.height {
            let line_y = (y * self.zoom + self.pan_y).max(rect.y);
            renderer.draw_line(rect.x, line_y, rect.x + rect.width, line_y, grid_color, 0.5);
            y += grid_size.max(1.0);
        }

        // Draw origin crosshair
        let origin_x = self.pan_x;
        let origin_y = self.pan_y;
        if origin_x >= rect.x && origin_x <= rect.x + rect.width {
            renderer.draw_line(origin_x, rect.y, origin_x, rect.y + rect.height, [0.2, 0.4, 0.6, 0.5], 1.0);
        }
        if origin_y >= rect.y && origin_y <= rect.y + rect.height {
            renderer.draw_line(rect.x, origin_y, rect.x + rect.width, origin_y, [0.2, 0.4, 0.6, 0.5], 1.0);
        }

        // Render children
        for item in &self.children {
            let item_x = (item.x * self.zoom + self.pan_x).max(rect.x);
            let item_y = (item.y * self.zoom + self.pan_y).max(rect.y);
            let item_w = (item.width * self.zoom).max(1.0);
            let item_h = (item.height * self.zoom).max(1.0);

            if item_x + item_w > rect.x && item_x < rect.x + rect.width &&
               item_y + item_h > rect.y && item_y < rect.y + rect.height {
                let item_rect = Rect { x: item_x, y: item_y, width: item_w, height: item_h };
                item.content.render(renderer, item_rect);
            }
        }
    }
}

impl LayoutView for InfiniteCanvas {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn LayoutView], _cache: &mut LayoutCache) -> Size {
        Size { width: 800.0, height: 600.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn LayoutView], _cache: &mut LayoutCache) {}
}

/// Minimap provides a small overview of the canvas.
pub struct Minimap {
    pub(crate) canvas: AnyView,
    pub(crate) minimap_size: f32,
}

impl Minimap {
    pub fn new(canvas: impl View + Clone + 'static, minimap_size: f32) -> Self {
        Self {
            canvas: canvas.erase(),
            minimap_size,
        }
    }
}

impl View for Minimap {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Draw minimap background
        renderer.fill_rounded_rect(rect, 4.0, [0.1, 0.1, 0.15, 1.0]);
        renderer.stroke_rounded_rect(rect, 4.0, [0.3, 0.5, 0.8, 1.0], 1.0);

        // Render canvas preview (simplified)
        let preview_rect = Rect {
            x: rect.x + 4.0,
            y: rect.y + 4.0,
            width: rect.width - 8.0,
            height: rect.height - 8.0,
        };
        self.canvas.render(renderer, preview_rect);
    }
}

impl LayoutView for Minimap {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn LayoutView], _cache: &mut LayoutCache) -> Size {
        Size { width: self.minimap_size, height: self.minimap_size }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn LayoutView], _cache: &mut LayoutCache) {}
}
