// =============================================================================
// P2-26: Heatmap LOD System
// =============================================================================

/// Level of detail for heatmap aggregation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeatmapLod {
    Full,
    Medium,
    Low,
    Minimal,
}

impl HeatmapLod {
    pub fn from_data_count(count: usize) -> Self {
        match count {
            0..=1000 => HeatmapLod::Full,
            1001..=10000 => HeatmapLod::Medium,
            10001..=100000 => HeatmapLod::Low,
            _ => HeatmapLod::Minimal,
        }
    }

    pub fn cell_size(&self) -> u32 {
        match self {
            HeatmapLod::Full => 1,
            HeatmapLod::Medium => 2,
            HeatmapLod::Low => 4,
            HeatmapLod::Minimal => 8,
        }
    }

    pub fn max_points(&self) -> usize {
        match self {
            HeatmapLod::Full => 1000,
            HeatmapLod::Medium => 10000,
            HeatmapLod::Low => 100000,
            HeatmapLod::Minimal => usize::MAX,
        }
    }
}

/// Progressive aggregation state for streaming heatmap updates.
#[derive(Clone, Debug)]
pub struct HeatmapAggregation {
    pub lod: HeatmapLod,
    pub cells: Vec<f32>,
    pub grid_width: u32,
    pub grid_height: u32,
}

impl HeatmapAggregation {
    pub fn new(width: u32, height: u32, lod: HeatmapLod) -> Self {
        let cell_size = lod.cell_size();
        let grid_width = width.div_ceil(cell_size);
        let grid_height = height.div_ceil(cell_size);
        Self {
            lod,
            cells: vec![0.0; (grid_width * grid_height) as usize],
            grid_width,
            grid_height,
        }
    }

    pub fn add_point(&mut self, x: f32, y: f32, value: f32) {
        let cell_size = self.lod.cell_size() as f32;
        let cx = (x / cell_size) as u32;
        let cy = (y / cell_size) as u32;
        if cx < self.grid_width && cy < self.grid_height {
            let idx = (cy * self.grid_width + cx) as usize;
            self.cells[idx] += value;
        }
    }

    pub fn downsample(&self) -> Self {
        let new_lod = match self.lod {
            HeatmapLod::Full => HeatmapLod::Medium,
            HeatmapLod::Medium => HeatmapLod::Low,
            HeatmapLod::Low => HeatmapLod::Minimal,
            HeatmapLod::Minimal => HeatmapLod::Minimal,
        };
        let mut result = Self::new(
            self.grid_width * self.lod.cell_size(),
            self.grid_height * self.lod.cell_size(),
            new_lod,
        );
        for (i, &val) in self.cells.iter().enumerate() {
            result.cells[i / 4] += val;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heatmap_lod_from_data_count() {
        assert_eq!(HeatmapLod::from_data_count(100), HeatmapLod::Full);
        assert_eq!(HeatmapLod::from_data_count(5000), HeatmapLod::Medium);
    }

    #[test]
    fn heatmap_aggregation_add_point() {
        let mut agg = HeatmapAggregation::new(100, 100, HeatmapLod::Full);
        agg.add_point(50.0, 50.0, 1.0);
        assert!(agg.cells.iter().any(|&v| v > 0.0));
    }
}
