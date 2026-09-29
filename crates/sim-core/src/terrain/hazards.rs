//! Tile-level fire and surface water: the physical side of fires and floods.
//!
//! Fire burns the fuel a tile's vegetation provides, spreads to neighbours with
//! a chance set by their fuel, dryness and the wind, stops at water, and leaves
//! scorched ground that regrows over about a year. Surface water flows to the
//! lowest neighbouring water surface, pools in basins, drains into rivers,
//! lakes and the sea, and is lost to evaporation and infiltration.
//! Only active tiles are processed, so an idle world costs nothing.
use super::{Biome, Ground, TileMap, CHUNK, TILE_SIZE};
use crate::world::Position;

/// Spread rate at full intensity into full, dry fuel (per day, per neighbour).
const SPREAD_PER_DAY: f32 = 5.0;
/// Fuel fraction a full-intensity fire consumes per day.
const BURN_PER_DAY: f32 = 0.85;
/// Scorch fraction regrown per day on ground no longer burning.
const REGROW_PER_DAY: f32 = 0.0025;
/// Share of the surface-height difference that can move per day.
const FLOW_PER_DAY: f32 = 12.0;
/// Evaporation plus infiltration, metres per day.
const WATER_LOSS_PER_DAY: f32 = 0.05;
const MIN_WATER: f32 = 0.004;

#[derive(Clone, Debug)]
pub struct Hazards {
    width: usize,
    height: usize,
    /// Fire intensity 0..1 per tile.
    pub fire: Vec<f32>,
    burning: Vec<u32>,
    /// Standing surface water on land, metres.
    pub water: Vec<f32>,
    wet: Vec<u32>,
    /// Burnt fraction 0..1: fuel left is base fuel times (1 - scorch).
    pub scorch: Vec<f32>,
    scorched: Vec<u32>,
    chunk_revision: Vec<u32>,
    /// Prevailing wind as a unit vector in tile space (x east, y south).
    pub wind: (f32, f32),
    ticks: u64,
    flow: Vec<f32>,
}

/// Burnable fuel a biome provides, 0..1.
pub fn base_fuel(b: Biome) -> f32 {
    match b {
        Biome::TemperateForest | Biome::Taiga => 1.0,
        Biome::Savanna => 0.9,
        Biome::Rainforest => 0.75,
        Biome::Grassland => 0.7,
        Biome::ColdSteppe => 0.55,
        Biome::Swamp => 0.35,
        Biome::Tundra => 0.2,
        Biome::Desert => 0.08,
        Biome::Beach | Biome::Alpine => 0.05,
        Biome::Ocean | Biome::Freshwater | Biome::Ice | Biome::Snowcap => 0.0,
    }
}

fn hash_unit(seed: u64, a: u64, b: u64) -> f32 {
    let mut x = seed ^ a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ b.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    x ^= x >> 30; x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27; x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x >> 40) as f32 / (1u64 << 24) as f32
}

impl Hazards {
    pub fn new(t: &TileMap, seed: u64) -> Self {
        let n = t.width * t.height;
        let a = hash_unit(seed, 17, 29) * std::f32::consts::TAU;
        Self {
            width: t.width, height: t.height,
            fire: vec![0.0; n], burning: Vec::new(),
            water: vec![0.0; n], wet: Vec::new(),
            scorch: vec![0.0; n], scorched: Vec::new(),
            chunk_revision: vec![0; t.chunks_x() * t.chunks_y()],
            wind: (a.cos(), a.sin()), ticks: 0, flow: vec![0.0; n],
        }
    }

    pub fn burning(&self) -> &[u32] { &self.burning }
    pub fn wet(&self) -> &[u32] { &self.wet }
    pub fn is_quiet(&self) -> bool { self.burning.is_empty() && self.wet.is_empty() }

    pub fn chunk_revision(&self, cx: usize, cy: usize) -> u32 {
        self.chunk_revision[cy * self.width.div_ceil(CHUNK) + cx]
    }
    fn touch(&mut self, i: usize) {
        let (x, y) = (i % self.width, i / self.width);
        let ci = (y / CHUNK) * self.width.div_ceil(CHUNK) + x / CHUNK;
        self.chunk_revision[ci] = self.chunk_revision[ci].wrapping_add(1);
    }

    /// Fuel left on a tile, 0..1.
    pub fn fuel(&self, t: &TileMap, i: usize) -> f32 {
        if !t.ground[i].walkable() || t.ground[i] == Ground::River { return 0.0; }
        base_fuel(t.biome[i]) * (1.0 - self.scorch[i]) * (1.0 - (self.water[i] / 0.2).min(1.0))
    }

    /// Walkers cannot enter a tile burning hard or flooded deeper than knee height.
    pub fn blocks(&self, i: usize) -> bool { self.fire[i] > 0.6 || self.water[i] > 0.7 }

    fn tiles_in(&self, t: &TileMap, center: Position, radius: f32) -> Vec<(usize, f32)> {
        let Some((cx, cy)) = t.tile_at(center) else { return Vec::new(); };
        let r = (radius / TILE_SIZE).max(0.6);
        let ri = r.ceil() as i32;
        let mut out = Vec::new();
        for dy in -ri..=ri { for dx in -ri..=ri {
            let d = ((dx * dx + dy * dy) as f32).sqrt();
            if d > r { continue; }
            let (x, y) = (cx as i32 + dx, cy as i32 + dy);
            if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 { continue; }
            out.push((y as usize * self.width + x as usize, d / r));
        }}
        out
    }

    /// Set fire to every burnable tile within `radius` world units.
    pub fn ignite(&mut self, t: &TileMap, center: Position, radius: f32, intensity: f32) -> usize {
        let mut lit = 0;
        for (i, _) in self.tiles_in(t, center, radius) {
            if self.fuel(t, i) < 0.05 { continue; }
            if self.fire[i] == 0.0 { self.burning.push(i as u32); }
            self.fire[i] = self.fire[i].max(intensity.clamp(0.1, 1.0));
            self.touch(i);
            lit += 1;
        }
        self.burning.sort_unstable();
        self.burning.dedup();
        lit
    }

    /// Pour water onto land within `radius`, deepest at the centre.
    pub fn add_water(&mut self, t: &TileMap, center: Position, radius: f32, depth_m: f32) {
        for (i, d) in self.tiles_in(t, center, radius) {
            if !t.ground[i].walkable() || t.ground[i] == Ground::River { continue; }
            if self.water[i] == 0.0 { self.wet.push(i as u32); }
            self.water[i] += depth_m.max(0.0) * (1.0 - 0.5 * d);
            self.touch(i);
        }
        self.wet.sort_unstable();
        self.wet.dedup();
    }

    /// Rain douses fires and wets the ground.
    pub fn rain(&mut self, t: &TileMap, center: Position, radius: f32, intensity: f32) {
        for (i, _) in self.tiles_in(t, center, radius) {
            if self.fire[i] > 0.0 {
                self.fire[i] *= (1.0 - intensity).clamp(0.0, 1.0) * 0.3;
                self.touch(i);
            }
        }
        self.add_water(t, center, radius, 0.03 * intensity);
    }

    pub fn step(&mut self, t: &TileMap, days: f32, seed: u64) {
        if days <= 0.0 { return; }
        self.ticks = self.ticks.wrapping_add(1);
        if !self.burning.is_empty() { self.step_fire(t, days, seed); }
        if !self.scorched.is_empty() { self.regrow(days); }
        if !self.wet.is_empty() { self.step_water(t, days); }
    }

    fn step_fire(&mut self, t: &TileMap, days: f32, seed: u64) {
        let (w, h) = (self.width as i32, self.height as i32);
        let mut ignitions: Vec<u32> = Vec::new();
        let burning = std::mem::take(&mut self.burning);
        for &bi in &burning {
            let i = bi as usize;
            let f = self.fire[i];
            if f <= 0.0 { continue; }
            // Burn fuel; intensity follows what is left and dies in water.
            let before = self.scorch[i];
            self.scorch[i] = (before + BURN_PER_DAY * f * days).min(1.0);
            if before == 0.0 { self.scorched.push(bi); }
            let left = self.fuel(t, i);
            self.fire[i] = if left < 0.03 { 0.0 } else { (left * 1.3).min(1.0).max(f * 0.6) };
            self.touch(i);
            if self.fire[i] == 0.0 { continue; }
            let (x, y) = ((i % self.width) as i32, (i / self.width) as i32);
            for dy in -1..=1 { for dx in -1..=1 {
                if dx == 0 && dy == 0 { continue; }
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w || ny >= h { continue; }
                let j = ny as usize * self.width + nx as usize;
                if self.fire[j] > 0.0 { continue; }
                let fuel = self.fuel(t, j);
                if fuel < 0.05 { continue; }
                let len = ((dx * dx + dy * dy) as f32).sqrt();
                let downwind = (dx as f32 * self.wind.0 + dy as f32 * self.wind.1) / len;
                let dryness = 1.0 - t.moisture[j] as f32 / 255.0 * 0.7;
                let rate = SPREAD_PER_DAY * self.fire[i] * fuel * dryness * (1.0 + 0.8 * downwind) / len;
                let p = 1.0 - (-rate.max(0.0) * days).exp();
                if hash_unit(seed, j as u64, self.ticks) < p { ignitions.push(j as u32); }
            }}
        }
        self.burning = burning.into_iter().filter(|&i| self.fire[i as usize] > 0.0).collect();
        for j in ignitions {
            if self.fire[j as usize] == 0.0 {
                self.fire[j as usize] = 0.4;
                self.burning.push(j);
                self.touch(j as usize);
            }
        }
        self.burning.sort_unstable();
        self.burning.dedup();
    }

    fn regrow(&mut self, days: f32) {
        let scorched = std::mem::take(&mut self.scorched);
        let mut keep = Vec::with_capacity(scorched.len());
        for si in scorched {
            let i = si as usize;
            if self.fire[i] == 0.0 {
                let before = self.scorch[i];
                self.scorch[i] = (before - REGROW_PER_DAY * days).max(0.0);
                // Redraw only when the visible shade steps.
                if (before * 8.0) as u8 != (self.scorch[i] * 8.0) as u8 { self.touch(i); }
            }
            if self.scorch[i] > 0.0 { keep.push(si); }
        }
        self.scorched = keep;
        self.scorched.sort_unstable();
        self.scorched.dedup();
    }

    fn step_water(&mut self, t: &TileMap, days: f32) {
        let substeps = ((days * FLOW_PER_DAY / 0.4).ceil() as usize).clamp(1, 12);
        let dt = days / substeps as f32;
        let k = (dt * FLOW_PER_DAY).min(0.5);
        let (w, h) = (self.width as i32, self.height as i32);
        for _ in 0..substeps {
            let wet = std::mem::take(&mut self.wet);
            let mut touched: Vec<u32> = Vec::with_capacity(wet.len() * 2);
            for &wi in &wet {
                let i = wi as usize;
                let depth = self.water[i];
                if depth <= 0.0 { continue; }
                let surface = t.height_m[i].max(0) as f32 + depth;
                let (x, y) = ((i % self.width) as i32, (i / self.width) as i32);
                let mut lower: [(i64, f32); 4] = [(-1, 0.0); 4];
                let mut total = 0.0;
                let mut steepest: f32 = 0.0;
                for (n, (dx, dy)) in [(0, -1), (-1, 0), (1, 0), (0, 1)].into_iter().enumerate() {
                    let (nx, ny) = (x + dx, y + dy);
                    // The map edge drains like the open sea.
                    if nx < 0 || ny < 0 || nx >= w || ny >= h { lower[n] = (-2, depth); total += depth; steepest = steepest.max(depth); continue; }
                    let j = ny as usize * self.width + nx as usize;
                    let sink = !t.ground[j].walkable() || t.ground[j] == Ground::River;
                    let other = if sink { t.height_m[j].min(0) as f32 } else { t.height_m[j].max(0) as f32 + self.water[j] };
                    let diff = surface - other;
                    if diff > 0.0 { lower[n] = (if sink { -2 } else { j as i64 }, diff); total += diff; steepest = steepest.max(diff); }
                }
                if total <= 0.0 { continue; }
                // Move at most half the steepest drop so surfaces settle instead of oscillating.
                let moved = depth.min(steepest * 0.5) * k;
                self.flow[i] -= moved;
                touched.push(wi);
                for (target, diff) in lower {
                    if target == -1 || diff <= 0.0 { continue; }
                    if target >= 0 {
                        self.flow[target as usize] += moved * diff / total;
                        touched.push(target as u32);
                    }
                }
            }
            touched.extend_from_slice(&wet);
            touched.sort_unstable();
            touched.dedup();
            let mut next = Vec::with_capacity(touched.len());
            for &ti in &touched {
                let i = ti as usize;
                let before = self.water[i];
                let mut d = (before + self.flow[i] - WATER_LOSS_PER_DAY * dt).max(0.0);
                self.flow[i] = 0.0;
                if d < MIN_WATER { d = 0.0; }
                self.water[i] = d;
                if (before * 10.0) as u32 != (d * 10.0) as u32 || (before > 0.0) != (d > 0.0) { self.touch(i); }
                if d > 0.0 {
                    next.push(ti);
                    // Standing water puts out fire.
                    if self.fire[i] > 0.0 && d > 0.05 { self.fire[i] = 0.0; self.touch(i); }
                }
            }
            self.wet = next;
        }
    }

    pub fn fire_at(&self, t: &TileMap, p: Position) -> f32 {
        t.tile_at(p).map(|(x, y)| self.fire[y * self.width + x]).unwrap_or(0.0)
    }
    pub fn water_at(&self, t: &TileMap, p: Position) -> f32 {
        t.tile_at(p).map(|(x, y)| self.water[y * self.width + x]).unwrap_or(0.0)
    }
    pub fn scorch_at(&self, t: &TileMap, p: Position) -> f32 {
        t.tile_at(p).map(|(x, y)| self.scorch[y * self.width + x]).unwrap_or(0.0)
    }

    /// Strongest fire within `radius` world units, weighted down by distance:
    /// (felt intensity 0..1, distance in world units, tile index).
    pub fn felt_fire(&self, t: &TileMap, p: Position, radius: f32) -> Option<(f32, f32, usize)> {
        if self.burning.is_empty() { return None; }
        let (cx, cy) = t.tile_at(p)?;
        let r = (radius / TILE_SIZE).ceil() as i32;
        let mut best: Option<(f32, f32, usize)> = None;
        for dy in -r..=r { for dx in -r..=r {
            let (x, y) = (cx as i32 + dx, cy as i32 + dy);
            if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 { continue; }
            let f = self.fire[y as usize * self.width + x as usize];
            if f <= 0.0 { continue; }
            let d = ((dx * dx + dy * dy) as f32).sqrt() * TILE_SIZE;
            if d > radius { continue; }
            let felt = f * (1.0 - d / radius.max(1.0));
            if best.map(|b| felt > b.0).unwrap_or(true) { best = Some((felt, d, y as usize * self.width + x as usize)); }
        }}
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::{MapSize, WorldTemplate};

    /// A flat grassland plain, optionally with a north-south river at column `river_x`
    /// and a height ramp falling toward the east.
    fn plain(river_x: Option<usize>, slope_m_per_tile: f32) -> TileMap {
        let mut m = TileMap::generate(1, MapSize::Small, WorldTemplate::Pangaea);
        for y in 0..m.height { for x in 0..m.width {
            let i = m.index(x, y);
            m.ground[i] = Ground::Lowland;
            m.biome[i] = Biome::Grassland;
            m.moisture[i] = 60;
            m.height_m[i] = (400.0 - x as f32 * slope_m_per_tile).max(1.0) as i16;
            if Some(x) == river_x { m.ground[i] = Ground::River; }
        }}
        m
    }

    fn centre(m: &TileMap, x: usize, y: usize) -> Position { m.tile_center(x, y) }

    #[test]
    fn fire_spreads_through_fuel_and_stops_at_a_river() {
        let m = plain(Some(60), 0.0);
        let mut hz = Hazards::new(&m, 7);
        hz.wind = (1.0, 0.0); // blowing toward the river
        hz.ignite(&m, centre(&m, 40, 128), 6.0, 1.0);
        for _ in 0..60 { hz.step(&m, 0.5, 7); }
        let burnt_near = (0..m.width * m.height).filter(|&i| i % m.width < 60 && hz.scorch[i] > 0.0).count();
        let burnt_far = (0..m.width * m.height).filter(|&i| i % m.width > 60 && hz.scorch[i] > 0.0).count();
        assert!(burnt_near > 500, "fire barely spread: {burnt_near}");
        assert_eq!(burnt_far, 0, "fire jumped the river");
    }

    #[test]
    fn fire_burns_out_and_ground_regrows() {
        let m = plain(None, 0.0);
        let mut hz = Hazards::new(&m, 3);
        hz.ignite(&m, centre(&m, 128, 128), 4.0, 1.0);
        let mut days = 0.0;
        while !hz.burning().is_empty() && days < 400.0 { hz.step(&m, 1.0, 3); days += 1.0; }
        assert!(hz.burning().is_empty(), "fire never burned out");
        let i = m.index(128, 128);
        let right_after = hz.scorch[i];
        assert!(right_after > 0.5);
        for _ in 0..200 { hz.step(&m, 1.0, 3); }
        assert!(hz.scorch[i] < right_after, "scorched ground never recovered");
    }

    #[test]
    fn rain_puts_fire_out() {
        let m = plain(None, 0.0);
        let mut hz = Hazards::new(&m, 5);
        let p = centre(&m, 100, 100);
        hz.ignite(&m, p, 8.0, 1.0);
        hz.rain(&m, p, 60.0, 1.0);
        hz.step(&m, 0.5, 5);
        assert!(hz.fire_at(&m, p) == 0.0, "rain left the centre burning");
    }

    #[test]
    fn water_runs_downhill_and_drains_into_a_river() {
        let m = plain(Some(150), 1.5);
        let mut hz = Hazards::new(&m, 1);
        hz.add_water(&m, centre(&m, 100, 128), 16.0, 2.0);
        let mean_x = |hz: &Hazards| {
            let (mut sx, mut sw) = (0.0, 0.0);
            for &i in hz.wet() { let i = i as usize; sx += (i % m.width) as f32 * hz.water[i]; sw += hz.water[i]; }
            if sw > 0.0 { sx / sw } else { f32::NAN }
        };
        let start = mean_x(&hz);
        let volume = |hz: &Hazards| hz.wet().iter().map(|&i| hz.water[i as usize]).sum::<f32>();
        let v0 = volume(&hz);
        for _ in 0..20 { hz.step(&m, 0.25, 1); }
        assert!(mean_x(&hz) > start + 3.0, "water did not move downhill (east)");
        // Nothing crosses to the far bank: the river takes it.
        assert!(hz.wet().iter().all(|&i| (i as usize % m.width) < 150));
        for _ in 0..200 { hz.step(&m, 0.5, 1); }
        assert!(volume(&hz) < v0 * 0.2, "water never drained");
    }

    #[test]
    fn water_pools_in_a_basin_and_cannot_climb_out() {
        let mut m = plain(None, 0.0);
        // A bowl: height rises with distance from (128,128).
        for y in 0..m.height { for x in 0..m.width {
            let d = (((x as f32 - 128.0).powi(2) + (y as f32 - 128.0).powi(2)).sqrt()).min(60.0);
            let i = m.index(x, y);
            m.height_m[i] = (100.0 + d * 5.0) as i16;
        }}
        let mut hz = Hazards::new(&m, 2);
        hz.add_water(&m, centre(&m, 128, 128), 20.0, 3.0);
        for _ in 0..40 { hz.step(&m, 0.5, 2); }
        assert!(hz.water[m.index(128, 128)] > 0.5, "the basin emptied");
        assert!(hz.wet().iter().all(|&i| {
            let (x, y) = ((i as usize % m.width) as f32, (i as usize / m.width) as f32);
            ((x - 128.0).powi(2) + (y - 128.0).powi(2)).sqrt() < 40.0
        }), "water climbed out of the bowl");
    }
}
