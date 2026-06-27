use cvkg_core::KvasirId;
use cvkg_flow::port::*;
use cvkg_flow::types::*;
use cvkg_flow::*;

/// This example demonstrates different edge path types, specifically Bezier curves.
pub fn create_bezier_demo() -> FlowGraph {
    let mut graph = FlowGraph::new();

    // Setup nodes
    let mut n1 = FlowNode::new(KvasirId(1), "Start", (100.0, 100.0));
    n1.add_port(FlowPort::new(
        PortId(11),
        KvasirId(1),
        PortPosition::Right,
        PortDirection::Output,
    ));

    let mut n2 = FlowNode::new(KvasirId(2), "Bezier End", (400.0, 50.0));
    n2.add_port(FlowPort::new(
        PortId(21),
        KvasirId(2),
        PortPosition::Left,
        PortDirection::Input,
    ));

    let mut n3 = FlowNode::new(KvasirId(3), "Straight End", (400.0, 200.0));
    n3.add_port(FlowPort::new(
        PortId(31),
        KvasirId(3),
        PortPosition::Left,
        PortDirection::Input,
    ));

    graph.add_node(n1);
    graph.add_node(n2);
    graph.add_node(n3);

    // Bezier Edge
    let e1 = FlowEdge::new(101, KvasirId(1), 0, KvasirId(2), 0);
    graph.add_edge(e1);

    // Straight Edge
    let e2 = FlowEdge::new(102, KvasirId(1), 0, KvasirId(3), 0);
    graph.add_edge(e2);

    graph
}

fn main() {
    let _graph = create_bezier_demo();
    println!("Bezier demo graph created.");
}
