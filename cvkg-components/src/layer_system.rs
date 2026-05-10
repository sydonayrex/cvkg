use cvkg_core::{layout::{LayoutCache, LayoutView, SizeProposal}, Rect, Renderer, Size, View, Never};

/// Layer in a layer system
pub struct Layer {
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
}

/// Layer system for managing canvas layers
pub struct LayerSystem {
    pub(crate) layers: Vec<Layer>,
    pub(crate) active_layer: Option<String>,
}

impl Default for LayerSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl LayerSystem {
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            active_layer: None,
        }
    }

    pub fn layer(mut self, id: &str, name: &str) -> Self {
        self.layers.push(Layer {
            id: id.to_string(),
            name: name.to_string(),
            visible: true,
            locked: false,
            opacity: 1.0,
        });
        self
    }

    pub fn visible(mut self, id: &str, visible: bool) -> Self {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.id == id) {
            layer.visible = visible;
        }
        self
    }

    pub fn opacity(mut self, id: &str, opacity: f32) -> Self {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.id == id) {
            layer.opacity = opacity.clamp(0.0, 1.0);
        }
        self
    }
}

impl View for LayerSystem {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let item_h = 32.0;
        let padding = 8.0;

        for (i, layer) in self.layers.iter().enumerate() {
            let y = rect.y + padding + i as f32 * item_h;
            let layer_rect = Rect {
                x: rect.x,
                y,
                width: rect.width,
                height: item_h,
            };

            let is_active = self.active_layer.as_deref() == Some(&layer.id);
            let bg = if is_active { [0.1, 0.2, 0.4, 1.0] } else { [0.06, 0.06, 0.1, 1.0] };
            renderer.fill_rounded_rect(layer_rect, 4.0, bg);
            if is_active {
                renderer.stroke_rounded_rect(layer_rect, 4.0, [0.0, 0.8, 1.0, 1.0], 1.0);
            }

            // Eye icon for visibility
            let icon = if layer.visible { "👁" } else { "🚫" };
            renderer.draw_text(icon, layer_rect.x + 8.0, layer_rect.y + 10.0, 12.0, [0.6, 0.8, 1.0, 1.0]);

            // Layer name
            renderer.draw_text(&layer.name, layer_rect.x + 32.0, layer_rect.y + 10.0, 12.0, [0.8, 0.8, 0.9, 1.0]);

            // Lock icon
            let lock_icon = if layer.locked { "🔒" } else { "🔓" };
            renderer.draw_text(lock_icon, layer_rect.x + layer_rect.width - 60.0, layer_rect.y + 10.0, 12.0, [0.7, 0.5, 0.4, 1.0]);
        }
    }
}

impl LayoutView for LayerSystem {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn LayoutView], _cache: &mut LayoutCache) -> Size {
        let height = (self.layers.len() as f32 * 40.0) + 16.0;
        Size { width: 200.0, height }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn LayoutView], _cache: &mut LayoutCache) {}
}
