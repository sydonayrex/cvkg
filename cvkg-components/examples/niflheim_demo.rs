// Niflheim Demo - CVKG 0.1.16
// Demonstrates Liquid Glass capabilities via NiflheimFrost with clean mode

use cvkg_components::{Text, Button, Toggle, NiflheimFrost};

fn main() {
    println!("Niflheim Demo - CVKG Components 0.1.16");
    
    // Demo the components including Liquid Glass (via NiflheimFrost.clean())
    let _text = Text::new("Welcome to Niflheim");
    let _button = Button::new("Click Me", || println!("Button clicked!"));
    let _toggle = Toggle::new("Enable Feature", false, |val| println!("Toggled: {}", val));
    
    // Liquid Glass demonstration (2026 UX Trend) using NiflheimFrost with clean mode
    let _glass_panel = NiflheimFrost::new(Text::new("Liquid Glass Panel"))
        .clean()  // Disable frost particles for clean glass aesthetic
        .blur_radius(16.0)
        .morph_progress(0.5)  // Animate corner radius
        .corner_radii(8.0, 16.0)
        .edge_color([0.0, 1.0, 1.0, 0.8]); // Neon cyan edge
    
    println!("{}", niflheim_demo());
}

fn niflheim_demo() -> &'static str {
    "Niflheim demo complete!"
}
