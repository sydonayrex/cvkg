//! DataTable component for displaying tabular data with sorting and selection.
//!
//! Provides a `DataTable` component with typed columns, row selection,
//! and optional sorting. Renders using the standard Renderer trait.

use crate::{Color, Never, Rect, Renderer, View};
use std::marker::PhantomData;

/// A column definition for DataTable.
pub struct Column<T: Clone> {
    /// Column header text.
    pub title: String,
    /// Extract the display value from a row item.
    pub accessor: fn(&T) -> String,
    /// Whether this column is sortable.
    pub sortable: bool,
    /// Width fraction (0.0-1.0).
    pub width: f32,
}

impl<T: Clone> Column<T> {
    pub fn new(title: &str, accessor: fn(&T) -> String) -> Self {
        Self {
            title: title.to_string(),
            accessor,
            sortable: false,
            width: 1.0,
        }
    }

    pub fn sortable(mut self, yes: bool) -> Self {
        self.sortable = yes;
        self
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = w.clamp(0.01, 1.0);
        self
    }
}

/// Sort direction for DataTable columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
    None,
}

/// A DataTable component.
pub struct DataTable<T: Clone + 'static> {
    columns: Vec<Column<T>>,
    data: Vec<T>,
    selected_row: Option<usize>,
    sort_column: Option<usize>,
    sort_direction: SortDirection,
    _phantom: PhantomData<T>,
}

impl<T: Clone + 'static> DataTable<T> {
    /// Create a new DataTable with the given columns and data.
    pub fn new(columns: Vec<Column<T>>, data: Vec<T>) -> Self {
        Self {
            columns,
            data,
            selected_row: None,
            sort_column: None,
            sort_direction: SortDirection::None,
            _phantom: PhantomData,
        }
    }

    /// Set the currently selected row index.
    pub fn selected_row(mut self, idx: Option<usize>) -> Self {
        self.selected_row = idx;
        self
    }

    /// Set the sort state.
    pub fn sort(mut self, column: usize, direction: SortDirection) -> Self {
        self.sort_column = Some(column);
        self.sort_direction = direction;
        self
    }
}

impl<T: Clone + Send + Sync + 'static> View for DataTable<T> {
    type Body = Never;
    fn body(self) -> Never {
        unreachable!("DataTable renders via render()")
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let row_height = 40.0;
        let header_height = 44.0;
        let num_rows = self.data.len();
        let num_cols = self.columns.len();

        if num_cols == 0 || num_rows == 0 {
            return;
        }

        // Draw header background
        renderer.fill_rect(
            Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: header_height,
            },
            [0.95, 0.95, 0.97, 1.0],
        );

        // Draw header text
        let mut x_offset = rect.x + 12.0;
        for (col_idx, col) in self.columns.iter().enumerate() {
            let col_width = rect.width * col.width / num_cols as f32;
            renderer.draw_text(
                &col.title,
                x_offset,
                rect.y + header_height / 2.0,
                13.0,
                [0.2, 0.2, 0.2, 1.0],
            );

            // Draw sort indicator
            if self.sort_column == Some(col_idx) {
                let indicator = match self.sort_direction {
                    SortDirection::Ascending => " ▲",
                    SortDirection::Descending => " ▼",
                    SortDirection::None => "",
                };
                renderer.draw_text(
                    indicator,
                    x_offset + col_width * 0.8,
                    rect.y + header_height / 2.0,
                    11.0,
                    [0.5, 0.5, 0.5, 1.0],
                );
            }

            x_offset += col_width;
        }

        // Draw rows
        for (row_idx, item) in self.data.iter().enumerate() {
            let row_y = rect.y + header_height + row_idx as f32 * row_height;

            // Highlight selected row
            if self.selected_row == Some(row_idx) {
                renderer.fill_rect(
                    Rect {
                        x: rect.x,
                        y: row_y,
                        width: rect.width,
                        height: row_height,
                    },
                    [0.9, 0.93, 1.0, 1.0],
                );
            }

            // Draw row cells
            let mut cell_x = rect.x + 12.0;
            for (_col_idx, col) in self.columns.iter().enumerate() {
                let col_width = rect.width * col.width / num_cols as f32;
                let value = (col.accessor)(item);
                renderer.draw_text(
                    &value,
                    cell_x,
                    row_y + row_height / 2.0,
                    13.0,
                    [0.15, 0.15, 0.15, 1.0],
                );
                cell_x += col_width;
            }

            // Draw row separator
            renderer.draw_line(
                rect.x,
                row_y + row_height,
                rect.x + rect.width,
                row_y + row_height,
                [0.9, 0.9, 0.9, 1.0],
                1.0,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestRow {
        name: String,
        age: u32,
        city: String,
    }

    fn make_data() -> Vec<TestRow> {
        vec![
            TestRow {
                name: "Alice".to_string(),
                age: 30,
                city: "NYC".to_string(),
            },
            TestRow {
                name: "Bob".to_string(),
                age: 25,
                city: "LA".to_string(),
            },
            TestRow {
                name: "Carol".to_string(),
                age: 35,
                city: "Chicago".to_string(),
            },
        ]
    }

    fn make_columns() -> Vec<Column<TestRow>> {
        vec![
            Column::new("Name", |r| r.name.clone()),
            Column::new("Age", |r| r.age.to_string()),
            Column::new("City", |r| r.city.clone()),
        ]
    }

    #[test]
    fn data_table_renders_without_panic() {
        let data = make_data();
        let columns = make_columns();
        let table = DataTable::new(columns, data);

        let mut renderer = crate::testing::MockRenderer::new();
        let rect = Rect::new(0.0, 0.0, 600.0, 200.0);
        table.render(&mut renderer, rect);

        // Should have drawn header text + row text + separators
        // Header: 3 texts + 3 sort indicators = 6
        // Rows: 3 rows * 3 cells = 9 texts
        // Separators: 3 row separators
        // Total: at least 15 draw calls
        renderer.assert_draw_call_count(16);
    }

    #[test]
    fn data_table_empty_data() {
        let columns: Vec<Column<TestRow>> = vec![];
        let data = vec![];
        let table = DataTable::new(columns, data);

        let mut renderer = crate::testing::MockRenderer::new();
        let rect = Rect::new(0.0, 0.0, 600.0, 200.0);
        table.render(&mut renderer, rect);

        // No data, no draw calls
        renderer.assert_draw_call_count(0);
    }

    #[test]
    fn data_table_with_selection() {
        let data = make_data();
        let columns = make_columns();
        let table = DataTable::new(columns, data).selected_row(Some(1));

        let mut renderer = crate::testing::MockRenderer::new();
        let rect = Rect::new(0.0, 0.0, 600.0, 200.0);
        table.render(&mut renderer, rect);

        // Should still render all rows
        renderer.assert_draw_call_count(17);
    }

    #[test]
    fn data_table_with_sort() {
        let data = make_data();
        let columns = make_columns();
        let table = DataTable::new(columns, data).sort(0, SortDirection::Ascending);

        let mut renderer = crate::testing::MockRenderer::new();
        let rect = Rect::new(0.0, 0.0, 600.0, 200.0);
        table.render(&mut renderer, rect);

        // Sort indicator adds one extra draw call
        renderer.assert_draw_call_count(17);
    }

    #[test]
    fn column_builder() {
        let col = Column::<TestRow>::new("Test", |r| r.name.clone())
            .sortable(true)
            .width(0.5);

        assert!(col.sortable);
        assert_eq!(col.width, 0.5);
    }
}
