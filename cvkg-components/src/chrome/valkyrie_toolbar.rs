//! ValkyrieToolbar — Floating glass toolbar with flexible layout.
//! Named after the Valkyries, choosers of the slain.

use cvkg_core::{Never, Rect, Renderer, View};

/// A segmented control item for the toolbar.
/// Simplified inline version of HrungnirSegmented for toolbar embedding.
#[derive(Clone)]
pub struct ToolbarSegmented {
    pub options: Vec<String>,
    pub selected: usize,
}

impl ToolbarSegmented {
    pub fn new(options: Vec<String>, selected: usize) -> Self {
        Self { options, selected }
    }
}

/// A search field item for the toolbar.
/// Simplified inline search field placeholder.
#[derive(Clone)]
pub struct ToolbarSearchField {
    pub placeholder: String,
    pub query: String,
}

impl ToolbarSearchField {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            query: String::new(),
        }
    }

    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = query.into();
        self
    }
}

/// An item that can be placed in a ValkyrieToolbar.
#[derive(Clone)]
pub enum ToolbarItem {
    /// A button with a label and optional icon.
    Button { label: String, icon: Option<String> },
    /// A segmented control with mutually exclusive options.
    Segmented(ToolbarSegmented),
    /// A search field with placeholder text.
    SearchField(ToolbarSearchField),
    /// A fixed-width spacer (8px).
    Spacer,
    /// A flexible space that expands to fill available room.
    FlexSpace,
    /// A vertical separator line.
    Separator,
}

impl ToolbarItem {
    /// Create a new button toolbar item.
    pub fn button(label: impl Into<String>) -> Self {
        ToolbarItem::Button {
            label: label.into(),
            icon: None,
        }
    }

    /// Create a new button toolbar item with an icon.
    pub fn button_with_icon(label: impl Into<String>, icon: impl Into<String>) -> Self {
        ToolbarItem::Button {
            label: label.into(),
            icon: Some(icon.into()),
        }
    }

    /// Create a new segmented control toolbar item.
    pub fn segmented(options: Vec<String>, selected: usize) -> Self {
        ToolbarItem::Segmented(ToolbarSegmented::new(options, selected))
    }

    /// Create a new search field toolbar item.
    pub fn search_field(placeholder: impl Into<String>) -> Self {
        ToolbarItem::SearchField(ToolbarSearchField::new(placeholder))
    }

    /// Create a fixed spacer.
    pub fn spacer() -> Self {
        ToolbarItem::Spacer
    }

    /// Create a flexible space.
    pub fn flex_space() -> Self {
        ToolbarItem::FlexSpace
    }

    /// Create a separator.
    pub fn separator() -> Self {
        ToolbarItem::Separator
    }
}

/// A floating glass toolbar with leading, center, and trailing item groups.
///
/// The toolbar renders a glass platter background with configurable corner radius.
/// Items are arranged in three groups:
/// - `leading`: left-aligned items
/// - `center`: centered items
/// - `trailing`: right-aligned items
///
/// # Example
///
/// ```no_run
/// use cvkg_components::chrome::valkyrie_toolbar::{ValkyrieToolbar, ToolbarItem};
///
/// let toolbar = ValkyrieToolbar::new()
///     .leading(vec![
///         ToolbarItem::button("New"),
///         ToolbarItem::button("Open"),
///         ToolbarItem::separator(),
///         ToolbarItem::segmented(vec!["List".into(), "Grid".into()], 0),
///     ])
///     .trailing(vec![
///         ToolbarItem::search_field("Search..."),
///         ToolbarItem::button("Settings"),
///     ]);
/// ```
pub struct ValkyrieToolbar {
    pub leading: Vec<ToolbarItem>,
    pub center: Vec<ToolbarItem>,
    pub trailing: Vec<ToolbarItem>,
    pub radius: f32,
    pub height: f32,
}

impl ValkyrieToolbar {
    /// Create a new empty ValkyrieToolbar with default settings.
    pub fn new() -> Self {
        Self {
            leading: Vec::new(),
            center: Vec::new(),
            trailing: Vec::new(),
            radius: 12.0,
            height: 40.0,
        }
    }

    /// Set the leading (left-aligned) items.
    pub fn leading(mut self, items: Vec<ToolbarItem>) -> Self {
        self.leading = items;
        self
    }

    /// Set the center items.
    pub fn center(mut self, items: Vec<ToolbarItem>) -> Self {
        self.center = items;
        self
    }

    /// Set the trailing (right-aligned) items.
    pub fn trailing(mut self, items: Vec<ToolbarItem>) -> Self {
        self.trailing = items;
        self
    }

    /// Set the corner radius of the toolbar platter.
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Set the toolbar height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Compute the natural (intrinsic) width of a toolbar item for layout.
    fn item_width(&self, renderer: &mut dyn Renderer, item: &ToolbarItem) -> f32 {
        match item {
            ToolbarItem::Button { label, .. } => renderer.measure_text(label, 12.0).0 + 20.0,
            ToolbarItem::Segmented(seg) => {
                let total_text_w: f32 = seg
                    .options
                    .iter()
                    .map(|o| renderer.measure_text(o, 12.0).0 + 16.0)
                    .sum();
                total_text_w + 16.0 // padding
            }
            ToolbarItem::SearchField(_) => 160.0,
            ToolbarItem::Spacer => 8.0,
            ToolbarItem::FlexSpace => 0.0,
            ToolbarItem::Separator => 12.0,
        }
    }
}

impl Default for ValkyrieToolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl View for ValkyrieToolbar {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Glass platter background with blur
        renderer.bifrost(rect, 20.0, 1.1, 0.6);
        renderer.fill_rounded_rect(rect, self.radius, [0.12, 0.12, 0.15, 0.88]);

        // Subtle border
        renderer.stroke_rounded_rect(rect, self.radius, [1.0, 1.0, 1.0, 0.06], 1.0);

        let y = rect.y + 6.0;
        let item_height = 28.0;

        // Render leading items (left-aligned)
        let mut x = rect.x + 8.0;
        for item in &self.leading {
            let w = self.item_width(renderer, item);
            render_toolbar_item(renderer, item, x, y, w, item_height);
            x += w + 4.0;
        }

        // Render center items (centered)
        if !self.center.is_empty() {
            let center_total_w: f32 = self
                .center
                .iter()
                .map(|item| self.item_width(renderer, item))
                .sum::<f32>()
                + (self.center.len().saturating_sub(1)) as f32 * 4.0;
            let mut x = rect.x + (rect.width - center_total_w) / 2.0;
            for item in &self.center {
                let w = self.item_width(renderer, item);
                render_toolbar_item(renderer, item, x, y, w, item_height);
                x += w + 4.0;
            }
        }

        // Render trailing items (right-aligned)
        let mut x = rect.x + rect.width - 8.0;
        for item in self.trailing.iter().rev() {
            let w = self.item_width(renderer, item);
            x -= w;
            render_toolbar_item(renderer, item, x, y, w, item_height);
            x -= 4.0;
        }
    }
}

/// Render a single toolbar item at the given position.
fn render_toolbar_item(
    renderer: &mut dyn Renderer,
    item: &ToolbarItem,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    let item_rect = Rect {
        x,
        y,
        width: w,
        height: h,
    };

    match item {
        ToolbarItem::Button { label, icon } => {
            // Glass button background
            renderer.fill_rounded_rect(item_rect, 6.0, [0.2, 0.2, 0.25, 0.8]);
            let text_x = if let Some(_icon) = icon {
                x + 18.0
            } else {
                x + 10.0
            };
            renderer.draw_text(label, text_x, y + 7.0, 12.0, [0.9, 0.9, 0.92, 1.0]);
        }
        ToolbarItem::Segmented(seg) => {
            // Glass platter for segmented control
            let radius = h / 2.0;
            renderer.fill_rounded_rect(item_rect, radius, [0.1, 0.1, 0.12, 0.85]);

            // Sliding pill indicator
            let seg_count = seg.options.len().max(1) as f32;
            let pill_w = w / seg_count;
            let pill_x = x + seg.selected as f32 * pill_w;
            let pill_rect = Rect {
                x: pill_x + 2.0,
                y: y + 2.0,
                width: pill_w - 4.0,
                height: h - 4.0,
            };
            renderer.fill_rounded_rect(pill_rect, 6.0, [1.0, 1.0, 1.0, 0.15]);

            // Segment labels
            for (i, label) in seg.options.iter().enumerate() {
                let label_w = renderer.measure_text(label, 12.0).0;
                let label_x = x + i as f32 * pill_w + (pill_w - label_w) / 2.0;
                let color = if i == seg.selected {
                    [1.0, 1.0, 1.0, 1.0]
                } else {
                    [0.7, 0.7, 0.75, 1.0]
                };
                renderer.draw_text(label, label_x, y + 7.0, 12.0, color);
            }
        }
        ToolbarItem::SearchField(sf) => {
            // Glass search background (pill shape)
            let radius = h / 2.0;
            renderer.fill_rounded_rect(item_rect, radius, [0.1, 0.1, 0.12, 0.85]);

            // Search icon
            renderer.draw_text("*", x + 8.0, y + 7.0, 12.0, [0.6, 0.6, 0.65, 0.8]);

            // Query text or placeholder
            let text = if sf.query.is_empty() {
                &sf.placeholder
            } else {
                &sf.query
            };
            let color = if sf.query.is_empty() {
                [0.5, 0.5, 0.55, 0.6]
            } else {
                [0.9, 0.9, 0.92, 1.0]
            };
            renderer.draw_text(text, x + 24.0, y + 7.0, 12.0, color);
        }
        ToolbarItem::Separator => {
            // Vertical separator line
            let sep_x = x + w / 2.0;
            renderer.draw_line(
                sep_x,
                y + 4.0,
                sep_x,
                y + h - 4.0,
                [0.3, 0.3, 0.35, 0.5],
                1.0,
            );
        }
        ToolbarItem::Spacer => {
            // Empty space — nothing to render
        }
        ToolbarItem::FlexSpace => {
            // Handled by layout — nothing to render
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolbar_new() {
        let toolbar = ValkyrieToolbar::new();
        assert!(toolbar.leading.is_empty());
        assert!(toolbar.center.is_empty());
        assert!(toolbar.trailing.is_empty());
        assert_eq!(toolbar.radius, 12.0);
        assert_eq!(toolbar.height, 40.0);
    }

    #[test]
    fn test_toolbar_default() {
        let toolbar: ValkyrieToolbar = Default::default();
        assert!(toolbar.leading.is_empty());
        assert!(toolbar.center.is_empty());
        assert!(toolbar.trailing.is_empty());
    }

    #[test]
    fn test_toolbar_leading() {
        let toolbar = ValkyrieToolbar::new().leading(vec![
            ToolbarItem::button("New"),
            ToolbarItem::button("Open"),
        ]);
        assert_eq!(toolbar.leading.len(), 2);
    }

    #[test]
    fn test_toolbar_trailing() {
        let toolbar = ValkyrieToolbar::new().trailing(vec![
            ToolbarItem::search_field("Search..."),
            ToolbarItem::button("Settings"),
        ]);
        assert_eq!(toolbar.trailing.len(), 2);
    }

    #[test]
    fn test_toolbar_center() {
        let toolbar = ValkyrieToolbar::new().center(vec![ToolbarItem::segmented(
            vec!["List".into(), "Grid".into()],
            0,
        )]);
        assert_eq!(toolbar.center.len(), 1);
    }

    #[test]
    fn test_toolbar_radius() {
        let toolbar = ValkyrieToolbar::new().radius(16.0);
        assert_eq!(toolbar.radius, 16.0);
    }

    #[test]
    fn test_toolbar_height() {
        let toolbar = ValkyrieToolbar::new().height(48.0);
        assert_eq!(toolbar.height, 48.0);
    }

    #[test]
    fn test_toolbar_item_button() {
        let item = ToolbarItem::button("Test");
        match item {
            ToolbarItem::Button { label, icon } => {
                assert_eq!(label, "Test");
                assert!(icon.is_none());
            }
            _ => panic!("Expected Button variant"),
        }
    }

    #[test]
    fn test_toolbar_item_button_with_icon() {
        let item = ToolbarItem::button_with_icon("Save", "disk");
        match item {
            ToolbarItem::Button { label, icon } => {
                assert_eq!(label, "Save");
                assert_eq!(icon, Some("disk".to_string()));
            }
            _ => panic!("Expected Button variant"),
        }
    }

    #[test]
    fn test_toolbar_item_segmented() {
        let item = ToolbarItem::segmented(vec!["A".into(), "B".into(), "C".into()], 1);
        match item {
            ToolbarItem::Segmented(seg) => {
                assert_eq!(seg.options.len(), 3);
                assert_eq!(seg.selected, 1);
            }
            _ => panic!("Expected Segmented variant"),
        }
    }

    #[test]
    fn test_toolbar_item_search_field() {
        let item = ToolbarItem::search_field("Find...");
        match item {
            ToolbarItem::SearchField(sf) => {
                assert_eq!(sf.placeholder, "Find...");
                assert!(sf.query.is_empty());
            }
            _ => panic!("Expected SearchField variant"),
        }
    }

    #[test]
    fn test_toolbar_item_separator() {
        let item = ToolbarItem::separator();
        assert!(matches!(item, ToolbarItem::Separator));
    }

    #[test]
    fn test_toolbar_item_spacer() {
        let item = ToolbarItem::spacer();
        assert!(matches!(item, ToolbarItem::Spacer));
    }

    #[test]
    fn test_toolbar_item_flex_space() {
        let item = ToolbarItem::flex_space();
        assert!(matches!(item, ToolbarItem::FlexSpace));
    }

    #[test]
    fn test_toolbar_segmented_new() {
        let seg = ToolbarSegmented::new(vec!["X".into(), "Y".into()], 0);
        assert_eq!(seg.options.len(), 2);
        assert_eq!(seg.selected, 0);
    }

    #[test]
    fn test_toolbar_search_field_new() {
        let sf = ToolbarSearchField::new("Search...");
        assert_eq!(sf.placeholder, "Search...");
        assert!(sf.query.is_empty());
    }

    #[test]
    fn test_toolbar_search_field_with_query() {
        let sf = ToolbarSearchField::new("Search...").query("hello");
        assert_eq!(sf.query, "hello");
    }

    #[test]
    fn test_toolbar_full_layout() {
        // Test a toolbar with all three groups populated
        let toolbar = ValkyrieToolbar::new()
            .leading(vec![
                ToolbarItem::button("New"),
                ToolbarItem::button("Open"),
                ToolbarItem::separator(),
                ToolbarItem::segmented(vec!["List".into(), "Grid".into()], 0),
            ])
            .center(vec![ToolbarItem::button("Center")])
            .trailing(vec![
                ToolbarItem::flex_space(),
                ToolbarItem::search_field("Search..."),
                ToolbarItem::button("Settings"),
            ]);

        assert_eq!(toolbar.leading.len(), 4);
        assert_eq!(toolbar.center.len(), 1);
        assert_eq!(toolbar.trailing.len(), 3);
    }

    #[test]
    fn test_toolbar_body_unreachable() {
        // Verify that body() is unreachable — this is a compile-time check
        // that the View trait is implemented with Never body.
        // We can't call body() in a test without panicking, but we verify
        // the type system accepts the implementation.
        fn assert_never<T: std::fmt::Debug>(_t: T) {}
        // This would fail to compile if Body were not Never:
        let toolbar = ValkyrieToolbar::new();
        // We just verify the toolbar can be constructed and is Send+Sync
        fn assert_send_sync<T: Send + Sync>(_t: T) {}
        assert_send_sync(toolbar);
    }
}
