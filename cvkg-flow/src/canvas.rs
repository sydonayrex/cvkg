use crate::edge::FlowEdge;
use crate::graph::FlowGraph;
#[cfg(test)]
use cvkg_core::KvasirId;
use crate::node::FlowNode;
use crate::ribbon::{RibbonBatch, build_ribbon_batch};
use crate::types::{EdgeId, LevelOfDetail, NodeId};
use cvkg_core::Rect;
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Camera transform for pan and zoom.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    /// Pan offset in canvas space.
    pub offset: Vec2,
    /// Zoom level. 1.0 = 100%, 0.5 = 50%, 2.0 = 200%.
    pub zoom: f32,
    /// Minimum allowed zoom level.
    pub min_zoom: f32,
    /// Maximum allowed zoom level.
    pub max_zoom: f32,
}

impl Camera {
    /// Creates a new camera at origin with 1:1 zoom.
    pub fn new() -> Self {
        Self {
            offset: Vec2::ZERO,
            zoom: 1.0,
            min_zoom: 0.1,
            max_zoom: 10.0,
        }
    }

    /// Sets the pan offset.
    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    /// Sets the zoom level.
    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom.clamp(self.min_zoom, self.max_zoom);
        self
    }

    /// Transforms a point from screen space to canvas space.
    pub fn screen_to_canvas(&self, screen_point: Vec2) -> Vec2 {
        (screen_point - self.offset) / self.zoom
    }

    /// Transforms a point from canvas space to screen space.
    pub fn canvas_to_screen(&self, canvas_point: Vec2) -> Vec2 {
        canvas_point * self.zoom + self.offset
    }

    /// Transforms a rectangle from canvas space to screen space.
    pub fn canvas_rect_to_screen(&self, rect: Rect) -> Rect {
        let tl = self.canvas_to_screen(Vec2::new(rect.x, rect.y));
        Rect {
            x: tl.x,
            y: tl.y,
            width: rect.width * self.zoom,
            height: rect.height * self.zoom,
        }
    }

    /// Zooms toward a specific screen point (keeps that point stationary).
    pub fn zoom_at(&mut self, screen_point: Vec2, new_zoom: f32) {
        let new_zoom = new_zoom.clamp(self.min_zoom, self.max_zoom);
        let canvas_point = self.screen_to_canvas(screen_point);
        self.zoom = new_zoom;
        self.offset = screen_point - canvas_point * self.zoom;
    }

    /// Pans the camera by the given screen-space delta.
    pub fn pan(&mut self, delta: Vec2) {
        self.offset += delta;
    }

    /// Returns the visible canvas rectangle in canvas space.
    pub fn visible_canvas_rect(&self, screen_width: f32, screen_height: f32) -> Rect {
        let tl = self.screen_to_canvas(Vec2::ZERO);
        let br = self.screen_to_canvas(Vec2::new(screen_width, screen_height));
        Rect {
            x: tl.x,
            y: tl.y,
            width: br.x - tl.x,
            height: br.y - tl.y,
        }
    }

    /// Returns the current Level of Detail (LoD) based on camera zoom factor.
    ///
    /// # Contract
    /// Categorizes rendering complexity thresholds:
    /// - Detailed: zoom >= 0.7
    /// - Medium: 0.35 <= zoom < 0.7
    /// - Simplified: zoom < 0.35
    pub fn level_of_detail(&self) -> LevelOfDetail {
        if self.zoom >= 0.7 {
            LevelOfDetail::Detailed
        } else if self.zoom >= 0.35 {
            LevelOfDetail::Medium
        } else {
            LevelOfDetail::Simplified
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

/// The main flow canvas that owns the graph, camera, and rendering state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowCanvas {
    /// The flow graph containing nodes and edges.
    pub graph: FlowGraph,
    /// Camera transform for pan and zoom.
    pub camera: Camera,
    /// Pre-built ribbon batch for GPU-instanced edge rendering.
    #[serde(skip)]
    pub ribbon_batch: RibbonBatch,
    /// Whether the ribbon batch needs to be rebuilt.
    pub ribbon_dirty: bool,
    /// Screen width in pixels.
    pub screen_width: f32,
    /// Screen height in pixels.
    pub screen_height: f32,
    /// Background color as RGBA.
    pub background_color: [f32; 4],
    /// Grid snap enabled.
    pub snap_to_grid: bool,
    /// Grid cell size in canvas space.
    pub grid_size: f32,
    /// Whether to show the grid.
    pub show_grid: bool,
    /// Grid color as RGBA.
    pub grid_color: [f32; 4],
}

impl FlowCanvas {
    /// Creates a new flow canvas with default settings.
    pub fn new() -> Self {
        Self {
            graph: FlowGraph::new(),
            camera: Camera::new(),
            ribbon_batch: RibbonBatch::new(),
            ribbon_dirty: true,
            screen_width: 1280.0,
            screen_height: 720.0,
            background_color: [0.08, 0.09, 0.12, 1.0],
            snap_to_grid: false,
            grid_size: 20.0,
            show_grid: true,
            grid_color: [0.15, 0.16, 0.2, 0.5],
        }
    }

    /// Sets the screen dimensions.
    pub fn with_screen_size(mut self, width: f32, height: f32) -> Self {
        self.screen_width = width;
        self.screen_height = height;
        self
    }

    /// Sets the background color.
    pub fn with_background(mut self, color: [f32; 4]) -> Self {
        self.background_color = color;
        self
    }

    /// Enables or disables grid snapping.
    pub fn with_snap_to_grid(mut self, enabled: bool) -> Self {
        self.snap_to_grid = enabled;
        self
    }

    /// Sets the grid size.
    pub fn with_grid_size(mut self, size: f32) -> Self {
        self.grid_size = size.max(1.0);
        self
    }

    /// Handles a pointer wheel (scroll) event for zooming.
    ///
    /// The `delta` parameter is the scroll delta (positive = zoom in, negative = zoom out).
    /// The `screen_x` and `screen_y` parameters are the pointer position in screen space.
    pub fn handle_scroll(&mut self, screen_x: f32, screen_y: f32, delta: f32) {
        let zoom_factor = if delta > 0.0 {
            1.1
        } else if delta < 0.0 {
            0.9
        } else {
            1.0
        };
        let new_zoom = self.camera.zoom * zoom_factor;
        self.camera.zoom_at(Vec2::new(screen_x, screen_y), new_zoom);
    }

    /// Handles a pan gesture (middle-click drag or two-finger scroll).
    pub fn handle_pan(&mut self, dx: f32, dy: f32) {
        self.camera.pan(Vec2::new(dx, dy));
    }

    /// Resets the camera to origin with 1:1 zoom.
    pub fn reset_camera(&mut self) {
        self.camera = Camera::new();
    }

    /// Fits all nodes in view.
    pub fn fit_to_content(&mut self) {
        if self.graph.nodes.is_empty() {
            return;
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for node in self.graph.nodes.values() {
            min_x = min_x.min(node.position.0);
            min_y = min_y.min(node.position.1);
            max_x = max_x.max(node.position.0 + node.size.0);
            max_y = max_y.max(node.position.1 + node.size.1);
        }

        let content_width = max_x - min_x;
        let content_height = max_y - min_y;
        let padding = 40.0;

        if content_width > 0.0 && content_height > 0.0 {
            let zoom_x = (self.screen_width - padding * 2.0) / content_width;
            let zoom_y = (self.screen_height - padding * 2.0) / content_height;
            let zoom = zoom_x
                .min(zoom_y)
                .clamp(self.camera.min_zoom, self.camera.max_zoom);

            self.camera.zoom = zoom;
            self.camera.offset = Vec2::new(
                (self.screen_width - content_width * zoom) / 2.0 - min_x * zoom,
                (self.screen_height - content_height * zoom) / 2.0 - min_y * zoom,
            );
        }
    }

    /// Rebuilds the ribbon batch from the current graph state.
    ///
    /// This should be called before rendering whenever edges or nodes
    /// have moved. The batch is stored in `self.ribbon_batch` and can
    /// be submitted to the GPU renderer.
    pub fn rebuild_ribbons(&mut self) {
        let edges: Vec<FlowEdge> = self.graph.edges.values().cloned().collect();
        let nodes = self.graph.nodes.clone();
        self.ribbon_batch = build_ribbon_batch(&edges, &nodes);
        self.ribbon_dirty = false;
    }

    /// Marks the ribbon batch as dirty (needs rebuild).
    pub fn invalidate_ribbons(&mut self) {
        self.ribbon_dirty = true;
        self.graph.spatial_index_dirty.set(true);
    }

    /// Returns a reference to the current ribbon batch, rebuilding if necessary.
    pub fn ribbons(&mut self) -> &RibbonBatch {
        if self.ribbon_dirty {
            self.rebuild_ribbons();
        }
        &self.ribbon_batch
    }

    /// Returns the ribbon batch vertex count (for GPU buffer sizing).
    pub fn ribbon_vertex_count(&self) -> usize {
        self.ribbon_batch.vertices.len()
    }

    /// Returns the ribbon batch index count (for GPU buffer sizing).
    pub fn ribbon_index_count(&self) -> usize {
        self.ribbon_batch.indices.len()
    }

    /// Adds a node to the canvas.
    pub fn add_node(&mut self, node: FlowNode) {
        self.graph.add_node(node);
        self.invalidate_ribbons();
    }

    /// Removes a node from the canvas by ID.
    pub fn remove_node(&mut self, id: NodeId) {
        self.graph.nodes.remove(&id);
        self.graph.spatial_index_dirty.set(true);
        self.invalidate_ribbons();
    }

    /// Adds an edge to the canvas.
    pub fn add_edge(&mut self, edge: FlowEdge) {
        self.graph.add_edge(edge);
        self.invalidate_ribbons();
    }

    /// Removes an edge from the canvas by ID.
    pub fn remove_edge(&mut self, edge_id: u64) {
        self.graph.edges.remove(&EdgeId(edge_id));
        self.invalidate_ribbons();
    }

    /// Returns the node at the given screen position, if any.
    /// Nodes with higher z_index take priority when overlapping.
    pub fn node_at_screen(&self, screen_x: f32, screen_y: f32) -> Option<NodeId> {
        let canvas_pos = self.camera.screen_to_canvas(Vec2::new(screen_x, screen_y));
        let mut candidates: Vec<_> = self.graph
            .nodes
            .iter()
            .filter(|(_, node)| {
                canvas_pos.x >= node.position.0
                    && canvas_pos.x <= node.position.0 + node.size.0
                    && canvas_pos.y >= node.position.1
                    && canvas_pos.y <= node.position.1 + node.size.1
            })
            .collect();
        // Sort by z_index descending so topmost node is returned
        candidates.sort_by(|a, b| b.1.z_index.partial_cmp(&a.1.z_index).unwrap_or(std::cmp::Ordering::Equal));
        candidates.first().map(|(id, _)| *id).copied()
    }

    /// Returns nodes within the given screen-space rectangle.
    pub fn nodes_in_screen_rect(
        &self,
        screen_x: f32,
        screen_y: f32,
        screen_w: f32,
        screen_h: f32,
    ) -> Vec<NodeId> {
        let tl = self.camera.screen_to_canvas(Vec2::new(screen_x, screen_y));
        let br = self
            .camera
            .screen_to_canvas(Vec2::new(screen_x + screen_w, screen_y + screen_h));
        self.graph
            .nodes_in_rect(tl.x, tl.y, br.x - tl.x, br.y - tl.y)
    }

    /// Returns nodes near the given screen point within a screen-space radius.
    pub fn nodes_near_screen_point(
        &self,
        screen_x: f32,
        screen_y: f32,
        screen_radius: f32,
    ) -> Vec<NodeId> {
        let canvas_point = self.camera.screen_to_canvas(Vec2::new(screen_x, screen_y));
        let canvas_radius = screen_radius / self.camera.zoom;
        self.graph.nodes_near_point(canvas_point, canvas_radius)
    }

    /// Updates edge animations. Call once per frame with delta time.
    pub fn tick_animations(&mut self, dt: f32) -> bool {
        let mut any_animating = false;
        for edge in self.graph.edges.values_mut() {
            if edge.tick_animation(dt) {
                any_animating = true;
            }
        }
        if any_animating {
            self.invalidate_ribbons();
        }
        any_animating
    }

    /// Returns the visible canvas rectangle for culling.
    pub fn visible_rect(&self) -> Rect {
        self.camera
            .visible_canvas_rect(self.screen_width, self.screen_height)
    }

    /// Returns the current Level of Detail (LoD) for the canvas based on the camera zoom.
    pub fn level_of_detail(&self) -> LevelOfDetail {
        self.camera.level_of_detail()
    }
}

impl Default for FlowCanvas {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::FlowEdge;

    #[test]
    fn test_canvas_creation() {
        let canvas = FlowCanvas::new();
        assert_eq!(canvas.screen_width, 1280.0);
        assert_eq!(canvas.screen_height, 720.0);
        assert_eq!(canvas.camera.zoom, 1.0);
        assert!(canvas.ribbon_dirty);
    }

    #[test]
    fn test_camera_screen_to_canvas() {
        let mut cam = Camera::new();
        assert_eq!(
            cam.screen_to_canvas(Vec2::new(100.0, 200.0)),
            Vec2::new(100.0, 200.0)
        );

        cam.offset = Vec2::new(50.0, 50.0);
        assert_eq!(
            cam.screen_to_canvas(Vec2::new(100.0, 200.0)),
            Vec2::new(50.0, 150.0)
        );

        cam.zoom = 2.0;
        cam.offset = Vec2::ZERO;
        assert_eq!(
            cam.screen_to_canvas(Vec2::new(100.0, 200.0)),
            Vec2::new(50.0, 100.0)
        );
    }

    #[test]
    fn test_camera_canvas_to_screen() {
        let mut cam = Camera::new();
        cam.zoom = 2.0;
        cam.offset = Vec2::new(10.0, 20.0);
        let screen = cam.canvas_to_screen(Vec2::new(50.0, 60.0));
        assert!((screen.x - 110.0).abs() < 0.001);
        assert!((screen.y - 140.0).abs() < 0.001);
    }

    #[test]
    fn test_camera_zoom_at() {
        let mut cam = Camera::new();
        cam.zoom_at(Vec2::new(100.0, 100.0), 2.0);
        assert!((cam.zoom - 2.0).abs() < 0.001);
        let before = Vec2::new(100.0, 100.0);
        let canvas_before = (before - Vec2::ZERO) / 1.0;
        let canvas_after = cam.screen_to_canvas(before);
        assert!((canvas_before.x - canvas_after.x).abs() < 0.001);
        assert!((canvas_before.y - canvas_after.y).abs() < 0.001);
    }

    #[test]
    fn test_camera_pan() {
        let mut cam = Camera::new();
        cam.pan(Vec2::new(10.0, -5.0));
        assert_eq!(cam.offset, Vec2::new(10.0, -5.0));
    }

    #[test]
    fn test_camera_visible_rect() {
        let cam = Camera::new();
        let rect = cam.visible_canvas_rect(800.0, 600.0);
        assert_eq!(rect.x, 0.0);
        assert_eq!(rect.y, 0.0);
        assert_eq!(rect.width, 800.0);
        assert_eq!(rect.height, 600.0);
    }

    #[test]
    fn test_camera_clamp_zoom() {
        let mut cam = Camera::new();
        cam.zoom_at(Vec2::ZERO, 100.0);
        assert_eq!(cam.zoom, 10.0);

        cam.zoom_at(Vec2::ZERO, 0.001);
        assert_eq!(cam.zoom, 0.1);
    }

    #[test]
    fn test_canvas_scroll_zoom() {
        let mut canvas = FlowCanvas::new();
        let initial_zoom = canvas.camera.zoom;

        canvas.handle_scroll(400.0, 300.0, 1.0);
        assert!(canvas.camera.zoom > initial_zoom);

        canvas.handle_scroll(400.0, 300.0, -1.0);
        assert!(canvas.camera.zoom < initial_zoom * 1.1);
    }

    #[test]
    fn test_canvas_pan() {
        let mut canvas = FlowCanvas::new();
        canvas.handle_pan(50.0, -30.0);
        assert_eq!(canvas.camera.offset, Vec2::new(50.0, -30.0));
    }

    #[test]
    fn test_canvas_reset_camera() {
        let mut canvas = FlowCanvas::new();
        canvas.handle_pan(100.0, 200.0);
        canvas.handle_scroll(400.0, 300.0, 5.0);
        canvas.reset_camera();
        assert_eq!(canvas.camera.zoom, 1.0);
        assert_eq!(canvas.camera.offset, Vec2::ZERO);
    }

    #[test]
    fn test_canvas_fit_to_content() {
        let mut canvas = FlowCanvas::new().with_screen_size(800.0, 600.0);

        let mut n1 = FlowNode::new(KvasirId(1), "A", (100.0, 100.0));
        n1.size = (200.0, 100.0);
        let mut n2 = FlowNode::new(KvasirId(2), "B", (400.0, 300.0));
        n2.size = (200.0, 100.0);

        canvas.add_node(n1);
        canvas.add_node(n2);
        canvas.fit_to_content();

        assert!(canvas.camera.zoom > 0.0);
    }

    #[test]
    fn test_canvas_fit_to_content_empty() {
        let mut canvas = FlowCanvas::new().with_screen_size(800.0, 600.0);
        canvas.fit_to_content();
        assert_eq!(canvas.camera.zoom, 1.0);
    }

    #[test]
    fn test_canvas_add_node_invalidates_ribbons() {
        let mut canvas = FlowCanvas::new();
        canvas.ribbon_dirty = false;
        let node = FlowNode::new(KvasirId(1), "Test", (0.0, 0.0));
        canvas.add_node(node);
        assert!(canvas.ribbon_dirty);
    }

    #[test]
    fn test_canvas_node_at_screen() {
        let mut canvas = FlowCanvas::new();
        let mut node = FlowNode::new(KvasirId(1), "Test", (100.0, 100.0));
        node.size = (150.0, 80.0);
        canvas.add_node(node);

        let found = canvas.node_at_screen(150.0, 120.0);
        assert_eq!(found, Some(KvasirId(1)));

        let found = canvas.node_at_screen(50.0, 50.0);
        assert_eq!(found, None);
    }

    #[test]
    fn test_canvas_nodes_in_screen_rect() {
        let mut canvas = FlowCanvas::new();
        canvas.add_node(FlowNode::new(KvasirId(1), "A", (50.0, 50.0)));
        canvas.add_node(FlowNode::new(KvasirId(2), "B", (500.0, 500.0)));

        let found = canvas.nodes_in_screen_rect(0.0, 0.0, 200.0, 200.0);
        assert!(found.contains(&KvasirId(1)));
        assert!(!found.contains(&KvasirId(2)));
    }

    #[test]
    fn test_canvas_tick_animations() {
        let mut canvas = FlowCanvas::new();

        let n1 = FlowNode::new(KvasirId(1), "A", (0.0, 0.0));
        let n2 = FlowNode::new(KvasirId(2), "B", (200.0, 0.0));
        canvas.add_node(n1);
        canvas.add_node(n2);

        let mut edge = FlowEdge::new(1, KvasirId(1), 0, KvasirId(2), 0);
        edge.restart_animation();
        canvas.add_edge(edge);

        assert!(canvas.tick_animations(0.016));

        for _ in 0..200 {
            canvas.tick_animations(0.016);
        }
        assert!(!canvas.tick_animations(0.016));
    }

    #[test]
    fn test_canvas_visible_rect() {
        let canvas = FlowCanvas::new().with_screen_size(800.0, 600.0);
        let rect = canvas.visible_rect();
        assert_eq!(rect.x, 0.0);
        assert_eq!(rect.y, 0.0);
        assert_eq!(rect.width, 800.0);
        assert_eq!(rect.height, 600.0);
    }

    #[test]
    fn test_canvas_ribbons_rebuild() {
        use crate::port::FlowPort;
        use crate::types::{PortDirection, PortPosition};
        let mut canvas = FlowCanvas::new();
        let mut n1 = FlowNode::new(KvasirId(1), "A", (0.0, 0.0));
        n1.add_port(FlowPort::new(
            crate::types::PortId(1),
            KvasirId(1),
            PortPosition::Right,
            PortDirection::Output,
        ));
        canvas.add_node(n1);

        let mut n2 = FlowNode::new(KvasirId(2), "B", (200.0, 0.0));
        n2.add_port(FlowPort::new(
            crate::types::PortId(2),
            KvasirId(2),
            PortPosition::Left,
            PortDirection::Input,
        ));
        canvas.add_node(n2);

        let edge = FlowEdge::new(1, KvasirId(1), 0, KvasirId(2), 0);
        canvas.add_edge(edge);

        let batch = canvas.ribbons();
        assert!(!batch.is_empty());
        assert!(canvas.ribbon_vertex_count() > 0);
        assert!(canvas.ribbon_index_count() > 0);
    }

    #[test]
    fn test_spatial_hash_grid_correctness() {
        let mut canvas = FlowCanvas::new();
        canvas.add_node(FlowNode::new(KvasirId(1), "NodeA", (10.0, 10.0)));
        canvas.add_node(FlowNode::new(KvasirId(2), "NodeB", (500.0, 500.0)));
        canvas.add_node(FlowNode::new(KvasirId(3), "NodeC", (1000.0, 10.0)));

        let in_rect = canvas.graph.nodes_in_rect(0.0, 0.0, 600.0, 600.0);
        assert_eq!(in_rect.len(), 2);
        assert!(in_rect.contains(&KvasirId(1)));
        assert!(in_rect.contains(&KvasirId(2)));
        assert!(!in_rect.contains(&KvasirId(3)));

        let near_b = canvas
            .graph
            .nodes_near_point(glam::Vec2::new(510.0, 510.0), 30.0);
        assert_eq!(near_b.len(), 1);
        assert_eq!(near_b[0], KvasirId(2));
    }

    #[test]
    fn test_camera_lod_bounds() {
        let mut canvas = FlowCanvas::new();

        canvas.camera.zoom = 1.0;
        assert_eq!(canvas.level_of_detail(), LevelOfDetail::Detailed);

        canvas.camera.zoom = 0.7;
        assert_eq!(canvas.level_of_detail(), LevelOfDetail::Detailed);

        canvas.camera.zoom = 0.5;
        assert_eq!(canvas.level_of_detail(), LevelOfDetail::Medium);

        canvas.camera.zoom = 0.35;
        assert_eq!(canvas.level_of_detail(), LevelOfDetail::Medium);

        canvas.camera.zoom = 0.2;
        assert_eq!(canvas.level_of_detail(), LevelOfDetail::Simplified);
    }
}
