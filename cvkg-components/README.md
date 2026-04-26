# cvkg-components

**cvkg-components** is the high-level UI library for CVKG, providing a suite of interactive and layout components.

## Library Structure

### Layout Containers
*   `HStack` / `VStack`: Linear layout containers.
*   `ZStack`: Overlapping layout container.
*   `List`: Efficient vertical scrollable container.
*   `Scrollable`: Arbitrary content scrolling.
*   `Spacer`: Adaptive layout gap.

### Interactive Components
*   `Button`: Clickable action element.
*   `Toggle`: Boolean state switch.
*   `Slider`: Linear value selector.
*   `TextField`: Single-line text input with full cursor and IME support.
*   `SecureField`: Password/Sensitive input.
*   `Picker` / `Dropdown`: Selection from a list of options with glassmorphic overlays.

### Visual Elements
*   `Text`: High-fidelity typography (Markdown-like spans supported).
*   `Image`: GPU-accelerated image rendering.
*   `Shape`: Primitives (RoundedRect, Circle, etc.) with Berserker styling.
*   `ProgressRing`: Radial progress indicator with emissive glow.
*   `StatusBar`: Global status monitoring for mission-critical telemetry.
*   `TelemetryView`: Real-time HUD for GPU performance (FPS, Draw Calls).

### Modals & Overlays
*   `.sheet()`: Fluent modifier for creating glassmorphic modal windows.
*   `.popover()`: Contextual tooltips and menus.
