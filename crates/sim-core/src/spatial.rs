//! Uniform-grid spatial index over point sets.
//! Queries return candidate indices in ascending order, so a caller that applies
//! its exact distance test to the candidates visits items in the same order as
//! a full linear scan — results stay bit-identical, only cheaper.
use crate::world::Position;
use std::collections::HashMap;

pub struct PointGrid {
    cell: f32,
    buckets: HashMap<(i32, i32), Vec<u32>>,
}

impl PointGrid {
    pub fn new(cell: f32, points: impl IntoIterator<Item = Position>) -> Self {
        let cell = cell.max(1.0);
        let mut buckets: HashMap<(i32, i32), Vec<u32>> = HashMap::new();
        for (i, p) in points.into_iter().enumerate() {
            buckets.entry(Self::key(cell, p)).or_default().push(i as u32);
        }
        Self { cell, buckets }
    }

    fn key(cell: f32, p: Position) -> (i32, i32) {
        ((p.x / cell).floor() as i32, (p.y / cell).floor() as i32)
    }

    /// Indices of every point that may lie within `radius` of `p`, ascending.
    /// A superset of the true answer: callers keep their exact distance test.
    pub fn candidates(&self, p: Position, radius: f32, out: &mut Vec<u32>) {
        out.clear();
        let (x0, y0) = Self::key(self.cell, Position { x: p.x - radius, y: p.y - radius });
        let (x1, y1) = Self::key(self.cell, Position { x: p.x + radius, y: p.y + radius });
        for gy in y0..=y1 {
            for gx in x0..=x1 {
                if let Some(b) = self.buckets.get(&(gx, gy)) {
                    out.extend_from_slice(b);
                }
            }
        }
        out.sort_unstable();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidates_cover_every_point_in_radius_in_scan_order() {
        let pts: Vec<Position> = (0..500)
            .map(|i| Position { x: ((i * 37) % 400) as f32 - 200.0, y: ((i * 53) % 300) as f32 - 150.0 })
            .collect();
        let grid = PointGrid::new(28.0, pts.iter().copied());
        let mut out = Vec::new();
        for (qi, q) in pts.iter().enumerate().step_by(7) {
            for r in [5.0, 28.0, 85.0] {
                grid.candidates(*q, r, &mut out);
                let exact: Vec<u32> = (0..pts.len() as u32).filter(|&i| q.distance(pts[i as usize]) <= r).collect();
                let filtered: Vec<u32> = out.iter().copied().filter(|&i| q.distance(pts[i as usize]) <= r).collect();
                assert_eq!(filtered, exact, "query {qi} r {r}");
            }
        }
    }
}
