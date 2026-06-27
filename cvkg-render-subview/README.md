# cvkg-render-subview

Stub crate for the iOS subrenderer mode.

## Target API

This crate will provide a render mode where CVKG draws into a caller-provided
`wgpu::Surface` instead of owning the window. This enables embedding CVKG inside
a SwiftUI/Metal host app on iOS.

## Status

**Not yet implemented.** This is a placeholder crate created during the UI/UX
Audit 4 implementation plan (Task 1.5). The actual implementation requires:

1. Extracting the render-to-texture path from `cvkg-render-gpu`
2. Creating a `SubviewRenderer` that accepts an external `wgpu::Surface`
3. Adding iOS-specific surface creation (CAMetalLayer)
4. Adding touch input translation

See `docs/platform-ios-feasibility.md` for the full feasibility analysis.
