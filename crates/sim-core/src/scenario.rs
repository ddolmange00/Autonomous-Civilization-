use crate::world::World;

#[derive(Clone, Debug)]
pub struct ScenarioResult {
    pub years: u32,
    pub surviving_civilizations: usize,
    pub extinct_civilizations: usize,
    pub total_population: f64,
}

pub fn run(mut world: World, max_years: u32) -> ScenarioResult {
    let mut years = 0;
    for _ in 0..max_years {
        world.step_year();
        years += 1;
        if world.civilizations.iter().all(|c| c.extinct) { break; }
    }
    let surviving = world.civilizations.iter().filter(|c| !c.extinct).count();
    ScenarioResult {
        years,
        surviving_civilizations: surviving,
        extinct_civilizations: world.civilizations.len() - surviving,
        total_population: world.civilizations.iter().map(|c| c.population).sum(),
    }
}
