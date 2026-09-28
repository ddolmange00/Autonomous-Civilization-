use sim_core::{
    lod::{represented_population, Cohort},
    sandbox::{Preset, Sandbox},
};

fn run_years(seed: u64, years: f64, preset: Preset) -> Sandbox {
    let mut sim = Sandbox::with_preset(seed, preset);
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
fn cohorts_and_eras_track_long_runs() {
    let sim = run_years(847291, 30.0, Preset::Default);
    let alive = sim.residents.iter().filter(|r| r.health > 0.0).count();
    let represented = represented_population(alive, &sim.cohorts);
    assert!(represented >= alive as f64);
    assert!(!sim.eras.is_empty());
    assert!(!sim.civilizations.is_empty());
    let largest = sim.civilizations.iter().map(|c| c.settlement_ids.len()).max().unwrap_or(0);
    assert!(largest >= 1);
}

#[test]
fn arid_preset_starves_faster_than_rich() {
    let arid = run_years(847291, 20.0, Preset::Arid);
    let rich = run_years(847291, 20.0, Preset::Rich);
    let alive_arid = arid.residents.iter().filter(|r| r.health > 0.0).count();
    let alive_rich = rich.residents.iter().filter(|r| r.health > 0.0).count();
    assert!(alive_rich >= alive_arid);
}

#[test]
fn cohort_counts_stay_finite() {
    let sim = run_years(847292, 30.0, Preset::Default);
    for c in &sim.cohorts {
        assert!(c.count.is_finite() && c.count >= 0.0);
    }
    let _: Vec<Cohort> = sim.cohorts.clone();
}
