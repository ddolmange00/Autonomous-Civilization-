use sim_core::{Civilization, World};
use sim_core::environment::EnvironmentCell;
use sim_core::monsters::{MonsterPopulation, MonsterSpecies};

fn main() {
    let apex = MonsterPopulation {
        species: MonsterSpecies {
            id: 1, body_mass_kg: 4_500.0, speed_m_s: 18.0, armor: 0.85,
            bite_force: 0.9, aggression: 0.95, intelligence: 0.4,
            reproduction_rate: 0.08, food_need_kg_day: 90.0,
        },
        count: 6.0, territory_food: 50_000.0,
    };
    let civ = Civilization { id: 1, population: 80.0, food_days: 40.0, defense_capacity: 12.0, resilience: 0.2, extinct: false };
    let mut world = World { year: 0.0, cells: Vec::<EnvironmentCell>::new(), civilizations: vec![civ], monsters: vec![apex] };
    for _ in 0..100 { world.step_year(); if world.civilizations[0].extinct { break; } }
    println!("year={} population={} extinct={}", world.year, world.civilizations[0].population, world.civilizations[0].extinct);
}
