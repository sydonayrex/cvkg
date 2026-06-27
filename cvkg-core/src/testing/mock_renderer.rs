//! MockRenderer -- test double that records all draw calls.

use crate::Renderer;
use crate::renderer::RendererCore;
use crate::renderer::{
    Renderer3D, RendererAccessibility, RendererBerserker, RendererClipping, RendererCompute,
    RendererCyberpunk, RendererDataViz, RendererEffects, RendererExport, RendererImages,
    RendererLayoutDebug, RendererMaterial, RendererOpacity, RendererPointer, RendererShapes,
    RendererTelemetry, RendererText, RendererTransforms, RendererVDOM, RendererVectorGraphics,
    RendererVolumetric, RendererZIndex,
};
use crate::*;

/// A recorded draw call from MockRenderer.
#[derive(Debug, Clone, PartialEq)]
pub enum DrawCall {
    FillRect {
        rect: Rect,
        color: [f32; 4],
    },
    FillRoundedRect {
        rect: Rect,
        radius: f32,
        color: [f32; 4],
    },
    FillEllipse {
        rect: Rect,
        color: [f32; 4],
    },
    FillGlassRect {
        rect: Rect,
        radius: f32,
        blur_radius: f32,
    },
    StrokeRect {
        rect: Rect,
        color: [f32; 4],
        stroke_width: f32,
    },
    StrokeRoundedRect {
        rect: Rect,
        radius: f32,
        color: [f32; 4],
        stroke_width: f32,
    },
    StrokeEllipse {
        rect: Rect,
        color: [f32; 4],
        stroke_width: f32,
    },
    DrawLine {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: [f32; 4],
        stroke_width: f32,
    },
    DrawText {
        text: String,
        x: f32,
        y: f32,
        size: f32,
        color: [f32; 4],
    },
    DrawTextCentered {
        text: String,
        x: f32,
        y: f32,
        size: f32,
        color: [f32; 4],
    },
    MeasureText {
        text: String,
        size: f32,
    },
    DrawShapedText {
        x: f32,
        y: f32,
    },
}

/// A test renderer that records all draw calls.
pub struct MockRenderer {
    pub calls: Vec<DrawCall>,
}

impl MockRenderer {
    pub fn new() -> Self {
        Self { calls: Vec::new() }
    }

    pub fn assert_draw_call_count(&self, expected: usize) {
        assert_eq!(
            self.calls.len(),
            expected,
            "Expected {} draw calls, got {}",
            expected,
            self.calls.len()
        );
    }

    pub fn assert_text_rendered(&self, expected_text: &str) {
        let found = self.calls.iter().any(|call| match call {
            DrawCall::DrawText { text, .. } | DrawCall::DrawTextCentered { text, .. } => {
                text.contains(expected_text)
            }
            _ => false,
        });
        assert!(
            found,
            "Expected text '{}' not found in recorded calls: {:?}",
            expected_text, self.calls
        );
    }

    pub fn assert_color_at(&self, x: f32, y: f32, expected_color: [f32; 4]) {
        let found = self.calls.iter().any(|call| match call {
            DrawCall::FillRect { rect, color }
            | DrawCall::FillRoundedRect { rect, color, .. }
            | DrawCall::FillEllipse { rect, color, .. } => {
                x >= rect.x
                    && x <= rect.x + rect.width
                    && y >= rect.y
                    && y <= rect.y + rect.height
                    && *color == expected_color
            }
            _ => false,
        });
        assert!(
            found,
            "Expected color {:?} at ({}, {}) not found in recorded calls: {:?}",
            expected_color, x, y, self.calls
        );
    }
}

impl Default for MockRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl ElapsedTime for MockRenderer {
    fn elapsed_time(&self) -> f32 {
        0.0
    }
    fn delta_time(&self) -> f32 {
        1.0 / 60.0
    }
}

impl RendererCore for MockRenderer {
    fn request_redraw(&mut self) {}
    fn is_over_budget(&self) -> bool {
        false
    }
}

impl RendererShapes for MockRenderer {}

impl RendererText for MockRenderer {}

impl Renderer3D for MockRenderer {}
impl RendererImages for MockRenderer {}
impl RendererDataViz for MockRenderer {}
impl RendererVectorGraphics for MockRenderer {}
impl RendererEffects for MockRenderer {}
impl RendererClipping for MockRenderer {}
impl RendererTransforms for MockRenderer {}
impl RendererOpacity for MockRenderer {}
impl RendererBerserker for MockRenderer {}
impl RendererExport for MockRenderer {}
impl RendererCyberpunk for MockRenderer {
    fn memoize(&mut self, _id: u64, _dh: u64, _f: &dyn Fn(&mut dyn Renderer)) {}
}
impl RendererCompute for MockRenderer {}
impl RendererVolumetric for MockRenderer {}
impl RendererAccessibility for MockRenderer {}
impl RendererTelemetry for MockRenderer {}
impl RendererVDOM for MockRenderer {}
impl RendererZIndex for MockRenderer {}
impl RendererMaterial for MockRenderer {}
impl RendererLayoutDebug for MockRenderer {}
impl RendererPointer for MockRenderer {}
impl RendererErrorHandler for MockRenderer {}

impl Renderer for MockRenderer {
    fn fill_rect(&mut self, rect: Rect, color: [f32; 4]) {
        self.calls.push(DrawCall::FillRect { rect, color });
    }
    fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4]) {
        self.calls.push(DrawCall::FillRoundedRect {
            rect,
            radius,
            color,
        });
    }
    fn fill_ellipse(&mut self, rect: Rect, color: [f32; 4]) {
        self.calls.push(DrawCall::FillEllipse { rect, color });
    }
    fn fill_glass_rect(&mut self, rect: Rect, radius: f32, blur_radius: f32) {
        self.calls.push(DrawCall::FillGlassRect {
            rect,
            radius,
            blur_radius,
        });
    }
    fn stroke_rect(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32) {
        self.calls.push(DrawCall::StrokeRect {
            rect,
            color,
            stroke_width,
        });
    }
    fn stroke_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4], stroke_width: f32) {
        self.calls.push(DrawCall::StrokeRoundedRect {
            rect,
            radius,
            color,
            stroke_width,
        });
    }
    fn stroke_ellipse(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32) {
        self.calls.push(DrawCall::StrokeEllipse {
            rect,
            color,
            stroke_width,
        });
    }
    fn draw_line(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: [f32; 4],
        stroke_width: f32,
    ) {
        self.calls.push(DrawCall::DrawLine {
            x1,
            y1,
            x2,
            y2,
            color,
            stroke_width,
        });
    }
    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        self.calls.push(DrawCall::DrawText {
            text: text.to_string(),
            x,
            y,
            size,
            color,
        });
    }
    fn draw_text_centered(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        self.calls.push(DrawCall::DrawTextCentered {
            text: text.to_string(),
            x,
            y,
            size,
            color,
        });
    }
    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        self.calls.push(DrawCall::MeasureText {
            text: text.to_string(),
            size,
        });
        (text.len() as f32 * size * 0.6, size)
    }
    fn draw_shaped_text(&mut self, _text: &cvkg_runic_text::ShapedText, x: f32, y: f32) {
        self.calls.push(DrawCall::DrawShapedText { x, y });
    }
    fn fill_polygon(&mut self, _vertices: &[[f32; 2]], _color: [f32; 4]) {}
    fn stroke_polygon(&mut self, _vertices: &[[f32; 2]], _color: [f32; 4], _stroke_width: f32) {}
    fn draw_background_image(&mut self, _image_name: &str, _rect: Rect) {}
    fn fill_glass_rect_with_intensity(
        &mut self,
        _rect: Rect,
        _radius: f32,
        _blur_radius: f32,
        _gi: f32,
    ) {
    }
    fn draw_image(&mut self, _iname: &str, _rect: Rect) {}
    fn prewarm_vram(&mut self, _assets: Vec<(String, Vec<u8>)>) {}
    fn draw_3d_cube(&mut self, _r: Rect, _c: [f32; 4], _rot: [f32; 3]) {}
    fn draw_mesh(&mut self, _m: &Mesh, _c: [f32; 4], _t: glam::Mat4) {}
    fn draw_texture(&mut self, _id: u32, _r: Rect) {}
    fn load_image(&mut self, _n: &str, _d: &[u8]) {}
    fn upload_data_texture(&mut self, _id: &str, _d: &[f32], _w: u32, _h: u32) {}
    fn draw_heatmap(&mut self, _id: &str, _r: Rect, _p: &str) {}
    fn load_svg(&mut self, _n: &str, _d: &[u8]) {}
    fn draw_svg(&mut self, _n: &str, _r: Rect) {}
    fn draw_linear_gradient(&mut self, _r: Rect, _sc: [f32; 4], _ec: [f32; 4], _a: f32) {}
    fn draw_radial_gradient(&mut self, _r: Rect, _ic: [f32; 4], _oc: [f32; 4]) {}
    fn draw_drop_shadow(&mut self, _r: Rect, _rad: f32, _c: [f32; 4], _b: f32, _s: f32) {}
    fn stroke_dashed_rounded_rect(
        &mut self,
        _r: Rect,
        _rad: f32,
        _c: [f32; 4],
        _w: f32,
        _d: f32,
        _g: f32,
    ) {
    }
    fn draw_9slice(&mut self, _n: &str, _r: Rect, _l: f32, _t: f32, _ri: f32, _b: f32) {}
    fn push_shadow(&mut self, _r: f32, _c: [f32; 4], _o: [f32; 2]) {}
    fn pop_shadow(&mut self) {}
    fn push_clip_rect(&mut self, _r: Rect) {}
    fn pop_clip_rect(&mut self) {}
    fn current_clip_rect(&self) -> Rect {
        Rect::new(-10000.0, -10000.0, 20000.0, 20000.0)
    }
    fn push_transform(&mut self, _t: [f32; 2], _s: [f32; 2], _r: f32) {}
    fn pop_transform(&mut self) {}
    fn push_opacity(&mut self, _o: f32) {}
    fn pop_opacity(&mut self) {}
    fn set_theme(&mut self, _t: ColorTheme) {}
    fn set_rage(&mut self, _r: f32) {}
    fn set_berserker_mode(&mut self, _s: RenderIntensityMode) {}
    fn trigger_shatter_event(&mut self, _o: [f32; 2], _f: f32) {}
    fn set_scene(&mut self, _s: &str) {}
    fn set_scene_preset(&mut self, _p: u32) {}
    fn set_fireball_pos(&mut self, _p: [f32; 2]) {}
    fn set_default_background_color(&mut self, _c: [f32; 4]) {}
    fn capture_png(&mut self) -> Vec<u8> {
        Vec::new()
    }
    fn print(&mut self) {}
    fn bifrost(&mut self, _r: Rect, _b: f32, _s: f32, _o: f32) {}
    fn gungnir(&mut self, _r: Rect, _c: [f32; 4], _rad: f32, _i: f32) {}
    fn gungnir_soft(&mut self, _r: Rect, _c: [f32; 4], _rad: f32, _i: f32) {}
    fn mani_glow(&mut self, _r: Rect, _c: [f32; 4], _rad: f32) {}
    fn push_mjolnir_slice(&mut self, _a: f32, _o: f32) {}
    fn pop_mjolnir_slice(&mut self) {}
    fn memoize(&mut self, _id: u64, _dh: u64, _f: &dyn Fn(&mut dyn Renderer)) {}
    fn mjolnir_shatter(&mut self, _r: Rect, _p: u32, _f: f32, _c: [f32; 4]) {}
    fn mjolnir_fluid_shatter(&mut self, _r: Rect, _p: u32, _f: f32, _c: [f32; 4]) {}
    fn draw_mjolnir_bolt(&mut self, _f: [f32; 2], _t: [f32; 2], _c: [f32; 4]) {}
    fn dispatch_particles(&mut self, _o: [f32; 2], _c: u32, _e: &str, _col: [f32; 4]) {}
    fn draw_hologram(&mut self, _r: Rect, _id: &str, _t: f32) {}
    fn set_aria_role(&mut self, _r: &str) {}
    fn set_aria_label(&mut self, _l: &str) {}
    fn set_aria_valuemin(&mut self, _m: f32) {}
    fn set_aria_valuemax(&mut self, _m: f32) {}
    fn set_aria_valuenow(&mut self, _n: f32) {}
    fn register_shared_element(&mut self, _id: &str, _r: Rect) {}
    fn set_key(&mut self, _k: &str) {}
    fn get_telemetry(&self) -> TelemetryData {
        TelemetryData::default()
    }
    fn push_vnode(&mut self, _r: Rect, _n: &'static str) {}
    fn pop_vnode(&mut self) {}
    fn register_handler(&mut self, _et: &str, _h: std::sync::Arc<dyn Fn(Event) + Send + Sync>) {}
    fn set_z_index(&mut self, _z: f32) {}
    fn get_z_index(&self) -> f32 {
        0.0
    }
    fn query_layout(&self, _nid: crate::scene_graph::NodeId) -> Option<Rect> {
        None
    }
    fn set_debug_layout(&mut self, _e: bool) {}
    fn get_debug_layout(&self) -> bool {
        false
    }
    fn get_pointer_position(&self) -> [f32; 2] {
        [0.0, 0.0]
    }
    fn set_material(&mut self, _m: crate::material::DrawMaterial) {}
    fn current_material(&self) -> crate::material::DrawMaterial {
        crate::material::DrawMaterial::Opaque
    }
}
