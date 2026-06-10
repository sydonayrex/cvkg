use crate::edge::FlowEdge;
use crate::node::FlowNode;
use crate::types::{NodeId, PortPosition};
use glam::Vec2;
use std::collections::HashMap;

/// A single vertex in a ribbon mesh.
///
/// Each ribbon segment has two vertices (left and right of the curve),
/// forming a quad strip that represents a thick bezier connection.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RibbonVertex {
    /// XY position in canvas space.
    pub position: [f32; 2],
    /// UV coordinates for texture sampling and edge effects.
    pub uv: [f32; 2],
    /// RGBA color with alpha for edge fading.
    pub color: [f32; 4],
    /// Ribbon width at this vertex in pixels.
    pub width: f32,
    /// Speed of the data pulse.
    pub flow_speed: f32,
    /// Color of the data pulse.
    pub pulse_color: [f32; 4],
}

/// A batch of ribbon geometry for GPU-instanced edge rendering.
///
/// Contains all ribbon vertices and indices for a single frame.
/// Submit this batch to the GPU renderer's vertex/index buffers
/// for a single draw call covering all edges.
#[derive(Debug, Clone, PartialEq)]
pub struct RibbonBatch {
    /// Vertex data for all ribbons.
    pub vertices: Vec<RibbonVertex>,
    /// Index data (triangle list) for all ribbons.
    pub indices: Vec<u32>,
}

impl RibbonBatch {
    /// Creates an empty ribbon batch.
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Reserves capacity for the given number of edges.
    ///
    /// Each edge generates approximately `segments * 2` vertices and
    /// `segments * 6` indices.
    pub fn reserve(&mut self, edge_count: usize, segments_per_edge: usize) {
        self.vertices.reserve(edge_count * segments_per_edge * 2);
        self.indices.reserve(edge_count * segments_per_edge * 6);
    }

    /// Returns the number of vertices in the batch.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of indices in the batch.
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    /// Returns true if the batch contains no geometry.
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Clears all vertices and indices from the batch.
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }
}

impl Default for RibbonBatch {
    fn default() -> Self {
        Self::new()
    }
}

/// Tessellates a cubic bezier curve into a polyline.
///
/// Evaluates the curve at `segments + 1` evenly spaced parameter values
/// from t=0.0 to t=1.0.
///
/// # Arguments
/// * `p0` - Start point.
/// * `p1` - First control point.
/// * `p2` - Second control point.
/// * `p3` - End point.
/// * `segments` - Number of line segments to generate.
///
/// # Returns
/// A `Vec<Vec2>` of `segments + 1` points along the curve.
pub fn tessellate_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, segments: usize) -> Vec<Vec2> {
    if segments == 0 {
        return vec![p0];
    }
    let mut points = Vec::with_capacity(segments + 1);
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let mt = 1.0 - t;
        let point =
            mt * mt * mt * p0 + 3.0 * mt * mt * t * p1 + 3.0 * mt * t * t * p2 + t * t * t * p3;
        points.push(point);
    }
    points
}

/// Tessellates a cubic Bezier curve into a polyline with uniform arc-length spacing.
///
/// # Contract
/// Generates a list of vertices positioned at uniform physical distance increments along the
/// Bezier contour. Returns the list of 2D coordinates and their corresponding normalized distance
/// fractions (0.0 to 1.0) along the curve to ensure smooth, constant-speed shader animations.
pub fn tessellate_bezier_uniform(
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
    segments: usize,
) -> (Vec<Vec2>, Vec<f32>) {
    if segments == 0 {
        return (vec![p0], vec![0.0]);
    }

    // 1. High-resolution sampling to approximate cumulative arc length
    const SAMPLE_COUNT: usize = 64;
    let mut t_samples = Vec::with_capacity(SAMPLE_COUNT + 1);
    let mut points_raw = Vec::with_capacity(SAMPLE_COUNT + 1);
    for i in 0..=SAMPLE_COUNT {
        let t = i as f32 / SAMPLE_COUNT as f32;
        let mt = 1.0 - t;
        let p = mt * mt * mt * p0 + 3.0 * mt * mt * t * p1 + 3.0 * mt * t * t * p2 + t * t * t * p3;
        points_raw.push(p);
        t_samples.push(t);
    }

    let mut cumulative_lengths = Vec::with_capacity(SAMPLE_COUNT + 1);
    cumulative_lengths.push(0.0f32);
    let mut current_length = 0.0f32;
    for i in 1..=SAMPLE_COUNT {
        current_length += points_raw[i].distance(points_raw[i - 1]);
        cumulative_lengths.push(current_length);
    }

    let total_length = current_length;

    // 2. Query target points at uniform physical distance steps
    let mut points = Vec::with_capacity(segments + 1);
    let mut uvs = Vec::with_capacity(segments + 1);

    for i in 0..=segments {
        let target_fraction = i as f32 / segments as f32;
        let target_dist = target_fraction * total_length;

        // Locate cumulative length interval
        let mut idx = 0;
        while idx < SAMPLE_COUNT && cumulative_lengths[idx + 1] < target_dist {
            idx += 1;
        }

        let t = if idx >= SAMPLE_COUNT {
            1.0f32
        } else {
            let l0 = cumulative_lengths[idx];
            let l1 = cumulative_lengths[idx + 1];
            let segment_len = l1 - l0;
            let factor = if segment_len > 0.0 {
                (target_dist - l0) / segment_len
            } else {
                0.0
            };
            let t0 = t_samples[idx];
            let t1 = t_samples[idx + 1];
            t0 + factor * (t1 - t0)
        };

        let mt = 1.0 - t;
        let p = mt * mt * mt * p0 + 3.0 * mt * mt * t * p1 + 3.0 * mt * t * t * p2 + t * t * t * p3;
        points.push(p);
        uvs.push(target_fraction);
    }

    (points, uvs)
}

/// Computes the center position of a port on a node.
///
/// The port position is determined by the port's `PortPosition` variant
/// (Top, Bottom, Left, Right) and the node's bounding rectangle.
fn port_center(node: &FlowNode, port_position: PortPosition) -> Vec2 {
    let (nx, ny) = node.position;
    let (nw, nh) = node.size;
    match port_position {
        PortPosition::Top => Vec2::new(nx + nw / 2.0, ny),
        PortPosition::Bottom => Vec2::new(nx + nw / 2.0, ny + nh),
        PortPosition::Left => Vec2::new(nx, ny + nh / 2.0),
        PortPosition::Right => Vec2::new(nx + nw, ny + nh / 2.0),
    }
}

/// Computes the outward direction vector for a port position.
fn port_direction(dir: PortPosition) -> Vec2 {
    match dir {
        PortPosition::Top => Vec2::new(0.0, -1.0),
        PortPosition::Bottom => Vec2::new(0.0, 1.0),
        PortPosition::Left => Vec2::new(-1.0, 0.0),
        PortPosition::Right => Vec2::new(1.0, 0.0),
    }
}

/// Finds the port position for a given port index on a node.
///
/// Returns `None` if the port index is out of bounds.
fn port_position_at(node: &FlowNode, port_idx: usize) -> Option<PortPosition> {
    node.ports.get(port_idx).map(|p| p.position)
}

/// Builds a ribbon batch from the given edges and nodes.
///
/// For each edge, the function:
/// 1. Looks up the source and target nodes.
/// 2. Computes the port center positions and outward directions.
/// 3. Derives cubic bezier control points offset from the ports.
/// 4. Tessellates the curve into a polyline.
/// 5. Generates a quad strip (two vertices per tessellation point, offset
///    perpendicular to the curve tangent) with appropriate UVs, colors, and widths.
///
/// Edges whose source or target node cannot be found are silently skipped.
///
/// # Arguments
/// * `edges` - Slice of `FlowEdge` connections to render.
/// * `nodes` - Map from `NodeId` to `FlowNode` for position/size lookups.
///
/// # Returns
/// A `RibbonBatch` containing all ribbon geometry.
pub fn build_ribbon_batch(edges: &[FlowEdge], nodes: &HashMap<NodeId, FlowNode>) -> RibbonBatch {
    let mut batch = RibbonBatch::new();
    const SEGMENTS: usize = 16;

    for edge in edges {
        // Look up source and target nodes
        let Some(src_node) = nodes.get(&edge.source_node) else {
            continue;
        };
        let Some(tgt_node) = nodes.get(&edge.target_node) else {
            continue;
        };

        // Get port positions
        let Some(src_port_pos) = port_position_at(src_node, edge.source_port_idx) else {
            continue;
        };
        let Some(tgt_port_pos) = port_position_at(tgt_node, edge.target_port_idx) else {
            continue;
        };

        let src_center = port_center(src_node, src_port_pos);
        let tgt_center = port_center(tgt_node, tgt_port_pos);
        let src_dir = port_direction(src_port_pos);
        let tgt_dir = port_direction(tgt_port_pos);

        // Compute bezier control points
        let handle_offset = (tgt_center - src_center).length() * 0.4;
        let p0 = src_center;
        let p1 = src_center + src_dir * handle_offset;
        let p2 = tgt_center + tgt_dir * handle_offset;
        let p3 = tgt_center;

        // Tessellate the bezier curve using uniform physical spacing
        let (points, uvs) = tessellate_bezier_uniform(p0, p1, p2, p3, SEGMENTS);

        // Get effective color and width from edge (includes hover/selection state)
        let color = edge.effective_color();
        let width = edge.effective_width();
        let flow_speed = edge.flow_speed;
        let pulse_color = edge.pulse_color;

        // Generate quad strip
        let base_index = batch.vertices.len() as u32;
        for (i, point) in points.iter().enumerate() {
            let t = uvs[i];

            // Compute tangent for perpendicular offset
            let tangent = if i + 1 < points.len() {
                (points[i + 1] - *point).normalize_or_zero()
            } else {
                (*point - points[i - 1]).normalize_or_zero()
            };
            let normal = Vec2::new(-tangent.y, tangent.x);

            // Taper width at endpoints
            let taper = if t < 0.1 {
                t / 0.1
            } else if t > 0.9 {
                (1.0 - t) / 0.1
            } else {
                1.0
            };
            let w = width * taper;

            // Left vertex
            batch.vertices.push(RibbonVertex {
                position: (point + normal * w * 0.5).into(),
                uv: [t, 0.0],
                color,
                width: w,
                flow_speed,
                pulse_color,
            });

            // Right vertex
            batch.vertices.push(RibbonVertex {
                position: (point - normal * w * 0.5).into(),
                uv: [t, 1.0],
                color,
                width: w,
                flow_speed,
                pulse_color,
            });

            // Generate indices for quad strip
            if i > 0 {
                let idx = base_index + (i as u32) * 2;
                // Triangle 1
                batch.indices.push(idx - 2);
                batch.indices.push(idx - 1);
                batch.indices.push(idx);
                // Triangle 2
                batch.indices.push(idx - 1);
                batch.indices.push(idx + 1);
                batch.indices.push(idx);
            }
        }
    }

    batch
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::FlowEdge;
    use crate::node::FlowNode;
    use crate::types::{NodeId, PortDirection, PortPosition};

    #[test]
    fn test_tessellate_bezier() {
        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(1.0, 2.0);
        let p2 = Vec2::new(3.0, 2.0);
        let p3 = Vec2::new(4.0, 0.0);

        let points = tessellate_bezier(p0, p1, p2, p3, 4);
        assert_eq!(points.len(), 5);
        assert_eq!(points[0], p0);
        assert_eq!(points[4], p3);
    }

    #[test]
    fn test_tessellate_bezier_zero_segments() {
        let points = tessellate_bezier(Vec2::ZERO, Vec2::X, Vec2::Y, Vec2::ONE, 0);
        assert_eq!(points.len(), 1);
        assert_eq!(points[0], Vec2::ZERO);
    }

    #[test]
    fn test_ribbon_batch_empty() {
        let batch = RibbonBatch::new();
        assert!(batch.is_empty());
        assert_eq!(batch.vertex_count(), 0);
        assert_eq!(batch.index_count(), 0);
    }

    #[test]
    fn test_ribbon_batch_clear() {
        let mut batch = RibbonBatch::new();
        batch.vertices.push(RibbonVertex {
            position: [0.0, 0.0],
            uv: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            width: 2.0,
            flow_speed: 0.0,
            pulse_color: [1.0, 1.0, 1.0, 1.0],
        });
        batch.indices.push(0);
        assert!(!batch.is_empty());
        batch.clear();
        assert!(batch.is_empty());
    }

    #[test]
    fn test_ribbon_batch_default() {
        let batch = RibbonBatch::default();
        assert!(batch.is_empty());
    }

    #[test]
    fn test_ribbon_batch_reserve() {
        let mut batch = RibbonBatch::new();
        batch.reserve(10, 16);
        assert!(batch.vertices.capacity() >= 320);
        assert!(batch.indices.capacity() >= 960);
    }

    #[test]
    fn test_build_ribbon_batch_single_edge() {
        let mut nodes = HashMap::new();

        let mut n1 = FlowNode::new(NodeId(1), "Source", (0.0, 0.0));
        n1.size = (100.0, 60.0);
        n1.ports.push(crate::port::FlowPort::new(
            crate::types::PortId(1),
            NodeId(1),
            PortPosition::Right,
            PortDirection::Output,
        ));
        nodes.insert(NodeId(1), n1);

        let mut n2 = FlowNode::new(NodeId(2), "Target", (300.0, 0.0));
        n2.size = (100.0, 60.0);
        n2.ports.push(crate::port::FlowPort::new(
            crate::types::PortId(2),
            NodeId(2),
            PortPosition::Left,
            PortDirection::Input,
        ));
        nodes.insert(NodeId(2), n2);

        let edge = FlowEdge::new(1, NodeId(1), 0, NodeId(2), 0);
        let edges = vec![edge];

        let batch = build_ribbon_batch(&edges, &nodes);
        assert!(!batch.is_empty());

        // 17 tessellation points * 2 vertices = 34 vertices
        assert_eq!(batch.vertex_count(), 34);
        // 16 segments * 6 indices = 96 indices
        assert_eq!(batch.index_count(), 96);
    }

    #[test]
    fn test_build_ribbon_batch_missing_node() {
        let mut nodes = HashMap::new();
        nodes.insert(NodeId(1), FlowNode::new(NodeId(1), "Only", (0.0, 0.0)));

        // Edge references non-existent target node
        let edge = FlowEdge::new(1, NodeId(1), 0, NodeId(99), 0);
        let edges = vec![edge];

        let batch = build_ribbon_batch(&edges, &nodes);
        assert!(batch.is_empty());
    }

    #[test]
    fn test_port_center_right() {
        let mut node = FlowNode::new(NodeId(1), "Test", (100.0, 200.0));
        node.size = (150.0, 80.0);
        let center = port_center(&node, PortPosition::Right);
        assert!((center.x - 250.0).abs() < 0.001);
        assert!((center.y - 240.0).abs() < 0.001);
    }

    #[test]
    fn test_port_center_left() {
        let mut node = FlowNode::new(NodeId(1), "Test", (100.0, 200.0));
        node.size = (150.0, 80.0);
        let center = port_center(&node, PortPosition::Left);
        assert!((center.x - 100.0).abs() < 0.001);
        assert!((center.y - 240.0).abs() < 0.001);
    }

    #[test]
    fn test_port_direction() {
        assert_eq!(port_direction(PortPosition::Right), Vec2::new(1.0, 0.0));
        assert_eq!(port_direction(PortPosition::Left), Vec2::new(-1.0, 0.0));
        assert_eq!(port_direction(PortPosition::Top), Vec2::new(0.0, -1.0));
        assert_eq!(port_direction(PortPosition::Bottom), Vec2::new(0.0, 1.0));
    }

    #[test]
    fn test_tessellate_bezier_uniform() {
        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(10.0, 20.0);
        let p2 = Vec2::new(20.0, -10.0);
        let p3 = Vec2::new(30.0, 0.0);

        let (points, uvs) = tessellate_bezier_uniform(p0, p1, p2, p3, 8);
        assert_eq!(points.len(), 9);
        assert_eq!(uvs.len(), 9);

        // Verify boundaries
        assert_eq!(points[0], p0);
        assert_eq!(points[8], p3);
        assert_eq!(uvs[0], 0.0);
        assert_eq!(uvs[8], 1.0);

        // Verify uniform step distances
        let first_dist = points[1].distance(points[0]);
        for i in 1..8 {
            let dist = points[i + 1].distance(points[i]);
            assert!(
                (dist - first_dist).abs() < 0.25,
                "Expected uniform step distance, got {} vs {}",
                dist,
                first_dist
            );
        }
    }
}
