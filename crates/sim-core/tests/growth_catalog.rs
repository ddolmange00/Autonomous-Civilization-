//! The shipped growth content: it must compile cleanly, cover every domain,
//! and give regions real character (no land reaches everything alone; trade
//! and contact open what the land withholds).
use sim_core::growth::{
    self,
    region::{local_resources, places_around, region_key},
    schema::{BiomeTag, Domain, Place},
    BitSet, Catalog, PlaceSet,
};
use sim_core::terrain::{Ground, MapSize, TileMap, WorldTemplate};
use std::collections::BTreeSet;

fn catalog() -> Catalog {
    match growth::builtin() {
        Ok(c) => c,
        Err(errors) => panic!("growth content has {} errors:\n{}", errors.len(),
            errors.iter().map(|e| format!("  {e}")).collect::<Vec<_>>().join("\n")),
    }
}

/// Resources a place set could hold in the luckiest region.
fn best_case_resources(c: &Catalog, places: PlaceSet) -> BitSet {
    let mut s = BitSet::default();
    for (i, r) in c.resources.iter().enumerate() {
        if r.sources.iter().any(|src| places.contains(src.place)) { s.insert(i as u16); }
    }
    s
}

#[test]
fn builtin_content_compiles_and_covers_every_domain() {
    let c = catalog();
    for d in Domain::ALL {
        let n = c.nodes.iter().filter(|n| n.domain == d).count();
        assert!(n >= 5, "domain {d:?} has only {n} nodes");
    }
    eprintln!("growth content: {} nodes, {} unlocks, {} resources", c.nodes.len(), c.unlocks.len(), c.resources.len());
}

#[test]
fn no_single_biome_reaches_everything_and_every_node_is_reachable_somewhere() {
    let c = catalog();
    let flags = [Place::Coast, Place::River, Place::Lake, Place::Hills, Place::Mountain];
    let mut union = BitSet::default();
    let mut distinct = BTreeSet::new();
    for b in BiomeTag::ALL {
        // A rich region: the biome plus every landform, best-case deposits.
        let mut places = PlaceSet::of(&flags);
        places.insert(Place::Biome(b));
        let (known, _) = c.closure(places, &best_case_resources(&c, places), &BitSet::default());
        assert!(known.len() < c.nodes.len(), "{b:?} reaches every node without contact");
        union.union_with(&known);
        // The bare biome alone, to measure regional character.
        let bare = PlaceSet::of(&[Place::Biome(b)]);
        let (bare_known, _) = c.closure(bare, &best_case_resources(&c, bare), &BitSet::default());
        distinct.insert(bare_known.iter().collect::<Vec<_>>());
    }
    let missing: Vec<&str> = (0..c.nodes.len() as u16).filter(|&n| !union.contains(n)).map(|n| c.nodes[n as usize].id.as_str()).collect();
    assert!(missing.is_empty(), "nodes no single region can reach on its own: {missing:?}");
    assert!(distinct.len() >= 12, "only {} distinct bare-biome trees out of 15", distinct.len());
}

#[test]
fn generated_worlds_give_sites_different_trees_and_trade_widens_them() {
    let c = catalog();
    let mut trees = BTreeSet::new();
    let mut trade_helped = 0;
    for (seed, template) in [(11u64, WorldTemplate::Continents), (12, WorldTemplate::Archipelago), (13, WorldTemplate::Pangaea)] {
        let map = TileMap::generate(seed, MapSize::Small, template);
        // Land sites on a coarse grid.
        let mut sites = Vec::new();
        for gy in (16..map.height - 16).step_by(40) {
            for gx in (16..map.width - 16).step_by(40) {
                if map.ground[map.index(gx, gy)].walkable() && map.ground[map.index(gx, gy)] != Ground::River { sites.push((gx, gy)); }
            }
        }
        assert!(sites.len() >= 4, "{template:?} seed {seed}: too few land sites");
        let regions: Vec<(PlaceSet, BitSet)> = sites.iter().map(|&(x, y)| {
            let places = places_around(&map, x, y, 24);
            (places, local_resources(&c, places, region_key(map.seed, x, y)))
        }).collect();
        for (i, (places, local)) in regions.iter().enumerate() {
            let (alone, _) = c.closure(*places, local, &BitSet::default());
            trees.insert(alone.iter().collect::<Vec<_>>());
            // Trading with the next site: its raw resources arrive.
            let (_, other) = &regions[(i + 1) % regions.len()];
            let mut raw = local.clone();
            raw.union_with(other);
            let (traded, _) = c.closure(*places, &raw, &BitSet::default());
            assert!(traded.len() >= alone.len());
            if traded.len() > alone.len() { trade_helped += 1; }
        }
    }
    assert!(trees.len() >= 5, "only {} distinct trees across generated sites", trees.len());
    assert!(trade_helped >= 3, "trade widened only {trade_helped} sites' trees");
}
