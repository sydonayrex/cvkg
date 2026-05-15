# ulfhednar

Primary desktop entry point and high-fidelity tactical UI orchestrator.

`ulfhednar` is the flagship binary crate of the Agent Ulfhednar project. it integrates all workspace modules into a stunning, GPU-accelerated desktop experience powered by the CVKG framework. It manages the global application state, background service loops, and multimodal neural interfaces.

## Boundaries
- **DOES**: Initialize the native application, manage the GUI state (AppState), coordinate background loops (Drauma, Scheduler), and handle desktop-specific resource paths.
- **DOES NOT**: Implement the core reasoning algorithms (see `ulf_core`) or define the underlying memory architecture (see `ulf_memory`).

## Architecture

### Main Modules
- `state`: The centralized `AppState` container using reactive signals and Arc-based service handles.
- `chat`: The primary neural interface for direct agent communication.
- `scheduler_ui`: Dashboard for managing recurring missions and history.
- `workflow_canvas`: SVG-based visualization of active task plans and reasoning flows.
- `memory_well`: Management interface for the MimirsWell persistent layers.
- `design`: Core aesthetic tokens and high-fidelity layout primitives.

### Initialization Loop (`run_native`)
1. **Bootstrap**: Configures logging and initialized the `tokio` runtime.
2. **Persistence**: Opens the `MimirsSession` and hydrates project/schedule repositories.
3. **Services**: Starts background loops for memory consolidation (Drauma) and schedule ticking.
4. **UI**: Launches the `cvkg` native renderer and enters the reactive event loop.

## Usage

### Build and Run
```bash
# Start the tactical interface on Linux/Desktop
cargo run --package ulfhednar --features native
```

### Configuration
Application behavior is primarily configured via `AgentConfig` and environment variables handled in `memory/session.rs`.

## Aesthetic Principles
- **Void Obsidian**: Deep dark backgrounds for high contrast and reduced eye strain.
- **Cyan Neon**: Primary accent color for active neural signals.
- **Bifrost Blur**: Frosted glass effects for hierarchical depth and mission focus.
- **Liquid Motion**: Smooth transitions and reactive micro-animations.

## Limitations
- **Platform Support**: Native GPU acceleration currently targets Linux and Desktop environments with Vulkan/Metal support.
- **WASM Status**: While core modules are WASM-compatible, the full desktop dashboard requires native system access for persistence and hardware acceleration.
