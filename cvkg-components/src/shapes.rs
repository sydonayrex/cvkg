use cvkg_core::{View, Rect, Renderer, Never, SizeProposal, Size};

/// Hvergelmir - A hexagonal shape primitive (Norse equivalent of Hexagon)
pub struct Hvergelmir {
    pub size: f32,
    pub color: [f32; 4],
    pub stroke_width: f32,
}

impl Hvergelmir {
    pub fn new(size: f32) -> Self {
        Self {
            size,
            color: [0.0, 0.8, 1.0, 1.0], // Cyan
            stroke_width: 2.0,
        }
    }
    
    pub fn color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

impl View for Hvergelmir {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }
    
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;
        let radius = (self.size / 2.0).min(rect.width / 2.0).min(rect.height / 2.0);
        
        let vertices: Vec<[f32; 2]> = (0..6)
            .map(|i| {
                let angle = std::f32::consts::PI / 3.0 * i as f32 - std::f32::consts::PI / 6.0;
                [
                    center_x + radius * angle.cos(),
                    center_y + radius * angle.sin(),
                ]
            })
            .collect();       
        renderer.fill_polygon(&vertices, self.color);
        renderer.stroke_polygon(&vertices, [1.0, 1.0, 1.0, 0.8], self.stroke_width);
    }
    
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let size = proposal.width.unwrap_or(self.size);
        Size { width: size, height: size }
    }

    fn layout(&self) -> Option<&dyn cvkg_core::layout::LayoutView> {
        Some(self)
    }
}

impl cvkg_core::layout::LayoutView for Hvergelmir {
    fn size_that_fits(&self, proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        let s = proposal.width.unwrap_or(self.size);
        Size { width: s, height: s }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Skjaldborg - A trapezoidal panel (Norse equivalent of Trapezoid / Shield Wall)
pub struct Skjaldborg {
    pub color: [f32; 4],
}

impl Skjaldborg {
    pub fn new(color: [f32; 4]) -> Self {
        Self { color }
    }
}

impl View for Skjaldborg {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }
    
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let vertices = [
            [rect.x, rect.y], // top left
            [rect.x + rect.width, rect.y], // top right
            [rect.x + rect.width * 0.85, rect.y + rect.height], // bottom right
            [rect.x + rect.width * 0.15, rect.y + rect.height], // bottom left
        ];
        
        renderer.fill_polygon(&vertices, self.color);
    }

    fn layout(&self) -> Option<&dyn cvkg_core::layout::LayoutView> {
        Some(self)
    }
}

impl cvkg_core::layout::LayoutView for Skjaldborg {
    fn size_that_fits(&self, proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { 
            width: proposal.width.unwrap_or(100.0), 
            height: proposal.height.unwrap_or(50.0) 
        }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Idavoll - A tactical octagonal container (8-sided).
/// Named after the plain in Asgard where the gods established their temples.
pub struct Idavoll<V: View> {
    pub content: V,
    pub color: [f32; 4],
    pub stroke_width: f32,
}

impl<V: View> Idavoll<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            color: [1.0, 0.84, 0.0, 0.9], // Viking Gold
            stroke_width: 1.5,
        }
    }

    pub fn color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

impl<V: View> View for Idavoll<V> {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;
        let radius = (rect.width / 2.0).min(rect.height / 2.0);

        // 8 vertices for Octagon
        let vertices: Vec<[f32; 2]> = (0..8)
            .map(|i| {
                let angle = (std::f32::consts::PI / 4.0) * i as f32 + (std::f32::consts::PI / 8.0);
                [
                    center_x + radius * angle.cos(),
                    center_y + radius * angle.sin(),
                ]
            })
            .collect();
        
        let mut bg_color = self.color;
        bg_color[3] = 0.1;
        renderer.fill_polygon(&vertices, bg_color);
        renderer.stroke_polygon(&vertices, self.color, self.stroke_width);
        
        // Inscribed square content area
        let content_size = radius * 1.414 / 1.5; 
        let content_rect = Rect {
            x: center_x - content_size / 2.0,
            y: center_y - content_size / 2.0,
            width: content_size,
            height: content_size,
        };
        self.content.render(renderer, content_rect);
    }
}

/// PolygonFrame - A geometric container with a specified number of sides.
/// Supports Hexagons (6), Octagons (8), and other tactical shapes.
pub struct PolygonFrame<V: View> {
    pub content: V,
    pub sides: u32,
    pub color: [f32; 4],
    pub stroke_width: f32,
    pub rotation: f32,
}

impl<V: View> PolygonFrame<V> {
    pub fn new(content: V, sides: u32) -> Self {
        Self {
            content,
            sides: sides.max(3),
            color: [1.0, 0.84, 0.0, 0.9], // Viking Gold
            stroke_width: 1.5,
            rotation: 0.0,
        }
    }

    pub fn rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

impl<V: View> View for PolygonFrame<V> {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if cvkg_core::load_system_state().realm == cvkg_core::Realm::Midgard {
            renderer.fill_rounded_rect(rect, 4.0, [0.1, 0.12, 0.15, 0.1]);
            renderer.stroke_rect(rect, self.color, self.stroke_width);
            self.content.render(renderer, rect.inset(4.0));
            return;
        }

        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;
        let radius = (rect.width / 2.0).min(rect.height / 2.0);

        let vertices: Vec<[f32; 2]> = (0..self.sides)
            .map(|i| {
                let angle = (2.0 * std::f32::consts::PI / self.sides as f32) * i as f32 + self.rotation;
                [
                    center_x + radius * angle.cos(),
                    center_y + radius * angle.sin(),
                ]
            })
            .collect();
        
        // 1. Background
        let mut bg_color = self.color;
        bg_color[3] = 0.1;
        renderer.fill_polygon(&vertices, bg_color);
        
        // 2. Border
        renderer.stroke_polygon(&vertices, self.color, self.stroke_width);
        
        // 3. Content
        // We calculate the largest inscribed square to ensure content fits
        let inner_radius = radius * (std::f32::consts::PI / self.sides as f32).cos();
        let content_size = (inner_radius * 2.0) / 1.414; // Approximate for square
        let content_rect = Rect {
            x: center_x - content_size / 2.0,
            y: center_y - content_size / 2.0,
            width: content_size,
            height: content_size,
        };
        self.content.render(renderer, content_rect);
    }
}
