use cvkg_core::mesh::Transform3D;
use cvkg_core::{ElapsedTime, Rect, Renderer, RendererErrorHandler};
use cvkg_vdom::WorldSpacePanel;
use cvkg_vdom::signals::{create_effect, create_signal};
use std::sync::{Arc, Mutex};

// ── DependencyGraph Reactivity Tests ──

#[test]
fn test_dependency_graph_selective_invalidation() {
    let (get_a, set_a) = create_signal(10);
    let (get_b, set_b) = create_signal(20);

    let ran_a = Arc::new(Mutex::new(0));
    let ran_b = Arc::new(Mutex::new(0));

    let ran_a_clone = Arc::clone(&ran_a);
    create_effect(move || {
        let _val = get_a();
        let mut count = ran_a_clone.lock().unwrap();
        *count += 1;
    });

    let ran_b_clone = Arc::clone(&ran_b);
    create_effect(move || {
        let _val = get_b();
        let mut count = ran_b_clone.lock().unwrap();
        *count += 1;
    });

    // Initial run of effects
    assert_eq!(*ran_a.lock().unwrap(), 1);
    assert_eq!(*ran_b.lock().unwrap(), 1);

    // Mutate A: only effect A should run again
    set_a(15);
    assert_eq!(*ran_a.lock().unwrap(), 2);
    assert_eq!(*ran_b.lock().unwrap(), 1);

    // Mutate B: only effect B should run again
    set_b(25);
    assert_eq!(*ran_a.lock().unwrap(), 2);
    assert_eq!(*ran_b.lock().unwrap(), 2);
}

// ── WorldSpacePanel Isolation Tests ──

struct MockPanelRenderer {
    pub current_panel_id: Option<u64>,
    pub panels: Vec<(u64, WorldSpacePanel)>,
    pub draw_call_panels: Vec<(String, Option<u64>)>,
}

impl MockPanelRenderer {
    fn new() -> Self {
        Self {
            current_panel_id: None,
            panels: Vec::new(),
            draw_call_panels: Vec::new(),
        }
    }
}

impl ElapsedTime for MockPanelRenderer {
    fn elapsed_time(&self) -> f32 {
        0.0
    }
    fn delta_time(&self) -> f32 {
        0.0
    }
}

impl RendererErrorHandler for MockPanelRenderer {}

impl Renderer for MockPanelRenderer {
    fn begin_world_space_panel(
        &mut self,
        node_id: u64,
        transform: &Transform3D,
        glass: Option<cvkg_materials::GlassMaterial>,
        pixels_per_unit: f32,
        world_size: (f32, f32),
    ) {
        self.current_panel_id = Some(node_id);
        self.panels.push((
            node_id,
            WorldSpacePanel {
                transform: transform.clone(),
                glass,
                pixels_per_unit,
                world_size,
                ..Default::default()
            },
        ));
    }

    fn end_world_space_panel(&mut self, _node_id: u64) {
        self.current_panel_id = None;
    }

    fn fill_rect(&mut self, _rect: Rect, _color: [f32; 4]) {
        self.draw_call_panels
            .push(("fill_rect".to_string(), self.current_panel_id));
    }

    fn fill_rounded_rect(&mut self, _rect: Rect, _radius: f32, _color: [f32; 4]) {}
    fn fill_ellipse(&mut self, _rect: Rect, _color: [f32; 4]) {}
    fn stroke_rect(&mut self, _rect: Rect, _color: [f32; 4], _width: f32) {}
    fn stroke_rounded_rect(&mut self, _rect: Rect, _radius: f32, _color: [f32; 4], _width: f32) {}
    fn stroke_ellipse(&mut self, _rect: Rect, _color: [f32; 4], _width: f32) {}
    fn draw_line(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _color: [f32; 4], _width: f32) {
    }
    fn memoize(&mut self, _id: u64, _data_hash: u64, render_fn: &dyn Fn(&mut dyn Renderer)) {
        render_fn(self);
    }
}

#[test]
fn test_world_space_panel_traversal_isolation() {
    let mut renderer = MockPanelRenderer::new();
    let panel_transform = Transform3D::default();

    // Simulating developer view rendering logic
    renderer.fill_rect(Rect::new(0.0, 0.0, 100.0, 100.0), [1.0, 0.0, 0.0, 1.0]);

    renderer.begin_world_space_panel(42, &panel_transform, None, 200.0, (1.0, 1.0));
    renderer.fill_rect(Rect::new(10.0, 10.0, 50.0, 50.0), [0.0, 1.0, 0.0, 1.0]);
    renderer.end_world_space_panel(42);

    renderer.fill_rect(Rect::new(0.0, 0.0, 10.0, 10.0), [0.0, 0.0, 1.0, 1.0]);

    assert_eq!(renderer.panels.len(), 1);
    assert_eq!(renderer.panels[0].0, 42);

    assert_eq!(renderer.draw_call_panels.len(), 3);
    // Draw calls 0 and 2 are outside any panel
    assert_eq!(renderer.draw_call_panels[0].1, None);
    assert_eq!(renderer.draw_call_panels[2].1, None);
    // Draw call 1 is isolated to panel 42
    assert_eq!(renderer.draw_call_panels[1].1, Some(42));
}
