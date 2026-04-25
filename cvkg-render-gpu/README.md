# cvkg-render-gpu (Surtr)

**Surtr** is the high-performance GPU rendering backend for CVKG, built on top of WGPU.

## Features

- **Muspelheim Pipeline**: 4-pass ping-pong Gaussian blur for cinematic neon bloom.
- **Niflheim Shaders**: Multi-layered shaders for Mist, Glow, and the Ginnungagap Void.
- **Vertex Generation**: Efficient conversion of high-level view structures into GPU buffers.
- **ShieldWall Integration**: Full support for AccessKit accessibility trees.

## Design Note

Surtr is designed for "High-Fidelity Tactical Interfaces". It prioritizes visual density and shader complexity, making it ideal for dashboard and control-system applications.
