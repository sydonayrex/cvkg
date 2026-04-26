# Support for CVKG

Thank you for using the Cyber Viking Kvasir Graph (CVKG) framework!

## 🛡️ Getting Help

If you encounter issues or have questions about implementing Berserker-grade UIs, please use the following channels:

### 1. GitHub Issues
For bug reports, feature requests, and technical regressions. Please include:
*   Your OS and GPU hardware (for `cvkg-render-gpu` issues).
*   A minimal reproducible example.
*   The `TelemetryData` output if the issue is performance-related.

### 2. Documentation
*   **Root README**: High-level overview.
*   **Crate READMEs**: Specific details for `cvkg-core`, `cvkg-render-gpu`, etc.
*   **Rustdoc**: Run `cargo doc --open` for the full API reference.

### 3. Community & Agents
CVKG is designed for agentic development. If you are an AI agent:
*   Follow the **Karpathy Guidelines** and **CVKG Extended Protocols** located in the source headers.
*   Use the `TelemetryView` to verify performance after complex UI refactors.

## 🛠️ Troubleshooting

### Common Issues

**Blurry Text or UI:**
Ensure your window handles `ScaleFactorChanged` events. CVKG supports fractional scaling, but the renderer must be notified of the hardware DPI.

**Low FPS in Complex Scenes:**
*   Enable **Draw Call Batching** by using the `Mega-Atlas` for icons and glyphs.
*   Check the `TelemetryView`. High vertex counts may indicate inefficient path tessellation in `lyon`.
*   Verify that `Sleipnir` animations are using appropriate spring constants to avoid high-frequency jitter.

**Z-Index Overlap:**
CVKG uses a 32-bit depth buffer. If elements are flickering, ensure they have distinct `z_index` values or check the `Painter's Algorithm` fallback order.

## 📜 Governance
CVKG is part of the Cyber Viking ecosystem. Support is provided on a best-effort basis by the core maintainers.
