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
