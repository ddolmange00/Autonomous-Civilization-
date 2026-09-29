//! What the land around a tribe offers: places present and natural resources.
use super::catalog::{BitSet, Catalog, PlaceSet, ResourceId};
use super::schema::{BiomeTag, Place};
use crate::terrain::{Biome, Ground, TileMap};

pub fn biome_tag(b: Biome) -> BiomeTag {
    match b {
        Biome::Ocean => BiomeTag::Ocean,
        Biome::Freshwater => BiomeTag::Freshwater,
        Biome::Beach => BiomeTag::Beach,
        Biome::Ice => BiomeTag::Ice,
        Biome::Tundra => BiomeTag::Tundra,
        Biome::Taiga => BiomeTag::Taiga,
        Biome::ColdSteppe => BiomeTag::ColdSteppe,
        Biome::Grassland => BiomeTag::Grassland,
        Biome::TemperateForest => BiomeTag::TemperateForest,
        Biome::Swamp => BiomeTag::Swamp,
        Biome::Savanna => BiomeTag::Savanna,
        Biome::Desert => BiomeTag::Desert,
        Biome::Rainforest => BiomeTag::Rainforest,
        Biome::Alpine => BiomeTag::Alpine,
        Biome::Snowcap => BiomeTag::Snowcap,
    }
}

/// A biome counts as present when it covers at least this share of the area.
const BIOME_MIN_SHARE: f32 = 0.03;

/// Places within `radius` tiles of tile (cx, cy).
pub fn places_around(map: &TileMap, cx: usize, cy: usize, radius: usize) -> PlaceSet {
    let mut set = PlaceSet::default();
    let mut biome_count = [0u32; 15];
    let mut total = 0u32;
    let r2 = (radius * radius) as isize;
    let (x0, x1) = (cx.saturating_sub(radius), (cx + radius).min(map.width - 1));
    let (y0, y1) = (cy.saturating_sub(radius), (cy + radius).min(map.height - 1));
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (dx, dy) = (x as isize - cx as isize, y as isize - cy as isize);
            if dx * dx + dy * dy > r2 { continue; }
            let i = map.index(x, y);
            total += 1;
            biome_count[biome_tag(map.biome[i]) as usize] += 1;
            match map.ground[i] {
                Ground::DeepOcean | Ground::Ocean | Ground::Shallow => set.insert(Place::Coast),
                Ground::Lake => set.insert(Place::Lake),
                Ground::River => set.insert(Place::River),
                Ground::Hills => set.insert(Place::Hills),
                Ground::Mountain | Ground::Peak => set.insert(Place::Mountain),
                Ground::Beach | Ground::Lowland => {}
            }
            if map.river[i] > 0 { set.insert(Place::River); }
        }
    }
    for b in BiomeTag::ALL {
        if total > 0 && biome_count[b as usize] as f32 >= BIOME_MIN_SHARE * total as f32 {
            set.insert(Place::Biome(b));
        }
    }
    set
}

/// Stable key for a region: nearby sites share deposits, distant ones do not.
pub fn region_key(world_seed: u64, cx: usize, cy: usize) -> u64 {
    mix(world_seed ^ ((cx as u64 / 32) << 32 | cy as u64 / 32))
}

fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

pub(super) fn unit(key: u64, a: u64, b: u64) -> f32 {
    (mix(key ^ mix(a.wrapping_mul(0x1000_0000_01B3) ^ b)) >> 40) as f32 / (1u64 << 24) as f32
}

/// Natural resources present in a region: a source counts when its place is
/// present and the region's deterministic roll falls under its chance.
pub fn local_resources(catalog: &Catalog, places: PlaceSet, key: u64) -> BitSet {
    let mut set = BitSet::with_len(catalog.resources.len());
    for (ri, r) in catalog.resources.iter().enumerate() {
        let found = r.sources.iter().enumerate().any(|(si, s)| {
            places.contains(s.place) && (s.chance >= 1.0 || unit(key, ri as u64, si as u64) < s.chance)
        });
        if found { set.insert(ri as ResourceId); }
    }
    set
}
