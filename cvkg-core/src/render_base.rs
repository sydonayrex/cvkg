/// Captures the depth of every stack-pushing operation on the `Renderer`.
///
/// Created via `Renderer::snapshot_render_state()` and consumed by
/// `Renderer::restore_render_state()`. The renderer uses this to recover
/// from mid-render panics -- any items pushed beyond the snapshot point
/// are popped so sibling views drawn afterward don't inherit leaked
/// clip / opacity / transform / shadow / vnode / mjolnir-slice state.
///
/// Frame-scoped: the renderer resets all stacks in `begin_frame()` so a
/// snapshot taken in one frame is meaningless in another.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RenderStateSnapshot {
    pub clip_depth: u32,
    pub opacity_depth: u32,
    pub slice_depth: u32,
    pub shadow_depth: u32,
    pub transform_depth: u32,
    pub vnode_depth: u32,
}

/// TelemetryData tracks real-time performance metrics for the GPU renderer.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TelemetryData {
    pub frame_time_ms: f32,
    /// Total frame budget in milliseconds for the active policy.
    pub frame_budget_ms: f32,
    /// Remaining frame budget after the frame completed; negative means over budget.
    pub frame_budget_remaining_ms: f32,
    /// Remaining layout budget after layout completed; negative means the layout slice was exceeded.
    pub layout_budget_remaining_ms: f32,
    /// Whether the frame exceeded the total budget.
    pub frame_over_budget: bool,
    /// Whether the layout phase exceeded its budget slice.
    pub layout_over_budget: bool,
    /// 99th percentile frame time over the last window, used to detect tail latency.
    pub p99_frame_time_ms: f32,
    /// Statistical jitter (variance in frame timing).
    pub frame_jitter_ms: f32,
    /// Indicates if a hardware stall (DRAM refresh, thermal spike) was detected.
    pub hardware_stall_detected: bool,

    // Pass timing
    pub input_time_ms: f32,
    pub state_flush_time_ms: f32,
    pub layout_time_ms: f32,
    pub draw_time_ms: f32,
    pub gpu_submit_time_ms: f32,

    pub draw_calls: u32,
    pub vertices: u32,

    /// Global Berserker Pipeline Intensity (0.0 - 1.0+)
    pub berserker_rage: f32,

    // Memory breakdown
    pub vram_usage_mb: f32,
    pub vram_textures_mb: f32,
    pub vram_buffers_mb: f32,
    pub vram_pipelines_mb: f32,
    /// Indicates if the Mega-Atlas or VRAM pools are at capacity.
    pub vram_exhausted: bool,
}

/// Configuration for render-loop frame timing and degradation strategies.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FrameBudget {
    /// Target frame time in milliseconds (default: 16.0 for 60FPS)
    pub target_ms: f32,
    /// If true, the renderer is allowed to dynamically skip non-critical effects
    /// (like heavy blurs or complex shadows) when the budget is exceeded.
    pub allow_degradation: bool,
}

impl Default for FrameBudget {
    fn default() -> Self {
        Self {
            target_ms: 16.0,
            allow_degradation: true,
        }
    }
}
