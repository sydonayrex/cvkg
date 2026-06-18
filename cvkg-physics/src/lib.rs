//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     -- State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     -- Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     -- Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    -- Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     -- Read the target, its surrounding context, and its full call graph
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     -- Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   -- Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.

//! # Tyr -- Rigid Body Physics Engine for CVKG
//!
//! 2D-oriented rigid body simulation with impulse-based constraint solving,
//! broad-phase culling via spatial hashing, and GJK/EPA narrow-phase collision.
//!
//! ## Architecture
//!
//! ```text
//! Application code
//!     │
//!     ▼
//! cvkg-physics (this crate)
//!     ├── world.rs          -- owns all bodies, runs simulation steps
//!     ├── body.rs           -- RigidBody: mass, velocity, restitution
//!     ├── shape.rs          -- Circle, AABB, ConvexHull, Capsule
//!     ├── collider.rs       -- binds a shape to a body with offset/rotation
//!     ├── constraint.rs     -- distance, pin, hinge, angular limit
//!     ├── solver.rs         -- Gauss-Seidel impulse solver
//!     ├── broadphase.rs     -- spatial hash for coarse culling
//!     ├── narrowphase.rs    -- GJK/EPA for convex-convex contact manifolds
//!     ├── integration.rs    -- semi-implicit Euler integrator
//!     └── scene_bridge.rs   -- reads/writes cvkg-scene NodeId transforms
//! ```
//!
//! ## Coordinate system
//!
//! All quantities use 2D coordinates (x, y) with the y-axis pointing down,
//! matching the CVKG screen coordinate convention. Angles are in radians,
//! positive rotation is clockwise.

pub mod body;
pub mod broadphase;
pub mod character;
pub mod collider;
pub mod constraint;
pub mod debug_draw;
pub mod gpu_broadphase;
pub mod heightmap;
pub mod integration;
pub mod lod;
pub mod mjolnir_bridge;
pub mod narrowphase;
pub mod queries;
pub mod ragdoll_bridge;
pub mod scene_bridge;
pub mod shape;
pub mod snapshot;
pub mod solver;
pub mod sph;
pub mod world;
pub mod xpbd;

pub use body::{BodyId, RigidBody};
pub use broadphase::SpatialHash;
pub use collider::Collider;
pub use constraint::{Constraint, ConstraintKind};
pub use integration::semi_implicit_euler;
pub use mjolnir_bridge::{shatter_at_constraint_break, shatter_at_position};
pub use narrowphase::{
    CcdResult3D, Contact, ContactManifold, GjkResult, epa, epa_with_simplex, gjk, gjk_ccd,
    gjk_ccd_3d, gjk_overlap,
};
pub use queries::{
    OverlapHit, QueryFilter, RaycastHit, RaycastHit3D, ShapeCastHit, ShapeCastHit3D,
};
pub use ragdoll_bridge::{BoneBodyMap, RagdollBridge, RagdollBridgeConfig};
pub use shape::{Shape, ShapeKind};
pub use solver::ImpulseSolver;
pub use world::{CollisionEvent, CollisionEventType, PhysicsWorld, StepResult, WorldConfig};
///
/// This is the handoff point to cvkg-anim: application code can use this
/// to trigger a Sleipnir spring animation to snap to grid/guide positions.
pub type OnSleepCallback = Box<dyn Fn(BodyId) + Send + Sync>;
