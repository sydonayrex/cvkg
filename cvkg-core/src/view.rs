use crate::*;
use std::sync::Arc;
pub trait View: Sized + Send {
    /// The concrete type produced after applying modifiers.
    /// For primitive views this is Self.
    type Body: View;

    fn body(self) -> Self::Body;

    /// Render this view into the provided renderer at the specified bounds.
    /// Primitive views override this to perform drawing operations.
    fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {}

    /// Calculate the natural (intrinsic) size of this view given proposed constraints.
    /// This allows views like Buttons or Labels to inform the layout engine of their needs.
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size::ZERO
    }

    /// Optionally provide a layout implementation for this view.
    fn layout(&self) -> Option<&dyn layout::LayoutView> {
        None
    }

    /// Returns the flex weight of this view for proportional distribution in stacks.
    fn flex_weight(&self) -> f32 {
        0.0
    }

    /// Returns the grid placement configuration for this view if it is laid out in a Grid.
    fn get_grid_placement(&self) -> Option<GridPlacement> {
        None
    }

    /// Provided modifier entry point
    fn modifier<M: ViewModifier>(self, m: M) -> ModifiedView<Self, M> {
        ModifiedView::new(self, m)
    }

    /// Apply a Bifrost (Frosted Glass) effect to the view
    fn bifrost(
        self,
        blur: f32,
        saturation: f32,
        opacity: f32,
    ) -> ModifiedView<Self, FrostedGlassModifier> {
        self.modifier(FrostedGlassModifier {
            blur,
            saturation,
            opacity,
            fresnel_strength: 1.0,
        })
    }

    /// Apply a Bifrost (Frosted Glass) effect with full parameter control.
    fn bifrost_full(
        self,
        blur: f32,
        saturation: f32,
        opacity: f32,
        fresnel_strength: f32,
    ) -> ModifiedView<Self, FrostedGlassModifier> {
        self.modifier(FrostedGlassModifier {
            blur,
            saturation,
            opacity,
            fresnel_strength,
        })
    }

    /// Apply a Gungnir (Neon Glow) effect to the view
    fn gungnir(
        self,
        color: impl Into<String>,
        radius: f32,
        intensity: f32,
    ) -> ModifiedView<Self, NeonGlowModifier> {
        self.modifier(NeonGlowModifier {
            color: color.into(),
            radius,
            intensity,
        })
    }

    /// Apply a Mjolnir Slice (Geometric cut) to the view
    fn mjolnir_slice(self, angle: f32, offset: f32) -> ModifiedView<Self, GeometricClipModifier> {
        self.modifier(GeometricClipModifier { angle, offset })
    }

    /// Apply a Mjolnir Shatter (Fragmented transition) to the view
    fn mjolnir_shatter(self, pieces: u32, force: f32) -> ModifiedView<Self, FragmentModifier> {
        self.modifier(FragmentModifier { pieces, force })
    }

    /// Mark this view as a Bifrost Bridge (Shared Element) for cross-view persistence
    fn bifrost_bridge(self, id: impl Into<String>) -> ModifiedView<Self, SharedElementModifier> {
        self.modifier(SharedElementModifier { id: id.into() })
    }

    /// Add a background color to this view
    fn background(self, color: [f32; 4]) -> ModifiedView<Self, BackgroundModifier> {
        self.modifier(BackgroundModifier { color })
    }

    /// Add padding to this view
    fn padding(self, amount: f32) -> ModifiedView<Self, PaddingModifier> {
        self.modifier(PaddingModifier { amount })
    }

    /// Set the opacity (alpha) of this view in the range [0.0, 1.0].
    fn opacity(self, opacity: f32) -> ModifiedView<Self, OpacityModifier> {
        self.modifier(OpacityModifier {
            opacity: opacity.clamp(0.0, 1.0),
        })
    }

    /// Override the foreground (text / icon) color of this view.
    fn foreground_color(self, color: [f32; 4]) -> ModifiedView<Self, ForegroundColorModifier> {
        self.modifier(ForegroundColorModifier { color })
    }

    /// Constrain this view to an explicit width and/or height.
    /// Constrains the size of this view using fixed width/height values.
    fn frame(self, width: Option<f32>, height: Option<f32>) -> ModifiedView<Self, FrameModifier> {
        self.modifier(FrameModifier {
            width,
            height,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            alignment: Alignment::Center,
        })
    }

    /// Give this view a flex weight for proportional space distribution in stacks.
    fn flex(self, weight: f32) -> ModifiedView<Self, FlexModifier> {
        self.modifier(FlexModifier { weight })
    }

    /// Specify the grid placement configuration (column, row, column_span, row_span) for this view.
    fn grid_placement(self, placement: GridPlacement) -> ModifiedView<Self, GridPlacementModifier> {
        self.modifier(GridPlacementModifier { placement })
    }

    /// Overlay a view on top of this view, aligned and offset relative to it.
    fn overlay<O: View + Clone + 'static>(
        self,
        overlay: O,
        alignment: Alignment,
        offset: [f32; 2],
        on_dismiss: Option<Arc<dyn Fn() + Send + Sync>>,
    ) -> ModifiedView<Self, OverlayModifier> {
        self.modifier(OverlayModifier {
            overlay: overlay.erase(),
            alignment,
            offset,
            on_dismiss,
        })
    }

    /// Automatically add padding to avoid overlapping with platform safe areas (notches, bars).
    fn safe_area_padding(self) -> ModifiedView<Self, SafeAreaModifier> {
        self.modifier(SafeAreaModifier { ignores: false })
    }

    /// Explicitly ignore platform safe areas and draw into the margins.
    fn ignores_safe_area(self) -> ModifiedView<Self, SafeAreaModifier> {
        self.modifier(SafeAreaModifier { ignores: true })
    }

    /// Clip all child drawing to this view's bounds.
    fn clip_to_bounds(self) -> ModifiedView<Self, ClipModifier> {
        self.modifier(ClipModifier)
    }

    /// Draw a colored border around this view.
    fn border(self, color: [f32; 4], width: f32) -> ModifiedView<Self, BorderModifier> {
        self.modifier(BorderModifier { color, width })
    }

    /// Add elevation (shadow) to the view. Level determines the shadow depth.
    fn elevation(self, level: f32) -> ModifiedView<Self, ElevationModifier> {
        self.modifier(ElevationModifier { level })
    }

    /// Apply an absolute-like position offset to this view.
    /// The view is shifted by (x, y) from its layout position.
    fn position(self, x: f32, y: f32) -> ModifiedView<Self, PositionModifier> {
        self.modifier(PositionModifier { x, y })
    }

    /// Set the z-index (render order) for this view.
    /// Higher values render on top of lower values.
    fn z_index(self, z: i32) -> ModifiedView<Self, ZIndexModifier> {
        self.modifier(ZIndexModifier { z_index: z })
    }

    /// Add a magnetic effect that pulls the view towards the cursor.
    fn magnetic(self, radius: f32, intensity: f32) -> ModifiedView<Self, MagneticPullModifier> {
        self.modifier(MagneticPullModifier { radius, intensity })
    }

    /// Add a ManiGlow (Lunar Illuminator) effect that glows near the cursor.
    fn mani_glow(self, color: [f32; 4], radius: f32) -> ModifiedView<Self, CursorGlowModifier> {
        self.modifier(CursorGlowModifier { color, radius })
    }

    /// Theme this view based on a specific memory layer.
    fn memory_layer(self, layer: MemoryLayer) -> ModifiedView<Self, MemoryLayerModifier> {
        self.modifier(MemoryLayerModifier { layer })
    }

    /// Enable Fafnir's Evolution: The component grows and glows as it is used.
    fn fafnir_evolve(self, id: u64) -> ModifiedView<Self, EvolvingInteractionModifier> {
        self.modifier(EvolvingInteractionModifier { id })
    }

    /// Enable Mimir's Intent: The component anticipates user interaction via pointer kinematics.
    fn mimir_intent(self) -> ModifiedView<Self, IntentPredictionModifier> {
        self.modifier(IntentPredictionModifier)
    }

    /// Enable Kvasir's Vibes: Subconscious telemetry representing cognitive complexity.
    fn kvasir_vibes(self, complexity: f32) -> ModifiedView<Self, ComplexityTelemetryModifier> {
        self.modifier(ComplexityTelemetryModifier { complexity })
    }

    /// Bestow Odin's Eye: Global omniscient observability layer.
    fn odins_eye(self) -> ModifiedView<Self, ObservabilityOverlayModifier> {
        self.modifier(ObservabilityOverlayModifier)
    }

    /// Trigger an action when the view appears
    fn on_appear<F: Fn() + Send + Sync + 'static>(
        self,
        action: F,
    ) -> ModifiedView<Self, LifecycleModifier> {
        self.modifier(LifecycleModifier {
            on_appear: Some(Arc::new(action)),
            on_disappear: None,
        })
    }

    /// Trigger an action when the view disappears
    fn on_disappear<F: Fn() + Send + Sync + 'static>(
        self,
        action: F,
    ) -> ModifiedView<Self, LifecycleModifier> {
        self.modifier(LifecycleModifier {
            on_appear: None,
            on_disappear: Some(Arc::new(action)),
        })
    }

    /// Trigger an action when the view is clicked
    fn on_click<F: Fn() + Send + Sync + 'static>(
        self,
        action: F,
    ) -> ModifiedView<Self, OnClickModifier> {
        self.modifier(OnClickModifier {
            action: Arc::new(action),
        })
    }

    /// Trigger an action when the pointer enters the view bounds
    fn on_pointer_enter<F: Fn() + Send + Sync + 'static>(
        self,
        action: F,
    ) -> ModifiedView<Self, OnPointerEnterModifier> {
        self.modifier(OnPointerEnterModifier {
            action: Arc::new(action),
        })
    }

    /// Trigger an action when the pointer leaves the view bounds
    fn on_pointer_leave<F: Fn() + Send + Sync + 'static>(
        self,
        action: F,
    ) -> ModifiedView<Self, OnPointerLeaveModifier> {
        self.modifier(OnPointerLeaveModifier {
            action: Arc::new(action),
        })
    }

    /// Trigger an action when the pointer moves inside the view bounds
    fn on_pointer_move<F: Fn(f32, f32) + Send + Sync + 'static>(
        self,
        action: F,
    ) -> ModifiedView<Self, OnPointerMoveModifier> {
        self.modifier(OnPointerMoveModifier {
            action: Arc::new(action),
        })
    }

    /// Trigger an action when the pointer is pressed down
    fn on_pointer_down<F: Fn() + Send + Sync + 'static>(
        self,
        action: F,
    ) -> ModifiedView<Self, OnPointerDownModifier> {
        self.modifier(OnPointerDownModifier {
            action: Arc::new(action),
        })
    }

    /// Trigger an action when the pointer is released
    fn on_pointer_up<F: Fn() + Send + Sync + 'static>(
        self,
        action: F,
    ) -> ModifiedView<Self, OnPointerUpModifier> {
        self.modifier(OnPointerUpModifier {
            action: Arc::new(action),
        })
    }

    /// Type-erase this view into AnyView
    fn erase(self) -> AnyView
    where
        Self: Clone + 'static,
    {
        AnyView::new(self)
    }

    // =============================================================================
    // ACCESSIBILITY
    // =============================================================================

    /// Return accessibility properties for this view.
    /// Override to expose semantic role, label, state to assistive technology.
    /// Default returns `None` (view is not explicitly accessible).
    fn aria_properties(&self) -> Option<AriaProperties> {
        None
    }

    /// Handle a keyboard navigation event.
    /// Return true if consumed, false to bubble.
    fn on_key_event(&self, _key: &str, _modifiers: KeyModifiers) -> bool {
        false
    }

    /// Return keyboard shortcuts this view responds to.
    fn key_shortcuts(&self) -> Vec<KeyShortcut> {
        vec![]
    }

    /// Return true when this view's render output has changed since last build.
    /// Default false (static views skip rebuild). Override for interactive views
    /// or views backed by dynamic state.
    /// When returns false, VDom::build skips this subtree entirely.
    fn changed(&self) -> bool {
        false
    }

    /// Return true when this view needs per-frame updates (animations, timers, etc.).
    /// Default false. Views that drive continuous updates (e.g., spring animations,
    /// particle effects, real-time data feeds) should override this to return true.
    ///
    /// Unlike `changed()`, this is checked at the top-level render loop to decide
    /// whether to request a new frame at all. A static UI with no animations should
    /// keep this false to avoid unnecessary frame processing.
    fn needs_update(&self) -> bool {
        false
    }

    /// Stable identity for diff keying. Return None for anonymous views.
    /// When Some(id), the VDOM layer uses this to match nodes across rebuilds,
    /// enabling handler survival and incremental patch generation.
    fn view_id(&self) -> Option<u64> {
        None
    }
}
