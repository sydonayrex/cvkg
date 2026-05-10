// CVKG Vanir Tools Example
// Demonstrates Phase 4 components: Freyr Inspector, Njord Theme, Skadi Scripting, Idunn Persistence
//
// Run with: cargo run --example vanir_tools_example

use cvkg_components::{
    FreyrInspector, PropertyValue,
    NjordTheme,
    SkadiScripting, ScriptNodeType,
    IdunnPersistence,
};
use std::collections::HashMap;

fn main() {
    println!("CVKG Vanir Tools Example");
    println!("========================\n");

    // Freyr Inspector - Property editing
    let freyr = FreyrInspector::new("Component Properties")
        .text_prop("name", "MyComponent", "The component name")
        .number_prop("opacity", 0.85, "Transparency level")
        .bool_prop("enabled", true, "Whether component is active");

    println!("Freyr Inspector: {} properties", freyr.properties.len());

    // Njord Theme - Theme engine
    let mut colors = HashMap::new();
    colors.insert("primary".to_string(), [0.0, 0.8, 1.0, 1.0]);
    colors.insert("secondary".to_string(), [0.8, 0.4, 0.9, 1.0]);

    let njord = NjordTheme::new()
        .token("spacing-md", "16px")
        .token("font-size-lg", "18px")
        .token("border-radius", "8px")
        .variant("dark", colors)
        .active("dark");

    println!("Njord Theme: {} tokens, {} variants", njord.tokens.len(), njord.variants.len());

    // Skadi Scripting - Visual scripting
    let skadi = SkadiScripting::new()
        .node(1, "Input", ScriptNodeType::Input, 100.0, 200.0)
        .node(2, "Process", ScriptNodeType::Process, 300.0, 200.0)
        .node(3, "Output", ScriptNodeType::Output, 500.0, 200.0)
        .connect(1, 2, "data")
        .connect(2, 3, "result")
        .state("running");

    println!("Skadi Scripting: {} nodes, {} connections", skadi.nodes.len(), skadi.connections.len());

    // Idunn Persistence - Workspace persistence
    let idunn = IdunnPersistence::new()
        .snapshot("snap_1", "Layout A", "{\"panels\": 3}")
        .snapshot("snap_2", "Layout B", "{\"panels\": 1}")
        .session("session_abc", "workspace_main")
        .auto_restore(true);

    println!("Idunn Persistence: {} snapshots", idunn.snapshots.len());

    println!("\n=== Vanir Tools Components Created Successfully ===");
}