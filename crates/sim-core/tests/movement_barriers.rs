use sim_core::{affordances::FeatureKind,sandbox::Sandbox};

#[test]
fn sandbox_contains_real_deep_water_barrier_features() {
    let s=Sandbox::new(7);
    assert!(s.features.iter().any(|f|f.kind==FeatureKind::DeepWater));
}

#[test]
fn migration_candidates_do_not_delete_barriers() {
    let s=Sandbox::new(11);
    let water=s.features.iter().filter(|f|f.kind==FeatureKind::DeepWater).count();
    assert!(water>5);
}
