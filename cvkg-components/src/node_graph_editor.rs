//! Interactive node graph editor with Bézier curve connections.
//!
//! Features:
//! - Draggable nodes with title bars and content areas
//! - Port-based connections (input/output) with click-drag wiring
//! - Cubic Bézier curve edges with tube rendering and arrow heads
//! - Selection, hover, and pending connection preview
//! - Background grid for spatial reference
//! - Two-layer state: persistent (nodes/edges) + interaction (drag/hover/pending)

use cvkg_core::{
    Event, Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

// ── State hash helpers ─────────────────────────────────────────────────────

/// Produce two distinct hash keys from an editor ID string.
/// `persistent` stores nodes/edges/selection; `interaction` stores drag/hover/pending.
fn state_hashes(id: &str) -> (u64, u64) {
    let mut hp = DefaultHasher::new();
    id.hash(&mut hp);
    b"persistent".hash(&mut hp);
    let persistent = hp.finish();

    let mut hi = DefaultHasher::new();
    id.hash(&mut hi);
    b"interaction".hash(&mut hi);
    let interaction = hi.finish();

    (persistent, interaction)
}

// ── Port ───────────────────────────────────────────────────────────────────

/// Port type determines connection direction and visual placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortType {
    Input,
    Output,
}

/// A connection point on a node.
#[derive(Debug, Clone)]
pub struct Port {
    pub id: String,
    pub label: String,
    pub port_type: PortType,
}

impl Port {
    pub fn new(id: &str, label: &str, port_type: PortType) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            port_type,
        }
    }
}

// ── Node ───────────────────────────────────────────────────────────────────

/// A draggable node in the graph.
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub input_ports: Vec<Port>,
    pub output_ports: Vec<Port>,
    pub color: [f32; 4],
}

impl GraphNode {
    pub fn new(id: &str, label: &str, x: f32, y: f32) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            x,
            y,
            width: 180.0,
            height: 120.0,
            input_ports: Vec::new(),
            output_ports: Vec::new(),
            color: [0.08, 0.08, 0.15, 1.0],
        }
    }

    pub fn with_size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }

    pub fn with_color(mut self, c: [f32; 4]) -> Self {
        self.color = c;
        self
    }

    pub fn with_input(mut self, id: &str, label: &str) -> Self {
        self.input_ports.push(Port::new(id, label, PortType::Input));
        self
    }

    pub fn with_output(mut self, id: &str, label: &str) -> Self {
        self.output_ports
            .push(Port::new(id, label, PortType::Output));
        self
    }

    /// Return the center of a port circle in absolute coordinates.
    pub fn port_position(&self, port: &Port) -> [f32; 2] {
        match port.port_type {
            PortType::Input => {
                let idx = self
                    .input_ports
                    .iter()
                    .position(|p| p.id == port.id)
                    .unwrap_or(0);
                let y_base = self.y + 30.0 + (idx as f32) * 22.0;
                [self.x + 8.0, y_base]
            }
            PortType::Output => {
                let idx = self
                    .output_ports
                    .iter()
                    .position(|p| p.id == port.id)
                    .unwrap_or(0);
                let y_base = self.y + 30.0 + (idx as f32) * 22.0;
                [self.x + self.width - 8.0, y_base]
            }
        }
    }
}

// ── Edge ───────────────────────────────────────────────────────────────────

/// A directed connection from one port to another.
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub id: String,
    pub from_node: String,
    pub from_port: String,
    pub to_node: String,
    pub to_port: String,
}

impl GraphEdge {
    pub fn new(id: &str, from_node: &str, from_port: &str, to_node: &str, to_port: &str) -> Self {
        Self {
            id: id.to_string(),
            from_node: from_node.to_string(),
            from_port: from_port.to_string(),
            to_node: to_node.to_string(),
            to_port: to_port.to_string(),
        }
    }
}

// ── Persistent state ───────────────────────────────────────────────────────

/// Stored in KnowledgeState.component_states under the "persistent" hash.
#[derive(Debug, Clone)]
pub struct NodeGraphPersistentState {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub selected_node: Option<String>,
    pub viewport_offset: [f32; 2],
}

impl NodeGraphPersistentState {
    pub fn new(nodes: Vec<GraphNode>, edges: Vec<GraphEdge>) -> Self {
        Self {
            nodes,
            edges,
            selected_node: None,
            viewport_offset: [0.0, 0.0],
        }
    }
}

// ── Interaction state ──────────────────────────────────────────────────────

/// Stored in KnowledgeState.component_states under the "interaction" hash.
#[derive(Debug, Clone, Default)]
pub struct GraphInteractionState {
    pub dragging_node: Option<String>,
    pub drag_offset: [f32; 2],
    pub hovered_port: Option<(String, String)>, // (node_id, port_id)
    pub selected_node: Option<String>,
    pub pending_edge: Option<PendingEdge>,
}

/// An edge currently being created by the user.
#[derive(Debug, Clone)]
pub struct PendingEdge {
    pub from_node: String,
    pub from_port: String,
    pub cursor_x: f32,
    pub cursor_y: f32,
}

// ── Editor ─────────────────────────────────────────────────────────────────

/// Interactive node graph editor component.
pub struct NodeGraphEditor {
    pub(crate) id: String,
    pub(crate) nodes: Vec<GraphNode>,
    pub(crate) edges: Vec<GraphEdge>,
    pub(crate) selected_node: Option<String>,
}

impl NodeGraphEditor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
            selected_node: None,
        }
    }

    pub fn node(mut self, node: GraphNode) -> Self {
        self.nodes.push(node);
        self
    }

    pub fn edge(mut self, edge: GraphEdge) -> Self {
        self.edges.push(edge);
        self
    }

    pub fn select(mut self, node_id: &str) -> Self {
        self.selected_node = Some(node_id.to_string());
        self
    }

    // ── Bézier drawing ─────────────────────────────────────────────────

    /// Evaluate a cubic Bézier at parameter t.
    fn bezier_point(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2], p3: [f32; 2], t: f32) -> [f32; 2] {
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        let t2 = t * t;
        let t3 = t2 * t;
        [
            mt3 * p0[0] + 3.0 * mt2 * t * p1[0] + 3.0 * mt * t2 * p2[0] + t3 * p3[0],
            mt3 * p0[1] + 3.0 * mt2 * t * p1[1] + 3.0 * mt * t2 * p2[1] + t3 * p3[1],
        ]
    }

    /// Evaluate the tangent of a cubic Bézier at parameter t.
    fn bezier_tangent(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2], p3: [f32; 2], t: f32) -> [f32; 2] {
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let t2 = t * t;
        [
            3.0 * mt2 * (p1[0] - p0[0])
                + 6.0 * mt * t * (p2[0] - p1[0])
                + 3.0 * t2 * (p3[0] - p2[0]),
            3.0 * mt2 * (p1[1] - p0[1])
                + 6.0 * mt * t * (p2[1] - p1[1])
                + 3.0 * t2 * (p3[1] - p2[1]),
        ]
    }

    /// Draw a cubic Bézier curve as a thick tube (line strip).
    fn draw_bezier_tube(
        &self,
        renderer: &mut dyn Renderer,
        from: [f32; 2],
        to: [f32; 2],
        color: [f32; 4],
        width: f32,
    ) {
        let dx = to[0] - from[0];
        let dy = to[1] - from[1];
        let dist = (dx * dx + dy * dy).sqrt();
        let ctrl_offset = (dist * 0.5).max(40.0);

        let p0 = from;
        let p1 = [from[0] + ctrl_offset, from[1]];
        let p2 = [to[0] - ctrl_offset, to[1]];
        let p3 = to;

        const SEGMENTS: usize = 32;
        for i in 0..SEGMENTS {
            let t0 = i as f32 / SEGMENTS as f32;
            let t1 = (i + 1) as f32 / SEGMENTS as f32;
            let a = Self::bezier_point(p0, p1, p2, p3, t0);
            let b = Self::bezier_point(p0, p1, p2, p3, t1);
            renderer.draw_line(a[0], a[1], b[0], b[1], color, width);
        }
    }

    /// Draw an arrow head at the end of a Bézier curve.
    fn draw_arrow_head(
        &self,
        renderer: &mut dyn Renderer,
        from: [f32; 2],
        to: [f32; 2],
        color: [f32; 4],
    ) {
        let dx = to[0] - from[0];
        let dy = to[1] - from[1];
        let dist = (dx * dx + dy * dy).sqrt();
        let ctrl_offset = (dist * 0.5).max(40.0);

        let p0 = from;
        let p1 = [from[0] + ctrl_offset, from[1]];
        let p2 = [to[0] - ctrl_offset, to[1]];
        let p3 = to;

        let tangent = Self::bezier_tangent(p0, p1, p2, p3, 1.0);
        let len = (tangent[0] * tangent[0] + tangent[1] * tangent[1]).sqrt();
        if len < 0.001 {
            return;
        }
        let nx = tangent[0] / len;
        let ny = tangent[1] / len;

        let arrow_len = 10.0;
        let arrow_width = 5.0;

        let tip = to;
        let base = [tip[0] - nx * arrow_len, tip[1] - ny * arrow_len];
        let left = [base[0] + ny * arrow_width, base[1] - nx * arrow_width];
        let right = [base[0] - ny * arrow_width, base[1] + nx * arrow_width];

        // Draw filled arrow (3 lines forming a triangle)
        renderer.draw_line(tip[0], tip[1], left[0], left[1], color, 2.0);
        renderer.draw_line(tip[0], tip[1], right[0], right[1], color, 2.0);
        renderer.draw_line(left[0], left[1], right[0], right[1], color, 2.0);
    }

    /// Draw a dashed Bézier curve (for pending edge preview).
    fn draw_dashed_bezier(
        &self,
        renderer: &mut dyn Renderer,
        from: [f32; 2],
        to: [f32; 2],
        color: [f32; 4],
    ) {
        let dx = to[0] - from[0];
        let dy = to[1] - from[1];
        let dist = (dx * dx + dy * dy).sqrt();
        let ctrl_offset = (dist * 0.5).max(40.0);

        let p0 = from;
        let p1 = [from[0] + ctrl_offset, from[1]];
        let p2 = [to[0] - ctrl_offset, to[1]];
        let p3 = to;

        const SEGMENTS: usize = 24;
        for i in 0..SEGMENTS {
            if i % 2 == 0 {
                continue; // skip every other segment for dash effect
            }
            let t0 = i as f32 / SEGMENTS as f32;
            let t1 = (i + 1) as f32 / SEGMENTS as f32;
            let a = Self::bezier_point(p0, p1, p2, p3, t0);
            let b = Self::bezier_point(p0, p1, p2, p3, t1);
            renderer.draw_line(a[0], a[1], b[0], b[1], color, 2.0);
        }
    }

    // ── Grid ───────────────────────────────────────────────────────────

    fn draw_grid(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let grid_size = 24.0;
        let grid_color = [0.06, 0.06, 0.1, 0.5];

        let start_x = (rect.x / grid_size).floor() * grid_size;
        let start_y = (rect.y / grid_size).floor() * grid_size;

        let mut x = start_x;
        while x < rect.x + rect.width {
            renderer.draw_line(x, rect.y, x, rect.y + rect.height, grid_color, 0.5);
            x += grid_size;
        }

        let mut y = start_y;
        while y < rect.y + rect.height {
            renderer.draw_line(rect.x, y, rect.x + rect.width, y, grid_color, 0.5);
            y += grid_size;
        }
    }
}

// ── View impl ──────────────────────────────────────────────────────────────

impl View for NodeGraphEditor {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let (persistent_hash, interaction_hash) = state_hashes(&self.id);

        // ── Load or initialize persistent state ────────────────────────
        let mut persistent = {
            let s = cvkg_core::load_system_state();
            s.get_component_state::<NodeGraphPersistentState>(persistent_hash)
                .and_then(|lock| lock.read().ok().map(|g| (*g).clone()))
                .unwrap_or_else(|| {
                    NodeGraphPersistentState::new(self.nodes.clone(), self.edges.clone())
                })
        };

        // Wrap in Arc for sharing across closures (clone Arc, not the data)
        let persistent_nodes = Arc::new(persistent.nodes.clone());
        let persistent_edges = Arc::new(persistent.edges.clone());

        // Apply initial selection if set
        if persistent.selected_node.is_none() && self.selected_node.is_some() {
            persistent.selected_node = self.selected_node.clone();
        }

        // ── Load or initialize interaction state ───────────────────────
        let interaction = {
            let s = cvkg_core::load_system_state();
            s.get_component_state::<GraphInteractionState>(interaction_hash)
                .and_then(|lock| lock.read().ok().map(|g| (*g).clone()))
                .unwrap_or_default()
        };

        // ── Register event handlers ────────────────────────────────────
        let _editor_id = self.id.clone();
        let ph = persistent_hash;
        let ih = interaction_hash;

        // pointerdown: start drag or start edge creation
        {
            let pn = persistent_nodes.clone();
            let pe = persistent_edges.clone();
            renderer.register_handler(
                "pointerdown",
                Arc::new(move |event| {
                    if let Event::PointerDown { x, y, .. } = event {
                        cvkg_core::update_system_state(|s| {
                            let mut s2 = s.clone();

                            // Load or init persistent state
                            let mut p = s2
                                .get_component_state::<NodeGraphPersistentState>(ph)
                                .and_then(|lock| lock.read().ok().map(|g| (*g).clone()))
                                .unwrap_or_else(|| {
                                    NodeGraphPersistentState::new((*pn).clone(), (*pe).clone())
                                });

                            let mut i: GraphInteractionState = s2
                                .get_component_state::<GraphInteractionState>(ih)
                                .and_then(|lock| lock.read().ok().map(|g| (*g).clone()))
                                .unwrap_or_default();

                            // Check if clicking on a port first
                            let mut hit_port = false;
                            for node in &p.nodes {
                                for port in node.input_ports.iter().chain(node.output_ports.iter())
                                {
                                    let [px, py] = node.port_position(port);
                                    let dx = x - px;
                                    let dy = y - py;
                                    if dx * dx + dy * dy <= 64.0 {
                                        if port.port_type == PortType::Output {
                                            i.pending_edge = Some(PendingEdge {
                                                from_node: node.id.clone(),
                                                from_port: port.id.clone(),
                                                cursor_x: x,
                                                cursor_y: y,
                                            });
                                            i.hovered_port =
                                                Some((node.id.clone(), port.id.clone()));
                                        }
                                        hit_port = true;
                                        break;
                                    }
                                }
                                if hit_port {
                                    break;
                                }
                            }

                            if !hit_port {
                                // Check if clicking on a node body
                                let mut hit_node = false;
                                for node in p.nodes.iter().rev() {
                                    if x >= node.x
                                        && x <= node.x + node.width
                                        && y >= node.y
                                        && y <= node.y + node.height
                                    {
                                        i.selected_node = Some(node.id.clone());
                                        p.selected_node = Some(node.id.clone());
                                        if y <= node.y + 24.0 {
                                            i.dragging_node = Some(node.id.clone());
                                            i.drag_offset = [x - node.x, y - node.y];
                                        }
                                        hit_node = true;
                                        break;
                                    }
                                }

                                if !hit_node {
                                    // Click on empty space: deselect
                                    p.selected_node = None;
                                    i.selected_node = None;
                                }
                            }

                            s2.set_component_state(ph, p);
                            s2.set_component_state(ih, i);
                            s2
                        });
                    }
                }),
            );
        }

        // pointermove: drag nodes or update pending edge cursor
        {
            let pn = persistent_nodes.clone();
            let pe = persistent_edges.clone();
            renderer.register_handler(
                "pointermove",
                Arc::new(move |event| {
                    if let Event::PointerMove { x, y } = event {
                        cvkg_core::update_system_state(|s| {
                            let mut s2 = s.clone();

                            let mut p = s2
                                .get_component_state::<NodeGraphPersistentState>(ph)
                                .and_then(|lock| lock.read().ok().map(|g| (*g).clone()))
                                .unwrap_or_else(|| {
                                    NodeGraphPersistentState::new((*pn).clone(), (*pe).clone())
                                });

                            let mut i: GraphInteractionState = s2
                                .get_component_state::<GraphInteractionState>(ih)
                                .and_then(|lock| lock.read().ok().map(|g| (*g).clone()))
                                .unwrap_or_default();

                            if let Some(ref node_id) = i.dragging_node
                                && let Some(node) = p.nodes.iter_mut().find(|n| &n.id == node_id)
                            {
                                node.x = x - i.drag_offset[0];
                                node.y = y - i.drag_offset[1];
                            }

                            if let Some(ref mut pending) = i.pending_edge {
                                pending.cursor_x = x;
                                pending.cursor_y = y;
                            }

                            // Update hovered port
                            i.hovered_port = None;
                            for node in &p.nodes {
                                for port in node.input_ports.iter().chain(node.output_ports.iter())
                                {
                                    let [px, py] = node.port_position(port);
                                    let dx = x - px;
                                    let dy = y - py;
                                    if dx * dx + dy * dy <= 64.0 {
                                        i.hovered_port = Some((node.id.clone(), port.id.clone()));
                                        break;
                                    }
                                }
                                if i.hovered_port.is_some() {
                                    break;
                                }
                            }

                            s2.set_component_state(ph, p);
                            s2.set_component_state(ih, i);
                            s2
                        });
                    }
                }),
            );
        }

        // pointerup: finish drag or complete/cancel edge creation
        {
            let pn = persistent_nodes.clone();
            let pe = persistent_edges.clone();
            renderer.register_handler(
                "pointerup",
                Arc::new(move |event| {
                    if let Event::PointerUp { x, y, .. } = event {
                        cvkg_core::update_system_state(|s| {
                            let mut s2 = s.clone();

                            let mut p = s2
                                .get_component_state::<NodeGraphPersistentState>(ph)
                                .and_then(|lock| lock.read().ok().map(|g| (*g).clone()))
                                .unwrap_or_else(|| {
                                    NodeGraphPersistentState::new((*pn).clone(), (*pe).clone())
                                });

                            let mut i: GraphInteractionState = s2
                                .get_component_state::<GraphInteractionState>(ih)
                                .and_then(|lock| lock.read().ok().map(|g| (*g).clone()))
                                .unwrap_or_default();

                            // If we were creating an edge, try to complete it
                            if let Some(ref pending) = i.pending_edge {
                                for node in &p.nodes {
                                    for port in &node.input_ports {
                                        let [px, py] = node.port_position(port);
                                        let dx = x - px;
                                        let dy = y - py;
                                        if dx * dx + dy * dy <= 64.0 {
                                            let edge_id = format!(
                                                "edge_{}_{}_{}_{}",
                                                pending.from_node,
                                                pending.from_port,
                                                node.id,
                                                port.id
                                            );
                                            let new_edge = GraphEdge::new(
                                                &edge_id,
                                                &pending.from_node,
                                                &pending.from_port,
                                                &node.id,
                                                &port.id,
                                            );
                                            p.edges.push(new_edge);
                                            break;
                                        }
                                    }
                                }
                                i.pending_edge = None;
                            }

                            i.dragging_node = None;
                            i.drag_offset = [0.0, 0.0];

                            s2.set_component_state(ph, p);
                            s2.set_component_state(ih, i);
                            s2
                        });
                    }
                }),
            );
        }

        // ── Drawing ─────────────────────────────────────────────────────

        // Background
        renderer.fill_rect(rect, [0.04, 0.04, 0.08, 1.0]);

        // Grid
        self.draw_grid(renderer, rect);

        // ── Draw edges ──────────────────────────────────────────────────
        for edge in persistent_edges.iter() {
            let from_node = persistent_nodes.iter().find(|n| n.id == edge.from_node);
            let to_node = persistent_nodes.iter().find(|n| n.id == edge.to_node);

            if let (Some(from), Some(to)) = (from_node, to_node) {
                let from_port = from.output_ports.iter().find(|p| p.id == edge.from_port);
                let to_port = to.input_ports.iter().find(|p| p.id == edge.to_port);

                if let (Some(fp), Some(tp)) = (from_port, to_port) {
                    let from_pos = from.port_position(fp);
                    let to_pos = to.port_position(tp);

                    let edge_color = [0.3, 0.5, 0.8, 0.8];

                    // Glow layer
                    self.draw_bezier_tube(renderer, from_pos, to_pos, [0.2, 0.4, 0.8, 0.2], 6.0);
                    // Main tube
                    self.draw_bezier_tube(renderer, from_pos, to_pos, edge_color, 2.0);
                    // Arrow head
                    self.draw_arrow_head(renderer, from_pos, to_pos, edge_color);
                }
            }
        }

        // ── Draw pending edge ───────────────────────────────────────────
        if let Some(ref pending) = interaction.pending_edge
            && let Some(from_node) = persistent_nodes.iter().find(|n| n.id == pending.from_node)
            && let Some(from_port) = from_node
                .output_ports
                .iter()
                .find(|p| p.id == pending.from_port)
        {
            let from_pos = from_node.port_position(from_port);
            let to_pos = [pending.cursor_x, pending.cursor_y];
            self.draw_dashed_bezier(renderer, from_pos, to_pos, [0.0, 0.8, 1.0, 0.6]);
            // Start point indicator
            renderer.fill_ellipse(
                Rect {
                    x: from_pos[0] - 6.0,
                    y: from_pos[1] - 6.0,
                    width: 12.0,
                    height: 12.0,
                },
                [0.0, 0.8, 1.0, 0.8],
            );
        }

        // ── Draw nodes ──────────────────────────────────────────────────
        for node in persistent_nodes.iter() {
            let is_selected = persistent.selected_node.as_deref() == Some(&node.id);
            let bg = if is_selected {
                [0.1, 0.2, 0.35, 1.0]
            } else {
                node.color
            };

            // Drop shadow
            renderer.draw_drop_shadow(
                Rect {
                    x: node.x + 2.0,
                    y: node.y + 2.0,
                    width: node.width,
                    height: node.height,
                },
                6.0,
                [0.0, 0.0, 0.0, 0.3],
                8.0,
                0.0,
            );

            // Node body
            renderer.fill_rounded_rect(
                Rect {
                    x: node.x,
                    y: node.y,
                    width: node.width,
                    height: node.height,
                },
                6.0,
                bg,
            );

            // Selection border
            if is_selected {
                renderer.stroke_rounded_rect(
                    Rect {
                        x: node.x,
                        y: node.y,
                        width: node.width,
                        height: node.height,
                    },
                    6.0,
                    [0.0, 0.8, 1.0, 1.0],
                    2.0,
                );
            }

            // Title bar
            renderer.fill_rounded_rect(
                Rect {
                    x: node.x,
                    y: node.y,
                    width: node.width,
                    height: 24.0,
                },
                6.0,
                [0.12, 0.12, 0.2, 0.8],
            );
            // Title separator
            renderer.draw_line(
                node.x + 4.0,
                node.y + 24.0,
                node.x + node.width - 4.0,
                node.y + 24.0,
                [0.2, 0.2, 0.3, 0.6],
                1.0,
            );

            // Title text
            renderer.draw_text(
                &node.label,
                node.x + 8.0,
                node.y + 8.0,
                12.0,
                [0.9, 0.95, 1.0, 1.0],
            );

            // ── Draw ports ───────────────────────────────────────────────
            // Input ports (left side)
            for port in &node.input_ports {
                let [px, py] = node.port_position(port);
                let is_hovered = interaction
                    .hovered_port
                    .as_ref()
                    .is_some_and(|(nid, pid)| nid == &node.id && pid == &port.id);

                let port_color = if is_hovered {
                    [0.0, 0.9, 1.0, 1.0]
                } else {
                    [0.4, 0.6, 0.9, 0.9]
                };

                renderer.fill_ellipse(
                    Rect {
                        x: px - 5.0,
                        y: py - 5.0,
                        width: 10.0,
                        height: 10.0,
                    },
                    port_color,
                );
                renderer.stroke_ellipse(
                    Rect {
                        x: px - 5.0,
                        y: py - 5.0,
                        width: 10.0,
                        height: 10.0,
                    },
                    [0.6, 0.8, 1.0, 0.8],
                    1.0,
                );

                // Port label
                renderer.draw_text(&port.label, px + 10.0, py - 4.0, 9.0, [0.6, 0.7, 0.8, 0.9]);
            }

            // Output ports (right side)
            for port in &node.output_ports {
                let [px, py] = node.port_position(port);
                let is_hovered = interaction
                    .hovered_port
                    .as_ref()
                    .is_some_and(|(nid, pid)| nid == &node.id && pid == &port.id);

                let port_color = if is_hovered {
                    [0.0, 0.9, 1.0, 1.0]
                } else {
                    [0.4, 0.9, 0.6, 0.9]
                };

                renderer.fill_ellipse(
                    Rect {
                        x: px - 5.0,
                        y: py - 5.0,
                        width: 10.0,
                        height: 10.0,
                    },
                    port_color,
                );
                renderer.stroke_ellipse(
                    Rect {
                        x: px - 5.0,
                        y: py - 5.0,
                        width: 10.0,
                        height: 10.0,
                    },
                    [0.6, 1.0, 0.8, 0.8],
                    1.0,
                );

                // Port label (right-aligned)
                let (tw, _) = renderer.measure_text(&port.label, 9.0);
                renderer.draw_text(
                    &port.label,
                    px - 10.0 - tw,
                    py - 4.0,
                    9.0,
                    [0.6, 0.7, 0.8, 0.9],
                );
            }
        }
    }
}

// ── LayoutView impl ────────────────────────────────────────────────────────

impl LayoutView for NodeGraphEditor {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        let max_x = self.nodes.iter().map(|n| n.x + n.width).fold(0.0, f32::max);
        let max_y = self
            .nodes
            .iter()
            .map(|n| n.y + n.height)
            .fold(0.0, f32::max);
        Size {
            width: max_x + 40.0,
            height: max_y + 40.0,
        }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}
