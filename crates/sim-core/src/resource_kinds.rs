use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct VegetationArchetype {
    pub food_value:f32,
    pub fiber_value:f32,
    pub wood_value:f32,
    pub growth_rate:f32,
    pub water_demand:f32,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MineralArchetype {
    pub hardness_hint:f32,
    pub density_hint:f32,
    pub metallic_fraction:f32,
    pub oxide_fraction:f32,
    pub rarity:f32,
}
impl Default for VegetationArchetype {
    fn default()->Self {Self{food_value:0.2,fiber_value:0.4,wood_value:0.5,growth_rate:0.4,water_demand:0.5}}
}
impl Default for MineralArchetype {
    fn default()->Self {Self{hardness_hint:0.6,density_hint:0.6,metallic_fraction:0.15,oxide_fraction:0.4,rarity:0.5}}
}
