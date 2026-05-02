use cvkg_core::{View, Rect, Renderer, Never};
use std::sync::Arc;

/// Vegvísir - A radial tactical menu (Norse compass)
pub struct Vegvísir {
    pub items: Vec<VegvísirItem>,
    pub is_open: bool,
    pub on_select: Arc<dyn Fn(usize) + Send + Sync>,
}

pub struct VegvísirItem {
    pub icon: String,
    pub label: String,
}

impl Vegvísir {
    pub fn new(on_select: impl Fn(usize) + Send + Sync + 'static) -> Self {
        Self {
            items: Vec::new(),
            is_open: false,
            on_select: Arc::new(on_select),
        }
    }
    
    pub fn add_item(mut self, icon: &str, label: &str) -> Self {
        self.items.push(VegvísirItem { icon: icon.to_string(), label: label.to_string() });
        self
    }
    
    pub fn open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }
} 

impl View for Vegvísir {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }
    
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.is_open || self.items.is_empty() {
            return;
        }
        
        renderer.push_vnode(rect, "Vegvísir");
        
        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;
        let radius = (rect.width / 2.0).min(rect.height / 2.0) * 0.6;
        
        let segment_angle = 2.0 * std::f32::consts::PI / self.items.len() as f32;
        
        for (i, item) in self.items.iter().enumerate() {
            let angle = segment_angle * i as f32 - std::f32::consts::PI / 2.0;
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            
            renderer.fill_rounded_rect(
                Rect { x: x - 30.0, y: y - 30.0, width: 60.0, height: 60.0 },
                30.0,
                [0.0, 0.5, 0.8, 0.8],
            );
            
            renderer.draw_text(&item.label, x - 20.0, y + 5.0, 10.0, [1.0, 1.0, 1.0, 1.0]);
        }

        let on_select = self.on_select.clone();
        let items_len = self.items.len();
        
        renderer.register_handler("pointerclick", Arc::new(move |event| {
            if let cvkg_core::Event::PointerClick { x, y } = event {
                let dx = x - center_x;
                let dy = y - center_y;
                let dist = (dx * dx + dy * dy).sqrt();
                
                if dist >= radius - 40.0 && dist <= radius + 40.0 {
                    let mut angle = dy.atan2(dx) + std::f32::consts::PI / 2.0;
                    if angle < 0.0 { angle += 2.0 * std::f32::consts::PI; }
                    
                    let idx = ((angle / segment_angle) + 0.5) as usize % items_len;
                    on_select(idx);
                }
            }
        }));

        renderer.pop_vnode();
    }
}
