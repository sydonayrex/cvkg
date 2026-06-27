use serde::{Deserialize, Serialize};

/// System appearance (Light/Dark mode)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Appearance {
    Light,
    Dark,
}
/// Orientation for layouts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orientation {
    Horizontal,
    Vertical,
}
/// Placement configuration for placing a view within a Grid layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridPlacement {
    /// 0-based column index. Negative values count from the end of columns.
    pub column: i32,
    /// Number of columns the view spans (default is 1).
    pub column_span: u32,
    /// 0-based row index. Negative values count from the end of rows.
    pub row: i32,
    /// Number of rows the view spans (default is 1).
    pub row_span: u32,
}
/// Cross-axis alignment for layout containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Alignment {
    #[default]
    Center,
    Leading,
    Trailing,
    Top,
    Bottom,
}
/// Main-axis distribution for linear layout containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Distribution {
    #[default]
    Fill,
    Center,
    Leading,
    Trailing,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}
