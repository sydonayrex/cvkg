use crate::*;
use crate::error_types::CvkgError;

pub trait ElapsedTime {
    /// Returns the cumulative time since the renderer started in seconds.
    fn elapsed_time(&self) -> f32;

    /// Returns the time elapsed since the last frame in seconds.
    fn delta_time(&self) -> f32;
}

/// The Renderer trait defines the atomic drawing operations for all CVKG backends.
/// This trait is object-safe and used by the View::render system.
/// # Implementation Requirements
/// 1. Coordinate system is origin-top-left (0,0) with Y increasing downwards.
/// 2. Colors are [R, G, B, A] in the [0.0, 1.0] range.
/// 3. All operations must be batchable by the underlying backend.
///
/// Sub-traits in `renderer/mod.rs` (RendererCore, RendererShapes, etc.) are
/// capability markers. Backends implement the monolithic `Renderer` trait.
/// The sub-traits exist so consumer code can depend on only the capability
/// slice it needs (e.g., `fn render<R: RendererShapes>(shapes: R)`).
/// Callback interface for renderer error reporting.
///
/// Backends override `on_render_error` to intercept non-fatal errors that occur
/// during drawing operations. The default implementation logs the error.
///
/// Design note: `render()` stays infallible to avoid proliferating `Result`
/// through the entire View trait hierarchy. Errors that cannot be recovered from
/// within a draw call are routed through this trait method instead.
pub trait RendererErrorHandler {
    /// Called when a non-fatal render error occurs during a draw operation.
    /// The renderer continues operating. Backends should log and optionally
    /// track error counts for health monitoring.
    fn on_render_error(&mut self, error: &CvkgError) {
        tracing::error!("[RenderError] {error}");
    }

    /// Called when a fatal error occurs that prevents further rendering.
    /// The backend should attempt graceful shutdown.
    fn on_fatal_error(&mut self, error: &CvkgError) {
        tracing::error!("[Fatal] {error}");
    }

    /// Returns true if the backend is in an error state.
    fn has_error(&self) -> bool {
        false
    }
}

pub trait Renderer: ElapsedTime + Send + RendererErrorHandler {
    /// Requests that the renderer redraws as soon as possible.
    /// Used for continuous animations.
    fn request_redraw(&mut self) {}

    /// Returns true if the current frame is over the time budget.
    /// This can be used to skip expensive visual effects.
    fn is_over_budget(&self) -> bool {
        false
    }

    // ── Filled shapes ────────────────────────────────────────────────────
    fn fill_rect(&mut self, rect: Rect, color: [f32; 4]);
    fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4]);
    /// Fill an ellipse/circle that fits inside `rect`.
    fn fill_ellipse(&mut self, rect: Rect, color: [f32; 4]);

    /// Draw a background image that fills the entire rect.
    /// This is a convenience wrapper around `draw_image` for the common case
    /// of a full-rect background. The image must have been pre-warmed via
    /// `prewarm_vram` before the first frame.
    fn draw_background_image(&mut self, image_name: &str, rect: Rect) {
        Renderer::draw_image(self, image_name, rect);
    }

    /// Fill a rounded rect with glass material for frosted backdrop effect.
    /// This is the proper way to render glass cards for macOS Tahoe-style blur.
    /// The blur_radius controls the intensity of the backdrop blur.
    fn fill_glass_rect(&mut self, rect: Rect, radius: f32, blur_radius: f32) {
        // Default no-op implementation; GPU backend overrides
        let _ = (rect, radius, blur_radius);
    }
    /// Fill a rounded rect with glass material with explicit intensity control.
    /// `glass_intensity` ranges from 0.0 (solid) to 1.0 (full glass). Default: 1.0.
    fn fill_glass_rect_with_intensity(
        &mut self,
        rect: Rect,
        radius: f32,
        blur_radius: f32,
        glass_intensity: f32,
    ) {
        let _ = (rect, radius, blur_radius, glass_intensity);
    }
    /// Fill a rounded rect with glass material with explicit tint color and intensity.
    /// `tint_color` is the glass fill color (RGBA). `glass_intensity` ranges from 0.0 (solid) to 1.0 (full glass).
    fn fill_glass_rect_with_tint(
        &mut self,
        rect: Rect,
        radius: f32,
        blur_radius: f32,
        tint_color: [f32; 4],
        glass_intensity: f32,
    ) {
        // Default: delegate to intensity-only version using tint color as a simple fill
        let _ = (rect, radius, blur_radius, tint_color, glass_intensity);
    }
    /// Fill a rounded rect with glass material, modulated by touch pressure.
    /// `pressure` ranges from 0.0 (no touch) to 1.0 (full pressure).
    /// When pressure > 0, refraction distortion is scaled by pressure amount.
    /// Desktop stub: pressure is always 1.0 for mouse clicks, 0.0 otherwise.
    fn fill_glass_rect_with_pressure(
        &mut self,
        rect: Rect,
        radius: f32,
        blur_radius: f32,
        pressure: f32,
    ) {
        // Default: delegate to standard glass with intensity = pressure
        Renderer::fill_glass_rect_with_intensity(self, rect, radius, blur_radius, pressure);
    }

    /// Fill a squircle (superellipse) for Apple-style icon silhouettes.
    /// `n` controls the squareness: 2.0 = rounded rect, 4.0 = classic squircle, higher = more square.
    fn fill_squircle(&mut self, rect: Rect, _n: f32, color: [f32; 4]) {
        // Default fallback to rounded rect
        Renderer::fill_rounded_rect(self, rect, rect.width.min(rect.height) * 0.22, color);
    }

    /// Stroke a squircle (superellipse) outline.
    fn stroke_squircle(&mut self, rect: Rect, _n: f32, color: [f32; 4], stroke_width: f32) {
        Renderer::stroke_rounded_rect(
            self,
            rect,
            rect.width.min(rect.height) * 0.22,
            color,
            stroke_width,
        );
    }

    /// Draw a focus ring around a rect (for keyboard navigation accessibility).
    /// `offset` is the gap between the rect and the ring, `width` is the ring thickness.
    fn draw_focus_ring(
        &mut self,
        rect: Rect,
        radius: f32,
        offset: f32,
        width: f32,
        color: [f32; 4],
    ) {
        // Default fallback to a stroked rounded rect
        let ring_rect = Rect {
            x: rect.x - offset,
            y: rect.y - offset,
            width: rect.width + 2.0 * offset,
            height: rect.height + 2.0 * offset,
        };
        Renderer::stroke_rounded_rect(self, ring_rect, radius + offset, color, width);
    }

    /// Draw a high-fidelity 3D cube inside the given rectangle using specialized shader logic.
    /// `rotation` is [pitch, yaw, roll] in radians.
    fn draw_3d_cube(&mut self, _rect: Rect, _color: [f32; 4], _rotation: [f32; 3]) {}

    // ── Stroked shapes ───────────────────────────────────────────────────
    fn stroke_rect(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32);
    fn stroke_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4], stroke_width: f32);
    /// Stroke an ellipse/circle that fits inside `rect`.
    fn stroke_ellipse(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32);
    /// Draw a straight line from (x1,y1) to (x2,y2).
    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: [f32; 4], stroke_width: f32);
    /// Fill a polygon defined by a set of vertices.
    fn fill_polygon(&mut self, _vertices: &[[f32; 2]], _color: [f32; 4]) {}
    /// Stroke a polygon defined by a set of vertices.
    fn stroke_polygon(&mut self, _vertices: &[[f32; 2]], _color: [f32; 4], _stroke_width: f32) {}

    // ── Text ─────────────────────────────────────────────────────────────
    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        let r = (color[0] * 255.0).clamp(0.0, 255.0) as u8;
        let g = (color[1] * 255.0).clamp(0.0, 255.0) as u8;
        let b = (color[2] * 255.0).clamp(0.0, 255.0) as u8;
        let a = (color[3] * 255.0).clamp(0.0, 255.0) as u8;

        let mut style = cvkg_runic_text::TextStyle::new("Inter", size);
        style.color = [r, g, b, a];
        let spans = [cvkg_runic_text::TextSpan::new(text, style)];

        if let Some(shaped) = self.shape_rich_text(
            &spans,
            None,
            cvkg_runic_text::TextAlign::Start,
            cvkg_runic_text::TextOverflow::Visible,
        ) {
            self.draw_shaped_text(&shaped, x, y);
        }
    }

    /// Draw centered text at the given position.
    fn draw_text_centered(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        self.draw_text(text, x, y, size, color)
    }

    /// Measure the width and height of the specified text.
    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        let span = cvkg_runic_text::TextSpan::new(
            text,
            cvkg_runic_text::TextStyle {
                family: "Inter".to_string(),
                font_size: size,
                fallback_families: vec![
                    "SF Pro".to_string(),
                    "SF Pro Text".to_string(),
                    "Helvetica Neue".to_string(),
                    "Helvetica".to_string(),
                    "Arial".to_string(),
                    "sans-serif".to_string(),
                ],
                ..Default::default()
            },
        );
        if let Some(shaped) = Renderer::shape_rich_text(
            self,
            &[span],
            None,
            cvkg_runic_text::TextAlign::Start,
            cvkg_runic_text::TextOverflow::Visible,
        ) {
            let scale = self.text_scale_factor().max(1.0);
            (shaped.width / scale, shaped.height / scale)
        } else {
            (0.0, 0.0)
        }
    }

    /// Return the baseline offset (ascent) for the given text and size.
    /// This is the distance from the text origin (y in draw_text) to the baseline.
    /// Default returns 0.0; override in renderers that support text shaping.
    fn measure_text_baseline(&mut self, text: &str, size: f32) -> f32 {
        let span = cvkg_runic_text::TextSpan::new(
            text,
            cvkg_runic_text::TextStyle {
                family: "Inter".to_string(),
                font_size: size,
                fallback_families: vec![
                    "SF Pro".to_string(),
                    "SF Pro Text".to_string(),
                    "Helvetica Neue".to_string(),
                    "Helvetica".to_string(),
                    "Arial".to_string(),
                    "sans-serif".to_string(),
                ],
                ..Default::default()
            },
        );
        if let Some(shaped) = Renderer::shape_rich_text(
            self,
            &[span],
            None,
            cvkg_runic_text::TextAlign::Start,
            cvkg_runic_text::TextOverflow::Visible,
        ) {
            shaped.ascent / self.text_scale_factor().max(1.0)
        } else {
            0.0
        }
    }

    /// Scale factor used by text measurement helpers.
    ///
    /// Renderers that shape text in device pixels should return their current
    /// device scale so `measure_text` and `measure_text_baseline` stay in logical pixels.
    fn text_scale_factor(&self) -> f32 {
        1.0
    }

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

    // ── Images & textures ────────────────────────────────────────────────
    /// Draw a texture (GPU-side) at the specified rect.
    fn draw_texture(&mut self, _texture_id: u32, _rect: Rect) {}
    /// Draw an image asset by name or path.
    fn draw_image(&mut self, _image_name: &str, _rect: Rect) {}
    /// Load an image asset from memory.
    fn load_image(&mut self, _name: &str, _data: &[u8]) {}
    /// Pre-warm the renderer with assets. Implementations can use this
    /// to populate texture atlases or warm up shader caches.
    fn prewarm_vram(&mut self, _assets: Vec<(String, Vec<u8>)>) {}

    /// Get the current pointer (mouse/touch) position.
    fn get_pointer_position(&self) -> [f32; 2] {
        [0.0, 0.0]
    }

    // ── Data Visualization ───────────────────────────────────────────────
    /// Upload raw float data as a GPU texture for heatmap rendering.
    fn upload_data_texture(&mut self, _id: &str, _data: &[f32], _width: u32, _height: u32) {}
    /// Draw a heatmap using a previously uploaded data texture.
    fn draw_heatmap(&mut self, _texture_id: &str, _rect: Rect, _palette: &str) {}

    // ── 3D Objects ───────────────────────────────────────────────────────
    /// Draw a 3D mesh.
    fn draw_mesh(&mut self, _mesh: &Mesh, _color: [f32; 4], _transform: glam::Mat4) {}

    /// Draw a 3D mesh with full material and transform support.
    fn draw_mesh_3d(&mut self, _mesh: &Mesh, _material: &Material3D, _transform: &Transform3D) {}

    /// Set the 3D camera for perspective/orthographic projection.
    /// If not called, rendering defaults to the 2D orthographic projection.
    fn set_camera_3d(&mut self, _camera: &Camera3D) {}

    /// Push a 3D transform onto the transform stack.
    /// All subsequent drawing is affected until `pop_transform_3d`.
    fn push_transform_3d(&mut self, _transform: &Transform3D) {}

    /// Pop the most recently pushed 3D transform.
    fn pop_transform_3d(&mut self) {}

    /// Render a 3D scene graph node. Reads position_3d, rotation_3d, scale_3d
    /// from the node and emits the appropriate draw call.
    /// Default implementation is a no-op; 3D renderers override this.
    ///
    /// `position`: [x, y, z] world-space position
    /// `rotation`: [x, y, z, w] quaternion rotation
    /// `scale`: [x, y, z] scale factors
    /// `color`: [r, g, b, a] base color for unlit rendering
    fn render_scene_node_3d(
        &mut self,
        _position: [f32; 3],
        _rotation: [f32; 4],
        _scale: [f32; 3],
        _color: [f32; 4],
        _meshes: &[Mesh],
    ) {
        // Default no-op: 2D renderers ignore 3D scene nodes
    }

    /// Draw a linear gradient between two colors at the specified angle.
    fn draw_linear_gradient(
        &mut self,
        _rect: Rect,
        _start_color: [f32; 4],
        _end_color: [f32; 4],
        _angle: f32,
    ) {
    }
    /// Draw a radial gradient between two colors.
    fn draw_radial_gradient(
        &mut self,
        _rect: Rect,
        _inner_color: [f32; 4],
        _outer_color: [f32; 4],
    ) {
    }
    /// Draw a multi-stop linear gradient (GPU-accelerated).
    /// stops: array of [R, G, B, position] where position is 0.0-1.0.
    /// angle: gradient angle in radians.
    fn draw_linear_gradient_multi(&mut self, _rect: Rect, _stops: &[[f32; 4]], _angle: f32) {}
    /// Draw a multi-stop radial gradient (GPU-accelerated).
    /// stops: array of [R, G, B, position] where position is 0.0-1.0.
    fn draw_radial_gradient_multi(&mut self, _rect: Rect, _stops: &[[f32; 4]]) {}
    /// Draw a high-fidelity drop shadow for a rounded rectangle.
    fn draw_drop_shadow(
        &mut self,
        _rect: Rect,
        _radius: f32,
        _color: [f32; 4],
        _blur: f32,
        _spread: f32,
    ) {
    }
    /// Draw a dashed border for a rounded rectangle.
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
    /// Draw a 9-slice / patch scaled image.
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

    // ── Clipping ─────────────────────────────────────────────────────────
    /// Push a clip rectangle.  All subsequent drawing is clipped to `rect`.
    /// Implementations that do not support clipping may ignore this call.
    fn push_clip_rect(&mut self, _rect: Rect) {}
    /// Pop the most recently pushed clip rectangle.
    fn pop_clip_rect(&mut self) {}
    /// Get the current clip rectangle in screen coordinates.
    /// Returns a rect covering the entire screen if no clip is active.
    fn current_clip_rect(&self) -> Rect {
        Rect::new(-10000.0, -10000.0, 20000.0, 20000.0)
    }

    // ── Global opacity ───────────────────────────────────────────────────
    /// Set a global opacity multiplier applied to all subsequent draw calls
    /// until `pop_opacity` is called.  `opacity` is in [0.0, 1.0].
    fn push_opacity(&mut self, _opacity: f32) {}
    /// Restore the previous opacity level.
    fn pop_opacity(&mut self) {}

    // ── Berserker Pipeline State ─────────────────────────────────────────
    fn set_theme(&mut self, _theme: ColorTheme) {}
    fn set_rage(&mut self, _rage: f32) {}
    fn set_berserker_mode(&mut self, _state: RenderIntensityMode) {}
    fn trigger_shatter_event(&mut self, _origin: [f32; 2], _force: f32) {}
    /// Set the fireball position for dynamic glass specular highlights.
    fn set_fireball_pos(&mut self, _pos: [f32; 2]) {}
    /// Set the desktop scene preset (Aurora, Void, Nebula, Glitch, Yggdrasil).
    fn set_scene(&mut self, _scene: &str) {}
    /// Set the desktop scene by name. Case-insensitive.
    /// Supports: "aurora", "void", "nebula", "glitch", "yggdrasil".
    /// Aliases: "empty", "none", "blank" → Void.
    fn set_scene_by_name(&mut self, name: &str) {
        if let Some(preset) = resolve_scene_by_name(name) {
            Renderer::set_scene_preset(self, preset);
        }
    }

    // ── Export & Print ───────────────────────────────────────────────────
    /// Capture the current frame as a PNG byte buffer.
    fn capture_png(&mut self) -> Vec<u8> {
        Vec::new()
    }
    /// Trigger a native print dialog or spooling operation.
    fn print(&mut self) {}

    fn set_scene_preset(&mut self, _preset: u32) {}

    // ── Cyberpunk Effects ────────────────────────────────────────────────
    /// Apply a Bifrost (Frosted Glass) effect to the specified rect.
    fn bifrost(&mut self, _rect: Rect, _blur: f32, _saturation: f32, _opacity: f32) {}
    /// Apply a Gungnir (Neon Glow) effect to the specified rect.
    fn gungnir(&mut self, _rect: Rect, _color: [f32; 4], _radius: f32, _intensity: f32) {}
    /// Soft glow variant -- half the intensity of gungnir(). Use for hover highlights.
    fn gungnir_soft(&mut self, _rect: Rect, _color: [f32; 4], _radius: f32, _intensity: f32) {}
    /// Set the default background color for the canvas (RGBA).
    /// Used when the app does not draw its own background.
    fn set_default_background_color(&mut self, _color: [f32; 4]) {}
    /// Apply a ManiGlow (Lunar Illuminator) effect.
    fn mani_glow(&mut self, _rect: Rect, _color: [f32; 4], _radius: f32) {}
    /// Push a Mjolnir Slice (geometric clipping).
    fn push_mjolnir_slice(&mut self, _angle: f32, _offset: f32) {}
    fn pop_mjolnir_slice(&mut self) {}
    /// Execute a render function with memoization.
    /// If the renderer supports caching and the `id` + `data_hash` match a previous run,
    /// it may replay cached commands instead of executing the function.
    fn memoize(&mut self, id: u64, data_hash: u64, render_fn: &dyn Fn(&mut dyn Renderer));
    /// Capture current renderer stack depths for later panic recovery.
    /// The default implementation returns `RenderStateSnapshot::default()`,
    /// which is safe but does nothing useful -- backends with stack state
    /// must override this to record their actual depths.
    fn snapshot_render_state(&self) -> RenderStateSnapshot {
        RenderStateSnapshot::default()
    }
    /// Restore renderer stack state by popping items pushed beyond the
    /// snapshot point. Used by `ErrorBoundary` to recover from mid-render
    /// panics so sibling views don't inherit leaked clip/opacity/transform
    /// state. Idempotent: a no-op if stacks are already at or below the
    /// snapshot depths. Default implementation is a no-op for backends
    /// that have no stack state.
    fn restore_render_state(&mut self, _snap: RenderStateSnapshot) {}
    /// Apply a Mjolnir Shatter effect (fragmentation) to the specified rect.
    fn mjolnir_shatter(&mut self, _rect: Rect, _pieces: u32, _force: f32, _color: [f32; 4]) {}
    fn mjolnir_fluid_shatter(&mut self, _rect: Rect, _pieces: u32, _force: f32, _color: [f32; 4]) {}
    fn draw_mjolnir_bolt(&mut self, _from: [f32; 2], _to: [f32; 2], _color: [f32; 4]) {}

    // ── Futuristic UI Compute & Volumetric ───────────────────────────────
    /// Dispatches a burst of GPU particles (e.g. fireworks, data streams).
    fn dispatch_particles(
        &mut self,
        _origin: [f32; 2],
        _count: u32,
        _effect_type: &str,
        _color: [f32; 4],
    ) {
    }

    /// Draws a volumetric hologram into the specified bounding rectangle.
    fn draw_hologram(&mut self, _rect: Rect, _hologram_id: &str, _time: f32) {}

    // ── Accessibility (ShieldWall) ───────────────────────────────────────
    fn set_aria_role(&mut self, _role: &str) {}
    fn set_aria_label(&mut self, _label: &str) {}
    fn set_aria_valuemin(&mut self, _min: f32) {}
    fn set_aria_valuemax(&mut self, _max: f32) {}
    fn set_aria_valuenow(&mut self, _now: f32) {}

    /// Push a focus trap onto the stack. While active, keyboard focus is
    /// trapped within the specified element and its children.
    /// Returns a trap ID that must be passed to `pop_focus_trap`.
    fn push_focus_trap(&mut self, _element_id: &str) -> u64 {
        0
    }

    /// Pop the most recently pushed focus trap.
    fn pop_focus_trap(&mut self, _trap_id: u64) {}

    /// Register a shared element for Bifrost Bridge transitions.
    fn register_shared_element(&mut self, _id: &str, _rect: Rect) {}

    /// Set a unique key for the current VDOM node to ensure stable identity during diffing.
    fn set_key(&mut self, _key: &str) {}

    // ── Telemetry ────────────────────────────────────────────────────────
    /// Get real-time performance telemetry.
    fn get_telemetry(&self) -> TelemetryData {
        TelemetryData::default()
    }

    // ── GPU State Management ─────────────────────────────────────────────
    /// Push a shadow state to the stack. All following draw calls will have this shadow.
    fn push_shadow(&mut self, _radius: f32, _color: [f32; 4], _offset: [f32; 2]) {}
    /// Pop the last shadow state from the stack.
    fn pop_shadow(&mut self) {}

    // ── VDOM & Scene Graph ───────────────────────────────────────────────
    /// Push a Virtual DOM node onto the stack for hierarchy tracking.
    fn push_vnode(&mut self, _rect: Rect, _name: &'static str) {}
    /// Pop the current Virtual DOM node from the stack.
    fn pop_vnode(&mut self) {}
    /// Register an event handler for the current VDOM node.
    fn register_handler(
        &mut self,
        _event_type: &str,
        _handler: std::sync::Arc<dyn Fn(Event) + Send + Sync>,
    ) {
    }

    // ── Z-Index & Depth ──────────────────────────────────────────────────
    /// Set the current Z-index for depth sorting.
    /// Higher values appear closer to the viewer.
    fn set_z_index(&mut self, _z: f32) {}
    /// Get the current Z-index.
    fn get_z_index(&self) -> f32 {
        0.0
    }

    // ── Vector Graphics ──────────────────────────────────────────────────
    /// Load an SVG model from raw bytes.
    fn load_svg(&mut self, _name: &str, _svg_data: &[u8]) {}
    /// Draw a pre-loaded SVG model.
    fn draw_svg(&mut self, _name: &str, _rect: Rect) {}
    /// Draw a pre-loaded SVG model with a per-instance animation time offset.
    /// The offset shifts the animation phase, allowing multiple draws of the same
    /// SVG to animate independently. Default delegates to draw_svg (no offset).
    fn draw_svg_with_offset(&mut self, name: &str, rect: Rect, _animation_time_offset: f32) {
        Renderer::draw_svg(self, name, rect);
    }
    /// Draw a pre-loaded SVG model with explicit draw_order for z-sorting.
    /// draw_order=200 renders above UI chrome (draw_order=0).
    fn draw_svg_with_order(&mut self, name: &str, rect: Rect, _draw_order: i32) {
        Renderer::draw_svg(self, name, rect);
    }
    /// Serialize a pre-loaded SVG model back to SVG XML markup.
    /// Returns the serialized SVG string, or an error if the model is not loaded
    /// or serialization is not supported by this renderer.
    fn serialize_svg(&mut self, _name: &str) -> Result<String, String> {
        Err("SVG serialization not supported by this renderer".into())
    }
    /// Apply an SVG filter to a pre-loaded SVG model by filter element ID.
    /// The filter is evaluated and the result composited back into the SVG.
    /// Returns the filtered SVG as XML, or an error if not supported.
    fn apply_svg_filter(
        &mut self,
        _name: &str,
        _filter_id: &str,
        _region: Rect,
    ) -> Result<String, String> {
        Err("SVG filter not supported by this renderer".into())
    }

    // ── GPU Transformations ──────────────────────────────────────────────
    /// Push a 2D transform (translation, scale, rotation) onto the stack.
    /// This transform should be applied to all subsequent draw calls until popped.
    /// Transform-only animations use this to avoid re-triggering the layout engine.
    fn push_transform(&mut self, _translation: [f32; 2], _scale: [f32; 2], _rotation: f32) {}
    /// Push a raw 2D affine transform matrix [a, b, c, d, e, f] corresponding to
    /// [m11, m12, m21, m22, tx, ty].
    fn push_affine(&mut self, _transform: [f32; 6]) {}
    /// Pop the last 2D transform from the stack.
    fn pop_transform(&mut self) {}
    /// Return the resolved layout bounds for a specific node ID if it exists.
    fn query_layout(&self, _node_id: scene_graph::NodeId) -> Option<Rect> {
        None
    }
    /// Enable or disable the layout debug overlay (bounds, padding, margin).
    fn set_debug_layout(&mut self, _enabled: bool) {}
    /// Check if the layout debug overlay is currently enabled.
    fn get_debug_layout(&self) -> bool {
        false
    }

    // ── Material Routing ─────────────────────────────────────────────────
    /// Set the active material for subsequent draw calls.
    /// Controls which pass a draw call is routed to in the multi-pass pipeline.
    fn set_material(&mut self, _material: crate::material::DrawMaterial) {}
    /// Return the currently active material (defaults to Opaque).
    fn current_material(&self) -> crate::material::DrawMaterial {
        crate::material::DrawMaterial::Opaque
    }

    // ── Vili Interaction Paradigm ──────────────────────────────────────────
    /// Compute the user's velocity/intent vector.
    fn mimir_intent(&self) -> [f32; 2] {
        [0.0, 0.0]
    }
    /// Calculate magnetic coordinate warp towards an anchor.
    fn magnetic_warp(&self, pointer: [f32; 2], anchor_rect: Rect, strength: f32) -> [f32; 2] {
        if strength <= 0.0 {
            return pointer;
        }
        let cx = anchor_rect.x + anchor_rect.width / 2.0;
        let cy = anchor_rect.y + anchor_rect.height / 2.0;
        let dx = pointer[0] - cx;
        let dy = pointer[1] - cy;
        let dist = (dx * dx + dy * dy).sqrt();
        let radius = 120.0;
        if dist < radius && dist > 0.0 {
            let force = (1.0 - dist / radius) * strength;
            [pointer[0] - dx * force, pointer[1] - dy * force]
        } else {
            pointer
        }
    }
    /// Calculate kinematic glow intensity based on proximity.
    fn mani_glow_intensity(&self, pointer: [f32; 2], bounds: Rect, radius: f32) -> f32 {
        let cx = bounds.x + bounds.width / 2.0;
        let cy = bounds.y + bounds.height / 2.0;
        let dist = ((pointer[0] - cx).powi(2) + (pointer[1] - cy).powi(2)).sqrt();
        if dist < radius {
            (1.0 - dist / radius).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
    /// Calculate dynamic element attention (scaling/morphing) statelessly per frame.
    fn fafnir_evolve(&self, pointer: [f32; 2], bounds: Rect, max_scale: f32) -> f32 {
        let prox = self.mani_glow_intensity(pointer, bounds, 120.0);
        1.0 + (max_scale - 1.0) * prox
    }
    /// Sets the precise Vili SDF Shape boundary for hit-testing.
    fn set_sdf_shape(&mut self, _shape: crate::layout::SdfShape) {}

    // -- Portal / PhaseGate rendering -----------------------------------------

    /// Begin rendering into the portal root layer instead of the inline tree.
    /// All draw calls between `enter_portal` and `exit_portal` are collected
    /// into a separate buffer that is composited AFTER the main tree.
    ///
    /// WHY separate buffer: The main tree may have clipping, transforms, or
    /// opacity that should NOT affect overlays. The portal layer renders on top
    /// of everything, ignoring the local coordinate system.
    fn enter_portal(&mut self, _z_index: i32) {}

    /// Exit the portal layer and return to inline rendering.
    /// The portal content collected since `enter_portal` is now sealed --
    /// no more draw calls will be appended to it.
    fn exit_portal(&mut self) {}

    /// Get the current viewport size in logical pixels.
    /// Used by portal content to size itself to the full screen.
    fn viewport_size(&self) -> Rect {
        Rect::new(0.0, 0.0, 1920.0, 1080.0)
    }

    // -- Accessibility announcements -----------------------------------------

    /// Announce a message to screen readers via the platform accessibility API.
    /// This call is non-blocking. The message is queued and the screen reader
    /// will speak it at its own pace.
    fn announce(&mut self, _message: &str, _priority: AnnouncementPriority) {}
}
