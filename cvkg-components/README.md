# cvkg-components

**cvkg-components** provides a high-fidelity tactical UI component library for CVKG, built with a strict **Cyberpunk Viking** aesthetic and GPU-native performance.

## Design Philosophy: Advanced Norse HUD Patterns

All components are designed with "Liquid Glass" aesthetics, leveraging three core rendering patterns:
- **Mimir's Refraction**: Deep refractive lensing using `renderer.bifrost()` for simulated thickness and background distortion.
- **Loki's Shape-shifting**: Coordinated state transitions and morphing geometry logic.
- **Surtur's Reactive Materials**: Kinetic materials that react to tactical input with dynamic resonance and glows.

## Component Inventory

### Data Display (The Great Library of Runes)
- `RunesTable`: Virtualized, sortable data grid with in-cell sparklines for mission telemetry.
- `YggdrasilTree`: Hierarchical data display for command structures and file systems.
- `RunesCard`: Tactically inscribed data containers with refractive glass depth.
- `UrdrTimeline`: Chronological event sequence display (The Past).

### Forms & Input (Input Manifests)
- `EikonaForm`: Schema-driven form validation with tactical feedback.
- `ValkyrSelect`: Searchable tactical chooser / Combobox.
- `TyrCalendar`: Temporal date and range selection system.
- `BifrostColorPicker`: Multi-realm color selection bridge.
- `ValhallaRating`: Tactical quality assessment with star resonance.

### Feedback & Overlays (The Sky Realms)
- `HiminnModal`: Elevated glassmorphic dialogs with refractive lensing.
- `GjallarAlert`: High-priority tactical notifications (toasts) using the Gjallarhorn signal aesthetic.
- `RunicTooltip`: Contextual information overlays with neon vibrancy.
- `DraumaSkeleton`: Spectral shimmer placeholders for asynchronous content.
- `SagaAccordion`: Multi-layered narrative content containers with collapsible flows.
- `ValkyrieAnalytics`: Real-time tactical gauges and radar charts for combat monitoring.

### Layout & Navigation
- `GjallarSplitter`: Resizable panel splitting with Mimir's Eye handle glow.
- `HringrPagination`: Cyclic navigation for traversing data loops.
- `MimirSpotlight`: Global command palette and tactical search bar.

## Usage Example

```rust
use cvkg_components::{VStack, RunesTable, EikonaForm, HiminnModal};
use cvkg_core::View;

fn tactical_dashboard() -> impl View {
    VStack::new(20.0)
        .child(
            RunesTable::new(mission_data)
                .sortable(true)
                .on_row_click(|row| /* handle */)
        )
        .child(
            HiminnModal::new("System Override")
                .content(EikonaForm::new(override_schema))
        )
}
```

## Performance & Rendering

- **GPU-Native**: Stateless functional views optimized for `wgpu` backends.
- **Deterministic VDOM**: 60+ FPS maintained via `push_vnode`/`pop_vnode` tracking.
- **Cross-Platform**: Consistent rendering across Native (Metal/DX12/Vulkan) and Web (WebGPU).