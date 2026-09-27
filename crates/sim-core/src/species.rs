use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Diet { Herbivore, Omnivore, Carnivore }

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct AnimalArchetype {
    pub body_mass_kg:f32,
    pub speed:f32,
    pub fear:f32,
    pub aggression:f32,
    pub reproduction:f32,
    pub diet:Diet,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MonsterArchetype {
    pub body_mass_kg:f32,
    pub speed:f32,
    pub armor:f32,
    pub aggression:f32,
    pub intelligence:f32,
    pub reproduction:f32,
}

impl Default for AnimalArchetype {
    fn default()->Self {Self{body_mass_kg:45.0,speed:0.55,fear:0.7,aggression:0.08,reproduction:0.25,diet:Diet::Herbivore}}
}
impl Default for MonsterArchetype {
    fn default()->Self {Self{body_mass_kg:900.0,speed:0.55,armor:0.35,aggression:0.75,intelligence:0.35,reproduction:0.08}}
}
