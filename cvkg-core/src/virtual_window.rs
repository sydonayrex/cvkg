// =============================================================================
// P1-41: LIST / TREE VIRTUALIZATION
// =============================================================================

/// Outcome of a `VirtualListWindow::compute` call — describes which rows
/// should be rendered and what scroll offset to apply to position them.
///
/// P1-41: IDE and visualization workloads with tens-of-thousands of rows must
/// only render the rows visible in the current viewport.  `VirtualListWindow`
/// computes the correct row range without building the full row list.
#[derive(Debug, Clone, PartialEq)]
pub struct VirtualWindow {
    /// Index of the first row that should be rendered (inclusive).
    pub first_visible: usize,
    /// Index one past the last row that should be rendered (exclusive).
    pub last_visible: usize,
    /// Total estimated height of all rows above `first_visible`, in logical
    /// pixels.  Use this as the scroll-offset padding above the rendered rows.
    pub offset_before: f32,
    /// Total estimated height of all rows below `last_visible`, in logical
    /// pixels.  Use this as the placeholder height below the rendered rows.
    pub offset_after: f32,
}

/// Computes the visible slice of a uniform-height virtual list.
///
/// Contract:
/// - `total_rows`: total number of items in the list (can be enormous).
/// - `row_height`: uniform logical height of every row in pixels.
/// - `viewport_y`: scroll offset of the viewport top edge.
/// - `viewport_height`: height of the visible window in pixels.
/// - `overscan`: number of extra rows to render above/below the viewport for
///   smooth scrolling.  Typically 2–5.
///
/// Returns a `VirtualWindow` describing the rendered slice and offset padding.
/// If `row_height` is ≤ 0 or `total_rows` is 0, returns a zero window.
pub fn compute_virtual_list_window(
    total_rows: usize,
    row_height: f32,
    viewport_y: f32,
    viewport_height: f32,
    overscan: usize,
) -> VirtualWindow {
    if total_rows == 0 || row_height <= 0.0 {
        return VirtualWindow {
            first_visible: 0,
            last_visible: 0,
            offset_before: 0.0,
            offset_after: 0.0,
        };
    }

    // How many rows fit in the viewport (rounded up for partial rows).
    let visible_rows = (viewport_height / row_height).ceil() as usize;

    // First row whose bottom edge is below the viewport top.
    let first = (viewport_y / row_height).floor() as isize - overscan as isize;
    let first = first.max(0) as usize;

    // Last row whose top edge is above the viewport bottom.
    let last = first + visible_rows + 2 * overscan;
    let last = last.min(total_rows);

    VirtualWindow {
        first_visible: first,
        last_visible: last,
        offset_before: first as f32 * row_height,
        offset_after: (total_rows - last) as f32 * row_height,
    }
}

/// Computes the visible slice of a variable-height virtual list using
/// a precomputed prefix-sum of row heights.
///
/// Contract:
/// - `prefix_heights[i]` is the cumulative height of all rows 0..i (not
///   including row i).  `prefix_heights.len()` must equal `total_rows + 1`
///   where `prefix_heights[0] == 0` and `prefix_heights[total_rows]` is the
///   total list height.
/// - `viewport_y` and `viewport_height` are in the same logical pixel units.
/// - `overscan` works the same as in `compute_virtual_list_window`.
///
/// This is O(log N) via binary search on the prefix-sum array.
pub fn compute_virtual_list_window_variable(
    prefix_heights: &[f32],
    viewport_y: f32,
    viewport_height: f32,
    overscan: usize,
) -> VirtualWindow {
    let total_rows = prefix_heights.len().saturating_sub(1);
    if total_rows == 0 {
        return VirtualWindow {
            first_visible: 0,
            last_visible: 0,
            offset_before: 0.0,
            offset_after: 0.0,
        };
    }

    // Binary search for the first row whose cumulative top is >= viewport_y.
    let first_idx = prefix_heights
        .partition_point(|&h| h < viewport_y)
        .saturating_sub(1);
    let first = first_idx.saturating_sub(overscan);

    // Binary search for the last row whose top < viewport_y + viewport_height.
    let viewport_bottom = viewport_y + viewport_height;
    let last_idx = prefix_heights.partition_point(|&h| h < viewport_bottom);
    let last = (last_idx + overscan).min(total_rows);

    VirtualWindow {
        first_visible: first,
        last_visible: last,
        offset_before: prefix_heights[first],
        offset_after: prefix_heights[total_rows] - prefix_heights[last],
    }
}
