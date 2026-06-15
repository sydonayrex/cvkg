//! Renderer sub-traits — logical groupings of the Renderer capability surface.
//!
//! The main `Renderer` trait (in the parent module) aggregates all of these.
//! Backends continue to implement `Renderer` as before. These sub-traits exist
//! so that consumer code can depend on only the capability slice it needs.

use super::{
    BerserkerMode, ColorTheme, Event, Mesh, Rect, TelemetryData,
};

// ══════════════════════════════════════════════════════════════════════════════
// Core — required by every backend
// ══════════════════════════════════════════════════════════════════════════════

/// Core rendering control. Every backend must implement these.
pub trait RendererCore: Send {
    /// Request a redraw as soon as possible.
    fn request_redraw(&mut self);
    /// Return true if the current frame is over its time budget.
    fn is_over_budget(&self) -> bool;
}

// ══════════════════════════════════════════════════════════════════════════════
// Shapes — 2D primitive drawing
// ══════════════════════════════════════════════════════════════════════════════

/// 2D shape drawing operations.
pub trait RendererShapes {
    fn fill_rect(&mut self, rect: Rect, color: [f32; 4]);
    fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4]);
    fn fill_ellipse(&mut self, rect: Rect, color: [f32; 4]);
    /// Fill a rounded rect with glass material for frosted backdrop effect.
    fn fill_glass_rect(&mut self, rect: Rect, radius: f32, blur_radius: f32);
    fn stroke_rect(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32);
    fn stroke_rounded_rect(
        &mut self,
        rect: Rect,
        radius: f32,
        color: [f32; 4],
        stroke_width: f32,
    );
    fn stroke_ellipse(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32);
    fn draw_line(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: [f32; 4],
        stroke_width: f32,
    );
    fn fill_polygon(&mut self, vertices: &[[f32; 2]], color: [f32; 4]) {}
    fn stroke_polygon(
        &mut self,
        vertices: &[[f32; 2]],
        color: [f32; 4],
        stroke_width: f32,
    ) {
        let _ = (vertices, color, stroke_width);
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// 3D — mesh and cube drawing
// ══════════════════════════════════════════════════════════════════════════════

/// 3D drawing operations. Optional — defaults to no-op.
pub trait Renderer3D {
    fn draw_3d_cube(&mut self, _rect: Rect, _color: [f32; 4], _rotation: [f32; 3]) {}
    fn draw_mesh(&mut self, _mesh: &Mesh, _color: [f32; 4], _transform: glam::Mat4) {}
}

// ══════════════════════════════════════════════════════════════════════════════
// Text — text layout and measurement
// ══════════════════════════════════════════════════════════════════════════════

pub trait RendererText {
    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]);
    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32);

    fn shape_rich_text(
        &mut self,
        _spans: &[cvkg_runic_text::TextSpan],
        _max_width: Option<f32>,
        _align: cvkg_runic_text::TextAlign,
        _overflow: cvkg_runic_text::TextOverflow,
    ) -> Option<cvkg_runic_text::ShapedText> {
        None
    }

    fn draw_shaped_text(&mut self, _text: &cvkg_runic_text::ShapedText, _x: f32, _y: f32) {}
}

// ══════════════════════════════════════════════════════════════════════════════
// Images — texture and image handling
// ══════════════════════════════════════════════════════════════════════════════

/// Image and texture operations.
pub trait RendererImages {
    fn draw_texture(&mut self, _texture_id: u32, _rect: Rect) {}
    fn draw_image(&mut self, _image_name: &str, _rect: Rect) {}
    fn load_image(&mut self, _name: &str, _data: &[u8]) {}
    fn prewarm_vram(&mut self, _assets: Vec<(String, Vec<u8>)>) {}
}

// ════════════════════════════════════════════════════════════════════════════──
// Data Viz — heatmap / data texture support
// ══════════════════════════════════════════════════════════════════════════════

/// Data-visualisation helpers.
pub trait RendererDataViz {
    fn upload_data_texture(
        &mut self,
        _id: &str,
        _data: &[f32],
        _width: u32,
        _height: u32,
    ) {
    }
    fn draw_heatmap(&mut self, _texture_id: &str, _rect: Rect, _palette: &str) {}
}

// ══════════════════════════════════════════════════════════════════════════════
// Vector Graphics — SVG loading and drawing
// ══════════════════════════════════════════════════════════════════════════════

/// SVG vector graphics.
pub trait RendererVectorGraphics {
    fn load_svg(&mut self, _name: &str, _svg_data: &[u8]) {}
    fn draw_svg(&mut self, _name: &str, _rect: Rect) {}
}

// ══════════════════════════════════════════════════════════════════════════════
// Effects — gradients, shadows, 9-slice, dashed strokes
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
// Clipping — scissor / clip-rect stack
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
// Transforms — 2D affine transform stack
// ══════════════════════════════════════════════════════════════════════════════

/// 2D transform stack.
pub trait RendererTransforms {
    fn push_transform(&mut self, _translation: [f32; 2], _scale: [f32; 2], _rotation: f32) {}
    fn pop_transform(&mut self) {}
}

// ══════════════════════════════════════════════════════════════════════════════
// Opacity — opacity stack
// ══════════════════════════════════════════════════════════════════════════════

/// Opacity stack operations.
pub trait RendererOpacity {
    fn push_opacity(&mut self, _opacity: f32) {}
    fn pop_opacity(&mut self) {}
}

// ══════════════════════════════════════════════════════════════════════════════
// Berserker — scene / rage / shatter pipeline
// ══════════════════════════════════════════════════════════════════════════════

/// Berserker pipeline (rage, shatter, scene presets).
pub trait RendererBerserker {
    fn set_theme(&mut self, _theme: ColorTheme) {}
    fn set_rage(&mut self, _rage: f32) {}
    fn set_berserker_mode(&mut self, _state: BerserkerMode) {}
    fn trigger_shatter_event(&mut self, _origin: [f32; 2], _force: f32) {}
    fn set_scene(&mut self, _scene: &str) {}
    fn set_scene_preset(&mut self, _preset: u32) {}
    /// Set the default background color for the canvas (RGBA).
    /// Used when the app does not draw its own background.
    fn set_default_background_color(&mut self, _color: [f32; 4]) {}
}

// ══════════════════════════════════════════════════════════════════════════════
// Export — PNG capture / print
// ══════════════════════════════════════════════════════════════════════════════

/// Frame export operations.
pub trait RendererExport {
    fn capture_png(&mut self) -> Vec<u8> {
        Vec::new()
    }
    fn print(&mut self) {}
}

// ══════════════════════════════════════════════════════════════════════════════
// Cyberpunk — bifrost, gungnir, mani, mjolnir, memoize
// ══════════════════════════════════════════════════════════════════════════════

/// Cyberpunk-specific visual effects.
pub trait RendererCyberpunk {
    fn bifrost(&mut self, _rect: Rect, _blur: f32, _saturation: f32, _opacity: f32) {}
    fn gungnir(&mut self, _rect: Rect, _color: [f32; 4], _radius: f32, _intensity: f32) {}
    /// Soft glow variant — half the intensity of gungnir(). Use for hover highlights.
    fn gungnir_soft(&mut self, _rect: Rect, _color: [f32; 4], _radius: f32, _intensity: f32) {}
    fn mani_glow(&mut self, _rect: Rect, _color: [f32; 4], _radius: f32) {}
    fn push_mjolnir_slice(&mut self, _angle: f32, _offset: f32) {}
    fn pop_mjolnir_slice(&mut self) {}
    fn memoize(&mut self, id: u64, data_hash: u64, render_fn: &dyn Fn(&mut dyn super::Renderer));
    fn mjolnir_shatter(
        &mut self,
        _rect: Rect,
        _pieces: u32,
        _force: f32,
        _color: [f32; 4],
    ) {
    }
    fn mjolnir_fluid_shatter(
        &mut self,
        _rect: Rect,
        _pieces: u32,
        _force: f32,
        _color: [f32; 4],
    ) {
    }
    fn draw_mjolnir_bolt(
        &mut self,
        _from: [f32; 2],
        _to: [f32; 2],
        _color: [f32; 4],
    ) {
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Compute & Particles — fluid simulations, generative physics
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
// Volumetric — holograms, raymarching
// ══════════════════════════════════════════════════════════════════════════════

/// Volumetric and raymarching projections.
pub trait RendererVolumetric {
    /// Draws a volumetric hologram into the specified bounding rectangle.
    fn draw_hologram(&mut self, _rect: Rect, _hologram_id: &str, _time: f32) {}
}

// ════════════════════════════════════════════════════════════════════════════──
// Accessibility — ARIA / shared elements / keys
// ══════════════════════════════════════════════════════════════════════════════

/// Accessibility helpers.
pub trait RendererAccessibility {
    fn set_aria_role(&mut self, _role: &str) {}
    fn set_aria_label(&mut self, _label: &str) {}
    fn register_shared_element(&mut self, _id: &str, _rect: Rect) {}
    fn set_key(&mut self, _key: &str) {}
}

// ══════════════════════════════════════════════════════════════════════════════
// Telemetry — frame budget / performance data
// ══════════════════════════════════════════════════════════════════════════════

/// Performance telemetry.
pub trait RendererTelemetry {
    fn get_telemetry(&self) -> TelemetryData {
        TelemetryData::default()
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// VDOM — virtual-DOM node tracking + event handler registration
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
// Z-Index — depth ordering
// ══════════════════════════════════════════════════════════════════════════════

/// Z-index ordering.
pub trait RendererZIndex {
    fn set_z_index(&mut self, _z: f32) {}
    fn get_z_index(&self) -> f32 {
        0.0
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Layout Debug — query / visualize layout
// ══════════════════════════════════════════════════════════════════════════════

/// Layout debugging helpers.
pub trait RendererLayoutDebug {
    fn query_layout(
        &self,
        _node_id: super::scene_graph::NodeId,
    ) -> Option<Rect> {
        None
    }
    fn set_debug_layout(&mut self, _enabled: bool) {}
    fn get_debug_layout(&self) -> bool {
        false
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Pointer — mouse / touch position query
// ══════════════════════════════════════════════════════════════════════════════

/// Pointer position query.
pub trait RendererPointer {
    fn get_pointer_position(&self) -> [f32; 2] {
        [0.0, 0.0]
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Material — draw call routing for multi-pass pipeline
// ══════════════════════════════════════════════════════════════════════════════

/// Material routing — controls which pass a draw call is routed to.
pub trait RendererMaterial {
    /// Set the active material for subsequent draw calls.
    fn set_material(&mut self, _material: crate::material::DrawMaterial) {}
    /// Return the currently active material.
    fn current_material(&self) -> crate::material::DrawMaterial {
        crate::material::DrawMaterial::Opaque
    }
}


