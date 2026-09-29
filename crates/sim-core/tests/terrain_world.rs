use sim_core::{
    affordances::FeatureKind,
    sandbox::{Preset, Sandbox},
    terrain::{Ground, MapSize, TileMap, WorldTemplate, CHUNK},
};

fn founded(template: WorldTemplate) -> Sandbox {
    Sandbox::on_terrain(847_291, Preset::Default, TileMap::generate(847_291, MapSize::Small, template))
}

#[test]
fn band_is_founded_on_land_with_features_from_real_tiles() {
    for template in WorldTemplate::ALL {
        let sim = founded(template);
        let t = sim.terrain.as_ref().unwrap();
        assert!(sim.residents.iter().all(|r| t.walkable_at(r.position)), "{template:?}");
        assert!(sim.features.iter().any(|f| f.kind == FeatureKind::Vegetation), "{template:?}");
        for f in &sim.features {
            let g = t.ground_at(f.position).unwrap();
            match f.kind {
                FeatureKind::DeepWater => assert!(g.is_water()),
                FeatureKind::RockFace => assert!(matches!(g, Ground::Mountain | Ground::Peak)),
                _ => assert!(g.walkable()),
            }
        }
    }
}

#[test]
fn walkers_stay_on_ground_unless_their_household_sails() {
    let mut sim = founded(WorldTemplate::Archipelago);
    while sim.year < 3.0 {
        sim.step(2.0);
        let t = sim.terrain.clone().unwrap();
        for r in sim.residents.iter().filter(|r| r.health > 0.0) {
            if t.walkable_at(r.position) {
                continue;
            }
            let migrating = r.life.kinship.household
                .and_then(|h| sim.households.iter().find(|x| x.id == h))
                .map(|h| h.migration_goal.is_some())
                .unwrap_or(false);
            assert!(migrating, "resident {} on {:?} without a voyage", r.id, t.ground_at(r.position));
        }
    }
}

#[test]
fn god_terrain_tools_change_tiles_and_mark_chunks() {
    let mut sim = founded(WorldTemplate::Continents);
    let p = sim.residents[0].position;
    let (x, y) = sim.terrain.as_ref().unwrap().tile_at(p).unwrap();
    let rev = sim.terrain.as_ref().unwrap().chunk_revision(x / CHUNK, y / CHUNK);
    sim.dig_water_at(p, 20.0);
    let t = sim.terrain.as_ref().unwrap();
    assert_eq!(t.ground_at(p), Some(Ground::Lake));
    assert_ne!(t.chunk_revision(x / CHUNK, y / CHUNK), rev);
    sim.raise_rock_at(p, 20.0);
    assert_eq!(sim.terrain.as_ref().unwrap().ground_at(p), Some(Ground::Mountain));
}

#[test]
fn features_materialise_where_people_go() {
    use sim_core::terrain::{Biome, TILE_SIZE};
    // Medium map: on a small one the founding area already covers most land.
    let mut sim = Sandbox::on_terrain(847_291, Preset::Default, TileMap::generate(847_291, MapSize::Medium, WorldTemplate::Pangaea));
    let t = sim.terrain.clone().unwrap();
    let reach = CHUNK as f32 * TILE_SIZE;
    // A forest tile outside the founding band's materialised area.
    let far = (0..t.biome.len())
        .filter(|&i| t.biome[i] == Biome::TemperateForest || t.biome[i] == Biome::Rainforest)
        .map(|i| t.tile_center(i % t.width, i / t.width))
        .find(|p| !sim.features.iter().any(|f| f.position.distance(*p) < reach))
        .expect("unexplored forest");
    sim.spawn_resident_at(far);
    sim.step(1.0);
    assert!(sim.features.iter().any(|f| f.kind == FeatureKind::Vegetation && f.position.distance(far) < reach));
}

#[test]
fn a_set_fire_spreads_and_nearby_residents_notice_it() {
    use sim_core::awareness::SituationKind;
    let mut sim = founded(WorldTemplate::Continents);
    let t = sim.terrain.clone().unwrap();
    // Light the most fuel-rich ground next to the band.
    let home = sim.residents[0].position;
    let spot = (0..t.biome.len())
        .filter(|&i| sim_core::terrain::hazards::base_fuel(t.biome[i]) >= 0.7)
        .map(|i| t.tile_center(i % t.width, i / t.width))
        .min_by(|a, b| a.distance(home).total_cmp(&b.distance(home)))
        .unwrap();
    let lit = sim.ignite_at(spot, 12.0, 1.0);
    assert!(lit > 0);
    for _ in 0..8 { sim.step(0.5); }
    let hz = sim.hazards.as_ref().unwrap();
    let burnt = hz.scorch.iter().filter(|&&s| s > 0.0).count();
    assert!(burnt > lit, "fire did not spread: {burnt} scorched vs {lit} lit");
    assert!(sim.residents.iter().any(|r| r.awareness.reports.contains_key(&SituationKind::Fire)), "nobody noticed the fire");
}

#[test]
fn a_flood_on_the_band_is_noticed_and_deep_water_leaves_people() {
    use sim_core::awareness::SituationKind;
    let mut sim = founded(WorldTemplate::Continents);
    let centre = sim.residents[0].position;
    sim.flood_at(centre, 30.0, 1.0);
    let t = sim.terrain.clone().unwrap();
    let deep = |sim: &Sandbox| sim.residents.iter().filter(|r| r.health > 0.0 && sim.hazards.as_ref().unwrap().water_at(&t, r.position) > 0.7).count();
    let caught = deep(&sim);
    assert!(caught > 0, "the flood missed the band");
    sim.step(0.5);
    assert!(sim.residents.iter().any(|r| r.awareness.reports.contains_key(&SituationKind::Flood)));
    for _ in 0..10 { sim.step(0.5); }
    assert!(deep(&sim) < caught, "nobody got out of deep water ({caught} caught)");
}
