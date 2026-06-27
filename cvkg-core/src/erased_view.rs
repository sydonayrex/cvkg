use crate::*;

pub trait ErasedView: Send {
    fn render_erased(&self, renderer: &mut dyn Renderer, rect: Rect);
    fn name(&self) -> &'static str;
    fn flex_weight_erased(&self) -> f32;
    fn layout_erased(&self) -> Option<&dyn layout::LayoutView>;
    fn grid_placement_erased(&self) -> Option<GridPlacement>;
    fn intrinsic_size_erased(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size;
    fn clone_box(&self) -> Box<dyn ErasedView>;
}

impl<V: View + Clone + 'static> ErasedView for V {
    fn render_erased(&self, renderer: &mut dyn Renderer, rect: Rect) {
        self.render(renderer, rect);
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<V>()
    }

    fn flex_weight_erased(&self) -> f32 {
        self.flex_weight()
    }

    fn layout_erased(&self) -> Option<&dyn layout::LayoutView> {
        self.layout()
    }

    fn grid_placement_erased(&self) -> Option<GridPlacement> {
        self.get_grid_placement()
    }

    fn intrinsic_size_erased(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        self.intrinsic_size(renderer, proposal)
    }

    fn clone_box(&self) -> Box<dyn ErasedView> {
        Box::new(self.clone())
    }
}

/// A view that memoizes its rendering based on a stable ID and data hash.
/// The renderer can use this to skip re-rendering the sub-tree if the data hasn't changed.
pub struct MemoView<V, F> {
    id: u64,
    data_hash: u64,
    builder: F,
    _v: std::marker::PhantomData<V>,
}

impl<V: View, F: Fn() -> V + Send + Sync> MemoView<V, F> {
    /// Create a new MemoView with a stable ID and a data hash.
    pub fn new(id: u64, data_hash: u64, builder: F) -> Self {
        Self {
            id,
            data_hash,
            builder,
            _v: std::marker::PhantomData,
        }
    }
}

impl<V: View + 'static, F: Fn() -> V + Send + Sync + 'static> View for MemoView<V, F> {
    type Body = Never;
    fn body(self) -> Self::Body {
        // SAFETY: `Never` is uninhabitable (zero variants). MemoView renders via
        // `render()` using the memoized builder closure and never exposes a body.
        unreachable!("MemoView does not have a body")
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.memoize(self.id, self.data_hash, &|r| {
            let view = (self.builder)();
            view.render(r, rect);
        });
    }
}

/// A type-erased View wrapper.
pub struct AnyView {
    inner: Box<dyn ErasedView>,
}

impl Clone for AnyView {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone_box(),
        }
    }
}

impl AnyView {
    pub fn new<V: View + Clone + 'static>(view: V) -> Self {
        Self {
            inner: Box::new(view),
        }
    }
}

impl View for AnyView {
    type Body = Never;
    fn body(self) -> Self::Body {
        // SAFETY: `Never` is uninhabitable. AnyView is a type-erased wrapper that
        // renders via `render_erased()` and never exposes a composable body.
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, self.inner.name());
        self.inner.render_erased(renderer, rect);
        renderer.pop_vnode();
    }

    fn flex_weight(&self) -> f32 {
        self.inner.flex_weight_erased()
    }

    fn layout(&self) -> Option<&dyn layout::LayoutView> {
        self.inner.layout_erased()
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        self.inner.intrinsic_size_erased(renderer, proposal)
    }

    fn get_grid_placement(&self) -> Option<GridPlacement> {
        self.inner.grid_placement_erased()
    }
}
