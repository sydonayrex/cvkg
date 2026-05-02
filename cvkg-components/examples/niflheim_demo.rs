// Niflheim Demo - CVKG 0.1.16
use cvkg_components::{VStack, Text, Button, Toggle};

fn main() {
    println!("Niflheim Demo - CVKG Components 0.1.16");
    
    // Demo the components
    let _text = Text::new("Welcome to Niflheim");
    let _button = Button::new("Click Me", || println!("Button clicked!"));
    let _toggle = Toggle::new("Enable Feature", false, |val| println!("Toggled: {}", val));
    let _vstack = VStack::new(8.0);
    
    println!("{}", niflheim_demo());
}

fn niflheim_demo() -> &'static str {
    "Niflheim demo complete!"
}