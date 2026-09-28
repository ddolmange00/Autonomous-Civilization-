use sim_core::sandbox::Sandbox;

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
fn settlements_carry_founding_lexicons() {
    let sim = run_years(847291, 10.0);
    let active: Vec<_> = sim.settlements.iter().filter(|s| !s.members.is_empty()).collect();
    assert!(!active.is_empty());
    for s in active {
        assert_eq!(s.lexicon.words.len(), sim_core::language::CONCEPTS as usize);
    }
}

#[test]
fn society_ledgers_stay_coherent() {
    let sim = run_years(847291, 20.0);
    for c in &sim.contacts {
        assert!(c.a != c.b);
        assert!(c.trust >= -1.0 && c.trust <= 1.0);
    }
    for i in &sim.institutions {
        assert!(i.strength > 0.0 && i.strength <= 1.0);
    }
    for n in &sim.narratives {
        assert!(!n.label.is_empty());
    }
    assert!(sim.trade_count + sim.theft_count < u64::MAX);
}
