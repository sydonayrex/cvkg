use cvkg_core::Rect;

/// A node entry stored in the spatial index after layout completes.
#[derive(Debug, Clone)]
pub struct LayoutSpatialEntry {
    /// Stable identity of the view — matches `LayoutView::view_hash()`.
    pub hash: u64,
    /// Post-layout bounding rect in the root coordinate space.
    pub rect: Rect,
}

/// Axis-aligned 2-D quadtree that indexes laid-out view bounding boxes.
pub struct LayoutSpatialIndex {
    root: Option<Box<QuadNode>>,
    /// Root bounds used when the tree was built — needed for hit queries.
    bounds: Rect,
}

const MAX_ITEMS_PER_NODE: usize = 16;
const MAX_TREE_DEPTH: u32 = 8;

struct QuadNode {
    bounds: Rect,
    entries: Vec<LayoutSpatialEntry>,
    children: Option<Box<[Box<QuadNode>; 4]>>,
}

impl QuadNode {
    fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            entries: Vec::new(),
            children: None,
        }
    }

    fn insert(&mut self, entry: LayoutSpatialEntry, depth: u32) {
        if !self.bounds.intersects(&entry.rect) {
            return;
        }
        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                if child.bounds.intersects(&entry.rect) {
                    child.insert(entry.clone(), depth + 1);
                }
            }
            return;
        }
        self.entries.push(entry);
        if self.entries.len() > MAX_ITEMS_PER_NODE && depth < MAX_TREE_DEPTH {
            self.split(depth);
        }
    }

    fn split(&mut self, depth: u32) {
        let hw = self.bounds.width * 0.5;
        let hh = self.bounds.height * 0.5;
        let mx = self.bounds.x + hw;
        let my = self.bounds.y + hh;
        let make = |x, y, w, h| Box::new(QuadNode::new(Rect { x, y, width: w, height: h }));
        let mut children = Box::new([
            make(self.bounds.x, self.bounds.y, hw, hh), // NW
            make(mx, self.bounds.y, hw, hh),            // NE
            make(self.bounds.x, my, hw, hh),            // SW
            make(mx, my, hw, hh),                       // SE
        ]);
        let entries = std::mem::take(&mut self.entries);
        for e in entries {
            for child in children.iter_mut() {
                if child.bounds.intersects(&e.rect) {
                    child.insert(e.clone(), depth + 1);
                }
            }
        }
        self.children = Some(children);
    }

    fn hit_test(&self, point: (f32, f32), out: &mut Vec<LayoutSpatialEntry>) {
        if !self.bounds.contains(point.0, point.1) {
            return;
        }
        for e in &self.entries {
            if e.rect.contains(point.0, point.1) {
                out.push(e.clone());
            }
        }
        if let Some(children) = &self.children {
            for child in children.iter() {
                child.hit_test(point, out);
            }
        }
    }

    fn query_region(&self, region: &Rect, out: &mut Vec<LayoutSpatialEntry>) {
        if !self.bounds.intersects(region) {
            return;
        }
        for e in &self.entries {
            if e.rect.intersects(region) {
                out.push(e.clone());
            }
        }
        if let Some(children) = &self.children {
            for child in children.iter() {
                child.query_region(region, out);
            }
        }
    }
}

impl LayoutSpatialIndex {
    /// Construct an empty index.
    pub fn new() -> Self {
        Self { root: None, bounds: Rect::zero() }
    }

    /// Rebuild the index from a flat list of (hash, rect) pairs produced after a layout pass.
    pub fn rebuild(&mut self, root_bounds: Rect, entries: impl IntoIterator<Item = LayoutSpatialEntry>) {
        self.bounds = root_bounds;
        let mut root = QuadNode::new(root_bounds);
        for e in entries {
            if e.rect.width > 0.0 && e.rect.height > 0.0 {
                root.insert(e, 0);
            }
        }
        self.root = Some(Box::new(root));
    }

    /// Return all entries whose bounding rect contains `(x, y)`, ordered front-to-back.
    pub fn hit_test(&self, x: f32, y: f32) -> Vec<LayoutSpatialEntry> {
        let mut out = Vec::new();
        if let Some(root) = &self.root {
            root.hit_test((x, y), &mut out);
        }
        out
    }

    /// Return all entries whose bounding rect overlaps `region`.
    pub fn query_region(&self, region: &Rect) -> Vec<LayoutSpatialEntry> {
        let mut out = Vec::new();
        if let Some(root) = &self.root {
            root.query_region(region, &mut out);
        }
        out
    }
}

impl Default for LayoutSpatialIndex {
    fn default() -> Self {
        Self::new()
    }
}
