//! Procedural pixel sprites: residents, trees, rocks, houses, building sites
//! and animals are drawn in code from simulation state (heritable traits,
//! household, biome, material, integrity), then cached as images. No asset
//! files; every people and every place gets its own look.
use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use std::collections::HashMap;

type Rgba = [u8; 4];
const CLEAR: Rgba = [0, 0, 0, 0];
const OUTLINE: Rgba = [34, 26, 22, 255];

/// A small RGBA canvas with pixel-art helpers.
struct Canvas { w: usize, h: usize, px: Vec<Rgba> }

impl Canvas {
    fn new(w: usize, h: usize) -> Self { Self { w, h, px: vec![CLEAR; w * h] } }
    fn set(&mut self, x: i32, y: i32, c: Rgba) {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h { self.px[y as usize * self.w + x as usize] = c; }
    }
    fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgba) {
        for yy in y..y + h { for xx in x..x + w { self.set(xx, yy, c); } }
    }
    /// Filled ellipse with light from the upper left.
    fn blob(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, base: Rgba, seed: u64) {
        for y in 0..self.h as i32 { for x in 0..self.w as i32 {
            let (dx, dy) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
            let d = dx * dx + dy * dy;
            if d > 1.0 { continue; }
            let light = -(dx + dy) * 0.5; // upper-left bright
            let n = hash_unit(seed ^ (x as u64 * 131 + y as u64 * 7919)) * 0.16 - 0.08;
            self.set(x, y, scale(base, 1.0 + light * 0.35 + n));
        }}
    }
    /// Dark 1px outline around the silhouette, for readability at any zoom.
    fn outline(&mut self) {
        let snapshot = self.px.clone();
        for y in 0..self.h as i32 { for x in 0..self.w as i32 {
            if snapshot[y as usize * self.w + x as usize][3] != 0 { continue; }
            let solid = [(0, 1), (0, -1), (1, 0), (-1, 0)].iter().any(|(dx, dy)| {
                let (nx, ny) = (x + dx, y + dy);
                nx >= 0 && ny >= 0 && (nx as usize) < self.w && (ny as usize) < self.h && snapshot[ny as usize * self.w + nx as usize][3] != 0
            });
            if solid { self.set(x, y, OUTLINE); }
        }}
    }
    fn into_image(self) -> Image {
        let data: Vec<u8> = self.px.iter().flat_map(|c| c.iter().copied()).collect();
        let mut img = Image::new(
            Extent3d { width: self.w as u32, height: self.h as u32, depth_or_array_layers: 1 },
            TextureDimension::D2, data, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default(),
        );
        img.sampler = ImageSampler::nearest();
        img
    }
}

fn hash_unit(mut x: u64) -> f32 {
    x ^= x >> 30; x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27; x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x >> 40) as f32 / (1u64 << 24) as f32
}
fn scale(c: Rgba, k: f32) -> Rgba {
    [(c[0] as f32 * k).clamp(0.0, 255.0) as u8, (c[1] as f32 * k).clamp(0.0, 255.0) as u8, (c[2] as f32 * k).clamp(0.0, 255.0) as u8, c[3]]
}
fn mix(a: Rgba, b: Rgba, t: f32) -> Rgba {
    let f = |i: usize| (a[i] as f32 + (b[i] as f32 - a[i] as f32) * t) as u8;
    [f(0), f(1), f(2), 255]
}

// ---------- residents ----------

/// What a resident looks like, quantised so identical looks share images.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Look { skin: u8, hair: u8, shirt: u8, long_hair: bool }

const SHIRTS: [Rgba; 12] = [
    [196, 64, 52, 255], [52, 104, 196, 255], [232, 180, 48, 255], [72, 156, 76, 255],
    [150, 78, 170, 255], [226, 122, 44, 255], [40, 150, 160, 255], [220, 220, 210, 255],
    [120, 84, 52, 255], [190, 70, 120, 255], [88, 96, 110, 255], [160, 170, 60, 255],
];
const HAIRS: [Rgba; 6] = [[30, 24, 20, 255], [70, 44, 26, 255], [120, 80, 40, 255], [206, 170, 90, 255], [150, 60, 30, 255], [180, 180, 175, 255]];

impl Look {
    /// `pigmentation` 0..1 from heritable traits; `family` keys the clothing colour.
    pub fn new(pigmentation: f32, family: u64, individual: u64, female: bool) -> Self {
        Self {
            skin: (pigmentation.clamp(0.0, 0.999) * 6.0) as u8,
            hair: (hash_unit(individual ^ 0x4A1) * HAIRS.len() as f32) as u8,
            shirt: (hash_unit(family ^ 0x5417) * SHIRTS.len() as f32) as u8,
            long_hair: female,
        }
    }
}

fn skin_colour(k: u8) -> Rgba {
    mix([244, 212, 178, 255], [92, 58, 38, 255], k as f32 / 5.0)
}

/// Poses: 0 and 2 stand, 1 and 3 stride (a walk cycle), 4 tool raised, 5 tool struck down.
pub const POSE_STAND: u8 = 0;
pub const POSE_WORK_UP: u8 = 4;
pub const POSE_WORK_DOWN: u8 = 5;

/// 12x18 person (the extra columns hold a raised or swung tool).
fn person(look: Look, frame: u8) -> Canvas {
    let mut c = Canvas::new(12, 18);
    let skin = skin_colour(look.skin);
    let hair = HAIRS[look.hair as usize % HAIRS.len()];
    let shirt = SHIRTS[look.shirt as usize % SHIRTS.len()];
    let pants = scale(mix(shirt, [70, 60, 50, 255], 0.7), 0.8);
    let working = frame >= POSE_WORK_UP;
    // Legs: stride on frames 1 and 3.
    let (l_off, r_off) = if working { (0, 0) } else { match frame % 4 { 1 => (-1, 1), 3 => (1, -1), _ => (0, 0) } };
    c.rect(3 + l_off.min(0), 12, 2, 5, pants);
    c.rect(5 + r_off.max(0), 12, 2, 5, pants);
    c.set(3 + l_off.min(0), 16, scale(pants, 0.6));
    c.set(6 + r_off.max(0), 16, scale(pants, 0.6));
    // Torso and arms (arms swing opposite the legs).
    c.rect(3, 6, 4, 6, shirt);
    c.rect(3, 6, 1, 6, scale(shirt, 0.82));
    if working {
        // Right arm works a hafted tool: raised overhead, then struck down in front.
        let wood = [128, 92, 56, 255];
        let stone = [150, 150, 158, 255];
        c.rect(2, 7, 1, 4, scale(shirt, 0.9));
        c.set(2, 11, skin);
        if frame == POSE_WORK_UP {
            c.rect(7, 3, 1, 4, scale(shirt, 0.9));
            c.set(7, 2, skin);
            c.rect(8, 0, 1, 3, wood);
            c.rect(8, 0, 3, 1, stone);
        } else {
            c.rect(7, 7, 1, 3, scale(shirt, 0.9));
            c.set(8, 10, skin);
            c.rect(9, 10, 2, 1, wood);
            c.rect(10, 11, 1, 3, stone);
        }
    } else {
        let arm_swing = match frame % 4 { 1 => 1, 3 => -1, _ => 0 };
        c.rect(2, 7 + arm_swing.max(0), 1, 4, scale(shirt, 0.9));
        c.rect(7, 7 + (-arm_swing).max(0), 1, 4, scale(shirt, 0.9));
        c.set(2, 11 + arm_swing.max(0), skin);
        c.set(7, 11 + (-arm_swing).max(0), skin);
    }
    // Head.
    c.rect(3, 1, 4, 5, skin);
    c.rect(3, 0, 4, 2, hair);
    c.set(3, 2, hair);
    if look.long_hair { c.rect(2, 1, 1, 6, hair); c.rect(7, 1, 1, 6, hair); c.set(6, 2, hair); }
    c.set(4, 3, [30, 24, 22, 255]);
    c.set(6, 3, [30, 24, 22, 255]);
    c.outline();
    c
}

// ---------- scenery ----------

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TreeKind { Broadleaf, Conifer, Jungle, Acacia, Cactus, Shrub }

impl TreeKind {
    pub fn for_biome(b: sim_core::terrain::Biome) -> Self {
        use sim_core::terrain::Biome::*;
        match b {
            Taiga | Tundra | Alpine | Snowcap | Ice => Self::Conifer,
            Rainforest | Swamp => Self::Jungle,
            Savanna => Self::Acacia,
            Desert | Beach => Self::Cactus,
            ColdSteppe => Self::Shrub,
            _ => Self::Broadleaf,
        }
    }
}

/// Leafy crown built from overlapping lit clumps.
fn crown(c: &mut Canvas, cx: f32, cy: f32, r: f32, base: Rgba, seed: u64) {
    let clumps = [(0.0, 0.0, 1.0), (-0.55, 0.25, 0.7), (0.55, 0.3, 0.72), (-0.2, -0.45, 0.66), (0.3, -0.35, 0.6)];
    for (k, &(dx, dy, s)) in clumps.iter().enumerate() {
        let shade = 0.84 + 0.08 * k as f32;
        c.blob(cx + dx * r, cy + dy * r, r * s, r * s * 0.9, scale(base, shade), seed ^ k as u64);
    }
    // Sun-catching highlights on the upper left.
    for k in 0..6u64 {
        let a = hash_unit(seed ^ (k * 97)) * std::f32::consts::PI + std::f32::consts::PI;
        let rr = r * 0.6 * hash_unit(seed ^ (k * 31));
        c.set((cx + a.cos() * rr) as i32, (cy + a.sin() * rr * 0.8) as i32, scale(base, 1.35));
    }
}

fn stump(c: &mut Canvas, cx: i32, base_y: i32) {
    let bark = [96, 64, 40, 255];
    c.rect(cx - 2, base_y - 4, 5, 4, bark);
    c.rect(cx - 1, base_y - 5, 3, 1, [176, 140, 96, 255]);
}

/// `fullness` 0..=4: how much harvestable growth is left (0 = stump).
fn tree(kind: TreeKind, fullness: u8, variant: u8) -> Canvas {
    let seed = variant as u64 * 977 + fullness as u64;
    let f = fullness as f32 / 4.0;
    let trunk = [98, 66, 40, 255];
    let bark_dark = [70, 46, 30, 255];
    match kind {
        TreeKind::Broadleaf | TreeKind::Jungle => {
            let big = kind == TreeKind::Jungle;
            let (w, h) = if big { (34, 48) } else { (28, 40) };
            let mut c = Canvas::new(w, h);
            let cx = w as i32 / 2;
            if fullness == 0 { stump(&mut c, cx, h as i32); c.outline(); return c; }
            let trunk_h = if big { 20 } else { 15 };
            c.rect(cx - 1, h as i32 - trunk_h, 3, trunk_h, trunk);
            c.rect(cx - 1, h as i32 - trunk_h, 1, trunk_h, bark_dark);
            c.rect(cx - 3, h as i32 - 2, 2, 2, trunk);
            c.rect(cx + 2, h as i32 - 2, 2, 2, trunk);
            let base = if big { [36, 112, 46, 255] } else { [68, 146, 58, 255] };
            let r = (if big { 13.0 } else { 10.5 }) * (0.55 + 0.45 * f);
            crown(&mut c, cx as f32 + 0.5, h as f32 - trunk_h as f32 - r * 0.55, r, base, seed);
            if big { for k in 0..5 { let x = cx - 6 + k * 3; c.rect(x, h as i32 - trunk_h - 2, 1, 5, [46, 96, 40, 255]); } }
            c.outline();
            c
        }
        TreeKind::Conifer => {
            let mut c = Canvas::new(20, 44);
            if fullness == 0 { stump(&mut c, 10, 44); c.outline(); return c; }
            c.rect(9, 36, 3, 8, trunk);
            let base = [42, 94, 66, 255];
            let tiers = 2 + fullness as i32;
            for k in 0..tiers {
                let bottom = 38 - k * 7;
                let half = 9 - k * 3 / 2;
                for dy in 0..9 {
                    let w = half * (dy + 1) / 9;
                    for dx in -w..=w {
                        let light = if dx < 0 { 1.12 } else { 0.86 };
                        c.set(10 + dx, bottom - 8 + dy, scale(base, light));
                    }
                }
                c.set(10 - half + 1, bottom, scale(base, 0.7));
            }
            c.outline();
            c
        }
        TreeKind::Acacia => {
            let mut c = Canvas::new(36, 30);
            if fullness == 0 { stump(&mut c, 18, 30); c.outline(); return c; }
            c.rect(17, 12, 2, 18, trunk);
            for k in 0..6 { c.set(16 - k, 12 - k / 2, trunk); c.set(19 + k, 13 - k / 2, trunk); }
            c.blob(18.0, 8.0, 7.0 + 9.0 * f, 3.0 + 2.5 * f, [130, 150, 62, 255], seed);
            c.blob(14.0, 6.5, 4.0 + 4.0 * f, 2.0 + 1.0 * f, [150, 168, 76, 255], seed ^ 3);
            c.outline();
            c
        }
        TreeKind::Cactus => {
            let mut c = Canvas::new(14, 24);
            let g = [84, 140, 70, 255];
            if fullness == 0 { c.rect(5, 18, 4, 6, scale(g, 0.6)); c.outline(); return c; }
            c.rect(5, 3, 4, 21, g);
            c.rect(5, 3, 1, 21, scale(g, 1.2));
            if fullness > 1 { c.rect(1, 10, 4, 3, g); c.rect(1, 5, 3, 6, g); c.rect(9, 12, 4, 3, g); c.rect(10, 7, 3, 6, g); }
            c.outline();
            c
        }
        TreeKind::Shrub => {
            let mut c = Canvas::new(20, 13);
            c.blob(10.0, 7.5, 3.0 + 6.0 * f.max(0.3), 2.5 + 3.0 * f.max(0.3), [108, 128, 68, 255], seed);
            c.outline();
            c
        }
    }
}

fn boulder(variant: u8) -> Canvas {
    let (w, h) = if variant % 2 == 0 { (30, 22) } else { (24, 18) };
    let mut c = Canvas::new(w, h);
    c.blob(w as f32 * 0.5, h as f32 * 0.6, w as f32 * 0.46, h as f32 * 0.4, [124, 120, 114, 255], variant as u64);
    c.blob(w as f32 * 0.6, h as f32 * 0.45, w as f32 * 0.26, h as f32 * 0.24, [150, 146, 140, 255], variant as u64 ^ 9);
    // Cracks.
    for k in 0..4 { c.set(w as i32 / 2 - 3 + k, h as i32 / 2 + k / 2, [84, 80, 76, 255]); }
    c.outline();
    c
}

fn stones(fullness: u8) -> Canvas {
    let mut c = Canvas::new(18, 10);
    let n = 1 + fullness as i32;
    for k in 0..n {
        let x = 3.0 + (k * 4 % 12) as f32;
        let y = 6.5 - (k / 3) as f32 * 2.0;
        c.blob(x + 1.0, y, 2.6, 1.9, [152, 134, 106, 255], k as u64);
    }
    c.outline();
    c
}

/// Isometric cottage, 52x46. `material` 0..3 picks mud-and-thatch, timber or
/// stone looks; `integrity` 0..=3 knocks holes in the roof as it decays.
fn house(material: u8, integrity: u8) -> Canvas {
    let mut c = Canvas::new(52, 46);
    let (wall, roof) = match material {
        0 => ([178, 142, 94, 255], [200, 170, 90, 255]),
        1 => ([152, 106, 66, 255], [150, 62, 42, 255]),
        _ => ([152, 148, 140, 255], [88, 98, 122, 255]),
    };
    // Footprint diamond: left corner (4,34), front (26,45), right (48,34), back (26,23).
    let wall_h = 14;
    for x in 4..=48i32 {
        let front = if x <= 26 { 34 + (x - 4) / 2 } else { 45 - (x - 26) / 2 };
        let face = if x <= 26 { scale(wall, 0.76) } else { wall };
        for y in (front - wall_h)..=front { c.set(x, y, face); }
        // Timber frame lines on the timber house.
        if material == 1 && (x % 6 == 0) { for y in (front - wall_h)..=front { c.set(x, y, scale(face, 0.7)); } }
    }
    if material == 2 { for y in (0..46).step_by(4) { for x in 4..=48i32 { if c.px[(y as usize) * 52 + x as usize][3] > 0 && (x + y) % 8 == 0 { c.set(x, y, scale(wall, 0.72)); } } } }
    // Door on the right face, window on the left.
    c.rect(33, 32, 4, 8, [72, 46, 30, 255]);
    c.rect(12, 27, 4, 4, [70, 90, 120, 255]);
    c.set(13, 27, [170, 200, 230, 255]);
    // Roof: gable ridge running back-left to front-right.
    for x in 2..=50i32 {
        let eave = if x <= 26 { 34 + (x - 4) / 2 - wall_h } else { 45 - (x - 26) / 2 - wall_h };
        let ridge = 6 + (x - 26).abs() / 4;
        let shade = if x <= 26 { scale(roof, 0.84) } else { roof };
        for y in ridge..=eave + 1 {
            let hole = integrity < 3 && hash_unit((x * 131 + y * 7) as u64 ^ integrity as u64) > 0.4 + integrity as f32 * 0.2;
            let stripe = (y - ridge) % 4 == 3;
            if !hole { c.set(x, y, if stripe { scale(shade, 0.85) } else { shade }); }
        }
    }
    if material == 2 { c.rect(38, 2, 4, 8, [110, 104, 98, 255]); }
    c.outline();
    c
}

/// Timber frame whose height tracks construction progress (0..=4), 52x40.
fn building_site(progress: u8) -> Canvas {
    let mut c = Canvas::new(52, 40);
    let wood = [160, 120, 72, 255];
    let dirt = [132, 104, 70, 255];
    for x in 4..=48i32 {
        let front = if x <= 26 { 30 + (x - 4) / 2 } else { 39 - (x - 26) / 2 };
        let back = if x <= 26 { 30 - (x - 4) / 2 } else { 19 + (x - 26) / 2 };
        c.set(x, front, dirt);
        c.set(x, back.max(0), dirt);
    }
    let h = 4 + progress as i32 * 4;
    for &(x, y) in &[(4, 30), (26, 19), (48, 30), (26, 39)] {
        for dy in 0..h { c.set(x, y - dy, wood); c.set(x + 1, y - dy, scale(wood, 0.8)); }
    }
    if progress >= 2 {
        for x in 4..=48i32 {
            let front = if x <= 26 { 30 + (x - 4) / 2 } else { 39 - (x - 26) / 2 };
            c.set(x, front - h, wood);
        }
    }
    // Stacked materials beside the plot.
    for k in 0..(1 + progress as i32).min(4) { c.rect(40 - k * 3, 33 - k, 6, 2, scale(wood, 0.9 + k as f32 * 0.05)); }
    c.outline();
    c
}

fn animal(variant: u8, dead: bool) -> Canvas {
    let mut c = Canvas::new(14, 10);
    let coat = [[150, 112, 70, 255], [120, 100, 84, 255], [190, 170, 130, 255], [90, 70, 50, 255]][variant as usize % 4];
    let coat = if dead { scale(coat, 0.55) } else { coat };
    c.blob(7.0, 4.5, 5.0, 2.8, coat, variant as u64);
    c.rect(10, 1, 3, 3, coat);
    c.set(12, 2, OUTLINE);
    for &x in &[3, 5, 8, 10] { c.rect(x, 6, 1, 3, scale(coat, 0.75)); }
    c.outline();
    c
}

/// 14x22 flame; `frame` 0..3 flickers the tongues.
fn flame(frame: u8) -> Canvas {
    let mut c = Canvas::new(14, 22);
    for y in 0..22i32 {
        let t = y as f32 / 21.0; // 0 at the tip, 1 at the base
        let sway = ((frame as f32 * 2.1 + y as f32 * 0.6).sin()) * (1.0 - t) * 2.2;
        let half = 0.6 + t * 5.4;
        for x in 0..14i32 {
            let d = (x as f32 + 0.5 - 7.0 - sway).abs();
            if d > half { continue; }
            let core = d < half * 0.45 && t > 0.35;
            let col = if core { [255, 236, 120, 255] } else if t < 0.4 { [255, 120, 40, 255] } else { [236, 76, 28, 255] };
            c.set(x, y, col);
        }
    }
    c
}

fn disc() -> Canvas {
    let mut c = Canvas::new(32, 32);
    for y in 0..32 { for x in 0..32 {
        let (dx, dy) = ((x as f32 + 0.5 - 16.0) / 16.0, (y as f32 + 0.5 - 16.0) / 16.0);
        let d = dx * dx + dy * dy;
        if d <= 1.0 { c.set(x, y, [255, 255, 255, (255.0 * (1.0 - d.powf(3.0))) as u8]); }
    }}
    c
}

// ---------- UI icons ----------

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Icon { Inspect, Person, Animal, Monster, Tree, Ore, Rock, Water, Rain, Sun, Fire, Flood, Quake, Pause }

/// 16x16 pixel-art icon for the god dock and speed controls.
fn icon(kind: Icon) -> Canvas {
    let mut c = Canvas::new(16, 16);
    match kind {
        Icon::Inspect => {
            for a in 0..32 { let t = a as f32 / 32.0 * std::f32::consts::TAU; c.set((6.5 + t.cos() * 4.5) as i32, (6.5 + t.sin() * 4.5) as i32, [220, 220, 230, 255]); }
            c.blob(6.5, 6.5, 3.2, 3.2, [120, 180, 230, 255], 3);
            for k in 0..5 { c.rect(10 + k, 10 + k, 2, 2, [150, 110, 70, 255]); }
        }
        Icon::Person => {
            let p = person(Look::new(0.35, 1, 5, false), 0);
            for y in 0..16 { for x in 0..12 { let v = p.px[(y + 1).min(17) * 12 + x]; if v[3] > 0 { c.set(x as i32 + 2, y as i32, v); } } }
            return c;
        }
        Icon::Animal => {
            let a = animal(0, false);
            for y in 0..10 { for x in 0..14 { let v = a.px[y * 14 + x]; if v[3] > 0 { c.set(x as i32 + 1, y as i32 + 4, v); } } }
            return c;
        }
        Icon::Monster => {
            c.blob(8.0, 9.0, 6.0, 5.5, [196, 50, 40, 255], 7);
            c.rect(3, 1, 2, 4, [230, 220, 200, 255]);
            c.rect(11, 1, 2, 4, [230, 220, 200, 255]);
            c.rect(5, 8, 2, 2, [250, 220, 60, 255]);
            c.rect(9, 8, 2, 2, [250, 220, 60, 255]);
            for x in 5..11 { c.set(x, 12, [250, 240, 230, 255]); }
        }
        Icon::Tree => {
            let t = tree(TreeKind::Broadleaf, 4, 0);
            // Downsample the 28x40 tree 2:1 into the icon.
            for y in 0..16 { for x in 0..14 { let v = t.px[(y * 2 + 8) * 28 + x * 2]; if v[3] > 0 { c.set(x as i32 + 1, y as i32, v); } } }
            return c;
        }
        Icon::Ore => {
            c.blob(8.0, 10.0, 6.5, 4.5, [110, 104, 98, 255], 11);
            c.rect(5, 8, 2, 2, [230, 190, 70, 255]);
            c.rect(9, 10, 2, 2, [230, 190, 70, 255]);
            c.rect(11, 3, 1, 3, [255, 245, 190, 255]);
            c.rect(10, 4, 3, 1, [255, 245, 190, 255]);
        }
        Icon::Rock => {
            for y in 3..15 { let half = (y - 3) / 2 + 1; for x in (8 - half)..(8 + half) { c.set(x, y, if x < 8 { [150, 144, 136, 255] } else { [112, 106, 100, 255] }); } }
            for y in 3..6 { let half = (y - 3) / 2 + 1; for x in (8 - half)..(8 + half) { c.set(x, y, [240, 244, 250, 255]); } }
        }
        Icon::Water => {
            for y in 2..15 { let r = if y < 8 { (y - 2) as f32 * 0.7 } else { 4.5 - ((y - 10) as f32).abs() * 0.4 }; for x in 0..16 { if ((x as f32 + 0.5) - 8.0).abs() < r { c.set(x, y, if x < 7 { [110, 180, 240, 255] } else { [60, 130, 210, 255] }); } } }
        }
        Icon::Rain => {
            c.blob(8.0, 5.0, 6.5, 3.5, [210, 216, 226, 255], 2);
            for &(x, y) in &[(4, 10), (8, 11), (12, 10), (6, 13), (10, 14)] { c.rect(x, y, 1, 2, [90, 150, 230, 255]); }
        }
        Icon::Sun => {
            c.blob(8.0, 8.0, 4.0, 4.0, [250, 200, 60, 255], 5);
            for &(x, y) in &[(8, 1), (8, 14), (1, 8), (14, 8), (3, 3), (12, 3), (3, 12), (12, 12)] { c.rect(x, y, 1, 1, [250, 170, 40, 255]); }
        }
        Icon::Fire => {
            for y in 2..15 { let w = ((y - 2) as f32 * 0.55).min(5.5) - if y > 11 { (y - 11) as f32 * 1.2 } else { 0.0 }; for x in 0..16 { let d = ((x as f32 + 0.5) - 8.0).abs(); if d < w { c.set(x, y, if d < w * 0.45 && y > 7 { [255, 230, 90, 255] } else { [236, 90, 30, 255] }); } } }
        }
        Icon::Flood => {
            for band in 0..3 { let y0 = 4 + band * 4; for x in 0..16 { let y = y0 + if (x / 3) % 2 == 0 { 0 } else { 1 }; c.rect(x, y, 1, 2, [70 + band as u8 * 30, 140, 220, 255]); } }
        }
        Icon::Quake => {
            c.rect(0, 9, 16, 6, [150, 116, 76, 255]);
            let mut x = 8; for y in 9..15 { c.set(x, y, [40, 30, 24, 255]); x += if y % 2 == 0 { 1 } else { -1 }; }
            c.rect(2, 4, 3, 3, [180, 150, 110, 255]);
            c.rect(11, 2, 2, 2, [180, 150, 110, 255]);
        }
        Icon::Pause => { c.rect(4, 3, 3, 10, [236, 232, 220, 255]); c.rect(9, 3, 3, 10, [236, 232, 220, 255]); }
    }
    c.outline();
    c
}

// ---------- cache ----------

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SpriteKey {
    Person(Look, u8),
    Tree(TreeKind, u8, u8),
    Boulder(u8),
    Stones(u8),
    House(u8, u8),
    Site(u8),
    Animal(u8, bool),
    Disc,
    Icon(Icon),
    Flame(u8),
}

#[derive(Resource, Default)]
pub struct SpriteBank(HashMap<SpriteKey, (Handle<Image>, Vec2)>);

impl SpriteBank {
    /// Cached image for a key and its natural size in view pixels.
    pub fn get(&mut self, key: SpriteKey, images: &mut Assets<Image>) -> (Handle<Image>, Vec2) {
        self.0.entry(key).or_insert_with(|| {
            let canvas = match key {
                SpriteKey::Person(look, frame) => person(look, frame),
                SpriteKey::Tree(kind, fullness, variant) => tree(kind, fullness, variant),
                SpriteKey::Boulder(v) => boulder(v),
                SpriteKey::Stones(f) => stones(f),
                SpriteKey::House(m, i) => house(m, i),
                SpriteKey::Site(p) => building_site(p),
                SpriteKey::Animal(v, dead) => animal(v, dead),
                SpriteKey::Disc => disc(),
                SpriteKey::Icon(i) => icon(i),
                SpriteKey::Flame(f) => flame(f),
            };
            let size = Vec2::new(canvas.w as f32, canvas.h as f32);
            (images.add(canvas.into_image()), size)
        }).clone()
    }
}

/// Quantise a 0..1 amount into 0..=4 steps (0 only when truly empty).
pub fn steps4(x: f32) -> u8 {
    if x <= 0.02 { 0 } else { (1.0 + x.clamp(0.0, 1.0) * 3.0).round() as u8 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opaque(c: &Canvas) -> usize { c.px.iter().filter(|p| p[3] == 255).count() }

    #[test]
    fn walk_frames_differ_and_stand_frames_match() {
        let look = Look::new(0.3, 7, 11, false);
        assert_eq!(person(look, 0).px, person(look, 2).px);
        assert_ne!(person(look, 1).px, person(look, 0).px);
        assert_ne!(person(look, 1).px, person(look, 3).px);
    }

    #[test]
    fn looks_follow_traits_and_family() {
        let light = Look::new(0.05, 1, 1, false);
        let dark = Look::new(0.95, 1, 1, false);
        assert_ne!(light.skin, dark.skin);
        // Same family, same clothes; long hair widens the silhouette.
        assert_eq!(Look::new(0.5, 42, 1, false).shirt, Look::new(0.5, 42, 2, true).shirt);
        assert!(opaque(&person(Look::new(0.5, 3, 3, true), 0)) > opaque(&person(Look::new(0.5, 3, 3, false), 0)));
    }

    #[test]
    fn harvested_trees_shrink_to_stumps() {
        for kind in [TreeKind::Broadleaf, TreeKind::Conifer, TreeKind::Jungle, TreeKind::Acacia] {
            assert!(opaque(&tree(kind, 4, 0)) > opaque(&tree(kind, 1, 0)), "{kind:?}");
            assert!(opaque(&tree(kind, 1, 0)) > opaque(&tree(kind, 0, 0)), "{kind:?}");
        }
    }

    #[test]
    fn decayed_roofs_have_holes() {
        assert!(opaque(&house(1, 3)) > opaque(&house(1, 0)));
    }
}
