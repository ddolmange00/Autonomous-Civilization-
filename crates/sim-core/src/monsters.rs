use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonsterSpecies {
    pub id: u32,
    pub body_mass_kg: f32,
    pub speed_m_s: f32,
    pub armor: f32,
    pub bite_force: f32,
    pub aggression: f32,
    pub intelligence: f32,
    pub reproduction_rate: f32,
    pub food_need_kg_day: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonsterPopulation {
    pub species: MonsterSpecies,
    pub count: f32,
    pub territory_food: f32,
}

impl MonsterPopulation {
    pub fn step_year(&mut self) {
        let carrying = (self.territory_food / self.species.food_need_kg_day.max(0.1)).max(1.0);
        let pressure = 1.0 - self.count / carrying;
        self.count = (self.count + self.count * self.species.reproduction_rate * pressure).max(0.0);
    }
    pub fn threat(&self) -> f32 {
        self.count * self.species.body_mass_kg.sqrt() * (0.4 + self.species.aggression) * (1.0 + self.species.armor)
    }
}
