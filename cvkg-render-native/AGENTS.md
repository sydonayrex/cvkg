# cvkg-render-native AGENTS.md

## Purpose
Own the native renderer backend: platform-specific windowing, surface management, and native rendering integration (Metal, DirectX, Vulkan).

## Ownership
- `src/lib.rs` — Native renderer implementation, window lifecycle
- Platform-specific backends (macOS/Metal, Windows/DX, Linux/Vulkan)
- Safe area insets, display scaling

## Local Contracts
- Must query OS for safe area insets (Dynamic Island, notch, taskbar).
- Display scaling must be handled correctly (HiDPI/Retina).
- Must integrate with cvkg-core's Renderer trait.

## Verification
- Run `cargo check -p cvkg-render-native`
- Build on target platform to verify
