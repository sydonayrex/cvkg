use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};
use cvkg_core::{Never, Rect, Renderer, Size, View};

/// A vertical stack of views
///
/// # Contract
/// Positions subviews vertically aligned according to layout constraints and computes intrinsic sizing.
#[derive(Clone)]
pub struct VStack {
    spacing: f32,
    alignment: cvkg_core::Alignment,
    distribution: cvkg_core::Distribution,
    children: Vec<cvkg_core::AnyView>,
    layout_cache: std::sync::Arc<std::sync::Mutex<LayoutCache>>,
    wrap: bool,
}

impl VStack {
    /// Create a new vertical stack with the given spacing between children.
    pub fn new(spacing: f32) -> Self {
        Self {
            spacing,
            alignment: cvkg_core::Alignment::Center,
            distribution: cvkg_core::Distribution::Fill,
            children: Vec::new(),
            layout_cache: std::sync::Arc::new(std::sync::Mutex::new(LayoutCache::new())),
            wrap: false,
        }
    }

    /// Configures the stack alignment.
    pub fn alignment(mut self, alignment: cvkg_core::Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Configures the distribution behavior.
    pub fn distribution(mut self, distribution: cvkg_core::Distribution) -> Self {
        self.distribution = distribution;
        self
    }

    /// Enable or disable flex-wrap behavior.
    /// When true, children will wrap to the next line when they overflow.
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Adds a child view.
    pub fn child<V: View + Clone + 'static>(mut self, view: V) -> Self {
        self.children.push(view.erase());
        self
    }
}

impl View for VStack {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        renderer.push_vnode(rect, "VStack");
        if self.children.is_empty() {
            renderer.pop_vnode();
            return;
        }

        let layouts: Vec<&dyn LayoutView> =
            self.children.iter().filter_map(|c| c.layout()).collect();

        let mut cache = self.layout_cache.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let rects = cvkg_layout::VStack::compute_layout(
            self.spacing,
            self.alignment,
            self.distribution,
            rect,
            &layouts,
            &mut cache,
        );

        let mut rect_idx = 0;
        for child in self.children.iter() {
            if child.layout().is_some() && rect_idx < rects.len() {
                child.render(renderer, rects[rect_idx]);
                rect_idx += 1;
            }
        }
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for (i, child) in self.children.iter().enumerate() {
            let child_size = child.intrinsic_size(renderer, proposal);
            width = width.max(child_size.width);
            height += child_size.height;
            if i < self.children.len() - 1 {
                height += self.spacing;
            }
        }

        Size { width, height }
    }

    fn layout(&self) -> Option<&dyn LayoutView> {
        Some(self)
    }
}

impl LayoutView for VStack {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for (i, child) in self.children.iter().enumerate() {
            if let Some(layout) = child.layout() {
                let child_size = layout.size_that_fits(proposal, &[], cache);
                width = width.max(child_size.width);
                height += child_size.height;
                if i < self.children.len() - 1 {
                    height += self.spacing;
                }
            }
        }

        Size { width, height }
    }

    fn place_subviews(
        &self,
        bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        cache: &mut LayoutCache,
    ) {
        let layouts: Vec<&dyn LayoutView> =
            self.children.iter().filter_map(|c| c.layout()).collect();

        let _rects = cvkg_layout::VStack::compute_layout(
            self.spacing,
            self.alignment,
            self.distribution,
            bounds,
            &layouts,
            cache,
        );
    }
}

/// A vertical stack that only renders visible children (efficient for long lists)
#[derive(Clone)]
pub struct LazyVStack {
    spacing: f32,
    children: Vec<cvkg_core::AnyView>,
}

impl LazyVStack {
    /// Creates a new LazyVStack with spacing.
    pub fn new(spacing: f32) -> Self {
        Self {
            spacing,
            children: Vec::new(),
        }
    }

    /// Adds a child view.
    pub fn child<V: View + Clone + 'static>(mut self, view: V) -> Self {
        self.children.push(view.erase());
        self
    }
}

impl View for LazyVStack {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.children.is_empty() {
            return;
        }

        let clip = renderer.current_clip_rect();
        let viewport_y = clip.y.max(rect.y);
        let viewport_bottom = (clip.y + clip.height).min(rect.y + rect.height);

        if viewport_bottom <= viewport_y {
            return;
        }

        let child_height = 40.0;

        let start_idx = ((viewport_y - rect.y) / (child_height + self.spacing)).floor() as usize;
        let visible_count =
            ((viewport_bottom - viewport_y) / (child_height + self.spacing)).ceil() as usize;
        let end_idx = (start_idx + visible_count + 1).min(self.children.len());

        for idx in start_idx..end_idx {
            let child = &self.children[idx];
            let child_y = rect.y + idx as f32 * (child_height + self.spacing);

            child.render(
                renderer,
                Rect {
                    x: rect.x,
                    y: child_y,
                    width: rect.width,
                    height: child_height,
                },
            );
        }
    }
}

/// A horizontal stack of views
#[derive(Clone)]
pub struct HStack {
    spacing: f32,
    alignment: cvkg_core::Alignment,
    distribution: cvkg_core::Distribution,
    children: Vec<cvkg_core::AnyView>,
    wrap: bool,
}

impl HStack {
    /// Create a new horizontal stack with the given spacing between children.
    pub fn new(spacing: f32) -> Self {
        Self {
            spacing,
            alignment: cvkg_core::Alignment::Center,
            distribution: cvkg_core::Distribution::Fill,
            children: Vec::new(),
            wrap: false,
        }
    }

    /// Configures the alignment.
    pub fn alignment(mut self, alignment: cvkg_core::Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Configures the distribution mode.
    pub fn distribution(mut self, distribution: cvkg_core::Distribution) -> Self {
        self.distribution = distribution;
        self
    }

    /// Enable or disable flex-wrap behavior.
    /// When true, children will wrap to the next line when they overflow.
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Adds a child view.
    pub fn child<V: View + Clone + 'static>(mut self, view: V) -> Self {
        self.children.push(view.erase());
        self
    }
}

impl View for HStack {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        if self.children.is_empty() {
            return;
        }

        let mut cache = LayoutCache::new();
        let layouts: Vec<&dyn LayoutView> =
            self.children.iter().filter_map(|c| c.layout()).collect();

        let rects = cvkg_layout::HStack::compute_layout(
            self.spacing,
            self.alignment,
            self.distribution,
            rect,
            &layouts,
            &mut cache,
        );

        let mut rect_idx = 0;
        for child in self.children.iter() {
            if child.layout().is_some() && rect_idx < rects.len() {
                child.render(renderer, rects[rect_idx]);
                rect_idx += 1;
            }
        }
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for (i, child) in self.children.iter().enumerate() {
            let child_size = child.intrinsic_size(renderer, proposal);
            width += child_size.width;
            height = height.max(child_size.height);
            if i < self.children.len() - 1 {
                width += self.spacing;
            }
        }

        Size { width, height }
    }

    fn layout(&self) -> Option<&dyn LayoutView> {
        Some(self)
    }
}

impl LayoutView for HStack {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for (i, child) in self.children.iter().enumerate() {
            if let Some(layout) = child.layout() {
                let child_size = layout.size_that_fits(proposal, &[], cache);
                width += child_size.width;
                height = height.max(child_size.height);
                if i < self.children.len() - 1 {
                    width += self.spacing;
                }
            }
        }

        Size { width, height }
    }

    fn place_subviews(
        &self,
        bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        cache: &mut LayoutCache,
    ) {
        let layouts: Vec<&dyn LayoutView> =
            self.children.iter().filter_map(|c| c.layout()).collect();

        let _rects = cvkg_layout::HStack::compute_layout(
            self.spacing,
            self.alignment,
            self.distribution,
            bounds,
            &layouts,
            cache,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cvkg_core::layout::{LayoutCache, LayoutView, Rect, SizeProposal};
    use cvkg_core::{Never, Size, View};

    #[derive(Clone)]
    struct FixedLayout;

    impl View for FixedLayout {
        type Body = Never;
        fn body(self) -> Self::Body {
            unreachable!()
        }
    }

    impl LayoutView for FixedLayout {
        fn size_that_fits(
            &self,
            _proposal: SizeProposal,
            _subviews: &[&dyn LayoutView],
            _cache: &mut LayoutCache,
        ) -> Size {
            Size { width: 80.0, height: 20.0 }
        }
        fn place_subviews(
            &self,
            _bounds: Rect,
            _subviews: &mut [&mut dyn LayoutView],
            _cache: &mut LayoutCache,
        ) {
        }
    }

    #[test]
    fn vstack_clone_shares_layout_cache() {
        let stack = VStack::new(8.0).child(FixedLayout);
        let cloned = stack.clone();
        assert!(std::sync::Arc::ptr_eq(&stack.layout_cache, &cloned.layout_cache));
    }
}
