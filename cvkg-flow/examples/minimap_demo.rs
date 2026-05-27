use cvkg_core::{Rect, Renderer};
use cvkg_flow::types::*;
use cvkg_flow::*;

/// This example shows how to render a FlowCanvas which automatically includes the mini-map.
pub fn render_flow_with_minimap(_renderer: &mut dyn Renderer, _canvas_rect: Rect) {
    let mut graph = FlowGraph::new();

    // Create a large sparse graph to show off the minimap
    for i in 0..10 {
        let x = (i as f32) * 300.0;
        let y = (i as f32) * 200.0;
        let node = FlowNode::new(NodeId(i), format!("Node {}", i), (x, y));
        graph.add_node(node);
    }

    let mut _canvas = FlowCanvas::new();
    _canvas.graph = graph;

    // The render call would handle grid, nodes, edges, and the minimap overlay
    // canvas.render(renderer, canvas_rect);
}

fn main() {
    println!("Minimap demo code snippet provided. Render this in a CVKG context.");
}
