use cvkg_core::{layout::{LayoutCache, LayoutView, SizeProposal}, Rect, Renderer, Size, View, Never, AnyView};
use crate::Spacer;

/// Column definition for a VirtualTable.
pub struct TableColumn<D> {
    pub header: String,
    pub width: f32,
    pub cell_builder: Box<dyn Fn(&D) -> AnyView + Send + Sync>,
}

/// A virtualized table that only renders rows in the visible viewport.
pub struct VirtualTable<D>
where
    D: Send + Sync + 'static,
{
    data: Vec<D>,
    row_height: f32,
    columns: Vec<TableColumn<D>>,
}

impl<D> VirtualTable<D>
where
    D: Send + Sync + 'static,
{
    pub fn new(data: Vec<D>) -> Self {
        Self {
            data,
            row_height: 30.0,
            columns: Vec::new(),
        }
    }

    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height;
        self
    }

    pub fn column<F, V>(mut self, header: &str, width: f32, builder: F) -> Self
    where
        F: Fn(&D) -> V + Send + Sync + 'static,
        V: View + 'static,
    {
        self.columns.push(TableColumn {
            header: header.to_string(),
            width,
            cell_builder: Box::new(move |d| builder(d).erase()),
        });
        self
    }
}

impl<D> View for VirtualTable<D>
where
    D: Send + Sync + 'static,
{
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Vertical virtualization
        for (idx, item) in self.data.iter().enumerate() {
            let row_y = idx as f32 * self.row_height;
            
            if row_y + self.row_height < 0.0 || row_y > rect.height {
                continue;
            }
            
            let mut current_x = rect.x;
            for col in &self.columns {
                let cell_rect = Rect {
                    x: current_x,
                    y: rect.y + row_y,
                    width: col.width,
                    height: self.row_height,
                };
                
                let view = (col.cell_builder)(item);
                view.render(renderer, cell_rect);
                current_x += col.width;
            }
        }
    }

    fn layout(&self) -> Option<&dyn LayoutView> {
        Some(self)
    }
}

impl<D> LayoutView for VirtualTable<D>
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
        let height = self.data.len() as f32 * self.row_height;
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
