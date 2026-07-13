//! cvkg 2D+3D UI Framework Integration Layer
//!
//! This module provides the bridge between cvkg-components and cvkg's core
//! rendering stack: vdom companions, 3D world-space rendering, spring-based
//! animations (cvkg-anim), physics (cvkg-physics), and glass materials
//! (cvkg-materials).
//!
//! ## Why this exists
//! cvkg is a true 2D+3D UI framework, but cvkg-components were originally
//! written for 2D-only rendering with stub `LayoutView` implementations and
//! no companion state. This module provides reusable building blocks so each
//! component can opt into the appropriate cvkg subsystems without every
//! component file re-implementing the same wiring.
//!
//! ## Usage patterns
//! Each helper below is designed to be held as a field on a component struct
//! and consulted from `render()`, `LayoutView`, or `companion_states()`. The
//! helpers are cheap to clone (most are `Option<...>`) and behave as no-ops
//! when unset, so existing 2D-only code paths remain unchanged.
//!
//! ```rust,ignore
//! use cvkg_components::integration::{CompanionBundle, SpringState, WorldSpaceConfig};
//!
//! pub struct Button {
//!     label: String,
//!     companions: CompanionBundle,   // focus + a11y
//!     press_spring: SpringState,     // press-depth animation
//!     world: WorldSpaceConfig,       // optional 3D placement
//! }
//! ```

use cvkg_anim::{SpringParams, SpringSolver};
use cvkg_core::companion::{A11yCompanion, Companion, FocusableCompanion};
use cvkg_core::mesh::Transform3D;
use cvkg_core::spring::SpringParams as CoreSpringParams;
use cvkg_core::PhysicsBody;
use cvkg_vdom::vnode::WorldSpacePanel;

// ───────────────────────────────────────────────────────────────────────────
// Companion bundles
// ───────────────────────────────────────────────────────────────────────────

/// A reusable bundle of `Companion` states that interactive components
/// typically need: `FocusableCompanion` (focus management) and
/// `A11yCompanion` (accessibility role/label).
///
/// Components return `bundle.to_vec()` from `View::companion_states()` so the
/// VDOM auto-initializes the right per-node state without the component having
/// to manually construct the vec at every render.
///
/// # Why a builder rather than always returning both
/// Some components are purely presentational (no focus) or purely decorative
/// (no a11y). The builder lets each component opt in to exactly the
/// companions it needs, keeping the VDOM node lean.
#[derive(Clone, Debug, Default)]
pub struct CompanionBundle {
    /// When true, the component participates in keyboard focus.
    pub focusable: bool,
    /// When true, the component publishes an ARIA role + label to the VDOM.
    pub a11y: bool,
    /// ARIA role string (e.g. "button", "checkbox", "dialog").
    pub role: Option<String>,
    /// ARIA label string (e.g. "Submit form", "Remember me").
    pub label: Option<String>,
    /// ARIA description string (longer-form context).
    pub description: Option<String>,
}

impl CompanionBundle {
    /// Create a bundle with focus enabled but no A11y yet.
    pub fn focusable() -> Self {
        Self {
            focusable: true,
            a11y: false,
            role: None,
            label: None,
            description: None,
        }
    }

    /// Attach an ARIA role to the bundle (enables A11y).
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.a11y = true;
        self.role = Some(role.into());
        self
    }

    /// Attach an ARIA label to the bundle.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.a11y = true;
        self.label = Some(label.into());
        self
    }

    /// Attach an ARIA description to the bundle.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.a11y = true;
        self.description = Some(description.into());
        self
    }

    /// Materialize the bundle into the `Vec<Box<dyn Companion>>` that
    /// `View::companion_states` returns. Empty when neither focus nor a11y
    /// is enabled.
    ///
    /// # Contract
    /// - The returned companions implement `Companion::default()` semantics,
    ///   i.e. they start in a neutral state and the VDOM owns their lifecycle
    ///   from that point on.
    /// - The order is fixed: focus first, then a11y. Downcast by type, not
    ///   by index, when retrieving.
    pub fn to_vec(&self) -> Vec<Box<dyn Companion>> {
        let mut out: Vec<Box<dyn Companion>> = Vec::new();
        if self.focusable {
            out.push(Box::new(FocusableCompanion::new()));
        }
        if self.a11y {
            let mut a11y = A11yCompanion::new();
            if let Some(role) = &self.role {
                a11y = a11y.with_role(role);
            }
            if let Some(label) = &self.label {
                a11y = a11y.with_label(label);
            }
            if let Some(description) = &self.description {
                a11y = a11y.with_description(description);
            }
            out.push(Box::new(a11y));
        }
        out
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Per-node spring animation state
// ───────────────────────────────────────────────────────────────────────────

/// A self-contained spring solver that components can hold as a field to
/// drive a single animated scalar (e.g. a press-depth scale factor, a toggle
/// rotation, a focus-ring opacity).
///
/// The component owns the `SpringState`, calls `set_target` when some UI
/// state changes (e.g. pressed → release), and calls `tick` inside its
/// `render()` using the renderer's elapsed-time delta. The returned value is
/// the current animated scalar.
///
/// # Why a wrapper and not just `SpringSolver` directly
/// `SpringSolver` requires a target and an initial value at construction
/// time, which is awkward for a component field that doesn't yet know its
/// starting value. `SpringState` lazily initializes the solver on first
/// `set_target`, so the field can be constructed with just params and a
/// resting value.
#[derive(Clone, Debug)]
pub struct SpringState {
    pub params: SpringParams,
    pub resting: f32,
    solver: Option<SpringSolver>,
    current: f32,
    /// When true, `tick` snaps to the target instantly (accessibility:
    /// prefers-reduced-motion). Components should forward the renderer's
    /// reduce-motion setting here.
    pub reduce_motion: bool,
}

impl SpringState {
    /// Create a spring state with the given params and resting value.
    pub fn new(params: SpringParams, resting: f32) -> Self {
        Self {
            params,
            resting,
            solver: None,
            current: resting,
            reduce_motion: false,
        }
    }

    /// A snappy spring resting at 0.0 — good default for press scales.
    pub fn press() -> Self {
        Self::new(SpringParams::snappy(), 0.0)
    }

    /// A fluid spring resting at 0.0 — good default for focus rings.
    pub fn focus_ring() -> Self {
        Self::new(SpringParams::fluid(), 0.0)
    }

    /// Update the spring's target. If the solver hasn't been initialized yet
    /// (first call), the solver is created with the current value as its
    /// starting point.
    pub fn set_target(&mut self, target: f32) {
        match &mut self.solver {
            Some(s) => s.set_target(target),
            None => {
                let mut s = SpringSolver::new(self.params, target, self.current);
                s.set_reduce_motion(self.reduce_motion);
                self.solver = Some(s);
            }
        }
    }

    /// Advance the simulation by `dt` seconds. Returns the current animated
    /// value. When `reduce_motion` is enabled, snaps to target instantly.
    pub fn tick(&mut self, dt: f32) -> f32 {
        if let Some(s) = &mut self.solver {
            s.set_reduce_motion(self.reduce_motion);
            self.current = s.tick(dt.max(0.0));
        }
        self.current
    }

    /// Read the current animated value without advancing the simulation.
    pub fn current(&self) -> f32 {
        self.current
    }

    /// Returns true when the spring has been initialized (i.e. `set_target`
    /// has been called at least once). Before initialization, `tick` is a
    /// no-op returning the resting value.
    pub fn is_animating(&self) -> bool {
        self.solver.is_some()
    }
}

impl Default for SpringState {
    fn default() -> Self {
        Self::new(SpringParams::default(), 0.0)
    }
}

// ───────────────────────────────────────────────────────────────────────────
// 3D world-space configuration
// ───────────────────────────────────────────────────────────────────────────

/// Configuration for rendering a component subtree into an offscreen texture
/// and compositing it as a 3D-positioned panel in the scene.
///
/// This is the primary integration point for cvkg's "true 2D+3D" story:
/// components that set `.world(Some(...))` on their `WorldSpaceConfig` field
/// will have their entire `render()` subtree redirected to an offscreen
/// target, then presented in 3D space with the configured transform, glass
/// material, and spring/physics parameters.
///
/// # 2D fast path
/// A default-constructed `WorldSpaceConfig` (zero `world_size`) is treated as
/// "no 3D panel". Components that don't specify a 3D transform should leave
/// it as `Default::default()` — the 2D path stays fast and allocation-free.
///
/// # Companions + world-space
/// A `WorldSpaceConfig` is orthogonal to `CompanionBundle`: a 3D dialog is
/// still focusable and still has ARIA semantics, it's just composited
/// differently. Both can be set on the same component.
#[derive(Clone, Debug)]
pub struct WorldSpaceConfig {
    /// The 3D transform (position, rotation, scale) in world space.
    pub transform: Transform3D,
    /// Logical size in world units (meters). The offscreen texture resolution
    /// = `world_size * pixels_per_unit`.
    pub world_size: (f32, f32),
    /// Pixel density for the offscreen texture. 200.0 = sharp UI on a panel
    /// the size of a monitor; lower for distant panels.
    pub pixels_per_unit: f32,
    /// Optional glass material so the panel refracts/blurs what's behind it.
    pub glass: Option<cvkg_materials::GlassMaterial>,
    /// Optional spring parameters for interactive settling (e.g. the panel
    /// bounces into place on present).
    pub spring: Option<SpringParams>,
    /// Optional physics body (core-local descriptor) so the panel participates
    /// in the cvkg-physics world simulation (gravity, collisions, constraints).
    /// The concrete renderer lowers this into a `cvkg_physics::RigidBody3D`.
    pub physics: Option<PhysicsBody>,
}

impl WorldSpaceConfig {
    /// A disabled (2D-only) world-space config.
    pub fn default() -> Self {
        Self {
            transform: Transform3D::default(),
            world_size: (0.0, 0.0),
            pixels_per_unit: 200.0,
            glass: None,
            spring: None,
            physics: None,
        }
    }

    /// Create a world-space config at a given transform with default size
    /// (0.5 m × 0.5 m) and 200 px/unit density.
    pub fn at(transform: Transform3D) -> Self {
        Self {
            transform,
            world_size: (0.5, 0.5),
            pixels_per_unit: 200.0,
            glass: None,
            spring: None,
            physics: None,
        }
    }

    /// Set the world size in meters.
    pub fn with_size(mut self, w: f32, h: f32) -> Self {
        self.world_size = (w, h);
        self
    }

    /// Set the pixel density.
    pub fn with_pixels_per_unit(mut self, ppu: f32) -> Self {
        self.pixels_per_unit = ppu;
        self
    }

    /// Attach a glass material to the panel.
    pub fn with_glass(mut self, glass: cvkg_materials::GlassMaterial) -> Self {
        self.glass = Some(glass);
        self
    }

    /// Attach spring parameters for interactive settling.
    pub fn with_spring(mut self, params: SpringParams) -> Self {
        self.spring = Some(params);
        self
    }

    /// Attach a physics body so the panel participates in the
    /// cvkg-physics world simulation (gravity, collisions, constraints).
    pub fn with_physics(mut self, body: PhysicsBody) -> Self {
        self.physics = Some(body);
        self
    }

    /// Convenience constructor for a frosted-glass world panel: enables
    /// world-space with a default glass material and the given size/ppu.
    /// The optional `spring` makes the panel settle into place on present.
    pub fn glass_panel(transform: Transform3D, size: (f32, f32)) -> Self {
        Self {
            transform,
            world_size: size,
            pixels_per_unit: 200.0,
            glass: Some(cvkg_materials::GlassMaterial::default()),
            spring: Some(SpringParams::default()),
            physics: None,
        }
    }

    /// Convert to a `WorldSpacePanel` for the VDOM. Returns `None` when the
    /// component should not participate in world-space rendering.
    pub fn to_panel(&self) -> Option<WorldSpacePanel> {
        // We only build a panel when the caller explicitly configured one.
        // A default-constructed `WorldSpaceConfig` (transform = identity,
        // zero size) is treated as "no panel".
        if self.world_size == (0.0, 0.0) {
            return None;
        }
        let panel = WorldSpacePanel {
            transform: self.transform,
            world_size: self.world_size,
            pixels_per_unit: self.pixels_per_unit,
            glass: self.glass.clone(),
            // Convert from cvkg_anim::SpringParams to the core SpringParams
            // type that WorldSpacePanel expects. The two types are
            // structurally identical (stiffness, damping, mass).
            spring: self.spring.map(|s| CoreSpringParams {
                stiffness: s.stiffness,
                damping: s.damping,
                mass: s.mass,
            }),
            physics: self.physics.clone(),
        };
        // sanity: never let pixels_per_unit be zero or negative
        let mut panel = panel;
        if panel.pixels_per_unit <= 0.0 {
            panel.pixels_per_unit = 200.0;
        }
        Some(panel)
    }

    /// Begin 3D world-space redirection on the renderer, forwarding the full
    /// panel (glass + spring + physics) when the caller configured one.
    ///
    /// This is the single integration seam for every wired component: call it
    /// once inside `render()` after `push_vnode_*` and before drawing. It
    /// uses `begin_world_space_panel_full` so `cvkg_physics` rigid bodies and
    /// `cvkg_anim` spring settling survive into the VDOM `WorldSpacePanel`.
    pub fn begin(&self, renderer: &mut dyn cvkg_core::Renderer, node_id: u64) {
        if !self.is_enabled() {
            return;
        }
        let panel = self.to_panel().expect("is_enabled() implies to_panel() Some");
        renderer.begin_world_space_panel_full(
            node_id,
            &panel.transform,
            panel.glass,
            panel.spring,
            panel.physics,
            panel.pixels_per_unit,
            panel.world_size,
        );
    }

    /// True when this config actually enables world-space rendering (i.e.
    /// has a non-zero world size). Components can use this to branch their
    /// `render()` between the 2D and 3D paths.
    pub fn is_enabled(&self) -> bool {
        self.world_size != (0.0, 0.0)
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Integration tests
// ───────────────────────────────────────────────────────────────────────────

// These verify the contract that the integration layer is actually wired into
// the VDOM: a component that opts into `CompanionBundle` + `WorldSpaceConfig`
// must surface (a) companion states on its `VNode.companions` map and
// (b) a `world_space` panel on the `VNode` when world-space is enabled.
//
// This is the decisive regression guard for the "true 2D+3D" integration:
// before this wiring, `companion_states()` was never invoked by the render
// pass and `world_space` was always `None`.

#[cfg(test)]
mod vdom_integration_tests {
    use super::*;
    use crate::{Button, Text};
    use cvkg_core::{Rect, Transform3D};
    use cvkg_vdom::VDom;
    use cvkg_core::PhysicsBody;

    fn find_node<'a>(vdom: &'a VDom, component_type: &str) -> Option<&'a cvkg_vdom::vnode::VNode> {
        vdom.nodes
            .values()
            .find(|n| n.component_type == component_type)
    }

    #[test]
    fn button_with_world_space_surfaces_panel_and_companions() {
        let world = WorldSpaceConfig::glass_panel(Transform3D::default(), (200.0, 50.0));
        let btn = Button::new("Submit", || {}).world(world);
        let vdom = VDom::build(&btn, Rect::new(0.0, 0.0, 200.0, 44.0));

        let node = find_node(&vdom, "Button").expect("Button VNode should exist");
        // World-space panel present
        assert!(
            node.world_space.is_some(),
            "Button with .world() must produce a world_space panel"
        );
        let panel = node.world_space.as_ref().unwrap();
        assert!(
            panel.glass.is_some(),
            "glass_panel must attach a glass material"
        );
        assert!(
            panel.spring.is_some(),
            "glass_panel must attach spring settling params"
        );
        // Companion states (Focusable + A11y) must be attached.
        assert!(
            !node.companions.is_empty(),
            "Button must publish companion states to the VDOM"
        );
        assert!(
            node.companions.contains_key("FocusableCompanion"),
            "Button must be focusable"
        );
        assert!(
            node.companions.contains_key("A11yCompanion"),
            "Button must publish ARIA companion"
        );
    }

    #[test]
    fn button_without_world_space_has_no_panel_but_has_companions() {
        let btn = Button::new("Submit", || {}); // default world = disabled
        let vdom = VDom::build(&btn, Rect::new(0.0, 0.0, 200.0, 44.0));

        let node = find_node(&vdom, "Button").expect("Button VNode should exist");
        assert!(
            node.world_space.is_none(),
            "Button without .world() must not produce a world_space panel"
        );
        assert!(
            !node.companions.is_empty(),
            "Button must still publish companions in 2D mode"
        );
    }

    #[test]
    fn text_publishes_companions_and_uses_dedicated_vnode() {
        let text = Text::new("Hello world");
        let vdom = VDom::build(&text, Rect::new(0.0, 0.0, 120.0, 20.0));

        let node = find_node(&vdom, "Text").expect("Text VNode should exist");
        assert!(
            !node.companions.is_empty(),
            "Text must publish companion states (role + label) to the VDOM"
        );
        assert!(
            node.companions.contains_key("A11yCompanion"),
            "Text must publish an ARIA companion with its content as label"
        );
    }

    #[test]
    fn companion_states_method_returns_focus_and_a11y() {
        // Direct unit-level check of the View::companion_states() hook.
        let btn = Button::new("Save", || {});
        let companions = cvkg_core::View::companion_states(&btn);
        assert_eq!(companions.len(), 2, "Button yields Focusable + A11y");
        assert_eq!(companions[0].type_name(), "FocusableCompanion");
        assert_eq!(companions[1].type_name(), "A11yCompanion");
    }

    #[test]
    fn world_space_panel_forwards_glass_spring_and_physics_to_vdom() {
        // Wire a full panel: glass + spring (via glass_panel) AND a cvkg-physics
        // rigid body. Before the begin_world_space_panel_full fix, spring and
        // physics were silently dropped. This guards that regression.
        use cvkg_core::Transform3D;

        let world = WorldSpaceConfig::glass_panel(
            Transform3D::default(),
            (200.0, 50.0),
        )
        .with_physics(PhysicsBody::at(&Transform3D::default()));

        let btn = Button::new("Orbit", || {}).world(world);
        let vdom = VDom::build(&btn, Rect::new(0.0, 0.0, 200.0, 50.0));

        let node = find_node(&vdom, "Button").expect("Button VNode should exist");
        let panel = node
            .world_space
            .as_ref()
            .expect("glass_panel must produce a world_space panel");
        assert!(
            panel.glass.is_some(),
            "glass_panel must attach a glass material"
        );
        assert!(
            panel.spring.is_some(),
            "glass_panel must attach spring settling params"
        );
        assert!(
            panel.physics.is_some(),
            "with_physics must forward the PhysicsBody into the VDOM panel"
        );
    }
}