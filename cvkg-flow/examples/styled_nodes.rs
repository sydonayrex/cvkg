use cvkg_flow::*;

/// This example shows different node types and styling options.
pub fn create_styled_nodes() -> FlowGraph {
    let mut graph = FlowGraph::new();

    // Input Node (Cyber Neon Green)
    let mut input = FlowNode::new(NodeId(1), "Data Input", (50.0, 50.0));
    input.node_type = NodeType::Input;
    input.add_port(FlowPort::new(PortId(11), NodeId(1), PortPosition::Bottom, PortDirection::Output));
    graph.add_node(input);

    // Group Node (Larger area)
    let mut group = FlowNode::new(NodeId(2), "Processing Cluster", (250.0, 50.0));
    group.size = (300.0, 200.0);
    group.node_type = NodeType::Group;
    graph.add_node(group);

    // Annotation Node (No ports)
    let mut note = FlowNode::new(NodeId(3), "Note: Optimize this path", (50.0, 200.0));
    note.node_type = NodeType::Annotation;
    graph.add_node(note);

    graph
}

fn main() {
    let _graph = create_styled_nodes();
    println!("Styled nodes created.");
}
