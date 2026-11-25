use cvkg_blocks::BlockGraph;
use cvkg_charts::{Chart, ChartSeries};
use cvkg_hooks::Hook;

#[test]
fn new_domain_crates_expose_minimal_contracts() {
    let mut hook = Hook::new();
    hook.subscribe(|| {});
    hook.run();

    let mut graph = BlockGraph::new();
    let first = graph.add("source");
    let second = graph.add("sink");
    graph.link(first, second);
    assert_eq!(graph.edges(), 1);

    let chart = Chart::new(vec![ChartSeries::new("fps", vec![60.0, 58.0])]);
    assert_eq!(chart.max_value(), 60.0);
}
