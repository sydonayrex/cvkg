// CVKG Component Feature Showcase
// Demonstrates all major component types and their usage patterns

use cvkg_components::{
    VStack, HStack,
    Button, Toggle, Checkbox,
    Text,
    ValkyrieIndicator,
    NiflheimFrost, Seiðr, LokiGlitch,
};
use cvkg_core::View;

fn showcase_view() -> impl View {
    VStack::new(16.0)
        .child(Text::new("CVKG Component Showcase").font_size(32.0))
        .child(
            HStack::new(10.0)
                .child(Button::new("Primary Action", || println!("Clicked!")))
                .child(Button::new("Secondary", || {}))
        )
        .child(
            VStack::new(8.0)
                .child(Toggle::new("Enable Bifrost", true, |_| {}))
                .child(Checkbox::new(false, |_| {}).label("Accept Terms"))
        )
        .child(
            NiflheimFrost::new(
                Text::new("Frosted Glass Component")
            )
            .blur_radius(10.0)
            .clean()
        )
        .child(Seiðr::default())
        .child(LokiGlitch::new("Digital Distortion"))
        .child(ValkyrieIndicator::new(40.0))
}

fn main() {
    println!("Running Component Feature Showcase...");
    // In a real app, we would pass showcase_view() to a renderer
    let _ = showcase_view();
}