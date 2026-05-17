use crate::graph::FlowGraph;
use crate::interaction::{DragState, FlowContainer, FlowSettings, InteractionState};
use crate::types::{EdgePath, PortPosition};
use cvkg_core::{Event, Never, Rect, Renderer, View};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Infinite canvas viewport with pan/zoom
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowCanvas {
    pub id: String,
    pub initial_graph: FlowGraph,
    pub min_scale: f32,
    pub max_scale: f32,
}

impl FlowCanvas {
    pub fn new(id: impl Into<String>, graph: FlowGraph) -> Self {
        Self {
            id: id.into(),
            initial_graph: graph,
            min_scale: 0.1,
            max_scale: 5.0,
        }
    }

    fn get_state_hash(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut s = std::collections::hash_map::DefaultHasher::new();
        self.id.hash(&mut s);
        s.finish()
    }
}

impl View for FlowCanvas {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "FlowCanvas");

        let id_hash = self.get_state_hash();

        // Load or initialize state
        let state = {
            let s = cvkg_core::load_system_state();
            s.get_component_state::<FlowContainer>(id_hash)
                .map(|v| v.read().unwrap().clone())
                .unwrap_or_else(|| FlowContainer {
                    graph: self.initial_graph.clone(),
                    interaction: InteractionState::default(),
                    settings: FlowSettings::default(),
                    offset: (0.0, 0.0),
                    scale: 1.0,
                    ..Default::default()
                })
        };

        // Background grid
        if state.settings.show_grid {
            self.render_grid(
                renderer,
                rect,
                state.offset,
                state.scale,
                state.settings.grid_size,
            );
        }

        // Push transform for pan/zoom
        renderer.push_transform(
            [state.offset.0, state.offset.1],
            [state.scale, state.scale],
            0.0,
        );

        // Render edges
        for edge in state.graph.edges.values() {
            self.render_edge(renderer, &state.graph, edge);
        }

        // Render nodes
        for node in state.graph.nodes.values() {
            self.render_node(renderer, node);
        }

        // Render active connection line
        if let Some(DragState::Connection {
            source_port,
            current_pos,
        }) = state.interaction.drag_state
            && let Some(source_node) = state.graph.get_node_by_port(source_port)
            && let Some(port) = source_node.ports.iter().find(|p| p.id == source_port)
        {
            let start = self.get_port_center(source_node, port);
            self.render_bezier_edge(
                renderer,
                start,
                current_pos,
                port.position,
                PortPosition::Left,
                [1.0, 1.0, 1.0, 0.5],
            );
        }

        renderer.pop_transform();

        // Render Selection Box (Marquee)
        if let Some(DragState::SelectionBox {
            start_pos,
            current_pos,
        }) = state.interaction.drag_state
        {
            let sel_rect = Rect {
                x: start_pos.0.min(current_pos.0),
                y: start_pos.1.min(current_pos.1),
                width: (current_pos.0 - start_pos.0).abs(),
                height: (current_pos.1 - start_pos.1).abs(),
            };
            renderer.fill_rect(sel_rect, [0.0, 0.8, 1.0, 0.1]);
            renderer.stroke_rect(sel_rect, [0.0, 0.9, 1.0, 0.5], 1.0);
        }

        // Render Mini-map
        self.render_minimap(renderer, rect, &state);

        // Register handlers
        self.register_interaction_handlers(renderer, rect, id_hash, state);

        renderer.pop_vnode();
    }
}

impl FlowCanvas {
    fn render_grid(
        &self,
        renderer: &mut dyn Renderer,
        rect: Rect,
        offset: (f32, f32),
        scale: f32,
        grid_size: f32,
    ) {
        let spacing = grid_size * scale;
        let off_x = offset.0 % spacing;
        let off_y = offset.1 % spacing;

        for x in (0..((rect.width / spacing) as i32 + 2)).map(|i| i as f32 * spacing + off_x) {
            for y in (0..((rect.height / spacing) as i32 + 2)).map(|i| i as f32 * spacing + off_y) {
                renderer.fill_ellipse(
                    Rect {
                        x: rect.x + x - 1.0,
                        y: rect.y + y - 1.0,
                        width: 2.0,
                        height: 2.0,
                    },
                    [0.2, 0.2, 0.3, 0.5],
                );
            }
        }
    }

    fn render_node(&self, renderer: &mut dyn Renderer, node: &crate::node::FlowNode) {
        let node_rect = Rect {
            x: node.position.0,
            y: node.position.1,
            width: node.size.0,
            height: node.size.1,
        };

        renderer.bifrost(node_rect, 12.0, 1.2, 0.9);
        renderer.fill_rounded_rect(node_rect, 8.0, [0.05, 0.05, 0.1, 0.8]);

        let border_color = if node.selected {
            [0.0, 0.9, 1.0, 1.0]
        } else {
            [0.2, 0.2, 0.3, 0.6]
        };
        renderer.stroke_rounded_rect(node_rect, 8.0, border_color, 1.5);

        renderer.draw_text(
            &node.label,
            node_rect.x + 15.0,
            node_rect.y + 12.0,
            14.0,
            [1.0, 1.0, 1.0, 1.0],
        );

        for port in &node.ports {
            self.render_port(renderer, node_rect, port);
        }
    }

    fn render_port(
        &self,
        renderer: &mut dyn Renderer,
        node_rect: Rect,
        port: &crate::port::FlowPort,
    ) {
        let port_size = 10.0;
        let (px, py) = match port.position {
            PortPosition::Top => (node_rect.x + node_rect.width / 2.0, node_rect.y),
            PortPosition::Bottom => (
                node_rect.x + node_rect.width / 2.0,
                node_rect.y + node_rect.height,
            ),
            PortPosition::Left => (node_rect.x, node_rect.y + node_rect.height / 2.0),
            PortPosition::Right => (
                node_rect.x + node_rect.width,
                node_rect.y + node_rect.height / 2.0,
            ),
        };

        let port_rect = Rect {
            x: px - port_size / 2.0,
            y: py - port_size / 2.0,
            width: port_size,
            height: port_size,
        };

        renderer.fill_ellipse(port_rect, [0.0, 0.8, 1.0, 1.0]);
        renderer.stroke_ellipse(port_rect, [1.0, 1.0, 1.0, 0.5], 1.0);
    }

    fn render_edge(
        &self,
        renderer: &mut dyn Renderer,
        graph: &FlowGraph,
        edge: &crate::edge::FlowEdge,
    ) {
        let mut source_pos = None;
        let mut target_pos = None;
        let mut source_dir = PortPosition::Right;
        let mut target_dir = PortPosition::Left;

        for node in graph.nodes.values() {
            for port in &node.ports {
                if port.id == edge.source {
                    source_pos = Some(self.get_port_center(node, port));
                    source_dir = port.position;
                }
                if port.id == edge.target {
                    target_pos = Some(self.get_port_center(node, port));
                    target_dir = port.position;
                }
            }
        }

        if let (Some(s), Some(t)) = (source_pos, target_pos) {
            let color = if edge.selected {
                [0.0, 0.9, 1.0, 1.0]
            } else {
                [0.4, 0.4, 0.5, 0.8]
            };
            match edge.path {
                EdgePath::Bezier => {
                    self.render_bezier_edge(renderer, s, t, source_dir, target_dir, color)
                }
                EdgePath::Straight => renderer.draw_line(s.0, s.1, t.0, t.1, color, 2.0),
                EdgePath::Step => renderer.draw_line(s.0, s.1, t.0, t.1, color, 2.0),
            }
        }
    }

    fn render_bezier_edge(
        &self,
        renderer: &mut dyn Renderer,
        s: (f32, f32),
        t: (f32, f32),
        s_dir: PortPosition,
        t_dir: PortPosition,
        color: [f32; 4],
    ) {
        let dx = (t.0 - s.0).abs();
        let dy = (t.1 - s.1).abs();
        let handle_dist = (dx.max(dy) * 0.5).max(30.0);

        let get_handle = |pos: (f32, f32), dir: PortPosition| match dir {
            PortPosition::Top => (pos.0, pos.1 - handle_dist),
            PortPosition::Bottom => (pos.0, pos.1 + handle_dist),
            PortPosition::Left => (pos.0 - handle_dist, pos.1),
            PortPosition::Right => (pos.0 + handle_dist, pos.1),
        };

        let h1 = get_handle(s, s_dir);
        let h2 = get_handle(t, t_dir);

        let segments = 20;
        let mut prev = s;
        for i in 1..=segments {
            let f = i as f32 / segments as f32;
            let current = self.cubic_bezier(s, h1, h2, t, f);
            renderer.draw_line(prev.0, prev.1, current.0, current.1, color, 2.0);
            prev = current;
        }
    }

    fn cubic_bezier(
        &self,
        p0: (f32, f32),
        p1: (f32, f32),
        p2: (f32, f32),
        p3: (f32, f32),
        t: f32,
    ) -> (f32, f32) {
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        let t2 = t * t;
        let t3 = t2 * t;
        (
            mt3 * p0.0 + 3.0 * mt2 * t * p1.0 + 3.0 * mt * t2 * p2.0 + t3 * p3.0,
            mt3 * p0.1 + 3.0 * mt2 * t * p1.1 + 3.0 * mt * t2 * p2.1 + t3 * p3.1,
        )
    }

    fn get_port_center(
        &self,
        node: &crate::node::FlowNode,
        port: &crate::port::FlowPort,
    ) -> (f32, f32) {
        match port.position {
            PortPosition::Top => (node.position.0 + node.size.0 / 2.0, node.position.1),
            PortPosition::Bottom => (
                node.position.0 + node.size.0 / 2.0,
                node.position.1 + node.size.1,
            ),
            PortPosition::Left => (node.position.0, node.position.1 + node.size.1 / 2.0),
            PortPosition::Right => (
                node.position.0 + node.size.0,
                node.position.1 + node.size.1 / 2.0,
            ),
        }
    }

    fn render_minimap(
        &self,
        renderer: &mut dyn Renderer,
        canvas_rect: Rect,
        state: &FlowContainer,
    ) {
        let mm_w = 200.0;
        let mm_h = 150.0;
        let mm_rect = Rect {
            x: canvas_rect.x + canvas_rect.width - mm_w - 20.0,
            y: canvas_rect.y + canvas_rect.height - mm_h - 20.0,
            width: mm_w,
            height: mm_h,
        };

        renderer.bifrost(mm_rect, 10.0, 1.0, 0.8);
        renderer.fill_rounded_rect(mm_rect, 4.0, [0.02, 0.02, 0.05, 0.7]);
        renderer.stroke_rounded_rect(mm_rect, 4.0, [0.2, 0.2, 0.3, 0.5], 1.0);

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        if state.graph.nodes.is_empty() {
            return;
        }

        for node in state.graph.nodes.values() {
            min_x = min_x.min(node.position.0);
            min_y = min_y.min(node.position.1);
            max_x = max_x.max(node.position.0 + node.size.0);
            max_y = max_y.max(node.position.1 + node.size.1);
        }

        let graph_w = (max_x - min_x).max(1000.0);
        let graph_h = (max_y - min_y).max(1000.0);
        let padding = 20.0;
        let scale_x = (mm_w - padding * 2.0) / graph_w;
        let scale_y = (mm_h - padding * 2.0) / graph_h;
        let scale = scale_x.min(scale_y);

        let center_off_x = (mm_w - graph_w * scale) / 2.0;
        let center_off_y = (mm_h - graph_h * scale) / 2.0;

        for node in state.graph.nodes.values() {
            let nx = mm_rect.x + center_off_x + (node.position.0 - min_x) * scale;
            let ny = mm_rect.y + center_off_y + (node.position.1 - min_y) * scale;
            let nw = node.size.0 * scale;
            let nh = node.size.1 * scale;
            renderer.fill_rect(
                Rect {
                    x: nx,
                    y: ny,
                    width: nw,
                    height: nh,
                },
                [0.0, 0.8, 1.0, 0.6],
            );
        }

        let v_x = mm_rect.x + center_off_x + ((-state.offset.0 / state.scale) - min_x) * scale;
        let v_y = mm_rect.y + center_off_y + ((-state.offset.1 / state.scale) - min_y) * scale;
        let v_w = (canvas_rect.width / state.scale) * scale;
        let v_h = (canvas_rect.height / state.scale) * scale;
        renderer.stroke_rect(
            Rect {
                x: v_x,
                y: v_y,
                width: v_w,
                height: v_h,
            },
            [1.0, 1.0, 1.0, 0.8],
            1.0,
        );
    }

    fn register_interaction_handlers(
        &self,
        renderer: &mut dyn Renderer,
        _rect: Rect,
        id_hash: u64,
        _state: FlowContainer,
    ) {
        renderer.register_handler(
            "pointerdown",
            Arc::new(move |event| {
                if let Event::PointerDown { x, y, .. } = event {
                    cvkg_core::update_system_state(|s| {
                        let mut s = s.clone();
                        let mut state = s
                            .get_component_state::<FlowContainer>(id_hash)
                            .unwrap()
                            .read()
                            .unwrap()
                            .clone();

                        let cx = (x - state.offset.0) / state.scale;
                        let cy = (y - state.offset.1) / state.scale;

                        let mut hit = false;
                        for node in state.graph.nodes.values_mut() {
                            let nr = Rect {
                                x: node.position.0,
                                y: node.position.1,
                                width: node.size.0,
                                height: node.size.1,
                            };
                            if nr.contains(cx, cy) {
                                for port in &node.ports {
                                    let pc = match port.position {
                                        PortPosition::Top => (nr.x + nr.width / 2.0, nr.y),
                                        PortPosition::Bottom => {
                                            (nr.x + nr.width / 2.0, nr.y + nr.height)
                                        }
                                        PortPosition::Left => (nr.x, nr.y + nr.height / 2.0),
                                        PortPosition::Right => {
                                            (nr.x + nr.width, nr.y + nr.height / 2.0)
                                        }
                                    };
                                    let dist = ((cx - pc.0).powi(2) + (cy - pc.1).powi(2)).sqrt();
                                    if dist < 10.0 {
                                        state.interaction.drag_state =
                                            Some(DragState::Connection {
                                                source_port: port.id,
                                                current_pos: (cx, cy),
                                            });
                                        hit = true;
                                        break;
                                    }
                                }

                                if !hit {
                                    state.interaction.drag_state = Some(DragState::Node {
                                        id: node.id,
                                        start_pos: node.position,
                                        mouse_start: (cx, cy),
                                    });
                                    node.selected = true;
                                    hit = true;
                                }
                                break;
                            } else {
                                node.selected = false;
                            }
                        }

                        if !hit {
                            state.interaction.drag_state = Some(DragState::Canvas {
                                start_offset: state.offset,
                                mouse_start: (x, y),
                            });
                        }

                        s.set_component_state(id_hash, state);
                        s
                    });
                }
            }),
        );

        renderer.register_handler(
            "pointermove",
            Arc::new(move |event| {
                if let Event::PointerMove { x, y } = event {
                    cvkg_core::update_system_state(|s| {
                        let mut s = s.clone();
                        let mut state = s
                            .get_component_state::<FlowContainer>(id_hash)
                            .unwrap()
                            .read()
                            .unwrap()
                            .clone();

                        match state.interaction.drag_state {
                            Some(DragState::Node {
                                id,
                                start_pos,
                                mouse_start,
                            }) => {
                                let cx = (x - state.offset.0) / state.scale;
                                let cy = (y - state.offset.1) / state.scale;
                                if let Some(node) = state.graph.nodes.get_mut(&id) {
                                    let mut nx = start_pos.0 + (cx - mouse_start.0);
                                    let mut ny = start_pos.1 + (cy - mouse_start.1);
                                    if state.settings.grid_snapping {
                                        nx = (nx / state.settings.grid_size).round()
                                            * state.settings.grid_size;
                                        ny = (ny / state.settings.grid_size).round()
                                            * state.settings.grid_size;
                                    }
                                    node.position = (nx, ny);
                                }
                            }
                            Some(DragState::Canvas {
                                start_offset,
                                mouse_start,
                            }) => {
                                state.offset = (
                                    start_offset.0 + (x - mouse_start.0),
                                    start_offset.1 + (y - mouse_start.1),
                                );
                            }
                            Some(DragState::Connection {
                                source_port: _,
                                ref mut current_pos,
                            }) => {
                                *current_pos = (
                                    (x - state.offset.0) / state.scale,
                                    (y - state.offset.1) / state.scale,
                                );
                            }
                            Some(DragState::SelectionBox {
                                start_pos: _,
                                ref mut current_pos,
                            }) => {
                                *current_pos = (x, y);
                            }
                            _ => {}
                        }

                        s.set_component_state(id_hash, state);
                        s
                    });
                }
            }),
        );

        renderer.register_handler(
            "pointerup",
            Arc::new(move |_| {
                cvkg_core::update_system_state(|s| {
                    let mut s = s.clone();
                    let mut state = s
                        .get_component_state::<FlowContainer>(id_hash)
                        .unwrap()
                        .read()
                        .unwrap()
                        .clone();

                    let mut target_port_id = None;
                    let mut source_port_id = None;

                    if let Some(DragState::Connection {
                        source_port,
                        current_pos,
                    }) = state.interaction.drag_state
                    {
                        source_port_id = Some(source_port);
                        for node in state.graph.nodes.values() {
                            for port in &node.ports {
                                let nr = Rect {
                                    x: node.position.0,
                                    y: node.position.1,
                                    width: node.size.0,
                                    height: node.size.1,
                                };
                                let pc = match port.position {
                                    PortPosition::Top => (nr.x + nr.width / 2.0, nr.y),
                                    PortPosition::Bottom => {
                                        (nr.x + nr.width / 2.0, nr.y + nr.height)
                                    }
                                    PortPosition::Left => (nr.x, nr.y + nr.height / 2.0),
                                    PortPosition::Right => {
                                        (nr.x + nr.width, nr.y + nr.height / 2.0)
                                    }
                                };
                                let dist = ((current_pos.0 - pc.0).powi(2)
                                    + (current_pos.1 - pc.1).powi(2))
                                .sqrt();
                                if dist < 15.0 && port.id != source_port {
                                    target_port_id = Some(port.id);
                                }
                            }
                        }
                    }

                    if let Some(target_port) = target_port_id {
                        if let Some(source_port) = source_port_id {
                            state.push_history();
                            let edge_id = crate::types::EdgeId(rand::random());
                            state.graph.edges.insert(
                                edge_id,
                                crate::edge::FlowEdge::new(edge_id, source_port, target_port),
                            );
                        }
                    } else if let Some(DragState::Node { .. }) = state.interaction.drag_state {
                        state.push_history();
                    } else if let Some(DragState::SelectionBox {
                        start_pos,
                        current_pos,
                    }) = state.interaction.drag_state
                    {
                        let sel_rect = Rect {
                            x: start_pos.0.min(current_pos.0),
                            y: start_pos.1.min(current_pos.1),
                            width: (current_pos.0 - start_pos.0).abs(),
                            height: (current_pos.1 - start_pos.1).abs(),
                        };
                        for node in state.graph.nodes.values_mut() {
                            let nx = state.offset.0 + node.position.0 * state.scale;
                            let ny = state.offset.1 + node.position.1 * state.scale;
                            if sel_rect.contains(nx, ny) {
                                node.selected = true;
                            }
                        }
                    }

                    state.interaction.drag_state = None;
                    s.set_component_state(id_hash, state);
                    s
                });
            }),
        );

        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key } = event {
                    if key == "z" {
                        cvkg_core::update_system_state(|s| {
                            let mut s = s.clone();
                            let mut state = s
                                .get_component_state::<FlowContainer>(id_hash)
                                .unwrap()
                                .read()
                                .unwrap()
                                .clone();
                            state.undo();
                            s.set_component_state(id_hash, state);
                            s
                        });
                    } else if key == "y" {
                        cvkg_core::update_system_state(|s| {
                            let mut s = s.clone();
                            let mut state = s
                                .get_component_state::<FlowContainer>(id_hash)
                                .unwrap()
                                .read()
                                .unwrap()
                                .clone();
                            state.redo();
                            s.set_component_state(id_hash, state);
                            s
                        });
                    }
                }
            }),
        );
    }
}
