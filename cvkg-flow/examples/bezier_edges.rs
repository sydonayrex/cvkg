use cvkg_flow::*;

/// This example demonstrates different edge path types, specifically Bezier curves.
pub fn create_bezier_demo() -> FlowGraph {
    let mut graph = FlowGraph::new();

    // Setup nodes
    let mut n1 = FlowNode::new(NodeId(1), "Start", (100.0, 100.0));
    n1.add_port(FlowPort::new(PortId(11), NodeId(1), PortPosition::Right, PortDirection::Output));
    
    let mut n2 = FlowNode::new(NodeId(2), "Bezier End", (400.0, 50.0));
    n2.add_port(FlowPort::new(PortId(21), NodeId(2), PortPosition::Left, PortDirection::Input));

    let mut n3 = FlowNode::new(NodeId(3), "Straight End", (400.0, 200.0));
    n3.add_port(FlowPort::new(PortId(31), NodeId(3), PortPosition::Left, PortDirection::Input));

    graph.add_node(n1);
    graph.add_node(n2);
    graph.add_node(n3);

    // Bezier Edge
    let mut e1 = FlowEdge::new(EdgeId(101), PortId(11), PortId(21));
    e1.path = EdgePath::Bezier;
    graph.add_edge(e1);

    // Straight Edge
    let mut e2 = FlowEdge::new(EdgeId(102), PortId(11), PortId(31));
    e2.path = EdgePath::Straight;
    graph.add_edge(e2);

    graph
}

fn main() {
    let _graph = create_bezier_demo();
    println!("Bezier demo graph created.");
}
