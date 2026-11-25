//! Chart data contracts.

/// A named series of numeric samples.
#[derive(Clone, Debug)]
pub struct ChartSeries {
    pub name: String,
    pub points: Vec<f32>,
}

impl ChartSeries {
    pub fn new(name: impl Into<String>, points: Vec<f32>) -> Self {
        Self {
            name: name.into(),
            points,
        }
    }
}

/// A chart made of one or more series.
#[derive(Clone, Debug)]
pub struct Chart {
    pub series: Vec<ChartSeries>,
}

impl Chart {
    pub fn new(series: Vec<ChartSeries>) -> Self {
        Self { series }
    }

    pub fn max_value(&self) -> f32 {
        self.series
            .iter()
            .flat_map(|series| series.points.iter().copied())
            .fold(0.0f32, f32::max)
    }
}
