use cvkg_core::Rect;

pub struct Quadtree {
    bounds: Rect,
    rects: Vec<Rect>,
    children: Option<Box<[Quadtree; 4]>>,
    max_rects: usize,
    max_depth: usize,
    depth: usize,
}

impl Quadtree {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            rects: Vec::new(),
            children: None,
            max_rects: 10,
            max_depth: 5,
            depth: 0,
        }
    }

    fn new_with_depth(bounds: Rect, depth: usize) -> Self {
        Self {
            bounds,
            rects: Vec::new(),
            children: None,
            max_rects: 10,
            max_depth: 5,
            depth,
        }
    }

    pub fn insert(&mut self, rect: Rect) {
        if !self.intersects(self.bounds, rect) {
            return;
        }

        if let Some(ref mut children) = self.children {
            for child in children.iter_mut() {
                child.insert(rect);
            }
            return;
        }

        self.rects.push(rect);

        if self.rects.len() > self.max_rects && self.depth < self.max_depth {
            self.subdivide();
        }
    }

    fn subdivide(&mut self) {
        let hw = self.bounds.width / 2.0;
        let hh = self.bounds.height / 2.0;
        let x = self.bounds.x;
        let y = self.bounds.y;
        let d = self.depth + 1;

        let mut children = Box::new([
            Quadtree::new_with_depth(Rect { x, y, width: hw, height: hh }, d),
            Quadtree::new_with_depth(Rect { x: x + hw, y, width: hw, height: hh }, d),
            Quadtree::new_with_depth(Rect { x, y: y + hh, width: hw, height: hh }, d),
            Quadtree::new_with_depth(Rect { x: x + hw, y: y + hh, width: hw, height: hh }, d),
        ]);

        for rect in self.rects.drain(..) {
            for child in children.iter_mut() {
                child.insert(rect);
            }
        }

        self.children = Some(children);
    }

    fn intersects(&self, a: Rect, b: Rect) -> bool {
        a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
    }

    pub fn retrieve(&self, rect: Rect, out: &mut Vec<Rect>) {
        if !self.intersects(self.bounds, rect) {
            return;
        }

        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.retrieve(rect, out);
            }
        } else {
            for r in &self.rects {
                out.push(*r);
            }
        }
    }
}
