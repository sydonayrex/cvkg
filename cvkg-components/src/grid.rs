use cvkg_core::{AnyView, LayoutCache, Never, Rect, Renderer, Size, SizeProposal, View};
pub use cvkg_layout::GridTrack;

/// A 2D grid layout container that arranges its children according to row and column track sizing.
#[derive(Clone)]
pub struct Grid {
    columns: Vec<GridTrack>,
    rows: Vec<GridTrack>,
    column_gap: f32,
    row_gap: f32,
    children: Vec<AnyView>,
    layout_cache: std::sync::Arc<std::sync::Mutex<LayoutCache>>,
}

impl Grid {
    /// Create a new Grid layout container.
    pub fn new(columns: Vec<GridTrack>, rows: Vec<GridTrack>) -> Self {
        Self {
            columns,
            rows,
            column_gap: 0.0,
            row_gap: 0.0,
            children: Vec::new(),
            layout_cache: std::sync::Arc::new(std::sync::Mutex::new(LayoutCache::new())),
        }
    }

    /// Set the gap between columns.
    pub fn column_gap(mut self, gap: f32) -> Self {
        self.column_gap = gap;
        self
    }

    /// Set the gap between rows.
    pub fn row_gap(mut self, gap: f32) -> Self {
        self.row_gap = gap;
        self
    }

    /// Set the gap between both rows and columns.
    pub fn gap(mut self, gap: f32) -> Self {
        self.column_gap = gap;
        self.row_gap = gap;
        self
    }

    /// Add a child view to the grid.
    pub fn child<V: View + Clone + 'static>(mut self, view: V) -> Self {
        self.children.push(view.erase());
        self
    }
}

impl View for Grid {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.children.is_empty() {
            return;
        }

        let mut layouts = Vec::new();
        let mut placements = Vec::new();
        for child in &self.children {
            if let Some(l) = child.layout() {
                layouts.push(l);
                placements.push(child.get_grid_placement());
            }
        }

        let grid_engine = cvkg_layout::Grid {
            columns: self.columns.clone(),
            rows: self.rows.clone(),
            column_gap: self.column_gap,
            row_gap: self.row_gap,
        };

        let mut cache = self.layout_cache.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let rects = grid_engine.compute_layout_rects(rect, &layouts, &placements, &mut cache);

        let mut rect_idx = 0;
        for child in &self.children {
            if child.layout().is_some() && rect_idx < rects.len() {
                child.render(renderer, rects[rect_idx]);
                rect_idx += 1;
            }
        }
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let mut layouts = Vec::new();
        let mut placements = Vec::new();
        for child in &self.children {
            if let Some(l) = child.layout() {
                layouts.push(l);
                placements.push(child.get_grid_placement());
            }
        }

        let grid_engine = cvkg_layout::Grid {
            columns: self.columns.clone(),
            rows: self.rows.clone(),
            column_gap: self.column_gap,
            row_gap: self.row_gap,
        };

        let width = proposal.width.unwrap_or(300.0);
        let height = proposal.height.unwrap_or(300.0);
        let bounds = Rect::new(0.0, 0.0, width, height);

        let mut cache = self.layout_cache.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let rects = grid_engine.compute_layout_rects(bounds, &layouts, &placements, &mut cache);

        if rects.is_empty() {
            return Size::ZERO;
        }
        let mut max_x = 0.0f32;
        let mut max_y = 0.0f32;
        for r in rects {
            max_x = max_x.max(r.x + r.width);
            max_y = max_y.max(r.y + r.height);
        }
        Size {
            width: max_x,
            height: max_y,
        }
    }
}
