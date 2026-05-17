use cvkg_flow::*;

/// This example shows how to configure the interactive settings of the FlowCanvas.
pub fn setup_interactive_flow() -> FlowCanvas {
    let graph = FlowGraph::new();
    let canvas = FlowCanvas::new("interactive_demo", graph);

    // Configure settings for the session
    // These would typically be updated via update_system_state in a real app

    // In a real application, you would initialize the system state with these settings:
    // let container = FlowContainer {
    //     graph: FlowGraph::new(),
    //     settings,
    //     ..Default::default()
    // };
    // system_state.set_component_state(canvas_id_hash, container);

    canvas
}

fn main() {
    println!("Interaction demo setup complete.");
}
