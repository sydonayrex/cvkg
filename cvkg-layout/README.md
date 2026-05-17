# cvkg-layout

![CVKG Component Showcase](../docs/images/cvkg_component_showcase.png)

`cvkg-layout` provides a high-performance, flexbox-inspired layout engine for CVKG, supporting complex hierarchical positioning with minimal overhead.

## Boundaries and Responsibilities

This crate handles the spatial arrangement of views. It does NOT perform drawing operations or manage state. Its responsibilities include:
- Implementing horizontal (`HStack`), vertical (`VStack`), and depth (`ZStack`) stacking.
- Calculating grid-based arrangements.
- Managing flexible space distribution via flex weights.
- Providing a cached layout pass to avoid redundant calculations.

## Public API Overview

### Layout Containers
- `HStack`: Distributes children horizontally with configurable spacing, alignment, and distribution.
- `VStack`: Distributes children vertically.
- `ZStack`: Overlays children on top of each other.
- `Grid`: Arranges children in a fixed-cell 2D grid.

### Layout Logic
- `LayoutView`: The trait implemented by layout containers and views to participate in the layout pass.
- `SizeProposal`: Communicates available space and constraints to child views.
- `LayoutCache`: Stores computed sizes to optimize multi-pass layout algorithms.

### Support Types
- `Alignment`: Controls positioning perpendicular to the stack axis (Top, Bottom, Leading, Trailing, Center).
- `Distribution`: Controls spacing along the stack axis (Leading, Trailing, Center, Fill, SpaceBetween, SpaceAround).

## Usage Example

```rust
use cvkg_layout::{HStack, VStack, Spacer};
use cvkg_core::{Alignment, Distribution};

// Create a vertical stack with horizontal leading alignment
let layout = VStack::new(10.0, Alignment::Leading, Distribution::Leading);

// In a real View, this is handled via the layout() method
// and placement happens during the render pass.
```

## Known Limitations
- Circular dependencies in layout (e.g., a parent sized by children that are sized by the parent) are resolved via the `SizeProposal` mechanism but may require explicit frame constraints.
- Grid layout currently assumes fixed-size cells.