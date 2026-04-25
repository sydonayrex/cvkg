# Surtr: High-Performance GPU Renderer

**Surtr** (`cvkg-render-gpu`) is the primary high-fidelity rendering backend for CVKG. It uses WGPU to provide hardware-accelerated drawing with support for complex multi-pass effects.

## The Niflheim Shader Suite

Surtr implements the **Niflheim** shader suite, which provides:
- **Mist**: Multi-pass Gaussian blur for glassmorphism.
- **Glow**: Additive blending for neon bloom effects.
- **Void**: The Ginnungagap true-black background.

## Muspelheim Pipeline

The Muspelheim pipeline is a 4-pass rendering engine designed for "Neon Bloom":
1. **Pass 1 (Extract)**: Isolate bright areas of the scene.
2. **Pass 2 (Horizontal Blur)**: First Gaussian pass.
3. **Pass 3 (Vertical Blur)**: Second Gaussian pass.
4. **Pass 4 (Composite)**: Add the blurred glow back onto the main scene.

## ShieldWall

Surtr integrates with **ShieldWall** (AccessKit) to provide native OS-level accessibility trees, ensuring that even high-performance GPU interfaces are fully accessible.
