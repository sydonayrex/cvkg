use crate::graph::FlowGraph;
use glam::Vec2;
use std::collections::HashMap;

/// Applies a force-directed auto-layout to the graph using the Fruchterman-Reingold algorithm.
///
/// This algorithm models nodes as repelling particles and edges as attractive springs,
/// settling the graph into an aesthetically pleasing organization.
///
/// # Arguments
/// * `graph` - The flow graph to layout.
/// * `iterations` - Number of simulation steps to run (e.g., 50-100).
pub fn apply_force_directed_layout(graph: &mut FlowGraph, iterations: usize) {
    if graph.nodes.is_empty() {
        return;
    }

    // Prevent giant jumps initially, bounded by canvas area approximations
    let area = 2000.0 * 2000.0;
    let mut k = (area / graph.nodes.len() as f32).sqrt();
    if k == 0.0 {
        k = 1.0;
    }

    // Initial temperature (max displacement per iteration)
    let mut temperatures: HashMap<_, _> = graph.nodes.keys().map(|&id| (id, k * 0.5)).collect();
    let node_ids: Vec<_> = graph.nodes.keys().copied().collect();

    for _ in 0..iterations {
        let mut displacements: HashMap<_, Vec2> =
            graph.nodes.keys().map(|&id| (id, Vec2::ZERO)).collect();

        // 1. Calculate repulsive forces between all nodes
        for i in 0..node_ids.len() {
            for j in (i + 1)..node_ids.len() {
                let v = node_ids[i];
                let u = node_ids[j];

                let p_v = Vec2::from(graph.nodes[&v].position);
                let p_u = Vec2::from(graph.nodes[&u].position);

                let mut delta = p_v - p_u;
                let mut dist = delta.length();

                // Jitter to prevent division by zero if nodes overlap exactly
                if dist < 0.01 {
                    delta = Vec2::new(rand::random::<f32>() - 0.5, rand::random::<f32>() - 0.5);
                    dist = delta.length().max(0.01);
                }

                let force = (k * k) / dist;
                let disp = (delta / dist) * force;

                *displacements.get_mut(&v).unwrap() += disp;
                *displacements.get_mut(&u).unwrap() -= disp;
            }
        }

        // 2. Calculate attractive forces between connected nodes
        for edge in graph.edges.values() {
            let v = edge.source_node;
            let u = edge.target_node;

            if v != u && graph.nodes.contains_key(&v) && graph.nodes.contains_key(&u) {
                let p_v = Vec2::from(graph.nodes[&v].position);
                let p_u = Vec2::from(graph.nodes[&u].position);

                let delta = p_v - p_u;
                let dist = delta.length().max(0.01);

                let force = (dist * dist) / k;
                let disp = (delta / dist) * force;

                *displacements.get_mut(&v).unwrap() -= disp;
                *displacements.get_mut(&u).unwrap() += disp;
            }
        }

        // 3. Apply displacements and cool down
        for (id, node) in graph.nodes.iter_mut() {
            let disp = displacements[id];
            let dist = disp.length();

            if dist > 0.0 {
                let t = temperatures[id];
                let movement = (disp / dist) * dist.min(t);

                node.position.0 += movement.x;
                node.position.1 += movement.y;

                // Cool down temperature
                temperatures.insert(*id, t * 0.95);
            }
        }
    }

    graph.spatial_index_dirty.set(true);
}
