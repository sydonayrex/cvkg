# cvkg-layout

**cvkg-layout** is the geometric heart of CVKG, implementing a flexbox-inspired layout engine.

## Features

*   **Geometric Primitives**: Defines `Rect`, `Point`, `Size`, and `Padding` with f32 precision.
*   **Flex Engine**: Implements the core logic for `HStack` and `VStack` distribution, including alignment (Start, Center, End) and spacing.
*   **Constraint Solving**: Handles nested layout constraints to determine final view frames.
*   **Z-Order Management**: Manages stacking order for layered components.
*   **Coordinate Translation**: Provides utilities for translating coordinates between global window space and local component space.

## Key Types
*   `LayoutNode`: Represents a node in the layout tree.
*   `Geometry`: Result of a layout pass.
*   `Alignment`: Axis-specific alignment options.
