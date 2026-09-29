//! Tile world: the objective terrain the sandbox stands on.
//! Residents never read these arrays directly; the sandbox samples them into
//! perceivable features and uses them for physical passability.
pub mod hazards;
pub mod iso;
pub mod noise;
pub mod palette;

use crate::world::Position;
use noise::Noise2;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

/// World units per tile edge.
pub const TILE_SIZE: f32 = 4.0;
/// Tiles per chunk edge (render and dirty-tracking unit).
pub const CHUNK: usize = 64;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ground { DeepOcean, Ocean, Shallow, Lake, River, Beach, Lowland, Hills, Mountain, Peak }

impl Ground {
    pub fn is_water(self) -> bool {
        matches!(self, Self::DeepOcean | Self::Ocean | Self::Shallow | Self::Lake)
    }
    /// Rivers are fordable; open water and summit rock are not walkable.
    pub fn walkable(self) -> bool {
        !matches!(self, Self::DeepOcean | Self::Ocean | Self::Shallow | Self::Lake | Self::Peak)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Biome {
    Ocean, Freshwater, Beach, Ice, Tundra, Taiga, ColdSteppe, Grassland,
    TemperateForest, Swamp, Savanna, Desert, Rainforest, Alpine, Snowcap,
}

impl Biome {
    /// Relative primary productivity (grassland = 1).
    pub fn productivity(self) -> f32 {
        match self {
            Self::Rainforest => 1.3, Self::TemperateForest => 1.1, Self::Grassland => 1.0,
            Self::Swamp => 0.9, Self::Savanna => 0.8, Self::Taiga => 0.7, Self::ColdSteppe => 0.55,
            Self::Tundra => 0.35, Self::Beach => 0.3, Self::Alpine => 0.25, Self::Desert => 0.2,
            Self::Ocean | Self::Freshwater | Self::Ice | Self::Snowcap => 0.0,
        }
    }
    /// Expected harvestable vegetation patches per tile.
    pub fn vegetation_density(self) -> f32 {
        match self {
            Self::Rainforest => 1.0 / 90.0, Self::TemperateForest => 1.0 / 120.0,
            Self::Taiga | Self::Swamp => 1.0 / 150.0, Self::Grassland => 1.0 / 260.0,
            Self::Savanna => 1.0 / 300.0, Self::ColdSteppe => 1.0 / 450.0, Self::Tundra => 1.0 / 900.0,
            Self::Alpine => 1.0 / 1200.0, Self::Beach => 1.0 / 1500.0, Self::Desert => 1.0 / 2500.0,
            Self::Ocean | Self::Freshwater | Self::Ice | Self::Snowcap => 0.0,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Ocean => "ocean", Self::Freshwater => "freshwater", Self::Beach => "beach", Self::Ice => "ice sheet",
            Self::Tundra => "tundra", Self::Taiga => "taiga", Self::ColdSteppe => "cold steppe",
            Self::Grassland => "grassland", Self::TemperateForest => "temperate forest", Self::Swamp => "swamp",
            Self::Savanna => "savanna", Self::Desert => "desert", Self::Rainforest => "rainforest",
            Self::Alpine => "alpine rock", Self::Snowcap => "snowcap",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldTemplate { Continents, Archipelago, Pangaea, Islands, Lakes }

impl WorldTemplate {
    pub const ALL: [Self; 5] = [Self::Continents, Self::Archipelago, Self::Pangaea, Self::Islands, Self::Lakes];
    pub fn label(self) -> &'static str {
        match self {
            Self::Continents => "continents", Self::Archipelago => "archipelago", Self::Pangaea => "pangaea",
            Self::Islands => "islands", Self::Lakes => "lakes",
        }
    }
    /// Share of tiles above sea level.
    pub fn land_fraction(self) -> f32 {
        match self {
            Self::Continents => 0.38, Self::Archipelago => 0.26, Self::Pangaea => 0.42,
            Self::Islands => 0.17, Self::Lakes => 0.68,
        }
    }
    fn base_frequency(self) -> f32 {
        match self {
            Self::Continents => 2.2, Self::Archipelago => 5.0, Self::Pangaea => 2.0,
            Self::Islands => 7.0, Self::Lakes => 2.6,
        }
    }
    fn macro_shape(self, u: f32, v: f32, n: &Noise2) -> f32 {
        match self {
            Self::Continents => 0.25 * n.fbm(u * 1.3 + 9.1, v * 1.3 + 2.7, 2),
            Self::Pangaea => {
                let r = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt();
                (0.45 - r) * 1.6
            }
            Self::Archipelago | Self::Islands | Self::Lakes => 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapSize { Small, Medium, Large, Huge }

impl MapSize {
    pub const ALL: [Self; 4] = [Self::Small, Self::Medium, Self::Large, Self::Huge];
    pub fn tiles(self) -> usize {
        match self { Self::Small => 256, Self::Medium => 512, Self::Large => 1024, Self::Huge => 2048 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainBrush { Water, Rock, Land }

#[derive(Clone)]
pub struct TileMap {
    pub width: usize,
    pub height: usize,
    pub seed: u64,
    pub template: WorldTemplate,
    /// Surface height above sea level in metres (negative = water depth).
    pub height_m: Vec<i16>,
    pub temperature_c: Vec<i8>,
    /// 0..=255 annual wetness index.
    pub moisture: Vec<u8>,
    pub ground: Vec<Ground>,
    pub biome: Vec<Biome>,
    /// River discharge class, 0 = no river.
    pub river: Vec<u8>,
    /// Tile distance to open ocean, capped at 255.
    pub coast_dist: Vec<u8>,
    /// Tile distance to a river or lake, capped at 255.
    pub fresh_dist: Vec<u8>,
    chunk_revision: Vec<u32>,
}

impl std::fmt::Debug for TileMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TileMap({}x{} {:?} seed {})", self.width, self.height, self.template, self.seed)
    }
}

const N8: [(i32, i32); 8] = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];
const N4: [(i32, i32); 4] = [(0, -1), (-1, 0), (1, 0), (0, 1)];

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn hash_unit(seed: u64, i: u64) -> f32 {
    let mut s = seed ^ i.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    s ^= s >> 30; s = s.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    s ^= s >> 27; s = s.wrapping_mul(0x94D0_49BB_1331_11EB);
    s ^= s >> 31;
    (s >> 40) as f32 / (1u64 << 24) as f32
}

/// Fill `out` row by row in parallel. Each cell is independent, so the
/// result does not depend on thread count.
fn par_rows<T: Send>(out: &mut [T], width: usize, f: impl Fn(usize, usize) -> T + Sync) {
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1).min(16);
    let rows = out.len() / width;
    let band = rows.div_ceil(threads).max(1);
    std::thread::scope(|s| {
        for (b, chunk) in out.chunks_mut(band * width).enumerate() {
            let f = &f;
            s.spawn(move || {
                for (i, cell) in chunk.iter_mut().enumerate() {
                    let y = b * band + i / width;
                    *cell = f(i % width, y);
                }
            });
        }
    });
}

/// Multi-source 4-neighbour BFS distance, capped at 255.
fn distance_field(w: usize, h: usize, sources: impl Iterator<Item = usize>) -> Vec<u8> {
    let mut d = vec![255u8; w * h];
    let mut q = VecDeque::new();
    for i in sources {
        d[i] = 0;
        q.push_back(i);
    }
    while let Some(i) = q.pop_front() {
        let (x, y) = ((i % w) as i32, (i / w) as i32);
        let next = d[i].saturating_add(1);
        if next == 255 { continue; }
        for (dx, dy) in N4 {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 { continue; }
            let j = ny as usize * w + nx as usize;
            if d[j] > next {
                d[j] = next;
                q.push_back(j);
            }
        }
    }
    d
}

pub fn classify_biome(ground: Ground, temp_c: f32, moisture: f32, height_m: f32) -> Biome {
    match ground {
        Ground::DeepOcean | Ground::Ocean | Ground::Shallow => return if temp_c < -20.0 { Biome::Ice } else { Biome::Ocean },
        Ground::Lake => return if temp_c < -8.0 { Biome::Ice } else { Biome::Freshwater },
        Ground::Beach => return if temp_c < -2.0 { Biome::Tundra } else { Biome::Beach },
        Ground::Peak => return if temp_c < 4.0 { Biome::Snowcap } else { Biome::Alpine },
        Ground::Mountain => return if temp_c < -4.0 { Biome::Snowcap } else { Biome::Alpine },
        _ => {}
    }
    let low = height_m < 350.0;
    if temp_c < -8.0 { Biome::Ice }
    else if temp_c < 0.0 { Biome::Tundra }
    else if temp_c < 8.0 { if moisture > 0.5 { Biome::Taiga } else { Biome::ColdSteppe } }
    else if temp_c < 20.0 {
        if moisture < 0.22 { Biome::Desert }
        else if moisture > 0.8 && low { Biome::Swamp }
        else if moisture < 0.48 { Biome::Grassland }
        else { Biome::TemperateForest }
    } else if moisture < 0.28 { Biome::Desert }
    else if moisture > 0.82 && low { Biome::Swamp }
    else if moisture > 0.6 { Biome::Rainforest }
    else { Biome::Savanna }
}

impl TileMap {
    pub fn generate(seed: u64, size: MapSize, template: WorldTemplate) -> Self {
        let (w, h) = (size.tiles(), size.tiles());
        let n = w * h;
        let base = Noise2::new(seed ^ 0xA11C_E5ED);
        let warp = Noise2::new(seed ^ 0x0B0B_5EED);
        let ridge = Noise2::new(seed ^ 0xC0FF_EE00);
        let clim = Noise2::new(seed ^ 0xD00D_1234);
        let wet = Noise2::new(seed ^ 0xE1E1_7777);
        // Bigger maps hold more landmasses, not just the same ones upscaled.
        let scale = (w as f32 / 512.0).sqrt();
        let freq = template.base_frequency() * scale;

        // 1. Raw elevation: warped fBm + ridged chains + template macro shape + ocean rim.
        let mut raw = vec![0f32; n];
        par_rows(&mut raw, w, |x, y| {
            let (u, v) = (x as f32 / w as f32, y as f32 / h as f32);
            let wx = u + 0.18 * warp.fbm(u * 3.0, v * 3.0, 3);
            let wy = v + 0.18 * warp.fbm(u * 3.0 + 5.2, v * 3.0 + 1.3, 3);
            let mut e = base.fbm(wx * freq, wy * freq, 7);
            e += 0.35 * ridge.ridged(wx * freq * 1.7, wy * freq * 1.7, 5) * (e + 0.3).clamp(0.0, 1.0);
            e += template.macro_shape(u, v, &base);
            let edge = u.min(1.0 - u).min(v).min(1.0 - v);
            e - (1.0 - smoothstep(0.0, 0.09, edge)) * 1.2
        });

        // 2. Sea level from the template's land share.
        let mut sorted = raw.clone();
        let k = (((1.0 - template.land_fraction()) * n as f32) as usize).min(n - 1);
        let (_, sea, _) = sorted.select_nth_unstable_by(k, |a, b| a.total_cmp(b));
        let sea = *sea;
        let (lo, hi) = raw.iter().fold((f32::MAX, f32::MIN), |(lo, hi), &e| (lo.min(e), hi.max(e)));
        let to_m = |e: f32| -> f32 {
            if e > sea {
                1.0 + ((e - sea) / (hi - sea).max(1e-6)).powf(2.0) * 4400.0
            } else {
                -1.0 - ((sea - e) / (sea - lo).max(1e-6)).powf(0.8) * 5200.0
            }
        };
        let mut height_m: Vec<i16> = raw.iter().map(|&e| to_m(e) as i16).collect();
        drop(sorted);

        // 3. Water connected to the map rim is ocean; enclosed water is lake.
        let mut ocean = vec![false; n];
        let mut q = VecDeque::new();
        for i in 0..n {
            let (x, y) = (i % w, i / w);
            if (x == 0 || y == 0 || x == w - 1 || y == h - 1) && height_m[i] <= 0 {
                ocean[i] = true;
                q.push_back(i);
            }
        }
        while let Some(i) = q.pop_front() {
            let (x, y) = ((i % w) as i32, (i / w) as i32);
            for (dx, dy) in N4 {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 { continue; }
                let j = ny as usize * w + nx as usize;
                if !ocean[j] && height_m[j] <= 0 {
                    ocean[j] = true;
                    q.push_back(j);
                }
            }
        }
        let coast_dist = distance_field(w, h, (0..n).filter(|&i| ocean[i]));

        // 4. Temperature (latitude + lapse rate) and first-pass moisture.
        let mut temperature = vec![0f32; n];
        par_rows(&mut temperature, w, |x, y| {
            let (u, v) = (x as f32 / w as f32, y as f32 / h as f32);
            let lat = (v - 0.5).abs() * 2.0;
            let alt_km = height_m[y * w + x].max(0) as f32 / 1000.0;
            31.0 - 46.0 * lat.powf(1.7) - 6.0 * alt_km + 4.0 * clim.fbm(u * 4.0, v * 4.0, 3)
        });
        let coast_reach = 24.0 * scale;
        let mut moisture = vec![0f32; n];
        par_rows(&mut moisture, w, |x, y| {
            let (u, v) = (x as f32 / w as f32, y as f32 / h as f32);
            let lat = (v - 0.5).abs() * 2.0;
            let i = y * w + x;
            let maritime = (-(coast_dist[i] as f32) / coast_reach).exp();
            // Subtropical high-pressure belt dries the ~30° latitudes.
            let hadley = (-((lat - 0.32) / 0.09).powi(2)).exp();
            0.46 + 0.4 * wet.fbm(u * 3.5 * scale, v * 3.5 * scale, 4) + 0.3 * maritime - 0.3 * hadley
        });

        // 5. Drainage: priority-flood from all water on the continuous (unquantised)
        // elevation so coastal lowlands do not collapse into tied flats.
        // The discovery tree is the flow direction.
        let key = |e: f32| -> u32 {
            let b = e.to_bits();
            if b & 0x8000_0000 != 0 { !b } else { b | 0x8000_0000 }
        };
        let mut filled = vec![f32::NAN; n];
        let mut down = vec![u32::MAX; n];
        let mut order: Vec<u32> = Vec::with_capacity(n);
        let mut heap = BinaryHeap::new();
        for i in 0..n {
            if height_m[i] <= 0 {
                filled[i] = raw[i];
                heap.push(Reverse((key(raw[i]), i as u32)));
            }
        }
        while let Some(Reverse((_, i))) = heap.pop() {
            let i = i as usize;
            let level = filled[i];
            let (x, y) = ((i % w) as i32, (i / w) as i32);
            for (dx, dy) in N8 {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 { continue; }
                let j = ny as usize * w + nx as usize;
                if !filled[j].is_nan() { continue; }
                filled[j] = raw[j].max(level);
                down[j] = i as u32;
                order.push(j as u32);
                heap.push(Reverse((key(filled[j]), j as u32)));
            }
        }
        let depression = |i: usize| filled[i] > raw[i];
        let mut acc: Vec<f32> = (0..n).map(|i| if height_m[i] > 0 { 0.25 + moisture[i].clamp(0.0, 1.0) } else { 0.0 }).collect();
        for &i in order.iter().rev() {
            let d = down[i as usize];
            if d != u32::MAX {
                acc[d as usize] += acc[i as usize];
            }
        }
        let mut land_acc: Vec<f32> = (0..n).filter(|&i| height_m[i] > 0).map(|i| acc[i]).collect();
        let threshold = if land_acc.is_empty() { f32::MAX } else {
            let k = ((land_acc.len() as f32 * 0.985) as usize).min(land_acc.len() - 1);
            let (_, q, _) = land_acc.select_nth_unstable_by(k, |a, b| a.total_cmp(b));
            q.max(40.0 * w as f32 / 256.0)
        };
        drop(land_acc);
        let mut lake = vec![false; n];
        let mut river = vec![0u8; n];
        // Closed basins hold standing water only when compact; sprawling noise
        // basins are treated as carved through (rivers cross them instead).
        let basin = |i: usize| height_m[i] > 0 && to_m(filled[i]) - height_m[i] as f32 >= 40.0;
        let (min_lake, max_lake) = (6usize, 60 + n / 2500);
        let mut seen = vec![false; n];
        for start in 0..n {
            if seen[start] || !basin(start) { continue; }
            let mut comp = vec![start];
            seen[start] = true;
            let mut k = 0;
            while k < comp.len() {
                let i = comp[k];
                k += 1;
                let (x, y) = ((i % w) as i32, (i / w) as i32);
                for (dx, dy) in N4 {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 { continue; }
                    let j = ny as usize * w + nx as usize;
                    if !seen[j] && basin(j) {
                        seen[j] = true;
                        comp.push(j);
                    }
                }
            }
            if comp.len() >= min_lake && comp.len() <= max_lake {
                for i in comp { lake[i] = true; }
            }
        }
        drop(seen);
        // Across flat filled basins the drainage tree fans out; keep only the
        // dominant inflow of each cell there so one channel crosses the flat.
        let mut main_in = vec![u32::MAX; n];
        for i in 0..n {
            let d = down[i];
            if d == u32::MAX { continue; }
            let m = main_in[d as usize];
            if m == u32::MAX || acc[i] > acc[m as usize] { main_in[d as usize] = i as u32; }
        }
        for i in 0..n {
            if height_m[i] <= 0 {
                lake[i] = !ocean[i];
            } else if lake[i] {
            } else if acc[i] >= threshold
                && (!depression(i) || down[i] == u32::MAX || main_in[down[i] as usize] == i as u32)
            {
                river[i] = ((acc[i] / threshold).log2() * 40.0 + 60.0).clamp(1.0, 255.0) as u8;
            }
        }
        // Pixel rivers must read as continuous lines: bridge diagonal flow steps
        // through an orthogonal tile, and widen major rivers to two tiles.
        let strong = 180u8;
        let mut extra: Vec<(usize, u8)> = Vec::new();
        for i in 0..n {
            if river[i] == 0 { continue; }
            let d = down[i];
            if d == u32::MAX { continue; }
            let (x, y) = (i % w, i / w);
            let (dx, dy) = (d as usize % w, d as usize / w);
            if x != dx && y != dy {
                let bridge = y * w + dx;
                if height_m[bridge] > 0 && !lake[bridge] && river[bridge] == 0 { extra.push((bridge, river[i])); }
            }
            if river[i] >= strong {
                let side = if x != dx { (y + 1).min(h - 1) * w + x } else { y * w + (x + 1).min(w - 1) };
                if height_m[side] > 0 && !lake[side] && river[side] == 0 { extra.push((side, river[i])); }
            }
        }
        // Applied after the scan so added tiles never seed further widening.
        for (j, r) in extra {
            river[j] = river[j].max(r);
        }
        drop(acc);
        drop(filled);
        drop(raw);
        let fresh_dist = distance_field(w, h, (0..n).filter(|&i| lake[i] || river[i] > 0));
        for i in 0..n {
            moisture[i] = (moisture[i] + 0.2 * (-(fresh_dist[i] as f32) / 5.0).exp()).clamp(0.0, 1.0);
        }

        // 6. Ground and biome.
        let mut ground = vec![Ground::Lowland; n];
        let mut biome = vec![Biome::Grassland; n];
        for i in 0..n {
            let hm = height_m[i] as f32;
            let g = if ocean[i] {
                if hm > -250.0 { Ground::Shallow } else if hm > -2000.0 { Ground::Ocean } else { Ground::DeepOcean }
            } else if lake[i] {
                Ground::Lake
            } else if river[i] > 0 {
                Ground::River
            } else if coast_dist[i] <= 1 && hm < 120.0 {
                Ground::Beach
            } else if hm < 700.0 {
                Ground::Lowland
            } else if hm < 1450.0 {
                Ground::Hills
            } else if hm < 2750.0 {
                Ground::Mountain
            } else {
                Ground::Peak
            };
            ground[i] = g;
            biome[i] = classify_biome(g, temperature[i], moisture[i], hm);
            if lake[i] && height_m[i] > 0 {
                height_m[i] = 0;
            }
        }

        let chunks = w.div_ceil(CHUNK) * h.div_ceil(CHUNK);
        Self {
            width: w, height: h, seed, template, height_m,
            temperature_c: temperature.iter().map(|t| t.round().clamp(-128.0, 127.0) as i8).collect(),
            moisture: moisture.iter().map(|m| (m * 255.0).round() as u8).collect(),
            ground, biome, river, coast_dist, fresh_dist,
            chunk_revision: vec![0; chunks],
        }
    }

    pub fn index(&self, x: usize, y: usize) -> usize { y * self.width + x }

    /// Tile containing a world position (world origin = map centre, +y = north).
    pub fn tile_at(&self, p: Position) -> Option<(usize, usize)> {
        let fx = (p.x / TILE_SIZE + self.width as f32 * 0.5).floor();
        let fy = (self.height as f32 * 0.5 - p.y / TILE_SIZE).floor();
        if fx < 0.0 || fy < 0.0 || fx >= self.width as f32 || fy >= self.height as f32 { return None; }
        Some((fx as usize, fy as usize))
    }

    pub fn tile_center(&self, x: usize, y: usize) -> Position {
        Position {
            x: (x as f32 + 0.5 - self.width as f32 * 0.5) * TILE_SIZE,
            y: (self.height as f32 * 0.5 - y as f32 - 0.5) * TILE_SIZE,
        }
    }

    pub fn ground_at(&self, p: Position) -> Option<Ground> {
        self.tile_at(p).map(|(x, y)| self.ground[self.index(x, y)])
    }

    pub fn walkable_at(&self, p: Position) -> bool {
        self.ground_at(p).map(Ground::walkable).unwrap_or(false)
    }

    /// Closest walkable tile centre within `max_tiles` rings, scanning outward.
    pub fn nearest_walkable(&self, p: Position, max_tiles: i32) -> Option<Position> {
        if self.walkable_at(p) { return Some(p); }
        let fx = (p.x / TILE_SIZE + self.width as f32 * 0.5).floor() as i32;
        let fy = (self.height as f32 * 0.5 - p.y / TILE_SIZE).floor() as i32;
        for r in 1..=max_tiles {
            let mut best: Option<(i32, usize, usize)> = None;
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r { continue; }
                    let (x, y) = (fx + dx, fy + dy);
                    if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 { continue; }
                    let (x, y) = (x as usize, y as usize);
                    if !self.ground[self.index(x, y)].walkable() { continue; }
                    let d = dx * dx + dy * dy;
                    if best.map(|b| d < b.0).unwrap_or(true) { best = Some((d, x, y)); }
                }
            }
            if let Some((_, x, y)) = best { return Some(self.tile_center(x, y)); }
        }
        None
    }

    pub fn land_fraction(&self) -> f32 {
        self.height_m.iter().filter(|&&h| h > 0).count() as f32 / self.height_m.len() as f32
    }

    /// Most habitable tile for a founding band: mild biome, fresh water close,
    /// and enough walkable ground around it to grow.
    pub fn start_site(&self) -> (usize, usize) {
        let (w, h) = (self.width, self.height);
        let ring: Vec<(i32, i32)> = (0..32).map(|k| {
            let a = k as f32 / 32.0 * std::f32::consts::TAU;
            ((a.cos() * 20.0) as i32, (a.sin() * 20.0) as i32)
        }).collect();
        let mut best = (f32::MIN, w / 2, h / 2);
        for y in (24..h.saturating_sub(24)).step_by(4) {
            for x in (24..w.saturating_sub(24)).step_by(4) {
                let i = self.index(x, y);
                let g = self.ground[i];
                if !g.walkable() || matches!(g, Ground::Mountain | Ground::River) { continue; }
                let b = self.biome[i];
                let climate = match b {
                    Biome::Grassland => 1.0, Biome::TemperateForest => 0.9, Biome::Savanna => 0.8,
                    Biome::Rainforest => 0.6, Biome::Taiga | Biome::ColdSteppe | Biome::Beach => 0.5,
                    Biome::Swamp => 0.3, Biome::Desert | Biome::Tundra => 0.15, _ => 0.05,
                };
                let fresh = if self.fresh_dist[i] <= 6 { 0.6 } else if self.fresh_dist[i] <= 15 { 0.3 } else { 0.0 };
                let coast = if self.coast_dist[i] <= 8 { 0.25 } else { 0.0 };
                let room = ring.iter().filter(|(dx, dy)| {
                    let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                    nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h
                        && self.ground[self.index(nx as usize, ny as usize)].walkable()
                }).count() as f32 / ring.len() as f32;
                let score = climate + fresh + coast + room;
                if score > best.0 { best = (score, x, y); }
            }
        }
        (best.1, best.2)
    }

    pub fn chunks_x(&self) -> usize { self.width.div_ceil(CHUNK) }
    pub fn chunks_y(&self) -> usize { self.height.div_ceil(CHUNK) }
    /// Monotonic per-chunk edit counter so renderers can redraw only what changed.
    pub fn chunk_revision(&self, cx: usize, cy: usize) -> u32 {
        self.chunk_revision[cy * self.chunks_x() + cx]
    }

    /// God-tool terrain edit over a disc. Returns the number of tiles changed.
    pub fn paint(&mut self, center: Position, radius: f32, brush: TerrainBrush) -> usize {
        let r_tiles = (radius / TILE_SIZE).max(0.5);
        let Some((cx, cy)) = self.tile_at(center) else { return 0; };
        let ri = r_tiles.ceil() as i32;
        let mut changed = 0;
        for dy in -ri..=ri {
            for dx in -ri..=ri {
                if (dx * dx + dy * dy) as f32 > r_tiles * r_tiles { continue; }
                let (x, y) = (cx as i32 + dx, cy as i32 + dy);
                if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 { continue; }
                let (x, y) = (x as usize, y as usize);
                let i = self.index(x, y);
                let (g, hm) = match brush {
                    TerrainBrush::Water => (Ground::Lake, (self.height_m[i] as i32).min(-30)),
                    TerrainBrush::Rock => (Ground::Mountain, (self.height_m[i] as i32).max(1800)),
                    TerrainBrush::Land => (Ground::Lowland, (self.height_m[i] as i32).clamp(40, 600)),
                };
                self.ground[i] = g;
                self.height_m[i] = hm as i16;
                self.river[i] = 0;
                self.biome[i] = classify_biome(g, self.temperature_c[i] as f32, self.moisture[i] as f32 / 255.0, hm as f32);
                let ci = (y / CHUNK) * self.chunks_x() + x / CHUNK;
                self.chunk_revision[ci] = self.chunk_revision[ci].wrapping_add(1);
                changed += 1;
            }
        }
        changed
    }

    /// Deterministic per-tile unit value for sampling decisions.
    pub fn tile_roll(&self, i: usize, salt: u64) -> f32 {
        hash_unit(self.seed ^ salt, i as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(seed: u64, t: WorldTemplate) -> TileMap {
        TileMap::generate(seed, MapSize::Small, t)
    }

    #[test]
    fn generation_is_deterministic_per_seed() {
        let a = map(11, WorldTemplate::Continents);
        let b = map(11, WorldTemplate::Continents);
        let c = map(12, WorldTemplate::Continents);
        assert_eq!(a.height_m, b.height_m);
        assert_eq!(a.ground, b.ground);
        assert_ne!(a.height_m, c.height_m);
    }

    #[test]
    fn templates_hit_their_land_share_and_rim_is_ocean() {
        for t in WorldTemplate::ALL {
            let m = map(5, t);
            // Lakes and filled basins are counted as water, so allow a margin below target.
            let land = m.land_fraction();
            assert!((land - t.land_fraction()).abs() < 0.03, "{t:?} land {land}");
            for x in 0..m.width {
                assert!(!m.ground[m.index(x, 0)].walkable());
                assert!(!m.ground[m.index(x, m.height - 1)].walkable());
            }
        }
        assert!(map(5, WorldTemplate::Islands).land_fraction() < map(5, WorldTemplate::Pangaea).land_fraction());
    }

    #[test]
    fn rivers_exist_and_run_downhill_to_water() {
        let m = map(21, WorldTemplate::Continents);
        let rivers: Vec<usize> = (0..m.ground.len()).filter(|&i| m.ground[i] == Ground::River).collect();
        assert!(rivers.len() > 50, "only {} river tiles", rivers.len());
        // Every river tile touches another river tile or open/standing water.
        for &i in &rivers {
            let (x, y) = ((i % m.width) as i32, (i / m.width) as i32);
            let linked = N8.iter().any(|(dx, dy)| {
                let (nx, ny) = (x + dx, y + dy);
                nx >= 0 && ny >= 0 && (nx as usize) < m.width && (ny as usize) < m.height && {
                    let g = m.ground[m.index(nx as usize, ny as usize)];
                    g == Ground::River || g.is_water()
                }
            });
            assert!(linked, "isolated river tile at {x},{y}");
        }
    }

    #[test]
    fn climate_orders_biomes_by_latitude_and_altitude() {
        let m = map(3, WorldTemplate::Pangaea);
        for i in 0..m.biome.len() {
            let t = m.temperature_c[i] as f32;
            match m.biome[i] {
                Biome::Rainforest | Biome::Savanna => assert!(t >= 19.0, "{:?} at {t}C", m.biome[i]),
                Biome::Tundra if m.ground[i] != Ground::Beach => assert!(t < 1.0),
                Biome::Snowcap => assert!(t < 5.0),
                _ => {}
            }
            if m.ground[i] == Ground::Peak { assert!(m.height_m[i] >= 2750); }
        }
        // Polar rows are colder than the equator row.
        let row_mean = |y: usize| (0..m.width).map(|x| m.temperature_c[m.index(x, y)] as f32).sum::<f32>() / m.width as f32;
        assert!(row_mean(4) < row_mean(m.height / 2) - 20.0);
    }

    #[test]
    fn start_site_is_walkable_with_room_to_grow() {
        for t in WorldTemplate::ALL {
            let m = map(9, t);
            let (x, y) = m.start_site();
            let i = m.index(x, y);
            assert!(m.ground[i].walkable(), "{t:?}");
            assert!(m.biome[i].productivity() > 0.0, "{t:?} start on {:?}", m.biome[i]);
        }
    }

    #[test]
    fn world_tile_round_trip_and_paint_marks_chunks() {
        let mut m = map(4, WorldTemplate::Continents);
        let p = m.tile_center(100, 37);
        assert_eq!(m.tile_at(p), Some((100, 37)));
        let before = m.chunk_revision(100 / CHUNK, 37 / CHUNK);
        let changed = m.paint(p, TILE_SIZE * 3.0, TerrainBrush::Water);
        assert!(changed >= 25);
        assert!(!m.walkable_at(p));
        assert_ne!(m.chunk_revision(100 / CHUNK, 37 / CHUNK), before);
        m.paint(p, TILE_SIZE * 3.0, TerrainBrush::Land);
        assert!(m.walkable_at(p));
    }
}
