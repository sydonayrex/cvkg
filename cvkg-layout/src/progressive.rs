use cvkg_core::{Alignment, Distribution, LayoutCache, LayoutView, Rect};

#[derive(Debug, Clone)]
pub struct ProgressiveChild {
    pub hash: u64,
    pub laid_out: bool,
    pub rect: Rect,
}

/// Opt-in wrapper that breaks a single layout pass into incremental batches.
///
/// # Contract
/// Saves partially computed layouts inside `LayoutCache` so the main UI thread does not stall for deep trees.
pub struct ProgressiveLayoutContext<'a> {
    pub children: &'a [&'a dyn LayoutView],
    pub entries: Vec<ProgressiveChild>,
    pub spacing: f32,
    pub alignment: Alignment,
    pub distribution: Distribution,
    pub bounds: Rect,
    pub completed: usize,
    pub fallback_applied: bool,
}

impl<'a> ProgressiveLayoutContext<'a> {
    /// Create a new progressive layout context for the given subviews.
    pub fn new(
        bounds: Rect,
        subviews: &'a [&'a dyn LayoutView],
        spacing: f32,
        alignment: Alignment,
        distribution: Distribution,
    ) -> Self {
        let entries = subviews
            .iter()
            .map(|v| ProgressiveChild {
                hash: v.view_hash(),
                laid_out: false,
                rect: Rect::zero(),
            })
            .collect();

        Self {
            children: subviews,
            entries,
            spacing,
            alignment,
            distribution,
            bounds,
            completed: 0,
            fallback_applied: false,
        }
    }

    /// Layout up to `batch_size` additional children.
    pub fn layout_next_batch(&mut self, batch_size: usize) -> bool {
        self.layout_next_batch_inner(batch_size, None);
        self.is_complete()
    }

    /// Variant of `layout_next_batch` that integrates with a persistent cache.
    pub fn layout_next_batch_with_cache(
        &mut self,
        batch_size: usize,
        cache: &mut LayoutCache,
    ) -> (bool, Vec<Rect>) {
        self.layout_next_batch_inner(batch_size, Some(cache));
        let new_rects: Vec<Rect> = self
            .entries
            .iter()
            .filter(|e| e.laid_out && e.rect != Rect::zero())
            .map(|e| e.rect)
            .collect();
        (self.is_complete(), new_rects)
    }

    fn layout_next_batch_inner(&mut self, batch_size: usize, mut cache: Option<&mut LayoutCache>) {
        let mut processed = 0;
        let mut batch_indices = Vec::new();
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.laid_out {
                continue;
            }
            if processed >= batch_size {
                break;
            }
            batch_indices.push(i);
            processed += 1;
        }

        if batch_indices.is_empty() {
            return;
        }

        let batch_subviews: Vec<&dyn LayoutView> =
            batch_indices.iter().map(|&i| self.children[i]).collect();

        let rects = match cache {
            Some(ref mut c) => crate::taffy_engine::HStack::compute_layout_incremental(
                self.spacing,
                self.alignment,
                self.distribution,
                self.bounds,
                0,
                &batch_subviews,
                c,
            ),
            None => {
                let mut tmp = LayoutCache::new();
                crate::taffy_engine::HStack::compute_layout_incremental(
                    self.spacing,
                    self.alignment,
                    self.distribution,
                    self.bounds,
                    0,
                    &batch_subviews,
                    &mut tmp,
                )
            }
        };

        for (local_idx, &global_idx) in batch_indices.iter().enumerate() {
            if local_idx < rects.len() {
                self.entries[global_idx].rect = rects[local_idx];
                self.entries[global_idx].laid_out = true;
                self.completed += 1;
            }
        }

        if let Some(c) = cache.as_mut() {
            for (local_idx, &global_idx) in batch_indices.iter().enumerate() {
                if local_idx < rects.len() {
                    let hash = self.entries[global_idx].hash;
                    if hash != 0 {
                        c.previous_rects.insert(hash, rects[local_idx]);
                    }
                }
            }
        }
    }

    /// Returns `true` when every child has been laid out or fallback has been applied.
    pub fn is_complete(&self) -> bool {
        self.fallback_applied || self.completed >= self.entries.len()
    }

    /// Returns `(completed, total)` progress counts.
    pub fn progress(&self) -> (usize, usize) {
        (self.completed, self.entries.len())
    }

    /// Apply fallback positioning to all children that have not yet been laid out.
    pub fn apply_remaining_fallback(&mut self, cache: &mut LayoutCache) -> Vec<Rect> {
        let mut fallback_rects = Vec::new();
        let remaining: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| !e.laid_out)
            .map(|(i, _)| i)
            .collect();

        if remaining.is_empty() {
            self.fallback_applied = true;
            return fallback_rects;
        }

        let cols = (remaining.len() as f32).sqrt().ceil() as usize;
        let rows = remaining.len().div_ceil(cols);
        let cell_w = self.bounds.width / cols as f32;
        let cell_h = self.bounds.height / rows as f32;

        for (offset, &idx) in remaining.iter().enumerate() {
            let hash = self.entries[idx].hash;
            let rect = if hash != 0 {
                cache.previous_rects.get(&hash).copied().unwrap_or_else(|| {
                    let col = offset % cols;
                    let row = offset / cols;
                    Rect {
                        x: self.bounds.x + col as f32 * cell_w,
                        y: self.bounds.y + row as f32 * cell_h,
                        width: cell_w,
                        height: cell_h,
                    }
                })
            } else {
                let col = offset % cols;
                let row = offset / cols;
                Rect {
                    x: self.bounds.x + col as f32 * cell_w,
                    y: self.bounds.y + row as f32 * cell_h,
                    width: cell_w,
                    height: cell_h,
                }
            };

            self.entries[idx].rect = rect;
            self.entries[idx].laid_out = true;
            self.completed += 1;
            if hash != 0 {
                cache.previous_rects.insert(hash, rect);
            }
            fallback_rects.push(rect);
        }

        self.fallback_applied = true;
        fallback_rects
    }

    /// Consume the context and return the final `Vec<Rect>` for all children in order.
    pub fn take_rects(self) -> Vec<Rect> {
        self.entries.into_iter().map(|e| e.rect).collect()
    }
}
