//! Isometric (2:1 diamond) presentation of the tile world. Presentation only:
//! the simulation keeps its square tile grid and world units.
//!
//! Screen space here is "iso pixels at full resolution" (`FULL_TILE_W` wide
//! tiles), x to the right and y downward. Every land tile is lifted by its real
//! height (exaggerated), and the gaps to lower neighbours are filled with
//! earth or rock faces, so plains roll gently and ranges stand up as mountains.
use super::{hazards::Hazards, palette::{tile_rgba, MapMode}, Ground, TileMap, CHUNK, TILE_SIZE};
use crate::world::Position;

/// Tile diamond width in pixels at full resolution.
pub const FULL_TILE_W: f32 = 16.0;
/// Tile diamond height (2:1 isometric).
pub const FULL_TILE_H: f32 = 8.0;
/// Rise of the highest possible summit, in full-resolution pixels (12 tile heights).
pub const MAX_RISE: f32 = 96.0;
/// Every shore stands this far above the water so coastlines read as banks.
const BANK: f32 = 2.0;

/// On-screen lift of a tile in full-resolution pixels: water 0, land rising
/// with real height (slightly concave so plains stay low and ranges tower).
pub fn rise(m: &TileMap, i: usize) -> f32 {
    if m.ground[i].is_water() { return 0.0; }
    BANK + (m.height_m[i].max(0) as f32 / 4400.0).clamp(0.0, 1.0).powf(1.15) * (MAX_RISE - BANK)
}

/// Continuous tile coordinates (x east, y south) of a world position.
pub fn world_to_tile(m: &TileMap, p: Position) -> (f32, f32) {
    (p.x / TILE_SIZE + m.width as f32 * 0.5, m.height as f32 * 0.5 - p.y / TILE_SIZE)
}

pub fn tile_to_world(m: &TileMap, tx: f32, ty: f32) -> Position {
    Position { x: (tx - m.width as f32 * 0.5) * TILE_SIZE, y: (m.height as f32 * 0.5 - ty) * TILE_SIZE }
}

/// Full-resolution iso screen position of continuous tile coordinates lifted by `lift` pixels.
pub fn tile_to_screen(tx: f32, ty: f32, lift: f32) -> (f32, f32) {
    ((tx - ty) * FULL_TILE_W * 0.5, (tx + ty) * FULL_TILE_H * 0.5 - lift)
}

/// Iso screen position of a world position, standing on the terrain under it.
pub fn world_to_screen(m: &TileMap, p: Position) -> (f32, f32) {
    let (tx, ty) = world_to_tile(m, p);
    let lift = m.tile_at(p).map(|(x, y)| rise(m, m.index(x, y))).unwrap_or(0.0);
    tile_to_screen(tx, ty, lift)
}

/// Inverse projection: the terrain point drawn at a screen position.
/// Walks the screen ray from the tallest possible lift (nearest the viewer)
/// back to sea level and returns the first tile whose surface reaches the ray.
pub fn screen_to_world(m: &TileMap, sx: f32, sy: f32) -> Position {
    let unproject = |lift: f32| {
        let a = sx / (FULL_TILE_W * 0.5);
        let b = (sy + lift) / (FULL_TILE_H * 0.5);
        ((a + b) * 0.5, (b - a) * 0.5)
    };
    let steps = (MAX_RISE * 2.0) as i32;
    for k in (0..=steps).rev() {
        let lift = k as f32 * 0.5;
        let (tx, ty) = unproject(lift);
        if tx < 0.0 || ty < 0.0 || tx >= m.width as f32 || ty >= m.height as f32 { continue; }
        if rise(m, m.index(tx as usize, ty as usize)) >= lift { return tile_to_world(m, tx, ty); }
    }
    let (tx, ty) = unproject(0.0);
    tile_to_world(m, tx, ty)
}

/// Full-resolution screen rectangle a baked chunk covers: (top-left, size).
/// Matches `bake_chunk` exactly, so renderers can cull before baking.
pub fn chunk_bounds(cx: usize, cy: usize) -> ((f32, f32), (f32, f32)) {
    let c = CHUNK as f32;
    let (sx, sy) = tile_to_screen((cx * CHUNK) as f32, (cy * CHUNK) as f32, 0.0);
    ((sx - c * FULL_TILE_W * 0.5, sy - MAX_RISE), (c * FULL_TILE_W, c * FULL_TILE_H + MAX_RISE + FULL_TILE_H))
}

/// A baked chunk image and where its top-left pixel sits in full-resolution screen space.
pub struct BakedChunk {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    /// Full-resolution screen position of pixel (0, 0).
    pub origin: (f32, f32),
    /// Full-resolution pixels per baked pixel.
    pub scale: f32,
}

fn shade(c: [u8; 4], k: f32) -> [u8; 4] {
    [(c[0] as f32 * k) as u8, (c[1] as f32 * k) as u8, (c[2] as f32 * k) as u8, 255]
}

fn cliff_colour(m: &TileMap, i: usize, top: [u8; 4]) -> [u8; 4] {
    // Exposed earth or rock on cliff faces, tinted by what grows on top.
    let base: [f32; 3] = match m.ground[i] {
        Ground::Mountain | Ground::Peak => [112.0, 104.0, 96.0],
        Ground::Beach => [196.0, 170.0, 110.0],
        _ => [120.0, 88.0, 58.0],
    };
    let t = 0.25;
    [
        (base[0] * (1.0 - t) + top[0] as f32 * t) as u8,
        (base[1] * (1.0 - t) + top[1] as f32 * t) as u8,
        (base[2] * (1.0 - t) + top[2] as f32 * t) as u8,
        255,
    ]
}

/// Rasterise one chunk in isometric projection at `tile_w` pixels per tile
/// (a power of two from 2 to 16). Back-to-front painter's order; cliff faces
/// are drawn toward lower neighbours, including neighbours in other chunks,
/// so adjacent chunks meet without seams.
pub fn bake_chunk(m: &TileMap, hz: Option<&Hazards>, cx: usize, cy: usize, mode: MapMode, tile_w: u32) -> BakedChunk {
    let tw = tile_w.max(2) as i32;
    let th = (tw / 2).max(1);
    let scale = FULL_TILE_W / tw as f32;
    let c = CHUNK as i32;
    let width = (c * tw) as u32;
    let top_pad = (MAX_RISE / scale).ceil() as i32;
    let lift_px = |i: usize| (rise(m, i) / scale).round() as i32;
    let height = (c * th + top_pad + th) as u32;
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    let (x0, y0) = ((cx * CHUNK) as i32, (cy * CHUNK) as i32);
    // Pixel of the top vertex of local tile (lx, ly) at level 0.
    let apex = |lx: i32, ly: i32| ((lx - ly) * tw / 2 + (c - 1) * tw / 2 + tw / 2, (lx + ly) * th / 2 + top_pad);
    let mut put = |px: i32, py: i32, col: [u8; 4]| {
        if px < 0 || py < 0 || px >= width as i32 || py >= height as i32 { return; }
        let o = ((py as u32 * width + px as u32) * 4) as usize;
        rgba[o..o + 4].copy_from_slice(&col);
    };
    let lift_at = |x: i32, y: i32| -> i32 {
        if x < 0 || y < 0 || x >= m.width as i32 || y >= m.height as i32 { return 0; }
        lift_px(m.index(x as usize, y as usize))
    };
    // Diagonals back to front.
    for d in 0..(2 * c - 1) {
        for lx in 0..c {
            let ly = d - lx;
            if ly < 0 || ly >= c { continue; }
            let (x, y) = (x0 + lx, y0 + ly);
            if x >= m.width as i32 || y >= m.height as i32 { continue; }
            let i = m.index(x as usize, y as usize);
            let lift = lift_px(i);
            let mut top = tile_rgba(m, x as usize, y as usize, mode);
            if mode == MapMode::Terrain && !m.ground[i].is_water() {
                // Relief light from the north-west: slopes facing it brighten, lee sides darken.
                let here = rise(m, i);
                let nw = if x > 0 && y > 0 { rise(m, m.index(x as usize - 1, y as usize - 1)) } else { here };
                let k = (1.0 + (here - nw) * 0.07).clamp(0.62, 1.3);
                top = [(top[0] as f32 * k).min(255.0) as u8, (top[1] as f32 * k).min(255.0) as u8, (top[2] as f32 * k).min(255.0) as u8, 255];
            }
            if let (Some(h), MapMode::Terrain) = (hz, mode) {
                // Burnt ground, glowing embers and standing floodwater.
                let blend = |c: [u8; 4], to: [f32; 3], t: f32| -> [u8; 4] {
                    let t = t.clamp(0.0, 1.0);
                    [(c[0] as f32 + (to[0] - c[0] as f32) * t) as u8, (c[1] as f32 + (to[1] - c[1] as f32) * t) as u8, (c[2] as f32 + (to[2] - c[2] as f32) * t) as u8, 255]
                };
                if h.scorch[i] > 0.0 { top = blend(top, [44.0, 36.0, 30.0], h.scorch[i] * 0.85); }
                if h.fire[i] > 0.0 { top = blend(top, [255.0, 128.0, 40.0], h.fire[i] * 0.55); }
                if h.water[i] > 0.01 { top = blend(top, [66.0, 128.0, 200.0], 0.35 + h.water[i] * 0.5); }
            }
            let (ax, ay) = apex(lx, ly);
            let ay = ay - lift;
            // Faces toward the south-west (left) and south-east (right) neighbours.
            let drop_left = lift - lift_at(x, y + 1);
            let drop_right = lift - lift_at(x + 1, y);
            if drop_left > 0 || drop_right > 0 {
                let side = cliff_colour(m, i, top);
                let half_h = th / 2;
                for col in 0..tw {
                    // Bottom edge of the top diamond at this column.
                    let edge_y = if col < tw / 2 { ay + half_h + (col * th / tw) } else { ay + half_h + ((tw - 1 - col) * th / tw) };
                    let (drop, k) = if col < tw / 2 { (drop_left, 0.62) } else { (drop_right, 0.80) };
                    if drop <= 0 { continue; }
                    let col_shade = shade(side, k);
                    // Strata every few pixels so tall faces read as rock, not flat paint.
                    let strata = (6.0 / scale).max(2.0) as i32;
                    for dy in 0..drop {
                        let band = if dy % strata == strata - 1 { shade(col_shade, 0.86) } else { col_shade };
                        put(ax - tw / 2 + col, edge_y + dy, band);
                    }
                }
            }
            // Top diamond.
            for r in 0..th {
                let half = if r < th / 2 { (r + 1) * tw / th } else { (th - r) * tw / th };
                for px in (ax - half)..(ax + half) {
                    put(px, ay + r, top);
                }
            }
        }
    }
    // Full-resolution screen position of pixel (0,0): invert `apex` for tile (x0, y0).
    let (sx, sy) = tile_to_screen(x0 as f32, y0 as f32, 0.0);
    let (ax, ay) = apex(0, 0);
    BakedChunk { width, height, rgba, origin: (sx - ax as f32 * scale, sy - ay as f32 * scale), scale }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::{MapSize, WorldTemplate};

    #[test]
    fn projection_round_trips_on_raised_ground() {
        let m = TileMap::generate(7, MapSize::Small, WorldTemplate::Pangaea);
        let mut checked = 0;
        for i in (0..m.ground.len()).step_by(97) {
            let (x, y) = (i % m.width, i / m.width);
            let p = m.tile_center(x, y);
            let (sx, sy) = world_to_screen(&m, p);
            let back = screen_to_world(&m, sx, sy);
            // A tile in front may be tall enough to hide this one; only check visible picks.
            if m.tile_at(back) == Some((x, y)) { checked += 1; }
        }
        assert!(checked > 300, "only {checked} tiles picked back to themselves");
    }

    #[test]
    fn baked_chunks_line_up_edge_to_edge() {
        let m = TileMap::generate(7, MapSize::Small, WorldTemplate::Continents);
        let a = bake_chunk(&m, None, 1, 1, MapMode::Terrain, 8);
        let b = bake_chunk(&m, None, 2, 1, MapMode::Terrain, 8);
        // Moving one chunk east moves CHUNK tiles along +x: half a diamond right and down per tile.
        let dx = b.origin.0 - a.origin.0;
        let dy = b.origin.1 - a.origin.1;
        assert!((dx - CHUNK as f32 * FULL_TILE_W * 0.5).abs() < 1e-3);
        assert!((dy - CHUNK as f32 * FULL_TILE_H * 0.5).abs() < 1e-3);
        assert_eq!(a.rgba.len(), (a.width * a.height * 4) as usize);
        for (baked, (cx, cy)) in [(&a, (1, 1)), (&b, (2, 1))] {
            let (o, size) = chunk_bounds(cx, cy);
            assert!((o.0 - baked.origin.0).abs() < 1e-3 && (o.1 - baked.origin.1).abs() < 1e-3);
            assert!((size.0 - baked.width as f32 * baked.scale).abs() < 1e-3);
            assert!((size.1 - baked.height as f32 * baked.scale).abs() < 1e-3);
        }
        assert!(a.rgba.chunks(4).any(|p| p[3] == 255));
    }

    #[test]
    fn water_is_lowest_and_mountains_rise() {
        let m = TileMap::generate(3, MapSize::Small, WorldTemplate::Pangaea);
        for i in 0..m.ground.len() {
            let r = rise(&m, i);
            if m.ground[i].is_water() { assert_eq!(r, 0.0); } else { assert!(r >= BANK); }
            // A peak stands more than six tile-heights above the sea.
            if m.ground[i] == Ground::Peak { assert!(r > 6.0 * FULL_TILE_H, "peak rise {r}"); }
        }
    }
}
