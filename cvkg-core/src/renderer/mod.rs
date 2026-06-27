//! Renderer sub-traits -- logical groupings of the Renderer capability surface.
//!
//! The main `Renderer` trait (in the parent module) aggregates all of these.
//! Backends continue to implement `Renderer` as before. These sub-traits exist
//! so that consumer code can depend on only the capability slice it needs.

use super::{ColorTheme, Event, Mesh, Rect, RenderIntensityMode, TelemetryData};

// ══════════════════════════════════════════════════════════════════════════════
// Core -- required by every backend
// ══════════════════════════════════════════════════════════════════════════════

/// Core rendering control. Every backend must implement these.
pub trait RendererCore: Send {
    /// Request a redraw as soon as possible.
    fn request_redraw(&mut self);
    /// Return true if the current frame is over its time budget.
    fn is_over_budget(&self) -> bool;
}

// ══════════════════════════════════════════════════════════════════════════════
// Shapes -- 2D primitive drawing
// ══════════════════════════════════════════════════════════════════════════════

/// 2D shape drawing operations.
///
/// All methods are defined in the main `Renderer` trait. This trait serves as
/// a capability marker for consumer code that only needs shape drawing.
pub trait RendererShapes: Send {}

// ══════════════════════════════════════════════════════════════════════════════
// 3D -- mesh and cube drawing
// ══════════════════════════════════════════════════════════════════════════════

/// 3D drawing operations. Optional -- defaults to no-op.
pub trait Renderer3D {
    fn draw_3d_cube(&mut self, _rect: Rect, _color: [f32; 4], _rotation: [f32; 3]) {}
    fn draw_mesh(&mut self, _mesh: &Mesh, _color: [f32; 4], _transform: glam::Mat4) {}
}

// ══════════════════════════════════════════════════════════════════════════════
// Text -- text layout and measurement
// ══════════════════════════════════════════════════════════════════════════════
/// Text layout and measurement.
///
/// All methods are defined in the main `Renderer` trait. This trait serves as
/// a capability marker for consumer code that only needs text rendering.
pub trait RendererText: Send {}

// ══════════════════════════════════════════════════════════════════════════════
// Images -- texture and image handling
// ══════════════════════════════════════════════════════════════════════════════

/// Image and texture operations.
pub trait RendererImages {
    fn draw_texture(&mut self, _texture_id: u32, _rect: Rect) {}
    fn draw_image(&mut self, _image_name: &str, _rect: Rect) {}
    fn load_image(&mut self, _name: &str, _data: &[u8]) {}
    fn prewarm_vram(&mut self, _assets: Vec<(String, Vec<u8>)>) {}
}

// ════════════════════════════════════════════════════════════════════════════──
// Data Viz -- heatmap / data texture support
// ══════════════════════════════════════════════════════════════════════════════

/// Data-visualisation helpers.
pub trait RendererDataViz {
    fn upload_data_texture(&mut self, _id: &str, _data: &[f32], _width: u32, _height: u32) {}
    fn draw_heatmap(&mut self, _texture_id: &str, _rect: Rect, _palette: &str) {}
}

// ══════════════════════════════════════════════════════════════════════════════
// Vector Graphics -- SVG loading and drawing
// ══════════════════════════════════════════════════════════════════════════════

/// SVG vector graphics.
pub trait RendererVectorGraphics {
    fn load_svg(&mut self, _name: &str, _svg_data: &[u8]) {}
    fn draw_svg(&mut self, _name: &str, _rect: Rect) {}
}

// ══════════════════════════════════════════════════════════════════════════════
// Effects -- gradients, shadows, 9-slice, dashed strokes
// ══════════════════════════════════════════════════════════════════════════════

/// Visual effects.
pub trait RendererEffects {
    fn draw_linear_gradient(
        &mut self,
        _rect: Rect,
        _start_color: [f32; 4],
        _end_color: [f32; 4],
        _angle: f32,
    ) {
    }
    fn draw_radial_gradient(
        &mut self,
        _rect: Rect,
        _inner_color: [f32; 4],
        _outer_color: [f32; 4],
    ) {
    }
    fn draw_drop_shadow(
        &mut self,
        _rect: Rect,
        _radius: f32,
        _color: [f32; 4],
        _blur: f32,
        _spread: f32,
    ) {
    }
    fn stroke_dashed_rounded_rect(
        &mut self,
        _rect: Rect,
        _radius: f32,
        _color: [f32; 4],
        _width: f32,
        _dash: f32,
        _gap: f32,
    ) {
    }
    fn draw_9slice(
        &mut self,
        _image_name: &str,
        _rect: Rect,
        _left: f32,
        _top: f32,
        _right: f32,
        _bottom: f32,
    ) {
    }
    fn push_shadow(&mut self, _radius: f32, _color: [f32; 4], _offset: [f32; 2]) {}
    fn pop_shadow(&mut self) {}
}

// ══════════════════════════════════════════════════════════════════════════════
// Clipping -- scissor / clip-rect stack
// ══════════════════════════════════════════════════════════════════════════════

/// Clip-rect stack operations.
pub trait RendererClipping {
    fn push_clip_rect(&mut self, _rect: Rect) {}
    fn pop_clip_rect(&mut self) {}
    fn current_clip_rect(&self) -> Rect {
        Rect::new(-10000.0, -10000.0, 20000.0, 20000.0)
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Transforms -- 2D affine transform stack
// ══════════════════════════════════════════════════════════════════════════════

/// 2D transform stack.
pub trait RendererTransforms {
    fn push_transform(&mut self, _translation: [f32; 2], _scale: [f32; 2], _rotation: f32) {}
    fn pop_transform(&mut self) {}
}

// ══════════════════════════════════════════════════════════════════════════════
// Opacity -- opacity stack
// ══════════════════════════════════════════════════════════════════════════════

/// Opacity stack operations.
pub trait RendererOpacity {
    fn push_opacity(&mut self, _opacity: f32) {}
    fn pop_opacity(&mut self) {}
}

// ══════════════════════════════════════════════════════════════════════════════
// Berserker -- scene / rage / shatter pipeline
// ══════════════════════════════════════════════════════════════════════════════

/// Berserker pipeline (rage, shatter, scene presets).
pub trait RendererBerserker {
    fn set_theme(&mut self, _theme: ColorTheme) {}
    fn set_rage(&mut self, _rage: f32) {}
    fn set_berserker_mode(&mut self, _state: RenderIntensityMode) {}
    fn trigger_shatter_event(&mut self, _origin: [f32; 2], _force: f32) {}
    fn set_scene(&mut self, _scene: &str) {}
    fn set_scene_preset(&mut self, _preset: u32) {}
    /// Set the fireball position for dynamic glass specular highlights.
    fn set_fireball_pos(&mut self, _pos: [f32; 2]) {}
    /// Set the default background color for the canvas (RGBA).
    /// Used when the app does not draw its own background.
    fn set_default_background_color(&mut self, _color: [f32; 4]) {}
}

// ══════════════════════════════════════════════════════════════════════════════
// Export -- PNG capture / print
// ══════════════════════════════════════════════════════════════════════════════

/// Frame export operations.
pub trait RendererExport {
    fn capture_png(&mut self) -> Vec<u8> {
        Vec::new()
    }
    fn print(&mut self) {}
}

// ══════════════════════════════════════════════════════════════════════════════
// Cyberpunk -- bifrost, gungnir, mani, mjolnir, memoize
// ══════════════════════════════════════════════════════════════════════════════

/// Cyberpunk-specific visual effects.
///
/// All methods except `memoize` are defined in the main `Renderer` trait.
/// `memoize` is a cyberpunk-specific optimization.
pub trait RendererCyberpunk: Send {
    fn memoize(&mut self, id: u64, data_hash: u64, render_fn: &dyn Fn(&mut dyn super::Renderer));
}

// ══════════════════════════════════════════════════════════════════════════════
// Compute & Particles -- fluid simulations, generative physics
// ══════════════════════════════════════════════════════════════════════════════

/// Generic compute and particle dispatch operations for futuristic UIs.
pub trait RendererCompute {
    /// Dispatches a burst of GPU particles (e.g. fireworks, data streams).
    fn dispatch_particles(
        &mut self,
        _origin: [f32; 2],
        _count: u32,
        _effect_type: &str,
        _color: [f32; 4],
    ) {
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Volumetric -- holograms, raymarching
// ══════════════════════════════════════════════════════════════════════════════

/// Volumetric and raymarching projections.
pub trait RendererVolumetric {
    /// Draws a volumetric hologram into the specified bounding rectangle.
    fn draw_hologram(&mut self, _rect: Rect, _hologram_id: &str, _time: f32) {}
}

// ════════════════════════════════════════════════════════════════════════════──
// Accessibility -- ARIA / shared elements / keys
// ══════════════════════════════════════════════════════════════════════════════

/// Accessibility helpers.
pub trait RendererAccessibility {
    fn set_aria_role(&mut self, _role: &str) {}
    fn set_aria_label(&mut self, _label: &str) {}
    fn set_aria_valuemin(&mut self, _min: f32) {}
    fn set_aria_valuemax(&mut self, _max: f32) {}
    fn set_aria_valuenow(&mut self, _now: f32) {}
    fn register_shared_element(&mut self, _id: &str, _rect: Rect) {}
    fn set_key(&mut self, _key: &str) {}
}

// ══════════════════════════════════════════════════════════════════════════════
// Telemetry -- frame budget / performance data
// ══════════════════════════════════════════════════════════════════════════════

/// Performance telemetry.
pub trait RendererTelemetry {
    fn get_telemetry(&self) -> TelemetryData {
        TelemetryData::default()
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// VDOM -- virtual-DOM node tracking + event handler registration
// ══════════════════════════════════════════════════════════════════════════════

/// VDOM integration.
pub trait RendererVDOM {
    fn push_vnode(&mut self, _rect: Rect, _name: &'static str) {}
    fn pop_vnode(&mut self) {}
    fn register_handler(
        &mut self,
        _event_type: &str,
        _handler: std::sync::Arc<dyn Fn(Event) + Send + Sync>,
    ) {
    }
    fn register_lifecycle(
        &mut self,
        _on_appear: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
        _on_disappear: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
    ) {
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Z-Index -- depth ordering
// ══════════════════════════════════════════════════════════════════════════════

/// Z-index ordering.
pub trait RendererZIndex {
    fn set_z_index(&mut self, _z: f32) {}
    fn get_z_index(&self) -> f32 {
        0.0
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Layout Debug -- query / visualize layout
// ══════════════════════════════════════════════════════════════════════════════

/// Layout debugging helpers.
pub trait RendererLayoutDebug {
    fn query_layout(&self, _node_id: super::scene_graph::NodeId) -> Option<Rect> {
        None
    }
    fn set_debug_layout(&mut self, _enabled: bool) {}
    fn get_debug_layout(&self) -> bool {
        false
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Pointer -- mouse / touch position query
// ══════════════════════════════════════════════════════════════════════════════

/// Pointer position query.
pub trait RendererPointer {
    fn get_pointer_position(&self) -> [f32; 2] {
        [0.0, 0.0]
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Material -- draw call routing for multi-pass pipeline
// ══════════════════════════════════════════════════════════════════════════════

/// Material routing -- controls which pass a draw call is routed to.
pub trait RendererMaterial {
    /// Set the active material for subsequent draw calls.
    fn set_material(&mut self, _material: crate::material::DrawMaterial) {}
    /// Return the currently active material.
    fn current_material(&self) -> crate::material::DrawMaterial {
        crate::material::DrawMaterial::Opaque
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Capabilities -- runtime feature detection (P2-35)
// ══════════════════════════════════════════════════════════════════════════════

/// P2-35: Capability flags for runtime feature detection.
/// Backends implement this to declare which features they support,
/// reducing the need for endless trait expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RendererCapabilities {
    pub shapes: bool,
    pub text: bool,
    pub images: bool,
    pub svg: bool,
    pub data_viz: bool,
    pub effects: bool,
    pub three_d: bool,
    pub volumetric: bool,
    pub compute: bool,
    pub accessibility: bool,
    pub export: bool,
    pub berserker: bool,
    pub cyberpunk: bool,
}

/// Returns the capabilities supported by this renderer.
/// Backends override this to declare their actual capabilities.
pub fn renderer_capabilities() -> RendererCapabilities {
    RendererCapabilities {
        shapes: true,
        text: true,
        images: true,
        svg: true,
        data_viz: false,
        effects: true,
        three_d: false,
        volumetric: false,
        compute: false,
        accessibility: true,
        export: false,
        berserker: false,
        cyberpunk: false,
    }
}
