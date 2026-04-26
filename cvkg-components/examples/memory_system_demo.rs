use cvkg_components::MemoryView;
use cvkg_core::{KnowledgeFragment, KnowledgeState, Rect, Renderer, State, View};
use cvkg_render_gpu::SurtrRenderer;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

struct MemoryApp {
    window: Option<Arc<Window>>,
    renderer: Option<SurtrRenderer>,
    memory_state: State<KnowledgeState>,
}

impl ApplicationHandler for MemoryApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("CVKG State-Native Memory Demo")
                        .with_inner_size(winit::dpi::LogicalSize::new(800u32, 600u32)),
                )
                .unwrap(),
        );

        let renderer = pollster::block_on(SurtrRenderer::forge(window.clone()));
        self.window = Some(window);
        self.renderer = Some(renderer);

        // Seed some initial knowledge
        let mut initial_state = KnowledgeState::default();
        initial_state.remember(KnowledgeFragment {
            id: "1".to_string(),
            summary: "Project Bifrost Security Protocol".to_string(),
            source: "docs/security/bifrost.md".to_string(),
            created_at: 100,
            accessed_count: 5,
            content: None,
        });
        initial_state.remember(KnowledgeFragment {
            id: "2".to_string(),
            summary: "Niflheim Mist Rendering Engine v2.0".to_string(),
            source: "src/render/niflheim.rs".to_string(),
            created_at: 150,
            accessed_count: 12,
            content: None,
        });
        initial_state.remember(KnowledgeFragment {
            id: "3".to_string(),
            summary: "Muspelheim Thermal Regulation Analysis".to_string(),
            source: "telemetry/muspel_01.log".to_string(),
            created_at: 200,
            accessed_count: 3,
            content: None,
        });

        // Initial search to show everything
        initial_state.process_query("");

        self.memory_state.set(initial_state);
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
            WindowEvent::KeyboardInput {
                event: kb_event, ..
            } => {
                if kb_event.state.is_pressed() {
                    // Cyclic search demonstration
                    let queries = ["", "bifrost", "niflheim", "thermal", "security"];
                    static mut QUERY_INDEX: usize = 0;
                    unsafe {
                        QUERY_INDEX = (QUERY_INDEX + 1) % queries.len();
                        let mut state = self.memory_state.get();
                        state.process_query(queries[QUERY_INDEX]);
                        self.memory_state.set(state);
                        println!("Searching for: '{}'", queries[QUERY_INDEX]);
                    }
                    self.window.as_ref().unwrap().request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                let encoder = renderer.begin_frame(self.window.as_ref().unwrap().id());

                // ── Background ──
                renderer.fill_rect(
                    Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 800.0,
                        height: 600.0,
                    },
                    [0.01, 0.01, 0.02, 1.0],
                );

                // ── Memory Inspection View ──
                let mem_rect = Rect {
                    x: 50.0,
                    y: 50.0,
                    width: 350.0,
                    height: 400.0,
                };
                let mem_view = MemoryView::new(self.memory_state.clone());
                mem_view.render(renderer, mem_rect);

                // ── AI Context Simulation ──
                renderer.draw_text(
                    "AI_CONTEXT_INJECTION",
                    450.0,
                    65.0,
                    16.0,
                    [1.0, 0.0, 1.0, 1.0], // MuspelMagenta
                );

                let state = self.memory_state.get();
                let mut ctx_y = 90.0;
                for id in &state.last_query_results {
                    if let Some(frag) = state.fragments.get(id) {
                        renderer.draw_text(
                            &format!("[RECALL:{}]", frag.id),
                            460.0,
                            ctx_y,
                            12.0,
                            [0.8, 0.8, 1.0, 0.9],
                        );
                        ctx_y += 20.0;
                    }
                }

                renderer.draw_text(
                    "Press any key to rotate search queries",
                    250.0,
                    520.0,
                    14.0,
                    [0.6, 0.6, 0.7, 1.0],
                );

                renderer.end_frame(encoder);
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = MemoryApp {
        window: None,
        renderer: None,
        memory_state: State::new(KnowledgeState::default()),
    };
    event_loop.run_app(&mut app).unwrap();
}
