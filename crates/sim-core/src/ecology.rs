use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ResourcePatch {
    pub biomass: f32,
    pub regeneration_per_year: f32,
    pub carrying_capacity: f32,
}

impl ResourcePatch {
    pub fn harvest(&mut self, requested: f32) -> f32 {
        let taken = requested.max(0.0).min(self.biomass);
        self.biomass -= taken;
        taken
    }
    pub fn step_year(&mut self) {
        let room = (1.0 - self.biomass / self.carrying_capacity.max(0.001)).clamp(0.0, 1.0);
        self.biomass = (self.biomass + self.regeneration_per_year * room).min(self.carrying_capacity);
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FoodWeb {
    pub plants: ResourcePatch,
    pub prey: f32,
    pub predators: f32,
}

impl FoodWeb {
    pub fn step_year(&mut self) {
        self.plants.step_year();
        let prey_capacity = self.plants.biomass.max(1.0) * 0.35;
        self.prey = (self.prey + self.prey * 0.18 * (1.0 - self.prey / prey_capacity) - self.predators * 0.06).max(0.0);
        let predator_capacity = self.prey.max(1.0) * 0.12;
        self.predators = (self.predators + self.predators * 0.10 * (1.0 - self.predators / predator_capacity)).max(0.0);
    }
}
