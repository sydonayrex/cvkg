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
        V: View + Clone + 'static,
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
        // Calculate visible range for O(visible) complexity using current clip
        let clip = renderer.current_clip_rect();
        
        // Find intersection of list bounds and clip rect to determine viewport
        let viewport_y = clip.y.max(rect.y);
        let viewport_bottom = (clip.y + clip.height).min(rect.y + rect.height);
        
        if viewport_bottom <= viewport_y {
            return; // Not visible
        }

        let start_idx = ((viewport_y - rect.y) / self.item_height).floor() as usize;
        let visible_count = ((viewport_bottom - viewport_y) / self.item_height).ceil() as usize;
        let end_idx = (start_idx + visible_count + 1).min(self.data.len());
        
        // Only iterate through visible items
        for idx in start_idx..end_idx {
            if let Some(item) = self.data.get(idx) {
                let item_y = rect.y + idx as f32 * self.item_height;
                let item_rect = Rect {
                    x: rect.x,
                    y: item_y,
                    width: rect.width,
                    height: self.item_height,
                };
                
                let view = (self.view_builder)(item);
                view.render(renderer, item_rect);
            }
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
