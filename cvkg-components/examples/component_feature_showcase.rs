// CVKG Component Feature Showcase
// Demonstrates all major component types and their usage patterns

use cvkg_components::{
    VStack, HStack, ZStack, NavigationStack, NavigationSplitView,
    Button, TextField, Toggle, Checkbox,
    Text, Image, Spacer,
    Blur, Shadow, Opacity,
    NavigationSplit,
    Grid,
};
use cvkg_core::View;

// ============================================================================
// CONTAINER COMPONENTS
// ============================================================================
// VStack: Arranges children vertically (top to bottom)
// HStack: Arranges children horizontally (left to right)
// ZStack: Layers children on top of each other (last on top)

fn container_example() -> VStack {
    VStack::new()
        .spacing(16.0)
        .child(Text::new(