use crate::{environment::EnvironmentCell, monsters::MonsterPopulation};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Civilization {
    pub id: u32,
    pub population: f64,
    pub food_days: f32,
    pub defense_capacity: f32,
    pub resilience: f32,
    pub extinct: bool,
}

impl Civilization {
    pub fn apply_monster_pressure(&mut self, threat: f32, years: f32) {
        if self.extinct { return; }
        let unresolved = (threat - self.defense_capacity).max(0.0);
        let mortality = (unresolved / (100.0 + unresolved) * years).clamp(0.0, 0.95);
        self.population *= 1.0 - mortality as f64;
        self.food_days = (self.food_days - unresolved * 0.015 * years).max(0.0);
        if self.population < 2.0 { self.population = 0.0; self.extinct = true; }
    }
}

#[derive(Clone, Debug)]
pub struct World {
    pub year: f64,
    pub cells: Vec<EnvironmentCell>,
    pub civilizations: Vec<Civilization>,
    pub monsters: Vec<MonsterPopulation>,
}

impl World {
    pub fn step_year(&mut self) {
        self.year += 1.0;
        for m in &mut self.monsters { m.step_year(); }
        let total_threat: f32 = self.monsters.iter().map(MonsterPopulation::threat).sum();
        for civ in &mut self.civilizations { civ.apply_monster_pressure(total_threat, 1.0); }
    }
}
