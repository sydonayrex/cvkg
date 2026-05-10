# cvkg-layout

**cvkg-layout** provides the flexbox-inspired geometric layout engine for CVKG. It computes positions and sizes for child views within container views.

## What This Crate Does

- Implements `HStack` for horizontal layout
- Implements `VStack` for vertical layout
- Provides `Alignment` and `Distribution` enums for positioning
- Supports flex weight distribution for proportional space allocation

## What This Crate Does NOT Do

- Does not provide UI components (see cvkg-components)
- Does not handle rendering (see cvkg-render-gpu)
- Does not maintain state

## Public API Overview

### Layout Containers

```rust
/// Horizontal stack that lays out children side-by-side
pub struct HStack {
    spacing: f32,
    alignment: Alignment,
    distribution: Distribution,
}

impl HStack {
    pub fn new(spacing: f32, alignment: Alignment, distribution: Distribution) -> Self;
    pub fn spacing(mut self, spacing: f32) -> Self;
    pub fn alignment(mut self, alignment: Alignment) -> Self;
    pub fn distribution(mut self, distribution: Distribution) -> Self;
    pub fn child<V: LayoutView>(mut self, child: V) -> Self;
}

/// Vertical stack that lays out children top-to-bottom
pub struct VStack {
    spacing: f32,
    alignment: Alignment,
    distribution: Distribution,
}

impl VStack {
    pub fn new(spacing: f32, alignment: Alignment, distribution: Distribution) -> Self;
}
```

### Layout Enums

```rust
pub enum Alignment {
    Top,      // Align to top edge
    Center,   // Center alignment
    Bottom,   // Align to bottom edge
    Leading,  // Align to leading edge (left in LTR)
    Trailing, // Align to trailing edge (right in LTR)
    Fill,     // Stretch to fill available space
}

pub enum Distribution {
    Leading,      // Pack to leading edge
    Center,       // Center the group
    Trailing,     // Pack to trailing edge
    Fill,         // Stretch children to fill
    SpaceBetween, // Evenly space, first/last at edges
}
```

### LayoutView Trait

```rust
// Implemented by layout containers
pub trait LayoutView {
    fn size_that_fits(&self, proposal: SizeProposal, subviews: &[&dyn LayoutView], cache: &mut LayoutCache) -> Size;
    fn place_subviews(&self, bounds: Rect, subviews: &mut [&mut dyn LayoutView], cache: &mut LayoutCache);
}
```

## Usage Example

```rust
use cvkg_layout::{HStack, VStack, Alignment, Distribution};
use cvkg_core::View;

// Create a horizontal stack with 16px spacing, center aligned
let horizontal = HStack::new(16.0, Alignment::Center, Distribution::Leading)
    .child(Text::new("Left"))
    .child(Text::new("Right"));

// Create a vertical stack with space-between distribution
let vertical = VStack::new(8.0, Alignment::Leading, Distribution::SpaceBetween)
    .child(Text::new("Top"))
    .child(Text::new("Bottom"));
```

## Known Limitations

- Layout calculations are performed synchronously; very deep hierarchies may impact frame rate
- Flex weight distribution does not support minimum/maximum constraints
- Percentage-based widths are not supported; use `frame()` modifier instead