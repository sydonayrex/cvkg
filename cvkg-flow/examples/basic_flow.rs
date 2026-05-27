use cvkg_flow::port::*;
use cvkg_flow::types::*;
use cvkg_flow::*;

/// This example demonstrates how to create a basic graph with two nodes and a connection.
pub fn create_basic_flow() -> FlowGraph {
    let mut graph = FlowGraph::new();

    // Create Source Node
    let mut node1 = FlowNode::new(NodeId(1), "Source Agent", (100.0, 100.0));
    node1.add_port(FlowPort::new(
        PortId(101),
        NodeId(1),
        PortPosition::Right,
        PortDirection::Output,
    ));
    graph.add_node(node1);

    // Create Target Node
    let mut node2 = FlowNode::new(NodeId(2), "Target Processor", (400.0, 150.0));
    node2.add_port(FlowPort::new(
        PortId(201),
        NodeId(2),
        PortPosition::Left,
        PortDirection::Input,
    ));
    graph.add_node(node2);

    // Create Edge connecting the ports
    graph.add_edge(FlowEdge::new(301, NodeId(1), 0, NodeId(2), 0));

    graph
}

fn main() {
    let graph = create_basic_flow();
    println!(
        "Created graph with {} nodes and {} edges",
        graph.nodes.len(),
        graph.edges.len()
    );
}
