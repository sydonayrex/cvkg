use cvkg_flow::*;

#[test]
fn test_complex_workflow_interaction() {
    let mut container = FlowContainer::default();
    
    // 1. Initial State
    let n1_id = NodeId(1);
    let p1_id = PortId(101);
    let mut n1 = FlowNode::new(n1_id, "Source", (0.0, 0.0));
    n1.add_port(FlowPort::new(p1_id, n1_id, PortPosition::Right, PortDirection::Output));
    container.graph.add_node(n1);
    
    // 2. Push history BEFORE adding n2
    container.push_history(); // history = [graph_with_n1]
    
    let n2_id = NodeId(2);
    let p2_id = PortId(201);
    let mut n2 = FlowNode::new(n2_id, "Target", (200.0, 200.0));
    n2.add_port(FlowPort::new(p2_id, n2_id, PortPosition::Left, PortDirection::Input));
    container.graph.add_node(n2);
    
    // 3. Push history BEFORE adding edge
    container.push_history(); // history = [graph_with_n1, graph_with_n1_n2]
    
    let e_id = EdgeId(301);
    container.graph.add_edge(FlowEdge::new(e_id, p1_id, p2_id));
    
    assert_eq!(container.graph.edges.len(), 1);
    
    // 4. Push history BEFORE movement
    container.push_history(); // history = [..., graph_with_edge]
    
    container.graph.nodes.get_mut(&n1_id).unwrap().position = (50.0, 50.0);
    
    assert_eq!(container.graph.nodes.get(&n1_id).unwrap().position, (50.0, 50.0));
    
    // 5. Undo movement
    container.undo(); // graph becomes graph_with_edge
    assert_eq!(container.graph.nodes.get(&n1_id).unwrap().position, (0.0, 0.0));
    assert_eq!(container.graph.edges.len(), 1);
    
    // 6. Undo edge
    container.undo(); // graph becomes graph_with_n1_n2
    assert_eq!(container.graph.edges.len(), 0);
    assert_eq!(container.graph.nodes.len(), 2);
    
    // 7. Redo edge
    container.redo();
    assert_eq!(container.graph.edges.len(), 1);
}
