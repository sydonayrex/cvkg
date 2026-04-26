use cvkg_core::{Rect, Renderer, View};
use cvkg_render_gpu::{
    ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, Node, NodeId, Role,
    ShieldWallAdapter, SurtrRenderer, Tree, TreeId, TreeUpdate,
};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

struct NiflheimApp {
    window: Option<Arc<Window>>,
    renderer: Option<SurtrRenderer>,
    /// ShieldWall — OS accessibility adapter (AccessKit backend)
    shieldwall: Option<ShieldWallAdapter>,
}

struct ShieldWallActivation(NodeId);
impl ActivationHandler for ShieldWallActivation {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        let root_id = self.0;
        Some(TreeUpdate {
            nodes: vec![(root_id, {
                let mut n = Node::new(Role::Window);
                n.set_label("Niflheim Mist Demo");
                n
            })],
            tree: Some(Tree::new(root_id)),
            tree_id: TreeId::ROOT,
            focus: root_id,
        })
    }
}

struct ShieldWallAction;
impl ActionHandler for ShieldWallAction {
    fn do_action(&mut self, _request: ActionRequest) {}
}

struct ShieldWallDeactivation;
impl DeactivationHandler for ShieldWallDeactivation {
    fn deactivate_accessibility(&mut self) {}
}

impl ApplicationHandler for NiflheimApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Window must be initially invisible so ShieldWall can be created
        // before first show (AccessKit requirement).
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Niflheim Mist Demo — CVKG Phase 6")
                        .with_inner_size(winit::dpi::LogicalSize::new(800u32, 600u32))
                        .with_visible(false),
                )
                .unwrap(),
        );

        // Build the ShieldWall adapter with a root Window node.
        let root_id = NodeId(1);
        let shieldwall = ShieldWallAdapter::with_direct_handlers(
            event_loop,
            &window,
            ShieldWallActivation(root_id),
            ShieldWallAction,
            ShieldWallDeactivation,
        );

        // Now safe to show the window.
        window.set_visible(true);

        let renderer = pollster::block_on(SurtrRenderer::forge(window.clone()));
        self.window = Some(window);
        self.renderer = Some(renderer);
        self.shieldwall = Some(shieldwall);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let renderer = match self.renderer.as_mut() {
            Some(r) => r,
            None => return,
        };
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let encoder = renderer.begin_frame(self.window.as_ref().unwrap().id());

                // ── Background void ──────────────────────────────────────────
                renderer.fill_rect(
                    Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 800.0,
                        height: 600.0,
                    },
                    [0.0, 0.0, 0.0, 1.0], // Ginnungagap
                );

                // ── Card body (Rounded, Transparent Glass — Bifrost) ──────────
                renderer.bifrost(
                    Rect {
                        x: 80.0,
                        y: 60.0,
                        width: 640.0,
                        height: 480.0,
                    },
                    10.0,
                    1.0,
                    0.1,
                );
                renderer.fill_rounded_rect(
                    Rect {
                        x: 80.0,
                        y: 60.0,
                        width: 640.0,
                        height: 480.0,
                    },
                    20.0,
                    [0.05, 0.03, 0.12, 0.8],
                );

                // ── Mjolnir Slice (Cut-off Corner) ──────────────────────────
                renderer.push_mjolnir_slice(45.0, 100.0);

                // ── Tactical Dashboard ──────────────────────────────────────
                // Label
                renderer.draw_text("SYSTEMS ONLINE", 100.0, 100.0, 24.0, [0.0, 1.0, 1.0, 1.0]);

                // Progress Bars using the high-level component
                let p1 = cvkg_components::ProgressView::new(0.75, 1.0);
                p1.render(
                    renderer,
                    Rect {
                        x: 100.0,
                        y: 150.0,
                        width: 280.0,
                        height: 20.0,
                    },
                );

                let p2 = cvkg_components::ProgressView::new(0.3, 1.0);
                p2.render(
                    renderer,
                    Rect {
                        x: 100.0,
                        y: 180.0,
                        width: 280.0,
                        height: 20.0,
                    },
                );

                // Tactical Chart
                let chart_data = vec![0.1, 0.5, 0.3, 0.8, 0.4, 0.9, 0.6];
                let chart =
                    cvkg_components::ChartView::new(cvkg_components::ChartType::Line, chart_data)
                        .color([1.0, 0.4, 0.0, 1.0]);
                chart.render(
                    renderer,
                    Rect {
                        x: 420.0,
                        y: 150.0,
                        width: 280.0,
                        height: 100.0,
                    },
                );

                renderer.pop_mjolnir_slice();

                // ── Gungnir Pulse Indicators ──
                for (cx, cy) in [
                    (80.0f32, 60.0f32),
                    (714.0, 60.0),
                    (80.0, 534.0),
                    (714.0, 534.0),
                ] {
                    renderer.fill_rect_with_mode(
                        Rect {
                            x: cx - 5.0,
                            y: cy - 5.0,
                            width: 10.0,
                            height: 10.0,
                        },
                        [0.0, 1.0, 1.0, 1.0],
                        1, // Pulse Mode
                        None,
                    );
                }

                // ── Quench the blade ─────────────────────────────────────────
                renderer.end_frame(encoder);
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = NiflheimApp {
        window: None,
        renderer: None,
        shieldwall: None,
    };
    event_loop.run_app(&mut app).unwrap();
}
