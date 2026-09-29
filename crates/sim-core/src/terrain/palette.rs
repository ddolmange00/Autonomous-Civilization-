//! Presentation-only tile colouring. Never feeds back into simulation.
use super::{Biome, Ground, TileMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapMode { Terrain, Height, Temperature, Moisture, Biome }

impl MapMode {
    pub const ALL: [Self; 5] = [Self::Terrain, Self::Height, Self::Temperature, Self::Moisture, Self::Biome];
    pub fn label(self) -> &'static str {
        match self {
            Self::Terrain => "terrain", Self::Height => "height", Self::Temperature => "temperature",
            Self::Moisture => "moisture", Self::Biome => "biome",
        }
    }
    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|m| *m == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }
}

type Rgb = [f32; 3];

fn lerp(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn biome_rgb(b: Biome) -> Rgb {
    match b {
        Biome::Ocean => [50.0, 100.0, 180.0],
        Biome::Freshwater => [70.0, 135.0, 205.0],
        Biome::Beach => [224.0, 206.0, 142.0],
        Biome::Ice => [226.0, 238.0, 248.0],
        Biome::Tundra => [168.0, 176.0, 158.0],
        Biome::Taiga => [52.0, 104.0, 74.0],
        Biome::ColdSteppe => [150.0, 162.0, 108.0],
        Biome::Grassland => [104.0, 176.0, 62.0],
        Biome::TemperateForest => [62.0, 138.0, 48.0],
        Biome::Swamp => [72.0, 104.0, 62.0],
        Biome::Savanna => [156.0, 166.0, 76.0],
        Biome::Desert => [222.0, 192.0, 122.0],
        Biome::Rainforest => [34.0, 114.0, 42.0],
        Biome::Alpine => [124.0, 118.0, 110.0],
        Biome::Snowcap => [246.0, 248.0, 252.0],
    }
}

fn is_forest(b: Biome) -> bool {
    matches!(b, Biome::TemperateForest | Biome::Rainforest | Biome::Taiga | Biome::Swamp)
}

/// RGBA8 colour of one tile in the given map mode.
pub fn tile_rgba(m: &TileMap, x: usize, y: usize, mode: MapMode) -> [u8; 4] {
    let i = m.index(x, y);
    let g = m.ground[i];
    let hm = m.height_m[i] as f32;
    let jitter = m.tile_roll(i, 0x7A11) * 2.0 - 1.0;
    let rgb: Rgb = match mode {
        MapMode::Height => {
            if hm <= 0.0 { lerp([20.0, 40.0, 110.0], [90.0, 150.0, 220.0], 1.0 + hm / 5200.0) }
            else if hm < 1400.0 { lerp([70.0, 150.0, 70.0], [200.0, 190.0, 110.0], hm / 1400.0) }
            else { lerp([200.0, 190.0, 110.0], [250.0, 250.0, 250.0], (hm - 1400.0) / 3000.0) }
        }
        MapMode::Temperature => {
            let t = (m.temperature_c[i] as f32 + 30.0) / 65.0;
            if t < 0.5 { lerp([40.0, 80.0, 220.0], [240.0, 240.0, 200.0], t * 2.0) }
            else { lerp([240.0, 240.0, 200.0], [220.0, 40.0, 30.0], t * 2.0 - 1.0) }
        }
        MapMode::Moisture => {
            if g.is_water() { [30.0, 60.0, 120.0] }
            else { lerp([190.0, 150.0, 90.0], [40.0, 110.0, 200.0], m.moisture[i] as f32 / 255.0) }
        }
        MapMode::Biome => {
            if g == Ground::River { [70.0, 135.0, 205.0] } else { biome_rgb(m.biome[i]) }
        }
        MapMode::Terrain => terrain_rgb(m, x, y, jitter),
    };
    let j = if mode == MapMode::Terrain { 1.0 } else { 1.0 + jitter * 0.02 };
    [
        (rgb[0] * j).clamp(0.0, 255.0) as u8,
        (rgb[1] * j).clamp(0.0, 255.0) as u8,
        (rgb[2] * j).clamp(0.0, 255.0) as u8,
        255,
    ]
}

fn terrain_rgb(m: &TileMap, x: usize, y: usize, jitter: f32) -> Rgb {
    let i = m.index(x, y);
    let g = m.ground[i];
    let hm = m.height_m[i] as f32;
    let b = m.biome[i];
    if g.is_water() {
        if b == Biome::Ice { return lerp(biome_rgb(Biome::Ice), [200.0, 220.0, 240.0], jitter.abs()); }
        let base = if g == Ground::Lake {
            biome_rgb(Biome::Freshwater)
        } else {
            // Depth ramp: surf-bright shallows to navy abyss.
            let d = (-hm / 4000.0).clamp(0.0, 1.0).powf(0.45);
            lerp([96.0, 170.0, 222.0], [26.0, 58.0, 132.0], d)
        };
        let surf = if m.coast_dist[i] == 0 && g == Ground::Shallow {
            // Lighter foam band hugging the shore.
            let near_land = neighbours(m, x, y).any(|j| !m.ground[j].is_water());
            if near_land { 0.18 } else { 0.0 }
        } else { 0.0 };
        let c = lerp(base, [210.0, 235.0, 245.0], surf);
        return [c[0] * (1.0 + jitter * 0.03), c[1] * (1.0 + jitter * 0.03), c[2] * (1.0 + jitter * 0.02)];
    }
    let mut c = if g == Ground::River {
        let s = m.river[i] as f32 / 255.0;
        lerp([90.0, 160.0, 220.0], [60.0, 120.0, 200.0], s)
    } else {
        biome_rgb(b)
    };
    if g == Ground::Hills { c = lerp(c, [120.0, 110.0, 80.0], 0.18); }
    if g == Ground::Mountain && b != Biome::Snowcap { c = lerp(c, [138.0, 132.0, 124.0], 0.35); }
    // Canopy speckle: forests read as clumps of trees, not flat paint.
    if is_forest(b) && g != Ground::River {
        let canopy = m.tile_roll(i, 0xF0_4E57);
        if canopy < 0.38 { c = lerp(c, [18.0, 60.0, 26.0], 0.35); }
        else if canopy > 0.9 { c = lerp(c, [150.0, 190.0, 80.0], 0.25); }
    }
    // Hillshade from the north-west neighbour.
    let nw = if x > 0 && y > 0 { m.height_m[m.index(x - 1, y - 1)].max(0) as f32 } else { hm };
    let slope = ((hm.max(0.0) - nw) / 140.0).clamp(-1.0, 1.0);
    let shade = 1.0 + slope * 0.22 + jitter * 0.045;
    [c[0] * shade, c[1] * shade, c[2] * shade]
}

fn neighbours(m: &TileMap, x: usize, y: usize) -> impl Iterator<Item = usize> + '_ {
    let (w, h) = (m.width as i32, m.height as i32);
    [(0, -1), (-1, 0), (1, 0), (0, 1)].into_iter().filter_map(move |(dx, dy)| {
        let (nx, ny) = (x as i32 + dx, y as i32 + dy);
        (nx >= 0 && ny >= 0 && nx < w && ny < h).then(|| m.index(nx as usize, ny as usize))
    })
}

/// Fill an RGBA8 buffer for one chunk (row 0 = north edge).
pub fn chunk_rgba(m: &TileMap, cx: usize, cy: usize, mode: MapMode, out: &mut Vec<u8>) {
    use super::CHUNK;
    out.clear();
    out.reserve(CHUNK * CHUNK * 4);
    for ty in 0..CHUNK {
        for tx in 0..CHUNK {
            let (x, y) = (cx * CHUNK + tx, cy * CHUNK + ty);
            if x < m.width && y < m.height {
                out.extend_from_slice(&tile_rgba(m, x, y, mode));
            } else {
                out.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
}
