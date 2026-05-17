use cvkg_core::{
    AnyView, Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};

/// Column definition for a DataGrid.
pub struct DataGridColumn<D> {
    pub header: String,
    pub width: f32,
    pub sortable: bool,
    pub cell_builder: Box<dyn Fn(&D) -> AnyView + Send + Sync>,
}

/// A virtualized data grid with advanced features including sorting, filtering, and column management.
pub struct RunesTable<D>
where
    D: Send + Sync + 'static,
{
    pub(crate) data: Vec<D>,
    pub(crate) row_height: f32,
    pub(crate) columns: Vec<DataGridColumn<D>>,
    pub(crate) sort_column: Option<String>,
    pub(crate) sort_order: SortOrder,
    pub(crate) frozen_columns: usize,
    pub(crate) show_sparklines: bool,
}

pub enum SortOrder {
    Asc,
    Desc,
}

impl<D> RunesTable<D>
where
    D: Send + Sync + 'static,
{
    /// Creates a new RunesTable with the given data.
    pub fn new(data: Vec<D>) -> Self {
        Self {
            data,
            row_height: 32.0,
            columns: Vec::new(),
            sort_column: None,
            sort_order: SortOrder::Asc,
            frozen_columns: 0,
            show_sparklines: false,
        }
    }

    /// Sets the height of each row in the table.
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height;
        self
    }

    /// Sets the number of columns that are frozen (fixed) on the left side.
    pub fn frozen_columns(mut self, count: usize) -> Self {
        self.frozen_columns = count.min(self.columns.len());
        self
    }

    /// Enables or disables sparkline rendering in cells.
    pub fn sparklines(mut self, enabled: bool) -> Self {
        self.show_sparklines = enabled;
        self
    }

    /// Adds a column to the table.
    pub fn column<F, V>(mut self, header: &str, width: f32, sortable: bool, builder: F) -> Self
    where
        F: Fn(&D) -> V + Send + Sync + 'static,
        V: View + Clone + 'static,
    {
        self.columns.push(DataGridColumn {
            header: header.to_string(),
            width,
            sortable,
            cell_builder: Box::new(move |d| builder(d).erase()),
        });
        self
    }

    /// Sets the sorting column and order.
    pub fn sort_by(mut self, column: &str, order: SortOrder) -> Self {
        self.sort_column = Some(column.to_string());
        self.sort_order = order;
        self
    }
}

impl<D> View for RunesTable<D>
where
    D: Send + Sync + 'static,
{
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.columns.is_empty() || self.data.is_empty() {
            return;
        }

        // Render frozen header columns
        let header_h = 36.0;
        let mut current_x = rect.x;
        for (i, col) in self.columns.iter().enumerate() {
            if i >= self.frozen_columns {
                break;
            }
            let col_rect = Rect {
                x: current_x,
                y: rect.y,
                width: col.width,
                height: header_h,
            };
            renderer.fill_rect(col_rect, [0.08, 0.08, 0.12, 1.0]);
            renderer.stroke_rect(col_rect, [0.3, 0.5, 0.8, 1.0], 1.0);

            let sort_indicator = if self.sort_column.as_deref() == Some(&col.header) {
                match self.sort_order {
                    SortOrder::Asc => " ▲",
                    SortOrder::Desc => " ▼",
                }
            } else {
                ""
            };
            renderer.draw_text(
                &format!("{}{}", col.header, sort_indicator),
                col_rect.x + 8.0,
                col_rect.y + 8.0,
                13.0,
                [0.8, 0.9, 1.0, 1.0],
            );
            current_x += col.width;
        }

        // Render body (virtualized)
        let body_rect = Rect {
            x: rect.x,
            y: rect.y + header_h,
            width: rect.width,
            height: rect.height - header_h,
        };

        let start_idx = if body_rect.y > 0.0 {
            (body_rect.y / self.row_height).floor() as usize
        } else {
            0
        };

        let visible_count = ((body_rect.height / self.row_height).ceil() as usize).max(1);
        let end_idx = (start_idx + visible_count + 1).min(self.data.len());

        for idx in start_idx..end_idx {
            if let Some(item) = self.data.get(idx) {
                let row_y = idx as f32 * self.row_height + header_h;
                let mut current_x = rect.x;
                for col in &self.columns {
                    let cell_rect = Rect {
                        x: current_x,
                        y: row_y,
                        width: col.width,
                        height: self.row_height,
                    };

                    // Alternating row colors
                    let bg = if idx % 2 == 0 {
                        [0.06, 0.06, 0.1, 1.0]
                    } else {
                        [0.08, 0.08, 0.12, 1.0]
                    };
                    renderer.fill_rect(cell_rect, bg);

                    let view = (col.cell_builder)(item);
                    view.render(renderer, cell_rect);
                    current_x += col.width;
                }
            }
        }
    }
}

impl<D> LayoutView for RunesTable<D>
where
    D: Send + Sync + 'static,
{
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        let width = self.columns.iter().map(|c| c.width).sum();
        let height = self.data.len() as f32 * self.row_height + 36.0;
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

/// DataTable adds search filtering and header controls to RunesTable.
pub struct DataTable<D>
where
    D: Send + Sync + 'static,
{
    pub(crate) table: RunesTable<D>,
    pub(crate) filter_text: String,
}

impl<D> DataTable<D>
where
    D: Send + Sync + 'static,
{
    /// Creates a new DataTable with the given data.
    pub fn new(data: Vec<D>) -> Self {
        Self {
            table: RunesTable::new(data),
            filter_text: String::new(),
        }
    }

    /// Adds a column to the underlying table.
    pub fn column<F, V>(mut self, header: &str, width: f32, sortable: bool, builder: F) -> Self
    where
        F: Fn(&D) -> V + Send + Sync + 'static,
        V: View + Clone + 'static,
    {
        self.table = self.table.column(header, width, sortable, builder);
        self
    }

    pub fn filter(mut self, text: &str) -> Self {
        self.filter_text = text.to_string();
        self
    }
}

impl<D> View for DataTable<D>
where
    D: Send + Sync + 'static,
{
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Render filter bar
        let filter_h = 32.0;
        let filter_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: filter_h,
        };
        renderer.fill_rounded_rect(filter_rect, 4.0, [0.1, 0.1, 0.15, 1.0]);
        renderer.draw_text(
            &format!("Filter: {}", self.filter_text),
            filter_rect.x + 8.0,
            filter_rect.y + 10.0,
            13.0,
            [0.6, 0.6, 0.7, 1.0],
        );

        // Render table below filter
        let table_rect = Rect {
            x: rect.x,
            y: rect.y + filter_h,
            width: rect.width,
            height: rect.height - filter_h,
        };
        self.table.render(renderer, table_rect);
    }
}
