CVKG Cross-Crate Architecture Maturity Implementation Plan
Background
The 
crosscrate.md
 audit identifies 10 critical findings that block CVKG from reaching the platform maturity tier occupied by Flutter, WPF, Qt, and Jetpack Compose. The workspace currently builds cleanly and has a solid crate graph. This plan implements the missing platform infrastructure in priority order.

Baseline State (Verified)
Workspace: builds cleanly (cargo test --workspace --no-run passes)
Identity: Fragmented — NodeId(u64) exists independently in cvkg-scene, cvkg-vdom, cvkg-flow, and cvkg-core
Invalidation: Per-crate dirty flags, no platform-wide model
Scheduler: None — updates execute immediately
Spatial indexing: Partial — SceneGraph has a spatial hash grid; cvkg-flow and cvkg-physics have nothing
Virtualization: Missing everywhere
Reflection: None
Materials: Stub types in cvkg-core::material.rs and cvkg-flow::GlassNodeMaterial but no unified crate
User Review Required
IMPORTANT

Identity Migration Strategy: Introducing KvasirId platform-wide means all existing NodeId(u64) types in cvkg-scene, cvkg-vdom, and cvkg-flow must either alias or wrap KvasirId. The safest approach is a type alias (pub type NodeId = KvasirId) so existing code is source-compatible. The alternative (newtype wrapping) would require mechanical refactoring across thousands of call sites. Decision needed: alias or newtype wrapper? This plan defaults to type alias.

WARNING

New crates add compilation time. Six new crates will add to incremental build times. Each is kept minimal (single lib.rs with tests for now) to minimize impact.

IMPORTANT

cvkg-vdom is not in workspace: The workspace Cargo.toml lists cvkg-vdom in [patch.crates-io] but not in [workspace] members. This means it is built as an external patch, not as a workspace member. Verify whether it should be added to workspace members. This plan will not change that unless you confirm.

Open Questions
KvasirId backing type: The audit suggests Uuid. However, Uuid adds a dependency and makes IDs non-sequential, which hurts cache locality in HashMap lookups. A u64 from an atomic counter is simpler and faster. Start with u64, add Uuid as an opt-in feature flag later.

cvkg-scheduler async runtime: Should it use tokio (already in workspace deps) or a custom frame-tick model? Integrate with the existing tokio features already declared (sync, rt, macros) rather than a new runtime.

cvkg-materials GPU tie-in: Glass/Mica/Acrylic require backend sampling. cvkg-materials define pure data structs (and cvkg-render-gpu consumes). pure data structs in cvkg-materials, pipeline descriptors in cvkg-render-gpu.

cvkg-accessibility vs existing cvkg-vdom a11y: cvkg-vdom already has A11yNodeEntry, AriaProps, and full accesskit integration. Should cvkg-accessibility supersede this, or extend it? New crate wraps and re-exports the vdom a11y types and adds the missing Semantics layer, Screen Reader bridge, and Focus Management.

Proposed Changes
Dependency Layer First (bottom-up ordering)
Phase 1: Unified Identity — cvkg-core
Priority: CRITICAL (Finding #2). Every other phase depends on this.

[MODIFY] 
lib.rs
Add KvasirId to cvkg-core — the platform-wide unique identifier. All crate-local NodeId/EdgeId/PortId types will alias this.

rust

/// Platform-wide unique identifier. All graph layers use this type.
/// Backed by a u64 from an atomic counter — sequential, cache-friendly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KvasirId(pub u64);
impl KvasirId {
    /// Allocate a new globally-unique KvasirId.
    pub fn new() -> Self { ... }
}
Phase 2: Unified Invalidation — cvkg-core
Priority: CRITICAL (Finding #3)

[MODIFY] 
lib.rs
Add DirtyFlags bitfield and InvalidationRecord to cvkg-core. Consumed by scene, layout, compositor, renderer, animation.

rust

bitflags! {
    pub struct DirtyFlags: u8 {
        const STATE      = 0b0000_0001;
        const LAYOUT     = 0b0000_0010;
        const PAINT      = 0b0000_0100;
        const COMPOSITE  = 0b0000_1000;
    }
}
pub struct InvalidationRecord {
    pub id: KvasirId,
    pub flags: DirtyFlags,
}
Phase 3: [NEW] cvkg-scheduler Crate
Priority: CRITICAL (Finding #4)

New crate at cvkg-scheduler/ providing:

FrameScheduler — coordinates render-frame tick, integrates with tokio
TaskScheduler — priority queue for background work (layout, physics, telemetry)
Priority enum — Critical, High, Normal, Idle
[NEW] cvkg-scheduler/Cargo.toml
[NEW] cvkg-scheduler/src/lib.rs
[NEW] cvkg-scheduler/src/frame.rs
[NEW] cvkg-scheduler/src/task.rs
Phase 4: [NEW] cvkg-spatial Crate
Priority: CRITICAL (Finding #5)

New crate at cvkg-spatial/ providing platform-wide spatial indexing.

Currently cvkg-scene has its own spatial hash and quadtree. These move here and are re-exported from scene.

Structures:

QuadTree<T> — generic 2D quad tree
BoundingVolumeHierarchy<T> — flat BVH for ray queries and physics
SpatialHash<T> — existing grid promoted to shared crate
RTree<T> — placeholder stub
[NEW] cvkg-spatial/Cargo.toml
[NEW] cvkg-spatial/src/lib.rs
[NEW] cvkg-spatial/src/quadtree.rs (move from cvkg-scene/src/quadtree.rs)
[NEW] cvkg-spatial/src/bvh.rs
[NEW] cvkg-spatial/src/spatial_hash.rs
[MODIFY] 
cvkg-scene/src/lib.rs
Remove local quadtree.rs and spatial hash impl
Re-export from cvkg-spatial
Phase 5: [NEW] cvkg-reflect Crate
Priority: HIGH (Finding #8)

New crate providing compile-time reflection metadata generated by cvkg-macros.

Types:

ReflectType — type descriptor
ReflectField — named field with type info and accessor
Reflected trait — implemented by #[derive(Reflect)] proc-macro
[NEW] cvkg-reflect/Cargo.toml
[NEW] cvkg-reflect/src/lib.rs
[MODIFY] cvkg-macros — Add #[derive(Reflect)] proc-macro
Phase 6: [NEW] cvkg-materials Crate
Priority: CRITICAL (Finding #7, blocks Native UI Parity)

New crate with pure data structs for all material types. No GPU code.

Materials:

GlassMaterial — blur, refraction, frost, tint (consolidates existing GlassNodeMaterial)
MicaMaterial — translucent frosted with system color integration
AcrylicMaterial — acrylic blur with tint
ElevationLevel — 0–5 elevation mapping
MaterialToken — semantic theme token for materials
[NEW] cvkg-materials/Cargo.toml
[NEW] cvkg-materials/src/lib.rs
[NEW] cvkg-materials/src/glass.rs
[NEW] cvkg-materials/src/mica.rs
[NEW] cvkg-materials/src/acrylic.rs
[NEW] cvkg-materials/src/elevation.rs
Phase 7: [NEW] cvkg-accessibility Crate
Priority: HIGH (Finding #10 partially, and Finding #7)

New crate that wraps cvkg-vdom's existing a11y types and adds:

AccessibilityTree — platform-facing tree built from VDom
SemanticNode — richer semantic annotations
FocusManager — keyboard navigation model
ScreenReaderBridge trait — platform-specific announcement
[NEW] cvkg-accessibility/Cargo.toml
[NEW] cvkg-accessibility/src/lib.rs
[NEW] cvkg-accessibility/src/tree.rs
[NEW] cvkg-accessibility/src/focus.rs
[NEW] cvkg-accessibility/src/bridge.rs
Phase 8: [NEW] cvkg-certification Crate
Priority: HIGH (Finding #9)

New crate providing cross-crate integration test certification framework:

CertificationSuite — runs platform-level test batteries
VisualCertification — pixel-diff reference testing
PipelineCertification — Scene→Layout→Render pipeline checks
PerformanceCertification — frame-time budget validation
[NEW] cvkg-certification/Cargo.toml
[NEW] cvkg-certification/src/lib.rs
[NEW] cvkg-certification/tests/pipeline_cert.rs
[NEW] cvkg-certification/tests/scene_layout_render.rs
Workspace Coordination
[MODIFY] 
Cargo.toml
Add 6 new crates to [workspace] members
Add 6 new crates to [workspace.dependencies]
Add 6 new crates to [patch.crates-io]
Add bitflags = "2" to workspace deps (for DirtyFlags)
Add uuid = { version = "1", features = ["v4"] } as optional dep for KvasirId
Verification Plan
Automated Tests
After each phase:

bash

cargo test --workspace 2>&1 | tail -20
Specific new test batteries:

bash

cargo test -p cvkg-scheduler
cargo test -p cvkg-spatial
cargo test -p cvkg-reflect
cargo test -p cvkg-materials
cargo test -p cvkg-accessibility
cargo test -p cvkg-certification
Cross-crate integration check:

bash

cargo test -p cvkg-certification -- --test-threads=1
TDD Gate
Each new crate follows RED → GREEN → REFACTOR:

Write failing tests first
Implement minimal code
Verify green
Refactor with tests still green
Manual Verification
Confirm validate_sync in cvkg-vdom still passes after identity migration
Confirm SceneGraph tests in cvkg-scene still pass after spatial move
Run existing demo builds: berserker, adele-web
Execution Order

Phase 1: KvasirId + DirtyFlags in cvkg-core           [~2h]
Phase 2: cvkg-scheduler                               [~2h]
Phase 3: cvkg-spatial (move quadtree)                 [~2h]
Phase 4: cvkg-reflect + macros                        [~2h]
Phase 5: cvkg-materials                               [~1h]
Phase 6: cvkg-accessibility                           [~2h]
Phase 7: cvkg-certification (cross-crate tests)       [~3h]
Phase 8: Wire up + workspace Cargo.toml               [~1h]
Total estimated: ~15h of focused implementation
