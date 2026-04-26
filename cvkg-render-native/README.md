# cvkg-render-native

**cvkg-render-native** provides the OS-level integration for CVKG applications on Desktop (Linux, Windows, macOS).

## Features

*   **Window Management**: Manages `winit` window creation and lifecycle.
*   **Event Loop**: Implements the main CVKG event loop, translating native `winit` events (WindowEvent, DeviceEvent) into CVKG `Event` variants.
*   **IME Integration**: Enables and routes OS-level Input Method Editor (IME) events for multi-key character composition.
*   **Accessibility Host**: Houses the `AccessKit` adapter for screen reader integration.
*   **Renderer Bridging**: Bridges the `SurtrRenderer` (GPU) with the `VDom` to create a complete interactive application.

## Main Functionality
The crate typically provides an `App` or `Window` runner that:
1.  Creates a GPU surface.
2.  Initializes the VDOM.
3.  Runs the event loop.
4.  Rebuilds and re-renders the UI on state changes or input.
