# cvkg-render-native

**cvkg-render-native** provides OS-level integration for CVKG applications on Desktop (Linux, Windows, macOS).

## 🚀 Quick Start

```rust
use cvkg_render_native::run_app;
use cvkg_components::{Text, VStack, Button};
use cvkg_core::View;

fn main() {
    run_app(