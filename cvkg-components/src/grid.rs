use cvkg_core::{AnyView, Never, Rect, Renderer, View};

/// A grid layout component
pub struct Grid {
    rows: usize,
    cols: usize,
    spacing: f32,
    children: Vec<AnyView>,
}

impl Grid {
    /// Create a new Grid with specified rows and columns.
    pub fn new(rows: usize, cols: usize, spacing: f32) -> Self {
        Self {
            rows,
            cols,
            spacing,
            children: Vec::new(),
        }
    }

    /// Add a child to the grid.
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
        if self.children.is_empty() || self.rows == 0 || self.cols == 0 {
            return;
        }

        let item_width = (rect.width - (self.cols - 1) as f32 * self.spacing) / self.cols as f32;
        let item_height = (rect.height - (self.rows - 1) as f32 * self.spacing) / self.rows as f32;

        for (i, child) in self.children.iter().enumerate() {
            let row = i / self.cols;
            let col = i % self.cols;

            if row >= self.rows {
                break;
            }

            let child_rect = Rect {
                x: rect.x + col as f32 * (item_width + self.spacing),
                y: rect.y + row as f32 * (item_height + self.spacing),
                width: item_width,
                height: item_height,
            };

            child.render(renderer, child_rect);
        }
    }
}
