// This example requires the GPU feature to be enabled
#![cfg(feature = "gpu")]

use cvkg_core::{Rect, Renderer};
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

struct ForgeEffectsApp {
    window: Option<Arc<Window>>,
    renderer: Option<SurtrRenderer>,
    shieldwall: Option<ShieldWallAdapter>,
}

struct ShieldWallActivation(NodeId);
impl ActivationHandler for ShieldWallActivation {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        let root_id = self.0;
        Some(TreeUpdate {
            nodes: vec![(root_id, {
                let mut n = Node::new(Role::Window);
                n.set_label("CVKG Forge Effects Demo");
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

impl ApplicationHandler for ForgeEffectsApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("CVKG Forge Effects — High-Fidelity GPU Test")
                        .with_inner_size(winit::dpi::LogicalSize::new(1024u32, 768u32))
                        .with_visible(false),
                )
                .unwrap(),
        );

        let root_id = NodeId(1);
        let shieldwall = ShieldWallAdapter::with_direct_handlers(
            event_loop,
            &window,
            ShieldWallActivation(root_id),
            ShieldWallAction,
            ShieldWallDeactivation,
        );

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

                // ── Background Void ──
                renderer.fill_rect(
                    Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 1024.0,
                        height: 768.0,
                    },
                    [0.0, 0.0, 0.0, 1.0],
                );

                // ── SDF Rounded Rectangles ──
                // Various radii
                for i in 0..5 {
                    let r = i as f32 * 15.0;
                    renderer.fill_rounded_rect(
                        Rect {
                            x: 50.0,
                            y: 50.0 + i as f32 * 80.0,
                            width: 200.0,
                            height: 60.0,
                        },
                        r,
                        [0.0, 1.0, 0.5, 1.0],
                    );
                }

                // ── SDF Ellipses ──
                renderer.fill_ellipse(
                    Rect {
                        x: 300.0,
                        y: 50.0,
                        width: 200.0,
                        height: 100.0,
                    },
                    [1.0, 0.0, 0.5, 1.0],
                );
                renderer.fill_ellipse(
                    Rect {
                        x: 350.0,
                        y: 120.0,
                        width: 100.0,
                        height: 200.0,
                    },
                    [0.0, 0.5, 1.0, 1.0],
                );

                // ── Mjolnir Slices (Geometric Cutting) ──
                renderer.push_mjolnir_slice(15.0, 650.0);
                renderer.fill_rect(
                    Rect {
                        x: 550.0,
                        y: 50.0,
                        width: 400.0,
                        height: 300.0,
                    },
                    [1.0, 0.5, 0.0, 0.8],
                );
                // Nested slice test
                renderer.push_mjolnir_slice(-45.0, 0.0);
                renderer.fill_rect(
                    Rect {
                        x: 600.0,
                        y: 100.0,
                        width: 300.0,
                        height: 200.0,
                    },
                    [0.0, 1.0, 1.0, 1.0],
                );
                renderer.pop_mjolnir_slice();
                renderer.pop_mjolnir_slice();

                // ── Hardware Scissor Clipping ──
                renderer.push_clip_rect(Rect {
                    x: 100.0,
                    y: 450.0,
                    width: 300.0,
                    height: 200.0,
                });
                // Background of clipped area
                renderer.fill_rect(
                    Rect {
                        x: 100.0,
                        y: 450.0,
                        width: 300.0,
                        height: 200.0,
                    },
                    [0.1, 0.1, 0.1, 1.0],
                );
                // This rect bleeds out but should be clipped
                renderer.fill_rounded_rect(
                    Rect {
                        x: 50.0,
                        y: 400.0,
                        width: 400.0,
                        height: 300.0,
                    },
                    30.0,
                    [1.0, 1.0, 1.0, 0.3],
                );
                renderer.pop_clip_rect();

                // ── Gungnir Neon Glow & Pulse ──
                // Dynamic breathing pulse (GungnirPulse logic implemented in core)
                let time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs_f32();
                let pulse = (time * 2.0).sin() * 0.5 + 0.5;

                renderer.stroke_rect(
                    Rect {
                        x: 600.0,
                        y: 450.0,
                        width: 200.0,
                        height: 200.0,
                    },
                    [0.0, 1.0, 1.0, pulse],
                    10.0,
                );

                // ── Mjolnir Shatter (Fragmented Rendering) ──
                // We'll manually implement the shatter loop here to demonstrate the renderer's capability
                let pieces = 8;
                for i in 0..pieces {
                    let angle = i as f32 / pieces as f32 * 360.0;
                    renderer.push_mjolnir_slice(angle, 0.0);
                    renderer.push_mjolnir_slice(angle + 45.0 + 180.0, 0.0);

                    let rad = (angle + 22.5).to_radians();
                    let dx = rad.cos() * 50.0;
                    let dy = rad.sin() * 50.0;

                    renderer.fill_rect(
                        Rect {
                            x: 450.0 + dx,
                            y: 450.0 + dy,
                            width: 100.0,
                            height: 100.0,
                        },
                        [1.0, 0.2, 0.2, 1.0],
                    );

                    renderer.pop_mjolnir_slice();
                    renderer.pop_mjolnir_slice();
                }

                renderer.end_frame(encoder);
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = ForgeEffectsApp {
        window: None,
        renderer: None,
        shieldwall: None,
    };
    event_loop.run_app(&mut app).unwrap();
}
