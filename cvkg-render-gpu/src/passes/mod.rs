pub mod accessibility;
pub mod bloom;
pub mod composite;
pub mod compute;
pub mod effects;
pub mod flow;
pub mod geometry;
pub mod glass;
pub mod pyramid;
pub mod ui;
pub mod volumetric;
pub mod backdrop_region;
// BackdropRegionNode is defined but not yet wired into the render graph.
// TODO: Wire into build_render_graph when per-element blur is needed.
