//! GPU-accelerated chart components for CVKG.

pub mod bar;
pub mod candlestick;
pub mod funnel;
pub mod gauge;
pub mod heatmap;
pub mod line;
pub mod pie;
pub mod radar;
pub mod range;
pub mod sankey;
pub mod scatter;
pub mod treemap;

pub use bar::{BarChart, Histogram};
pub use candlestick::{Candle, CandlestickChart};
pub use funnel::FunnelChart;
pub use gauge::GaugeChart;
pub use heatmap::HeatmapChart;
pub use line::{LineChart, SparkLineChart};
pub use pie::PieChart;
pub use radar::RadarChart;
pub use range::RangeChart;
pub use sankey::SankeyChart;
pub use scatter::ScatterPlot;
pub use treemap::{TreemapChart, TreemapNode};
