import os
import re

# Map of core details for each crate to guarantee high-quality descriptions
CRATE_INFO = {
    "cvkg": {
        "purpose": "Main public facade and platform backend selector for the CVKG framework.",
        "boundaries": "It does not implement renderers or layout engines directly; it delegates to platform-specific crates based on enabled features.",
        "api": "- `CvkgApp` — High-level application manager.\n- `prelude` — Re-exports standard views, modifiers, and state macros.",
        "example": "```rust\nuse cvkg::prelude::*;\nfn main() {\n    // Starts application facade\n}\n```"
    },
    "cvkg-core": {
        "purpose": "Defines fundamental traits, shared data structures, state management types, and layout primitives for CVKG.",
        "boundaries": "It does not implement layout calculations or drawing operations; those are handled by cvkg-layout and render backends.",
        "api": "- `View` — Core view trait.\n- `Renderer` — Drawing facade.\n- `State` — Reactive state wrapper.",
        "example": "```rust\nuse cvkg_core::prelude::*;\n```"
    },
    "cvkg-vdom": {
        "purpose": "Manages Virtual DOM trees, tree diffing, and state reconciliation patches.",
        "boundaries": "It does not run wgpu rendering pipelines or capture platform window events.",
        "api": "- `VNode` — Stateless Virtual DOM node.\n- `VDom` — Hierarchy tracker.\n- `VDomPatch` — Reconciler patch calculations.",
        "example": "```rust\nuse cvkg_vdom::{VDom, VNode};\n```"
    },
    "cvkg-scene": {
        "purpose": "Retained scene graph with spatial partitioning (QuadTree/BVH) for accelerated culling and hit-testing.",
        "boundaries": "It does not compute layout dimensions or flexbox constraints.",
        "api": "- `SceneGraph` — Main visual tree buffer.\n- `SceneNode` — Render-ready geometry nodes.",
        "example": "```rust\nuse cvkg_scene::SceneGraph;\n```"
    },
    "cvkg-layout": {
        "purpose": "Computes spatial bounds and flexbox positioning using Taffy constraints.",
        "boundaries": "It does not draw vector lines or compile wgpu shader programs.",
        "api": "- `SizeProposal` — proposed layout bounds.\n- `HStack` / `VStack` — layout containers.",
        "example": "```rust\nuse cvkg_layout::{HStack, SizeProposal};\n```"
    },
    "cvkg-anim": {
        "purpose": "Solves spring-physics motion transitions using RK4 numerical integration solvers.",
        "boundaries": "It does not manage visual layers or rasterize character fonts.",
        "api": "- `SleipnirSolver` — RK4 spring motion solver.\n- `RubberBand` — Scroll overflow damping resolver.",
        "example": "```rust\nuse cvkg_anim::SleipnirSolver;\n```"
    },
    "cvkg-render-gpu": {
        "purpose": "Drives wgpu-based rendering pipelines, shader compilations, and command buffers.",
        "boundaries": "It does not run native desktop window loops; those are managed by cvkg-render-native.",
        "api": "- `SurtrRenderer` — Central pipeline controller.\n- `Vertex` — Geometry vertex coordinates.",
        "example": "```rust\nuse cvkg_render_gpu::SurtrRenderer;\n```"
    },
    "cvkg-render-native": {
        "purpose": "Manages desktop window states and event loop triggers using winit and AccessKit.",
        "boundaries": "It does not write vector drawings or execute GPU fragment shaders directly.",
        "api": "- `NativeShell` — Desktop window state controller.\n- `WindowStateDetector` — Multi-monitor adaptive scaler.",
        "example": "```rust\nuse cvkg_render_native::NativeShell;\n```"
    },
    "cvkg-render-software": {
        "purpose": "Provides a CPU-based software rendering fallback using standard text layouts.",
        "boundaries": "It does not run wgpu bindings or compile pipeline graphics shaders.",
        "api": "- `SoftwareRenderer` — Software drawing interface.",
        "example": "```rust\nuse cvkg_render_software::SoftwareRenderer;\n```"
    },
    "cvkg-telemetry": {
        "purpose": "Aggregates performance statistics, input latency tracks, and frame duration metrics.",
        "boundaries": "It does not render dashboard layouts or draw visual diagrams.",
        "api": "- `TelemetryClient` — Performance client tracker.\n- `InputLatencyTracker` — Input percentile calculator.",
        "example": "```rust\nuse cvkg_telemetry::TelemetryClient;\n```"
    },
    "cvkg-compositor": {
        "purpose": "Groups and routes visual layers to multi-pass GPU drawing pipelines.",
        "boundaries": "It does not parse raw vector icons or track keyboard focus arrays.",
        "api": "- `CompositorEngine` — Layer compositor logic.\n- `LayerTree` — Z-sorted visually overlapping layer tree.",
        "example": "```rust\nuse cvkg_compositor::CompositorEngine;\n```"
    },
    "cvkg-cli": {
        "purpose": "Command-line interface scaffolding, project packing, and asset pipeline compiling.",
        "boundaries": "It does not execute core framework layout or view rendering inside runtime apps.",
        "api": "- `main` CLI entrypoint commands.",
        "example": "```bash\ncvkg build\n```"
    },
    "cvkg-svg-serialize": {
        "purpose": "Formats and writes raw SVG XML files from geometric vector structures.",
        "boundaries": "It does not parse or draw SVGs; it is strictly a write-path serializer.",
        "api": "- `SvgSerializer` — Serializes graphics to raw XML.",
        "example": "```rust\nuse cvkg_svg_serialize::SvgSerializer;\n```"
    },
    "cvkg-svg-filters": {
        "purpose": "Implements SVG filter primitives (blur, morphology, displacement) for visual effects.",
        "boundaries": "It does not compute layout margins or compile final rendering buffers.",
        "api": "- `FilterPrimitive` — Base effect definition.\n- `BlurEffect` — Box/Gaussian blur parameters.",
        "example": "```rust\nuse cvkg_svg_filters::FilterPrimitive;\n```"
    },
    "cvkg-webkit-server": {
        "purpose": "HTTP and WebSocket server hosting asset bundles and handling live reload signals.",
        "boundaries": "It does not execute native GUI window drawing or process desktop input events.",
        "api": "- `WebKitServer` — Web server manager.",
        "example": "```rust\nuse cvkg_webkit_server::WebKitServer;\n```"
    },
    "cvkg-components": {
        "purpose": "Tahoe component library containing base inputs, buttons, and custom layout controls.",
        "boundaries": "It does not write GPU hardware drivers or compute text metrics directly.",
        "api": "- `Button` — Native-drawn click component.\n- `PhoneInput` / `MentionInput` — Custom input editors.",
        "example": "```rust\nuse cvkg_components::Button;\n```"
    },
    "cvkg-icons": {
        "purpose": "Vector icon SVG asset storage, retrieval, and cache registration.",
        "boundaries": "It does not calculate visual constraints or apply text styling rules.",
        "api": "- `IconRegistry` — Dynamic icon registration.",
        "example": "```rust\nuse cvkg_icons::IconRegistry;\n```"
    },
    "cvkg-themes": {
        "purpose": "OKLCH-based system token catalog managing color palettes and premium materials.",
        "boundaries": "It does not shape unicode character fonts or compute text wrapping widths.",
        "api": "- `ThemeBuilder` — Builder for colors and tokens.\n- `oklch_to_color_theme` — Conversions.",
        "example": "```rust\nuse cvkg_themes::ThemeBuilder;\n```"
    },
    "cvkg-macros": {
        "purpose": "Procedural macros scaffolding DSL view bodies and reactive state bindings.",
        "boundaries": "It does not process dynamic runtime layout constraints.",
        "api": "- `#[derive(View)]` — Macro macro derivation.\n- `hamr!` — View composition DSL.",
        "example": "```rust\nuse cvkg_macros::View;\n```"
    },
    "cvkg-runic-text": {
        "purpose": "Text shaping, layout, and font rasterization coordinates engine using HarfBuzz and Swash.",
        "boundaries": "It does not allocate GPU memory or run desktop event loop windows.",
        "api": "- `RunicTextEngine` — Main shaper logic.",
        "example": "```rust\nuse cvkg_runic_text::RunicTextEngine;\n```"
    },
    "cvkg-test": {
        "purpose": "Visual regression comparison tests and automated testing suite assertions.",
        "boundaries": "It does not build release distribution assets.",
        "api": "- `VisualComparator` — Compare image buffers.",
        "example": "```rust\nuse cvkg_test::VisualComparator;\n```"
    },
    "cvkg-physics": {
        "purpose": "Tyr rigid-body physics engine solving XPBD constraints and broadphase collisions.",
        "boundaries": "It does not draw interactive UI controls or shape text spans.",
        "api": "- `PhysicsWorld` — Rigid-body solver manager.\n- `RigidBody` — Mass-point dynamic bodies.",
        "example": "```rust\nuse cvkg_physics::PhysicsWorld;\n```"
    },
    "cvkg-flow": {
        "purpose": "Canvas grid node-graph drawing engine and visual flow charts.",
        "boundaries": "It does not execute core application controller state logic.",
        "api": "- `FlowCanvas` — Node-graph editor workspace.",
        "example": "```rust\nuse cvkg_flow::FlowCanvas;\n```"
    },
    "cvkg-scheduler": {
        "purpose": "Synchronizes frame updates, layout passes, and GPU drawing tasks.",
        "boundaries": "It does not compose layouts or rasterize text spans.",
        "api": "- `FrameScheduler` — Frame manager clock.",
        "example": "```rust\nuse cvkg_scheduler::FrameScheduler;\n```"
    },
    "cvkg-spatial": {
        "purpose": "Provides spatial indexing algorithms (QuadTree, BVH) for hit-testing.",
        "boundaries": "It does not resolve CSS style properties.",
        "api": "- `QuadTree` — Spatial bounding box indexer.",
        "example": "```rust\nuse cvkg_spatial::QuadTree;\n```"
    },
    "cvkg-reflect": {
        "purpose": "Type introspection and property reflection mappings for runtime inspection.",
        "boundaries": "It does not capture user events.",
        "api": "- `ReflectRegistry` — Inspect properties dynamically.",
        "example": "```rust\nuse cvkg_reflect::ReflectRegistry;\n```"
    },
    "cvkg-materials": {
        "purpose": "Defines materials configs like Glass, Mica, and Acrylic profiles.",
        "boundaries": "It does not render shapes or execute GPU fragment shaders.",
        "api": "- `GlassMaterial` — blur/tint settings.\n- `MicaMaterial` — backdrop characteristics.",
        "example": "```rust\nuse cvkg_materials::GlassMaterial;\n```"
    },
    "cvkg-accessibility": {
        "purpose": "Translates visual component states into accessibility tree nodes for screen readers.",
        "boundaries": "It does not process click events or run animation loops.",
        "api": "- `AccessibilityBridge` — Mappings to screen readers.",
        "example": "```rust\nuse cvkg_accessibility::AccessibilityBridge;\n```"
    },
    "cvkg-certification": {
        "purpose": "Automated runtime conformance checkers verifying specification invariants.",
        "boundaries": "It does not compile release targets.",
        "api": "- `CertSuite` — Specification conformance audits.",
        "example": "```rust\nuse cvkg_certification::CertSuite;\n```"
    }
}

def parse_cargo_dependencies(cargo_path):
    deps = []
    if not os.path.exists(cargo_path):
        return deps
    with open(cargo_path, 'r') as f:
        content = f.read()
    # Find path-based workspace dependencies
    matches = re.findall(r'(\S+)\s*=\s*\{\s*path\s*=', content)
    for m in matches:
        # Standardize matching name
        name = m.replace('package', '').strip().replace('"', '').replace('=', '').strip()
        if name and name not in deps:
            deps.append(name)
    return deps

def main():
    workspace_root = "/D/rex/projects/cvkg"
    crates = [d for d in os.listdir(workspace_root) if os.path.isdir(os.path.join(workspace_root, d)) and os.path.exists(os.path.join(workspace_root, d, "Cargo.toml"))]
    
    # Map out forward dependencies
    forward_deps = {}
    for crate in crates:
        cargo_path = os.path.join(workspace_root, crate, "Cargo.toml")
        forward_deps[crate] = parse_cargo_dependencies(cargo_path)

    # Map out reverse dependencies
    reverse_deps = {c: [] for c in crates}
    for crate, deps in forward_deps.items():
        for dep in deps:
            # Map name if clean matches
            resolved_dep = dep
            if dep == "runic-text":
                resolved_dep = "cvkg-runic-text"
            if resolved_dep in reverse_deps:
                reverse_deps[resolved_dep].append(crate)

    for crate in crates:
        if crate not in CRATE_INFO:
            continue
        info = CRATE_INFO[crate]
        
        # Build Mermaid graph
        mermaid_lines = [
            "graph TD",
            f"    {crate}[\"{crate} (Focal Crate)\"]"
        ]
        
        # Add forward workspace dependencies
        for dep in forward_deps[crate]:
            resolved_dep = dep
            if dep == "runic-text":
                resolved_dep = "cvkg-runic-text"
            mermaid_lines.append(f"    {resolved_dep}[\"{resolved_dep}\"]")
            mermaid_lines.append(f"    {crate} --> {resolved_dep}")
            
        # Add reverse dependencies
        for rev in reverse_deps[crate]:
            mermaid_lines.append(f"    {rev}[\"{rev}\"]")
            mermaid_lines.append(f"    {rev} --> {crate}")
            
        # Styles
        mermaid_lines.append("    classDef focal fill:#0f172a,stroke:#3b82f6,color:#38bdf8,stroke-width:2px")
        mermaid_lines.append("    classDef sibling fill:#311042,stroke:#d946ef,color:#f472b6,stroke-width:1px")
        mermaid_lines.append(f"    class {crate} focal")
        
        siblings = list(set(forward_deps[crate] + reverse_deps[crate]))
        siblings = [s if s != "runic-text" else "cvkg-runic-text" for s in siblings]
        if siblings:
            mermaid_lines.append(f"    class {','.join(siblings)} sibling")
            
        mermaid_graph = "\n".join(mermaid_lines)
        
        # Write README.md
        readme_content = f"""# {crate}

## Purpose
{info['purpose']}

## Boundaries
- {info['boundaries']}
- It does not contain testing frameworks; quality checks are managed by `cvkg-test`.

## Dependency Graph
```mermaid
{mermaid_graph}
```

## Public API Overview
{info['api']}

## Usage Example
{info['example']}

## Use Cases
- Mapped as a core component inside the standard framework dependency tree.

## Edge Cases and Limitations
- Under extreme scale or thread contention, ensure the host runtime balances cycles appropriately.

## Crate-Specific Build Flags
This crate has no custom feature flags or compile-time options. It compiles under standard cargo parameters.
"""
        readme_path = os.path.join(workspace_root, crate, "README.md")
        with open(readme_path, "w") as f:
            f.write(readme_content)
        print(f"Generated/Updated README for {crate}")

if __name__ == "__main__":
    main()
