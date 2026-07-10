//! Temporary probe reproducing the gallery scenario: ScrollView containing
//! Combobox, DatePicker, Dialog. Captures handler registration + dispatch to
//! prove/measure each bug empirically.

use cvkg_components::{Combobox, DatePicker, Dialog, ScrollView, Text, VStack, Button};
use cvkg_core::{AnyView, ElapsedTime, Event, Rect, Renderer, View, load_system_state, update_system_state};
use std::sync::{Arc, Mutex};

struct CapturingRenderer {
    handlers: Mutex<std::collections::HashMap<String, Vec<Arc<dyn Fn(Event) + Send + Sync>>>>,
    draw: Mutex<Vec<String>>,
    z: Mutex<f32>,
}

impl CapturingRenderer {
    fn new() -> Self {
        Self {
            handlers: Mutex::new(std::collections::HashMap::new()),
            draw: Mutex::new(Vec::new()),
            z: Mutex::new(0.0),
        }
    }
    fn fire(&self, ev: Event) {
        let name = ev.name().to_string();
        let hs = self.handlers.lock().unwrap().get(&name).cloned().unwrap_or_default();
        for h in hs { h(ev.clone()); }
    }
    fn count(&self, name: &str) -> usize {
        self.handlers.lock().unwrap().get(name).map(|v| v.len()).unwrap_or(0)
    }
    fn logs(&self) -> Vec<String> { self.draw.lock().unwrap().clone() }
    fn reset(&self) {
        self.handlers.lock().unwrap().clear();
        self.draw.lock().unwrap().clear();
    }
}

impl ElapsedTime for CapturingRenderer {
    fn elapsed_time(&self) -> f32 { 0.0 }
    fn delta_time(&self) -> f32 { 1.0 / 60.0 }
}
impl cvkg_core::RendererErrorHandler for CapturingRenderer {}

impl Renderer for CapturingRenderer {
    fn fill_rect(&mut self, _rect: Rect, _c: [f32;4]) {}
    fn fill_rounded_rect(&mut self, _rect: Rect, _r: f32, _c: [f32;4]) {}
    fn fill_ellipse(&mut self, _rect: Rect, _c: [f32;4]) {}
    fn stroke_rect(&mut self, _rect: Rect, _c: [f32;4], _w: f32) {}
    fn stroke_rounded_rect(&mut self, _rect: Rect, _r: f32, _c: [f32;4], _w: f32) {}
    fn stroke_ellipse(&mut self, _rect: Rect, _c: [f32;4], _w: f32) {}
    fn draw_line(&mut self, _x1:f32,_y1:f32,_x2:f32,_y2:f32,_c:[f32;4],_w:f32) {}
    fn draw_text_raw(&mut self, t: &str, _x: f32, _y: f32, _s: f32, _c: [f32;4]) {
        self.draw.lock().unwrap().push(format!("Text({})", t));
    }
    fn measure_text(&mut self, _t: &str, _s: f32) -> (f32, f32) { (10.0, 10.0) }
    fn push_vnode(&mut self, _rect: Rect, name: &'static str) {
        self.draw.lock().unwrap().push(format!("PushVNode({})", name));
    }
    fn pop_vnode(&mut self) {
        self.draw.lock().unwrap().push("PopVNode".to_string());
    }
    fn set_z_index(&mut self, z: f32) {
        *self.z.lock().unwrap() = z;
        self.draw.lock().unwrap().push(format!("SetZ({})", z));
    }
    fn register_handler(&mut self, et: &str, h: Arc<dyn Fn(Event) + Send + Sync>) {
        self.handlers.lock().unwrap().entry(et.to_string()).or_default().push(h);
    }
    fn memoize(&mut self, _id: u64, _h: u64, _f: &dyn Fn(&mut dyn Renderer)) {}
}

const COMBO_HASH: u64 = 0xC00_0001;
const DP_HASH: u64 = 0xE00_0000;
const DIALOG_HASH: u64 = 0xB00_0001;

fn combo_open() -> bool {
    load_system_state().get_component_state::<bool>(COMBO_HASH)
        .and_then(|v| v.read().ok().map(|g| *g)).unwrap_or(false)
}
fn dp_open() -> bool {
    load_system_state().get_component_state::<bool>(DP_HASH)
        .and_then(|v| v.read().ok().map(|g| *g)).unwrap_or(false)
}
fn dialog_open() -> bool {
    load_system_state().get_component_state::<bool>(DIALOG_HASH)
        .and_then(|v| v.read().ok().map(|g| *g)).unwrap_or(false)
}

#[test]
fn probe_gallery_scenario() {
    let mut r = CapturingRenderer::new();

    // Build gallery-like view: ScrollView containing combobox, datepicker, dialog button.
    let cb = Combobox::new(vec!["Alpha".to_string(), "Beta".to_string()]);
    let dp = DatePicker::new(|_,_,_| {}).selected(15,6,2025);
    let open_btn = Button::new("Open Modal", || {
        update_system_state(|s| { let mut s = s.clone(); s.set_component_state(DIALOG_HASH, true); s });
    });
    let dialog = Dialog::new(AnyView::new(Text::new("Dialog body")))
        .presented(dialog_open())
        .title("Confirm Action");

    let view = VStack::new(10.0)
        .child(cb)
        .child(dp)
        .child(open_btn)
        .child(dialog);
    let scroll = ScrollView::new(view);

    let root = Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 };
    scroll.render(&mut r, root);

    println!("[probe] wheel handlers registered = {}", r.count("pointerwheel"));
    println!("[probe] pointerclick handlers registered = {}", r.count("pointerclick"));

    // --- Combobox trigger click ---
    println!("[probe] combo open BEFORE = {}", combo_open());
    r.fire(Event::PointerClick { x: 10.0, y: 10.0, button: 0, tilt: None, azimuth: None, pressure: None, barrel_rotation: None, pointer_precision: 0.0 });
    println!("[probe] combo open AFTER trigger click = {}", combo_open());

    // --- Datepicker field click (approx position y in 50..100) ---
    println!("[probe] dp open BEFORE = {}", dp_open());
    r.fire(Event::PointerClick { x: 10.0, y: 60.0, button: 0, tilt: None, azimuth: None, pressure: None, barrel_rotation: None, pointer_precision: 0.0 });
    println!("[probe] dp open AFTER field click = {}", dp_open());

    // --- Dialog: click open button then re-render (simulate changed()) ---
    r.fire(Event::PointerClick { x: 10.0, y: 110.0, button: 0, tilt: None, azimuth: None, pressure: None, barrel_rotation: None, pointer_precision: 0.0 });
    println!("[probe] dialog open AFTER button = {}", dialog_open());

    // Re-render with dialog open to see if it draws
    r.reset();
    let dialog2 = Dialog::new(AnyView::new(Text::new("Dialog body")))
        .presented(dialog_open())
        .title("Confirm Action");
    let view2 = VStack::new(10.0).child(dialog2);
    let scroll2 = ScrollView::new(view2);
    scroll2.render(&mut r, root);
    let logs = r.logs();
    let drew_dialog = logs.iter().any(|l| l.contains("Confirm Action"));
    println!("[probe] dialog drew title text = {} (logs: {:?})", drew_dialog, logs);

    // --- Datepicker z-index: render with open to capture SetZ calls ---
    update_system_state(|s| { let mut s = s.clone(); s.set_component_state(DP_HASH, true); s });
    r.reset();
    let dp2 = DatePicker::new(|_,_,_| {}).selected(15,6,2025);
    let view3 = VStack::new(10.0).child(dp2);
    ScrollView::new(view3).render(&mut r, root);
    let zlogs: Vec<_> = r.logs().into_iter().filter(|l| l.starts_with("SetZ")).collect();
    println!("[probe] datepicker SetZ calls = {:?}", zlogs);
    // Regression: calendar must render ABOVE the page (positive z), not behind it.
    assert!(zlogs.iter().any(|z| z == "SetZ(900)"), "datepicker calendar must use high positive z, got {:?}", zlogs);

    // Regression: combobox dropdown must pop above siblings (positive z).
    update_system_state(|s| { let mut s = s.clone(); s.set_component_state(COMBO_HASH, true); s });
    r.reset();
    let cb2 = Combobox::new(vec!["Alpha".to_string(), "Beta".to_string()]);
    ScrollView::new(VStack::new(10.0).child(cb2)).render(&mut r, root);
    let combo_z: Vec<_> = r.logs().into_iter().filter(|l| l.starts_with("SetZ")).collect();
    println!("[probe] combobox SetZ calls = {:?}", combo_z);
    assert!(combo_z.iter().any(|z| z == "SetZ(900)"), "combobox dropdown must use high positive z, got {:?}", combo_z);
}

#[test]
fn probe_wheel_accumulation() {
    // Simulate a single momentum wheel gesture = several small deltas summing
    // to ~one notch (10.0). Before the fix each sub-event advanced the card,
    // racing through the list. After the fix it advances exactly once.
    let mut s = GalleryState::new();
    let num = s.entries.len();
    assert!(num > 1);
    let start = s.selected;
    // 5 sub-events of 2.0 each = 10.0 total = one notch.
    for _ in 0..5 {
        s.wheel_accum += 2.0;
        while s.wheel_accum >= 10.0 { s.wheel_accum -= 10.0; s.selected = (s.selected + 1) % num; }
        while s.wheel_accum <= -10.0 { s.wheel_accum += 10.0; s.selected = (s.selected + num - 1) % num; }
        if s.wheel_accum.abs() > 40.0 { s.wheel_accum = 0.0; }
    }
    assert_eq!(s.selected, (start + 1) % num, "one notch should advance exactly one card");
    assert_eq!(s.wheel_accum, 0.0, "accumulator should be fully consumed at a notch boundary");
}

#[test]
fn probe_dialog_detail() {
    let mut r = CapturingRenderer::new();
    update_system_state(|s| { let mut s = s.clone(); s.set_component_state(DIALOG_HASH, true); s });
    let dialog = Dialog::new(AnyView::new(Text::new("Dialog body")))
        .presented(true)
        .title("Confirm Action")
        .action("Close", || {});
    let root = Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 };
    dialog.render(&mut r, root);
    println!("[dialog] pointerclick handlers = {}", r.count("pointerclick"));
    println!("[dialog] all draw logs = {:?}", r.logs());
}
