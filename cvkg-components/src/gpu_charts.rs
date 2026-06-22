//! GPU-accelerated chart components for CVKG.

pub mod bar;
pub mod line;
pub mod scatter;
pub mod radar;
pub mod pie;
pub mod heatmap;
pub mod candlestick;
pub mod funnel;
pub mod sankey;
pub mod gauge;
pub mod treemap;
pub mod range;

pub use bar::{BarChart, Histogram};
pub use line::{LineChart, SparkLineChart};
pub use scatter::ScatterPlot;
pub use radar::RadarChart;
pub use pie::PieChart;
pub use heatmap::HeatmapChart;
pub use candlestick::{CandlestickChart, Candle};
pub use funnel::FunnelChart;
pub use sankey::SankeyChart;
pub use gauge::GaugeChart;
pub use treemap::{TreemapChart, TreemapNode};
pub use range::RangeChart;
