use cvkg_core::{layout::{LayoutCache, LayoutView, SizeProposal}, Rect, Renderer, Size, View, Never, AnyView};
use crate::Spacer;

/// A virtualized list that only renders items in the visible viewport.
pub struct VirtualList<D>
where
    D: Send + Sync + 'static,
{
    data: Vec<D>,
    item_height: f32,
    view_builder: Box<dyn Fn(&D) -> AnyView + Send + Sync>,
}

impl<D> VirtualList<D>
where
    D: Send + Sync + 'static,
{
    /// Create a new VirtualList from a vector of data.
    pub fn new(data: Vec<D>) -> Self {
        Self {
            data,
            item_height: 24.0, // default
            view_builder: Box::new(|_| Spacer::new(0.0).erase()),
        }
    }

    /// Set the height of each item (fixed).
    pub fn item_height(mut self, height: f32) -> Self {
        self.item_height = height;
        self
    }

    /// Set a function that builds a view for each item.
    pub fn view_builder<F, V>(mut self, builder: F) -> Self
    where
        F: Fn(&D) -> V + Send + Sync + 'static,
        V: View + 'static,
    {
        self.view_builder = Box::new(move |d| builder(d).erase());
        self
    }
}

impl<D> View for VirtualList<D>
where
    D: Send + Sync + 'static,
{
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Basic vertical virtualization
        // We assume rect.y is the coordinate in the scrollable content.
        // If we want to be truly virtual, we should probably know the scroll offset.
        // For now, we'll just iterate and check bounds, which is O(N).
        // A better way is O(visible) by calculating start/end index.
        
        // Assuming rect.y is the offset of the visible window relative to the list top
        // and rect.height is the window height.
        // But in cvkg, render rect is usually the absolute drawing area.
        
        // Let's use a simple heuristic: if we are rendering at a very large negative Y,
        // we are likely clipped by a parent ScrollView.
        
        for (idx, item) in self.data.iter().enumerate() {
            let item_y = idx as f32 * self.item_height;
            
            // Only render if it intersects with the provided rect
            // (Note: this rect is in local coordinates if the parent is a container)
            if item_y + self.item_height < 0.0 || item_y > rect.height {
                continue;
            }
            
            let item_rect = Rect {
                x: rect.x,
                y: rect.y + item_y,
                width: rect.width,
                height: self.item_height,
            };
            
            let view = (self.view_builder)(item);
            view.render(renderer, item_rect);
        }
    }

    fn layout(&self) -> Option<&dyn LayoutView> {
        Some(self)
    }
}

impl<D> LayoutView for VirtualList<D>
where
    D: Send + Sync + 'static,
{
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        let width = proposal.width.unwrap_or(0.0); 
        let height = self.data.len() as f32 * self.item_height;
        Size { width, height }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}
