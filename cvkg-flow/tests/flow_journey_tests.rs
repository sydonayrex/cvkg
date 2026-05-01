use cvkg_flow::graph::FlowGraph;
use cvkg_flow::node::FlowNode;
use cvkg_flow::edge::FlowEdge;
use cvkg_flow::port::FlowPort;
use cvkg_flow::types::{NodeId, PortId, EdgeId, PortPosition, PortDirection};
use cvkg_core::{Rect, Event};

#[test]
fn journey_node_connection_flow() {
    let mut graph = FlowGraph::new();
    
    let mut n1 = FlowNode::new(NodeId(1), "Source", (10.0, 10.0));
    n1.add_port(FlowPort::new(PortId(10), NodeId(1), PortPosition::Right, PortDirection::Output));
    graph.add_node(n1);
    
    let mut n2 = FlowNode::new(NodeId(2), "Target", (200.0, 100.0));
    n2.add_port(FlowPort::new(PortId(20), NodeId(2), PortPosition::Left, PortDirection::Input));
    graph.add_node(n2);
    
    graph.add_edge(FlowEdge::new(EdgeId(100), PortId(10), PortId(20)));
    
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
    
    let edge = graph.edges.get(&EdgeId(100)).unwrap();
    assert_eq!(edge.source, PortId(10));
    assert_eq!(edge.target, PortId(20));
}
