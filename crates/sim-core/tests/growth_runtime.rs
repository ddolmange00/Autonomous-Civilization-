//! Growth trees in the running world: knowledge lives in residents, founding
//! bands carry the basics, tribes discover on their own, acceleration makes a
//! tribe outpace itself, and knowledge dies with its last carrier.
use sim_core::{
    causal_log::CausalNode,
    growth::runtime::{key, KEY_PREFIX},
    sandbox::{Preset, Sandbox},
    terrain::{MapSize, TileMap, WorldTemplate},
};

fn world(seed: u64) -> Sandbox {
    Sandbox::on_terrain(seed, Preset::Default, TileMap::generate(seed, MapSize::Small, WorldTemplate::Continents))
}

fn run_years(sim: &mut Sandbox, years: f64) {
    let end = sim.year + years;
    while sim.year < end { sim.step(2.0); }
}

fn discoveries(sim: &Sandbox) -> u64 { sim.growth.acquired.0 }

#[test]
fn founding_bands_carry_the_basics_and_the_tribe_reads_them_from_people() {
    let mut sim = world(847_291);
    run_years(&mut sim, 2.0);
    assert!(!sim.growth.tribes.is_empty(), "no settlement became a tribe");
    let c = sim.growth.catalog.clone();
    let fire = c.node("fire").unwrap();
    for (sid, tg) in &sim.growth.tribes {
        assert!(tg.tribe.known.contains(fire), "tribe {sid} lacks fire");
        let s = sim.settlements.iter().find(|s| s.id == *sid).unwrap();
        let carriers = s.members.iter()
            .filter_map(|id| sim.residents.iter().find(|r| r.id == *id))
            .filter(|r| r.health > 0.0 && r.knowledge.items.contains_key(&key(&c, fire)))
            .count();
        assert!(carriers > 0, "tribe {sid} knows fire but nobody carries it");
        assert_eq!(sim.growth.era(*sid), Some("stone-using"));
    }
}

#[test]
fn tribes_discover_on_their_own_and_acceleration_outpaces_it() {
    let mut calm = world(4_242);
    let mut rushed = world(4_242);
    run_years(&mut calm, 1.0);
    run_years(&mut rushed, 1.0);
    for id in rushed.growth.tribes.keys().copied().collect::<Vec<_>>() { rushed.growth.set_acceleration(id, 10.0); }
    // The causal log compacts old entries, so look for discovery records as they are written.
    let mut logged = false;
    while calm.year < 31.0 {
        calm.step(2.0);
        logged |= calm.causal_log.nodes.iter().any(|(_, n)| matches!(n, CausalNode::Outcome { label, .. } if label.starts_with("discovered")));
    }
    run_years(&mut rushed, 30.0);
    let (a, b) = (discoveries(&calm), discoveries(&rushed));
    eprintln!("30 years: {a} discoveries unassisted, {b} with x10 acceleration; tribes {} vs {}, residents alive {} vs {}",
        calm.growth.tribes.len(), rushed.growth.tribes.len(),
        calm.residents.iter().filter(|r| r.health > 0.0).count(), rushed.residents.iter().filter(|r| r.health > 0.0).count());
    assert!(a > 0, "no tribe discovered anything in 30 years");
    assert!(b > a * 2, "acceleration x10 gave {b} vs {a}");
    assert!(logged, "discoveries never reached the causal log");
}

#[test]
fn knowledge_dies_with_its_last_carrier() {
    let mut sim = world(847_291);
    run_years(&mut sim, 2.0);
    let c = sim.growth.catalog.clone();
    let herbs = key(&c, c.node("herbalism").unwrap());
    // One resident alone learns herbalism, then dies.
    let sid = *sim.growth.tribes.keys().next().unwrap();
    let s = sim.settlements.iter().find(|s| s.id == sid).unwrap().clone();
    for r in sim.residents.iter_mut().filter(|r| s.members.contains(&r.id)) { r.knowledge.items.remove(&herbs); }
    let lone = sim.residents.iter().position(|r| r.id == s.members[0]).unwrap();
    sim.residents[lone].knowledge.learn(herbs.clone(), 1.0, 0.9);
    sim.residents[lone].health = 0.0;
    run_years(&mut sim, 0.5);
    let herbalism = c.node("herbalism").unwrap();
    let still_carried = sim.residents.iter().any(|r| r.health > 0.0 && r.knowledge.items.get(&herbs).is_some_and(|k| k.confidence >= 0.05));
    let tribe_knows = sim.growth.tribes.get(&sid).is_some_and(|t| t.tribe.known.contains(herbalism));
    assert_eq!(tribe_knows, still_carried, "tribe knowledge must follow living carriers");
    assert!(sim.residents.iter().all(|r| r.health <= 0.0 || r.knowledge.items.keys().all(|k| !k.starts_with(KEY_PREFIX) || c.node(&k[KEY_PREFIX.len()..]).is_some())));
}
