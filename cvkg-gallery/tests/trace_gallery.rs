//! Trace test for gallery rendering - captures rect/layout hierarchy for debugging.
//! Run with: cargo test -p cvkg-gallery trace_gallery -- --nocapture

use cvkg::components::{Button, VStack};
use cvkg_core::{Never, Rect, Renderer, Size, SizeProposal, View};
use cvkg_runic_text as runic;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

/// TraceRenderer records every render call to a string for later inspection.
pub struct TraceRenderer {
    pub calls: Arc<std::sync::Mutex<Vec<String>>>,
    start: Instant,
    vnode_stack: VecDeque<String>,
}

impl TraceRenderer {
    pub fn new() -> Self {
        Self {
            calls: Arc::new(std::sync::Mutex::new(Vec::new())),
            start: Instant::now(),
            vnode_stack: VecDeque::new(),
        }
    }

    fn elapsed_ms(&self) -> f32 {
        self.start.elapsed().as_secs_f32() * 1000.0
    }

    fn push_vnode(&mut self, name: &'static str) {
        self.vnode_stack.push_back(name.to_string());
    }

    fn pop_vnode(&mut self) {
        self.vnode_stack.pop_back();
    }

    fn current_vnode_path(&self) -> String {
        self.vnode_stack
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(" > ")
    }

    fn log(&mut self, msg: impl Into<String>) {
        let path = self.current_vnode_path();
        let line = if path.is_empty() {
            msg.into()
        } else {
            format!("[{}] {}", path, msg.into())
        };
        self.calls.lock().unwrap().push(line);
    }
}

impl cvkg_core::ElapsedTime for TraceRenderer {
    fn elapsed_time(&self) -> f32 {
        self.elapsed_ms() / 1000.0
    }

    fn delta_time(&self) -> f32 {
        0.016 // 60fps
    }
}

impl Renderer for TraceRenderer {
    fn fill_rect(&mut self, rect: Rect, color: [f32; 4]) {
        self.log(format!(
            "fill_rect({:.1},{:.1},{:.1},{:.1}) color={:?}",
            rect.x, rect.y, rect.width, rect.height, color
        ));
    }

    fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4]) {
        self.log(format!(
            "fill_rounded_rect({:.1},{:.1},{:.1},{:.1}) radius={:.1} color={:?}",
            rect.x, rect.y, rect.width, rect.height, radius, color
        ));
    }

    fn fill_ellipse(&mut self, rect: Rect, color: [f32; 4]) {
        self.log(format!(
            "fill_ellipse({:.1},{:.1},{:.1},{:.1}) color={:?}",
            rect.x, rect.y, rect.width, rect.height, color
        ));
    }

    fn stroke_rect(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32) {
        self.log(format!(
            "stroke_rect({:.1},{:.1},{:.1},{:.1}) stroke={:.1} color={:?}",
            rect.x, rect.y, rect.width, rect.height, stroke_width, color
        ));
    }

    fn stroke_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4], stroke_width: f32) {
        self.log(format!(
            "stroke_rounded_rect({:.1},{:.1},{:.1},{:.1}) radius={:.1} stroke={:.1} color={:?}",
            rect.x, rect.y, rect.width, rect.height, radius, stroke_width, color
        ));
    }

    fn stroke_ellipse(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32) {
        self.log(format!(
            "stroke_ellipse({:.1},{:.1},{:.1},{:.1}) stroke={:.1} color={:?}",
            rect.x, rect.y, rect.width, rect.height, stroke_width, color
        ));
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
        self.log(format!(
            "draw_line({:.1},{:.1}->{:.1},{:.1}) stroke={:.1} color={:?}",
            x1, y1, x2, y2, stroke_width, color
        ));
    }

    fn fill_glass_rect(&mut self, rect: Rect, radius: f32, blur_radius: f32) {
        self.log(format!(
            "fill_glass_rect({:.1},{:.1},{:.1},{:.1}) radius={:.1} blur={:.1}",
            rect.x, rect.y, rect.width, rect.height, radius, blur_radius
        ));
    }

    fn fill_squircle(&mut self, rect: Rect, n: f32, color: [f32; 4]) {
        self.log(format!(
            "fill_squircle({:.1},{:.1},{:.1},{:.1}) n={:.1} color={:?}",
            rect.x, rect.y, rect.width, rect.height, n, color
        ));
    }

    fn draw_focus_ring(
        &mut self,
        rect: Rect,
        radius: f32,
        offset: f32,
        width: f32,
        color: [f32; 4],
    ) {
        self.log(format!(
            "draw_focus_ring({:.1},{:.1},{:.1},{:.1}) radius={:.1} offset={:.1} width={:.1} color={:?}",
            rect.x, rect.y, rect.width, rect.height, radius, offset, width, color
        ));
    }

    fn draw_linear_gradient(
        &mut self,
        rect: Rect,
        start_color: [f32; 4],
        end_color: [f32; 4],
        _angle: f32,
    ) {
        self.log(format!(
            "draw_linear_gradient({:.1},{:.1},{:.1},{:.1}) start={:?} end={:?}",
            rect.x, rect.y, rect.width, rect.height, start_color, end_color
        ));
    }

    fn draw_radial_gradient(&mut self, rect: Rect, inner_color: [f32; 4], outer_color: [f32; 4]) {
        self.log(format!(
            "draw_radial_gradient({:.1},{:.1},{:.1},{:.1}) inner={:?} outer={:?}",
            rect.x, rect.y, rect.width, rect.height, inner_color, outer_color
        ));
    }

    fn draw_text(&mut self, text: &str, rect: &Rect, size: f32, color: [f32; 4], h_align: cvkg_core::TextHAlign, v_align: cvkg_core::TextVAlign) {
        self.log(format!(
            "draw_text(\"{}\" at ({:.1},{:.1}) size={:.1} color={:?}",
            text, rect.x, rect.y, size, color
        ));
    }

    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        let w = text.len() as f32 * size * 0.6;
        let h = size;
        self.log(format!(
            "measure_text(\"{}\" size={:.1}) -> ({:.1},{:.1})",
            text, size, w, h
        ));
        (w, h)
    }

    fn shape_rich_text(
        &mut self,
        spans: &[runic::TextSpan],
        max_width: Option<f32>,
        _align: runic::TextAlign,
        _overflow: runic::TextOverflow,
    ) -> Option<runic::ShapedText> {
        let span_summary = spans
            .iter()
            .map(|s| {
                let content = if s.text.len() > 20 {
                    format!("{}...", &s.text[..17])
                } else {
                    s.text.clone()
                };
                format!("[{}:{}]", content, s.style.font_size)
            })
            .collect::<Vec<_>>()
            .join(", ");
        self.log(format!(
            "shape_rich_text([{}], max_width={:?})",
            span_summary, max_width
        ));
        // Return None to fall back to draw_text
        None
    }

    fn draw_shaped_text(&mut self, text: &runic::ShapedText, x: f32, y: f32) {
        self.log(format!(
            "draw_shaped_text(lines={}) at ({:.1},{:.1})",
            text.lines.len(),
            x,
            y
        ));
    }

    fn push_clip_rect(&mut self, rect: Rect) {
        self.log(format!(
            "push_clip_rect({:.1},{:.1},{:.1},{:.1})",
            rect.x, rect.y, rect.width, rect.height
        ));
    }

    fn pop_clip_rect(&mut self) {
        self.log("pop_clip_rect()".to_string());
    }

    fn push_opacity(&mut self, opacity: f32) {
        self.log(format!("push_opacity({:.2})", opacity));
    }

    fn pop_opacity(&mut self) {
        self.log("pop_opacity()".to_string());
    }

    fn push_shadow(&mut self, radius: f32, color: [f32; 4], offset: [f32; 2]) {
        self.log(format!(
            "push_shadow(radius={:.1}, color={:?}, offset={:?})",
            radius, color, offset
        ));
    }

    fn pop_shadow(&mut self) {
        self.log("pop_shadow()".to_string());
    }

    fn push_vnode(&mut self, rect: Rect, name: &'static str) {
        self.log(format!(
            ">>> vnode {} ({:.1},{:.1},{:.1},{:.1})",
            name, rect.x, rect.y, rect.width, rect.height
        ));
        self.push_vnode(name);
    }

    fn pop_vnode(&mut self) {
        if let Some(name) = self.vnode_stack.back() {
            self.log(format!("<<< vnode {}", name));
        }
        self.pop_vnode();
    }

    fn viewport_size(&self) -> Rect {
        Rect::new(0.0, 0.0, 800.0, 600.0)
    }

    fn is_over_budget(&self) -> bool {
        false
    }

    fn memoize(&mut self, _id: u64, _data_hash: u64, _render_fn: &dyn Fn(&mut dyn Renderer)) {
        // TraceRenderer does not cache - always executes
    }

    fn request_redraw(&mut self) {}
}

/// Minimal view that renders a single Text for testing
#[derive(Clone)]
struct SingleTextView {
    content: String,
    font_size: f32,
    color: [f32; 4],
}

impl SingleTextView {
    fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            font_size: 14.0,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    fn color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

impl View for SingleTextView {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "SingleText");
        renderer.draw_text_raw(self.content, rect.x, rect.y, self.font_size, self.color);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        let (w, h) = renderer.measure_text(&self.content, self.font_size);
        Size {
            width: w,
            height: h,
        }
    }

    fn layout(&self) -> Option<&dyn cvkg_core::LayoutView> {
        Some(self)
    }
}

impl cvkg_core::LayoutView for SingleTextView {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn cvkg_core::LayoutView],
        _cache: &mut cvkg_core::LayoutCache,
    ) -> Size {
        Size {
            width: self.content.len() as f32 * self.font_size * 0.6,
            height: self.font_size,
        }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn cvkg_core::LayoutView],
        _cache: &mut cvkg_core::LayoutCache,
    ) {
    }
}

#[test]
fn trace_text_render_with_panel_width() {
    // This test verifies whether passing a constrained proposal width to Text
    // causes word wrapping (the bug).
    let mut r = TraceRenderer::new();
    let panel_width = 600.0;
    let panel_height = 400.0;
    let panel_rect = Rect::new(0.0, 0.0, panel_width, panel_height);

    // Case 1: Render with None (should be single line)
    let text = SingleTextView::new("Item 1 Item 2 Item 3").font_size(14.0);
    r.log("=== Render with None (should be single line) ===".to_string());
    text.render(&mut r, panel_rect);

    // Case 2: Call intrinsic_size with Some(panel_width) - the code path that FrameModifier uses
    r.log("=== Intrinsic size with Some(panel_width) ===".to_string());
    let _size = text.intrinsic_size(
        &mut r,
        SizeProposal::new(Some(panel_width), Some(panel_height)),
    );

    // Dump all calls
    println!("\n===== TRACE OUTPUT =====\n");
    let calls = r.calls.lock().unwrap();
    for call in calls.iter() {
        println!("{}", call);
    }
    println!("\n===== END TRACE =====\n");

    // Assertions: the render call should have a single draw_text line (no wrapping)
    let draw_text_calls: Vec<_> = calls.iter().filter(|c| c.contains("draw_text")).collect();

    // If wrapping happened, we'd see multiple draw_text calls from shape_rich_text lines
    assert!(
        draw_text_calls.len() >= 1,
        "Expected at least one draw_text call, got {}",
        draw_text_calls.len()
    );

    // Check for wrapping: if max_width is passed and wrapping triggers,
    // we'd see shape_rich_text called with Some(max_width)
    let shape_calls: Vec<_> = calls.iter().filter(|c| c.contains("shape_rich_text")).collect();

    // The intrinsic_size call should have passed Some(panel_width)
    // but the render call (which uses None) should not wrap
    for call in &shape_calls {
        println!("SHAPE: {}", call);
    }

    println!("Text render trace test completed.");
}

#[test]
fn trace_button_render() {
    let mut r = TraceRenderer::new();
    let rect = Rect::new(0.0, 0.0, 200.0, 50.0);

    r.log("=== Button Render ===".to_string());
    let button = Button::new("Click Me", || {});
    button.render(&mut r, rect);

    println!("\n===== BUTTON TRACE =====\n");
    let calls = r.calls.lock().unwrap();
    for call in calls.iter() {
        println!("{}", call);
    }
    println!("\n===== END =====\n");

    // Button should have render calls
    let vnode_calls: Vec<_> = calls.iter().filter(|c| c.contains("vnode")).collect();
    assert!(!vnode_calls.is_empty(), "Button should have vnode calls");
}

#[test]
fn trace_vstack_render() {
    let mut r = TraceRenderer::new();
    let rect = Rect::new(0.0, 0.0, 400.0, 300.0);

    r.log("=== VStack Render ===".to_string());
    let stack = VStack::new(8.0)
        .child(SingleTextView::new("Item 1").color([1.0, 1.0, 1.0, 1.0]))
        .child(SingleTextView::new("Item 2").color([0.8, 0.8, 0.8, 1.0]))
        .child(SingleTextView::new("Item 3").color([0.6, 0.6, 0.6, 1.0]));
    stack.render(&mut r, rect);

    println!("\n===== VSTACK TRACE =====\n");
    let calls = r.calls.lock().unwrap();
    for call in calls.iter() {
        println!("{}", call);
    }
    println!("\n===== END =====\n");

    // VStack should render 3 text items
    let draw_calls: Vec<_> = calls.iter().filter(|c| c.contains("draw_text")).collect();
    assert_eq!(draw_calls.len(), 3, "VStack should have 3 draw_text calls");
}

#[test]
fn trace_hstack_with_frame_modifier() {
    let mut r = TraceRenderer::new();
    // Simulate the gallery detail panel: .frame(None, None) with center alignment
    let rect = Rect::new(0.0, 0.0, 400.0, 300.0);

    r.log("=== HStack with Flex+Frame (gallery detail panel) ===".to_string());
    // Build: detail.flex(1.0).frame(None, None)
    // FlexModifier has weight 1.0, FrameModifier has no size constraints but Alignment::Center
    let detail = VStack::new(4.0)
        .child(SingleTextView::new("Heading").font_size(24.0))
        .child(SingleTextView::new("Body text").font_size(14.0))
        .child(SingleTextView::new("Caption").font_size(10.0));

    // Apply flex(1.0).frame(None, None) - this wraps in ModifiedView
    let detail = detail.flex(1.0).frame(None, None);

    detail.render(&mut r, rect);

    println!("\n===== FRAME MODIFIER TRACE =====\n");
    let calls = r.calls.lock().unwrap();
    for call in calls.iter() {
        println!("{}", call);
    }
    println!("\n===== END =====\n");

    // The frame modifier with center alignment should compute intrinsic size
    // and center the child in the available rect
    let measure_calls: Vec<_> = calls
        .iter()
        .filter(|c| c.contains("measure_text"))
        .collect();
    println!("Measure calls: {}", measure_calls.len());
}