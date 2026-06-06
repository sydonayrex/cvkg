//! Heim packing utilities.
//!
//! SundrPacker implements a skyline bin-packing algorithm for the
//! Mega-Heim texture array. It tracks horizontal segments and finds
//! the best position for each new rectangle.

#[derive(Clone, Copy)]
pub struct SkylineSegment {
    pub x: u32,
    pub y: u32,
    pub w: u32,
}

pub struct SundrPacker {
    width: u32,
    height: u32,
    skyline: Vec<SkylineSegment>,
}

impl SundrPacker {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            skyline: vec![SkylineSegment {
                x: 0,
                y: 0,
                w: width,
            }],
        }
    }

    pub fn pack(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        if w > self.width || h > self.height {
            return None;
        }

        let mut best_idx = None;
        let mut best_y = u32::MAX;
        let mut best_w = u32::MAX;

        for i in 0..self.skyline.len() {
            let seg = &self.skyline[i];
            if seg.x + w > self.width {
                continue;
            }

            let mut y = seg.y;
            let mut remaining = w;
            let mut j = i;
            let mut fits = true;

            while remaining > 0 {
                if j >= self.skyline.len() {
                    fits = false;
                    break;
                }
                let s = &self.skyline[j];
                y = y.max(s.y);
                if y + h > self.height {
                    fits = false;
                    break;
                }
                if s.w >= remaining {
                    break;
                }
                remaining -= s.w;
                j += 1;
            }

            if fits && (y < best_y || (y == best_y && seg.w < best_w)) {
                best_y = y;
                best_idx = Some(i);
                best_w = seg.w;
            }
        }

        if let Some(idx) = best_idx {
            let x = self.skyline[idx].x;
            let y = best_y;

            let new_seg = SkylineSegment { x, y: y + h, w };
            let mut remaining = w;
            let insert_idx = idx;

            while remaining > 0 {
                if self.skyline[insert_idx].w <= remaining {
                    remaining -= self.skyline[insert_idx].w;
                    self.skyline.remove(insert_idx);
                } else {
                    self.skyline[insert_idx].x += remaining;
                    self.skyline[insert_idx].w -= remaining;
                    remaining = 0;
                }
            }
            self.skyline.insert(insert_idx, new_seg);

            let mut i = 0;
            while i < self.skyline.len() - 1 {
                if self.skyline[i].y == self.skyline[i + 1].y {
                    let w = self.skyline[i + 1].w;
                    self.skyline[i].w += w;
                    self.skyline.remove(i + 1);
                } else {
                    i += 1;
                }
            }

            return Some((x, y));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shelf_packer_basic() {
        let mut packer = SundrPacker::new(100, 100);
        assert_eq!(packer.pack(10, 10), Some((0, 0)));
        assert_eq!(packer.pack(20, 15), Some((10, 0)));
    }

    #[test]
    fn test_shelf_packer_wrap() {
        let mut packer = SundrPacker::new(100, 100);
        packer.pack(60, 10);
        assert_eq!(packer.pack(50, 20), Some((0, 10)));
    }

    #[test]
    fn test_shelf_packer_oversized() {
        let mut packer = SundrPacker::new(10, 10);
        assert_eq!(packer.pack(11, 5), None);
        assert_eq!(packer.pack(5, 11), None);
    }
}
