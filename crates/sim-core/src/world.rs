use crate::{environment::EnvironmentCell, monsters::MonsterPopulation};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Position { pub x: f32, pub y: f32 }

impl Position {
    pub fn distance(self, other: Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Civilization {
    pub id: u32,
    pub position: Position,
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
pub struct LocatedMonsterPopulation {
    pub position: Position,
    pub influence_radius: f32,
    pub population: MonsterPopulation,
}

#[derive(Clone, Debug)]
pub struct World {
    pub year: f64,
    pub cells: Vec<EnvironmentCell>,
    pub civilizations: Vec<Civilization>,
    pub monsters: Vec<LocatedMonsterPopulation>,
}

impl World {
    pub fn step_year(&mut self) {
        self.year += 1.0;
        for m in &mut self.monsters { m.population.step_year(); }
        for civ in &mut self.civilizations {
            let local_threat: f32 = self.monsters.iter()
                .filter(|m| civ.position.distance(m.position) <= m.influence_radius)
                .map(|m| {
                    let d = civ.position.distance(m.position);
                    let falloff = 1.0 - (d / m.influence_radius.max(0.001)).clamp(0.0, 1.0);
                    m.population.threat() * falloff
                })
                .sum();
            civ.apply_monster_pressure(local_threat, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monsters::{MonsterPopulation, MonsterSpecies};

    fn monster() -> LocatedMonsterPopulation {
        LocatedMonsterPopulation {
            position: Position { x: 0.0, y: 0.0 },
            influence_radius: 20.0,
            population: MonsterPopulation {
                species: MonsterSpecies {
                    id: 1, body_mass_kg: 4500.0, speed_m_s: 18.0, armor: 0.85,
                    bite_force: 0.9, aggression: 0.95, intelligence: 0.4,
                    reproduction_rate: 0.0, food_need_kg_day: 90.0,
                },
                count: 6.0, territory_food: 50_000.0,
            },
        }
    }

    #[test]
    fn monster_pressure_is_local_not_global() {
        let near = Civilization { id: 1, position: Position{x: 1.0,y: 0.0}, population: 80.0, food_days: 40.0, defense_capacity: 0.0, resilience: 0.0, extinct: false };
        let far = Civilization { id: 2, position: Position{x: 200.0,y: 0.0}, population: 80.0, food_days: 40.0, defense_capacity: 0.0, resilience: 0.0, extinct: false };
        let mut world = World { year: 0.0, cells: vec![], civilizations: vec![near, far], monsters: vec![monster()] };
        world.step_year();
        assert!(world.civilizations[0].population < 80.0);
        assert_eq!(world.civilizations[1].population, 80.0);
    }
}
