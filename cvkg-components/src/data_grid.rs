use crate::theme;
use cvkg_core::{
    AnyView, Event, Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};
use std::sync::Arc;

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
    pub(crate) selected_index: Option<usize>,
    pub(crate) on_select: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    pub(crate) on_sort: Option<Arc<dyn Fn(String, SortOrder) + Send + Sync>>,
}

#[derive(Clone, PartialEq)]
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
            selected_index: None,
            on_select: None,
            on_sort: None,
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

    /// Sets the row selection callback.
    pub fn on_select(mut self, callback: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(callback));
        self
    }

    /// Sets the column sort callback.
    pub fn on_sort(mut self, callback: impl Fn(String, SortOrder) + Send + Sync + 'static) -> Self {
        self.on_sort = Some(Arc::new(callback));
        self
    }

    /// Sets the selected row index.
    pub fn selected(mut self, index: Option<usize>) -> Self {
        self.selected_index = index;
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

        renderer.push_vnode(rect, "RunesTable");

        let header_h = 36.0;
        let mut current_x = rect.x;

        // ── Column headers (clickable for sortable columns) ──
        for (i, col) in self.columns.iter().enumerate() {
            if i >= self.frozen_columns && current_x >= rect.x + rect.width {
                break;
            }
            let col_rect = Rect {
                x: current_x,
                y: rect.y,
                width: col.width.clamp(40.0, 500.0),
                height: header_h,
            };
            let is_sorted = self.sort_column.as_deref() == Some(&col.header);
            let header_bg = if is_sorted {
                theme::table_header_bg()
            } else {
                theme::surface_elevated()
            };
            renderer.fill_rect(col_rect, header_bg);
            renderer.stroke_rect(
                col_rect,
                [0.3, 0.5, 0.8, if is_sorted { 0.8 } else { 0.4 }],
                1.0,
            );

            let sort_indicator = if is_sorted {
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
                col_rect.y + 10.0,
                13.0,
                if is_sorted {
                    theme::accent()
                } else {
                    theme::text()
                },
            );

            // ── Sort click handler ──
            if col.sortable {
                let col_name = col.header.clone();
                let on_sort = self.on_sort.clone();
                let current_order = self.sort_order.clone();
                let cr = col_rect;
                renderer.register_handler(
                    "pointerclick",
                    Arc::new(move |event| {
                        if let Event::PointerClick { x, y, .. } = event
                            && cr.contains(x, y)
                            && let Some(ref cb) = on_sort
                        {
                            let new_order = if current_order == SortOrder::Asc {
                                SortOrder::Desc
                            } else {
                                SortOrder::Asc
                            };
                            (cb)(col_name.clone(), new_order);
                        }
                    }),
                );
            }

            current_x += col.width;
        }

        // ── Body (virtualized row rendering) ──
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
                let row_y = rect.y + header_h + idx as f32 * self.row_height
                    - start_idx as f32 * self.row_height;
                let row_rect = Rect {
                    x: rect.x,
                    y: row_y,
                    width: rect.width,
                    height: self.row_height,
                };
                let is_selected = self.selected_index == Some(idx);

                // Row background: alternating + selection highlight
                let bg = if is_selected {
                    [0.0, 0.4, 0.8, 0.4]
                } else if idx % 2 == 0 {
                    theme::input_bg()
                } else {
                    theme::surface_elevated()
                };
                renderer.fill_rect(row_rect, bg);
                if is_selected {
                    renderer.stroke_rect(row_rect, [0.0, 0.8, 1.0, 0.6], 1.5);
                }

                let mut cx = rect.x;
                for col in &self.columns {
                    let cell_rect = Rect {
                        x: cx,
                        y: row_y,
                        width: col.width,
                        height: self.row_height,
                    };
                    let view = (col.cell_builder)(item);
                    view.render(renderer, cell_rect);
                    cx += col.width;
                }

                // ── Row selection click handler ──
                let row_idx = idx;
                let on_select = self.on_select.clone();
                let rr = row_rect;
                renderer.register_handler(
                    "pointerclick",
                    Arc::new(move |event| {
                        if let Event::PointerClick { x, y, .. } = event
                            && rr.contains(x, y)
                            && let Some(ref cb) = on_select
                        {
                            (cb)(row_idx);
                        }
                    }),
                );
            }
        }

        renderer.pop_vnode();
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
