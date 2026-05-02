# cvkg-components

**cvkg-components** is the high-level UI library for CVKG, providing a suite of interactive and layout components with a Cyberpunk Viking aesthetic.

## 📦 Library Structure

### Layout Containers

| Component | Description |
|-----------|-------------|
| `HStack` | Horizontal linear layout container |
| `VStack` | Vertical linear layout container |
| `ZStack` | Overlapping layout container |
| `List` | Efficient vertical scrollable container |
| `Scrollable` | Arbitrary content scrolling |

### Interactive Components

| Component | Description |
|-----------|-------------|
| `Button` | Clickable action element |
| `Toggle` | Boolean state switch |
| `Slider` | Linear value selector |
| `TextField` | Single-line text input with full cursor and IME support |
| `SecureField` | Password/Sensitive input |
| `Picker` / `Dropdown` | Selection from a list of options with glassmorphic overlays |

### Visual Elements

| Component | Description |
|-----------|-------------|
| `Text` | High-fidelity typography (Markdown-like spans supported) |
| `Image` | GPU-accelerated image rendering |
| `Shape` | Primitives (RoundedRect, Circle, etc.) with Berserker styling |
| `ProgressRing` | Radial progress indicator with emissive glow |
| `StatusBar` | Global status monitoring for mission-critical telemetry |
| `TelemetryView` | Real-time HUD for GPU performance (FPS, Draw Calls) |

### Display & Navigation

| Component | Description |
|-----------|-------------|
| `BifrostTabs` | Tabbed interface with frosted glass styling |
| `Skjaldborg` | Modal dialog system with cyber aesthetic |
| `Seiðr` | Wizard/stepper component |
| `Hvergelmir` | Progress/loading visualization |
| `ValkyrieIndicator` | Animated circular progress indicator |

### Game Components

| Component | Description |
|-----------|-------------|
| `MjöllnirFrame` | Animated button frame with lightning effects |
| `RunestoneEditor` | Interactive text/code editor |
| `RavenMessenger` | Chat/message interface |
| `OracleOrb` | Predictive state visualization |
| `WyrdHUD` | Runic text display system |

## 🚀 Quick Start

### Basic Usage

```rust
use cvkg_components::{Text, VStack, Button, Hvergelmir};
use cvkg_core::View;

// Create a simple UI
let app = VStack::new(16.0)
    .alignment(cvkg_core::Alignment::Center)
    .child(
        Text::new(