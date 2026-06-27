use crate::theme;
use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};
use cvkg_core::{Never, Rect, Renderer, Size, View};

/// A single candlestick data point.
pub struct Candle {
    /// Open price.
    pub open: f32,
    /// High price.
    pub high: f32,
    /// Low price.
    pub low: f32,
    /// Close price.
    pub close: f32,
}

/// Financial Candlestick chart displaying stock/market price trends.
pub struct CandlestickChart {
    pub(crate) data: Vec<Candle>,
}

impl Default for CandlestickChart {
    fn default() -> Self {
        Self::new()
    }
}

impl CandlestickChart {
    /// Create a new empty CandlestickChart.
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Set candle items.
    pub fn candles(mut self, candles: Vec<Candle>) -> Self {
        self.data = candles;
        self
    }
}

impl View for CandlestickChart {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.data.is_empty() {
            return;
        }

        let padding = 30.0;
        let chart_w = rect.width - padding * 2.0;
        let chart_h = rect.height - padding * 2.0;

        let max_val = self
            .data
            .iter()
            .map(|c| c.high)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(1.0);
        let min_val = self
            .data
            .iter()
            .map(|c| c.low)
            .min_by(|a, b| a.total_cmp(b))
            .unwrap_or(0.0);
        let range = (max_val - min_val).max(0.001);

        let count = self.data.len();
        let candle_w = chart_w / count as f32;

        for (i, candle) in self.data.iter().enumerate() {
            let x = rect.x + padding + i as f32 * candle_w;
            let cx = x + candle_w * 0.5;

            let y_high =
                rect.y + rect.height - padding - ((candle.high - min_val) / range) * chart_h;
            let y_low = rect.y + rect.height - padding - ((candle.low - min_val) / range) * chart_h;
            let y_open =
                rect.y + rect.height - padding - ((candle.open - min_val) / range) * chart_h;
            let y_close =
                rect.y + rect.height - padding - ((candle.close - min_val) / range) * chart_h;

            let color = if candle.close >= candle.open {
                theme::success()
            } else {
                theme::error_color()
            };

            // High/low wick line
            renderer.draw_line(cx, y_high, cx, y_low, color, 1.0);

            // Candle body
            let body_y = y_open.min(y_close);
            let body_h = (y_open - y_close).abs().max(2.0);
            let body_w = candle_w * 0.7;

            renderer.fill_rect(
                Rect {
                    x: cx - body_w * 0.5,
                    y: body_y,
                    width: body_w,
                    height: body_h,
                },
                color,
            );
        }
    }
}

impl LayoutView for CandlestickChart {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 400.0,
            height: 200.0,
        }
    }
    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}
