# Web Architecture: High-Fidelity in the Browser

**cvkg-render-web** brings the CVKG aesthetic to the web using a dual-mode rendering strategy.

## Dual-Mode Rendering
1. **WebGPU Path**: For modern browsers, we use a specialized WebGPU backend that mirror the **Surtr** GPU pipeline, ensuring parity in bloom and glow effects.
2. **WebGL2 Fallback**: For older environments, we provide a robust WebGL2 fallback that maintains the core aesthetic through optimized shader translations.

## Virtual DOM (vDOM)
The web backend is powered by **cvkg-vdom**, a lightweight, high-performance virtual DOM designed specifically for reactive UI state. It supports:
- **Structural Diffing**: Only update what changed in the scene graph.
- **Fast Patching**: Native JS bridges for rapid DOM manipulation.
- **Inspector Support**: Real-time inspection via the `cvkg-webkit-server` bridge.
