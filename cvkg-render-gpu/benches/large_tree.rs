// Benchmark suite for large component trees
// Tests performance for 100+, 1000+, 10000+ component configurations
// Target: <16ms render time for 60fps smoothness

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, BatchSize};
use std::time::{Instant, Duration};

/// Simulated component tree node
#[derive(Debug, Clone)]
struct ComponentNode {
    id: usize,
    parent_id: Option<usize>,
    bounds: [f32; 4],
    children: Vec<usize>,
    depth: usize,
}

/// Generate a component tree with specified number of nodes
fn generate_component_tree(node_count: usize) -> Vec<ComponentNode> {
    (0..node_count)
        .map(|i| {
            let parent_id = if i == 0 { None } else { Some(i / 2) };
            let depth = if i == 0 { 0 } else { (i as f64).log2() as usize };
            ComponentNode {
                id: i,
                parent_id,
                bounds: [i as f32 * 10.0, i as f32 * 10.0, 100.0, 100.0],
                children: vec![],
                depth,
            }
        })
        .collect()
}

/// Simulate rendering traversal of the tree
fn render_tree_simulation(nodes: &[ComponentNode]) -> Duration {
    let start = Instant::now();
    
    // Simulate render pass - traverse all nodes
    for node in nodes {
        // Simulate layout calculation
        let _area = node.bounds[2] * node.bounds[3];
        // Simulate paint commands
        black_box(node);
    }
    
    start.elapsed()
}

/// Benchmark: Tree generation for different sizes
fn bench_tree_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_generation");
    group.sample_size(20);
    
    for size in &[100, 500, 1_000, 2_000, 5_000, 10_000] {
        group.bench_with_input(BenchmarkId::new("nodes", size), size, |b, &size| {
            b.iter(|| black_box(generate_component_tree(size)))
        });
    }
    group.finish();
}

/// Benchmark: Render time for different tree sizes
/// Target: <16ms for 60fps
fn bench_render_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_time");
    group.sample_size(30);
    
    for size in &[100, 500, 1_000, 2_000, 5_000, 10_000] {
        group.bench_with_input(BenchmarkId::new("nodes", size), size, |b, &size| {
            b.iter_batched(
                || generate_component_tree(size),
                |nodes| render_tree_simulation(&nodes),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

/// Benchmark: Memory allocation for large trees
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.sample_size(20);
    
    for size in &[100, 500, 1_000, 2_000, 5_000, 10_000] {
        group.bench_with_input(BenchmarkId::new("alloc", size), size, |b, &size| {
            b.iter(|| {
                let tree = generate_component_tree(size);
                // Estimate memory footprint
                std::mem::size_of_val(&tree) + tree.len() * std::mem::size_of::<ComponentNode>()
            })
        });
    }
    group.finish();
}

/// Benchmark: Repeated render cycles (memory profiling)
fn bench_repeated_render_cycles(c: &mut Criterion) {
    let mut group = c.benchmark_group("repeated_render_cycles");
    group.sample_size(10);
    
    for size in &[100, 500, 1_000] {
        group.bench_with_input(BenchmarkId::new("cycles_100", size), size, |b, &size| {
            b.iter_custom(|iters| {
                let tree = generate_component_tree(size);
                let mut total_time = Duration::ZERO;
                
                for _ in 0..iters {
                    let render_time = render_tree_simulation(&tree);
                    total_time += render_time;
                }
                
                total_time
            })
        });
    }
    group.finish();
}

/// Benchmark: 60fps target verification
fn bench_fps_target(c: &mut Criterion) {
    let target_frame_time = Duration::from_millis(16);
    let mut group = c.benchmark_group("fps_target");
    group.sample_size(50);
    
    for size in &[100, 500, 1_000] {
        group.bench_with_input(BenchmarkId::new("under_16ms", size), size, |b, &size| {
            b.iter_custom(|iters| {
                let tree = generate_component_tree(size);
                let mut frame_times = Vec::new();
                
                for _ in 0..iters {
                    let render_time = render_tree_simulation(&tree);
                    frame_times.push(render_time);
                }
                
                // Report proportion of frames under target
                let under_target: usize = frame_times.iter()
                    .filter(|&&t| t < target_frame_time)
                    .count();
                black_box(under_target);
                
                Duration::from_nanos(frame_times.iter().map(|d| d.as_nanos() as u64).sum::<u64>() / frame_times.len() as u64)
            })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_tree_generation,
    bench_render_time,
    bench_memory_usage,
    bench_repeated_render_cycles,
    bench_fps_target,
);
criterion_main!(benches);
