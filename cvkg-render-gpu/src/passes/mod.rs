pub mod accessibility;
pub mod backdrop_region;
pub mod bloom;
pub mod composite;
pub mod effects;
pub mod geometry;
pub mod glass;
pub mod pyramid;
pub mod tonemap;
pub mod ui;
pub mod volumetric;
// BackdropRegionNode is defined but not yet wired into the render graph.
// TODO: Wire into build_render_graph when per-element blur is needed.
