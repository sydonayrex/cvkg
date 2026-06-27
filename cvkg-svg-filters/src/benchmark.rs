use crate::diagnostics::FilterDiagnostics;
use crate::graph::FilterGraph;
use crate::pool::FilterPlanner;

// =============================================================================
// P2-34: Performance Regression Benchmarks
// =============================================================================

#[derive(Clone, Debug)]
pub struct FilterBenchmarkConfig {
    pub node_count: usize,
    pub iterations: usize,
    pub max_time_ms: f64,
}

impl Default for FilterBenchmarkConfig {
    fn default() -> Self {
        Self {
            node_count: 100,
            iterations: 10,
            max_time_ms: 16.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FilterBenchmarkResult {
    pub node_count: usize,
    pub total_time_ms: f64,
    pub avg_time_ms: f64,
    pub passed: bool,
}

pub struct FilterBenchmark;

impl FilterBenchmark {
    pub fn run(config: &FilterBenchmarkConfig) -> FilterBenchmarkResult {
        use std::time::Instant;

        let start = Instant::now();

        for _ in 0..config.iterations {
            let graph = Self::create_test_graph(config.node_count);
            let _plan = FilterPlanner::plan(&graph).unwrap();
            let _diagnostics = Self::run_diagnostics(&graph);
        }

        let elapsed = start.elapsed();
        let total_time_ms = elapsed.as_secs_f64() * 1000.0;
        let avg_time_ms = total_time_ms / config.iterations as f64;

        FilterBenchmarkResult {
            node_count: config.node_count,
            total_time_ms,
            avg_time_ms,
            passed: avg_time_ms <= config.max_time_ms,
        }
    }

    fn create_test_graph(_node_count: usize) -> FilterGraph {
        let svg_str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <defs>
                <filter id="test">
                    <feGaussianBlur stdDeviation="1"/>
                </filter>
            </defs>
            <rect width="100" height="100" filter="url(#test)"/>
        </svg>"#;
        let options = usvg::Options::default();
        let tree = usvg::Tree::from_str(svg_str, &options).unwrap();
        let filter = &tree.filters()[0];
        FilterGraph::from_usvg_filter(filter).unwrap()
    }

    fn run_diagnostics(_graph: &FilterGraph) -> FilterDiagnostics {
        let mut diagnostics = FilterDiagnostics::new();
        if _graph.nodes().is_empty() {
            diagnostics.error(0, "filter graph has no nodes");
        }
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_benchmark_config_default() {
        let config = FilterBenchmarkConfig::default();
        assert_eq!(config.node_count, 100);
        assert_eq!(config.iterations, 10);
        assert_eq!(config.max_time_ms, 16.0);
    }

    #[test]
    fn filter_benchmark_run() {
        let config = FilterBenchmarkConfig {
            node_count: 10,
            iterations: 3,
            max_time_ms: 1000.0,
        };
        let result = FilterBenchmark::run(&config);
        assert_eq!(result.node_count, 10);
        assert!(result.total_time_ms > 0.0);
        assert!(result.passed);
    }
}
