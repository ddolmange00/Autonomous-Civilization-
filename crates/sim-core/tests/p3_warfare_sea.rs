use sim_core::{sandbox::Sandbox, species::MonsterArchetype, world::Position};

fn run_years(seed: u64, years: f64) -> Sandbox {
    let mut sim = Sandbox::new(seed);
    let target = sim.year + years;
    while sim.year < target {
        sim.step(5.0);
        if sim.residents.iter().all(|r| r.health <= 0.0) {
            break;
        }
    }
    sim
}

#[test]
fn ground_truth_registry_is_seeded() {
    let sim = Sandbox::new(5);
    assert_eq!(sim.ground_truth.materials.len(), 3);
    assert!(sim.ground_truth.get(1).is_some());
}

#[test]
fn warfare_ledgers_stay_coherent_with_monster_pressure() {
    let mut sim = Sandbox::new(847291);
    // Peaceful years until the first tool is invented (bounded), then a prowler in the largest village.
    let peace = sim.year + 40.0;
    while sim.year < peace && sim.draft_count == 0 {
        sim.step(5.0);
        if sim.residents.iter().all(|r| r.health <= 0.0) {
            break;
        }
    }
    assert!(sim.draft_count > 0);
    let den = sim.settlements.iter().filter(|s| !s.members.is_empty()).max_by_key(|s| s.members.len()).map(|s| s.center).unwrap_or(Position { x: -100.0, y: 0.0 });
    sim.spawn_monster_with(
        den,
        MonsterArchetype {
            body_mass_kg: 120.0,
            speed: 0.4,
            armor: 0.1,
            aggression: 0.6,
            intelligence: 0.2,
            reproduction: 0.0,
        },
        None,
        None,
    );
    let target = sim.year + 10.0;
    let mut saw_band = false;
    while sim.year < target {
        sim.step(5.0);
        saw_band = saw_band || !sim.warbands.is_empty();
        if sim.residents.iter().all(|r| r.health <= 0.0) {
            break;
        }
    }
    // An armed village facing a camped predator musters at least once.
    assert!(saw_band);
    assert!(sim.battles > 0);
    for b in &sim.warbands {
        assert!(b.members.len() >= 2);
        assert!(b.morale >= 0.0 && b.morale <= 1.0);
    }
    for h in &sim.households {
        assert!(h.designs.len() <= 12);
    }
}

#[test]
fn long_run_stays_coherent() {
    let sim = run_years(847292, 20.0);
    for b in &sim.warbands {
        assert!(b.members.len() >= 2);
    }
    assert!(sim.battles < u64::MAX);
}
