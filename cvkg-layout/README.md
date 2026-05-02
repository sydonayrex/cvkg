# cvkg-layout

**cvkg-layout** is the geometric heart of CVKG, implementing a flexbox-inspired layout engine.

## 🚀 Quick Start

```rust
use cvkg_layout::{HStack, VStack, Alignment};
use cvkg_components::Text;
use cvkg_core::View;

// Create layouts
let horizontal = HStack::new(16.0)
    .alignment(Alignment::Center)
    .child(Text::new(