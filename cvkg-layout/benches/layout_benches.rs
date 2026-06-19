use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use cvkg_core::{Alignment, Distribution, LayoutCache, LayoutView, Rect, Size, SizeProposal};
use cvkg_layout::{HStack, VStack};

// ---------------------------------------------------------------------------
// MockView: a lightweight LayoutView for benchmarking
// ---------------------------------------------------------------------------
struct MockView {
    size: Size,
    flex: f32,
}

impl LayoutView for MockView {
    fn size_that_fits(
        &self,
        _p: SizeProposal,
        _s: &[&dyn LayoutView],
        _c: &mut LayoutCache,
    ) -> Size {
        self.size
    }
    fn place_subviews(&self, _b: Rect, _s: &mut [&mut dyn LayoutView], _c: &mut LayoutCache) {}
    fn flex_weight(&self) -> f32 {
        self.flex
    }
}

fn make_views(n: usize) -> Vec<MockView> {
    (0..n)
        .map(|_| MockView {
            size: Size {
                width: 50.0,
                height: 30.0,
            },
            flex: 0.0,
        })
        .collect()
}

fn collect_subviews(views: &[MockView]) -> Vec<&dyn LayoutView> {
    views.iter().map(|v| v as &dyn LayoutView).collect()
}

// ---------------------------------------------------------------------------
// P2-48: HStack benchmarks -- single-threaded vs parallel
// ---------------------------------------------------------------------------
fn bench_hstack_single_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("hstack_single_threaded");
    for &n in &[100, 1000, 10000] {
        let views = make_views(n);
        let subviews = collect_subviews(&views);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &_n| {
            b.iter(|| {
                let mut cache = LayoutCache::new();
                let bounds = Rect {
                    x: 0.0,
                    y: 0.0,
                    width: n as f32 * 60.0,
                    height: 200.0,
                };
                HStack::compute_layout(
                    0.0,
                    Alignment::Leading,
                    Distribution::Leading,
                    bounds,
                    &subviews,
                    &mut cache,
                )
            });
        });
    }
    group.finish();
}

#[cfg(feature = "parallel")]
fn bench_hstack_parallel(c: &mut Criterion) {
    use rayon::prelude::*;

    let mut group = c.benchmark_group("hstack_parallel");
    for &n in &[100, 1000, 10000] {
        let views = make_views(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &_n| {
            b.iter(|| {
                let bounds = Rect {
                    x: 0.0,
                    y: 0.0,
                    width: n as f32 * 60.0,
                    height: 200.0,
                };
                // Parallel size computation on concrete type (Send + Sync)
                let sizes: Vec<Size> = views.par_iter().map(|v| v.size).collect();
                let _ = sizes;
                let subviews = collect_subviews(&views);
                let mut cache = LayoutCache::new();
                HStack::compute_layout(
                    0.0,
                    Alignment::Leading,
                    Distribution::Leading,
                    bounds,
                    &subviews,
                    &mut cache,
                )
            });
        });
    }
    group.finish();
}

#[cfg(not(feature = "parallel"))]
fn bench_hstack_parallel(_c: &mut Criterion) {}

// ---------------------------------------------------------------------------
// P2-48: Deep tree benchmarks -- single-threaded vs parallel
// ---------------------------------------------------------------------------
fn build_deep_tree(depth: usize) -> Box<dyn LayoutView> {
    if depth <= 1 {
        return Box::new(HStack::new(0.0, Alignment::Leading, Distribution::Leading));
    }
    let inner: Box<dyn LayoutView> = build_deep_tree(depth - 1);
    let _ = inner;
    Box::new(HStack::new(0.0, Alignment::Leading, Distribution::Leading))
}

fn bench_deep_tree_single_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("deep_tree_single_threaded");
    for &depth in &[10, 50, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &d| {
            b.iter(|| {
                let root = build_deep_tree(d);
                let mut cache = LayoutCache::new();
                root.size_that_fits(SizeProposal::unspecified(), &[], &mut cache)
            });
        });
    }
    group.finish();
}

#[cfg(feature = "parallel")]
fn bench_deep_tree_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("deep_tree_parallel");
    for &depth in &[10, 50, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &d| {
            b.iter(|| {
                let root = build_deep_tree(d);
                let mut cache = LayoutCache::new();
                root.size_that_fits(SizeProposal::unspecified(), &[], &mut cache)
            });
        });
    }
    group.finish();
}

#[cfg(not(feature = "parallel"))]
fn bench_deep_tree_parallel(_c: &mut Criterion) {}

// ---------------------------------------------------------------------------
// P2-48: Wide tree benchmarks -- single vs parallel
// ---------------------------------------------------------------------------
fn bench_wide_tree(c: &mut Criterion) {
    let mut group = c.benchmark_group("wide_tree");
    let n = 1000;
    let views = make_views(n);
    let subviews = collect_subviews(&views);

    group.bench_function("single_threaded", |b| {
        b.iter(|| {
            let mut cache = LayoutCache::new();
            let bounds = Rect {
                x: 0.0,
                y: 0.0,
                width: n as f32 * 60.0,
                height: 200.0,
            };
            HStack::compute_layout(
                0.0,
                Alignment::Leading,
                Distribution::Leading,
                bounds,
                &subviews,
                &mut cache,
            )
        });
    });

    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;
        group.bench_function("parallel", |b| {
            b.iter(|| {
                let bounds = Rect {
                    x: 0.0,
                    y: 0.0,
                    width: n as f32 * 60.0,
                    height: 200.0,
                };
                // Parallel size queries on concrete type
                let _sizes: Vec<Size> = views.par_iter().map(|v| v.size).collect();
                let subviews = collect_subviews(&views);
                let mut cache = LayoutCache::new();
                HStack::compute_layout(
                    0.0,
                    Alignment::Leading,
                    Distribution::Leading,
                    bounds,
                    &subviews,
                    &mut cache,
                )
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion group registration
// ---------------------------------------------------------------------------
criterion_group!(
    benches,
    bench_hstack_single_threaded,
    bench_hstack_parallel,
    bench_deep_tree_single_threaded,
    bench_deep_tree_parallel,
    bench_wide_tree,
);
criterion_main!(benches);
