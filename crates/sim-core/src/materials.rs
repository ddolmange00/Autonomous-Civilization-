use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MaterialProperties {
    pub density: f32,
    pub tensile_strength: f32,
    pub compressive_strength: f32,
    pub fracture_toughness: f32,
    pub hardness: f32,
    pub elastic_modulus: f32,
    pub flexibility: f32,
    pub friction: f32,
    pub thermal_conductivity: f32,
    pub ignition_temperature: f32,
    pub corrosion_resistance: f32,
    pub water_absorption: f32,
    pub permeability: f32,
    pub buoyancy_factor: f32,
    pub workability: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Material {
    pub id: u32,
    pub name: String,
    pub truth: MaterialProperties,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Belief {
    pub mean: f32,
    pub uncertainty: f32,
    pub observations: u32,
}

impl Belief {
    pub fn observe(&mut self, evidence: f32, reliability: f32) {
        let w = reliability.clamp(0.01, 1.0);
        let prior = if self.observations == 0 { evidence } else { self.mean };
        self.mean = prior * (1.0 - w) + evidence * w;
        self.uncertainty = (self.uncertainty.max(0.05) * (1.0 - 0.35 * w)).max(0.01);
        self.observations += 1;
    }
}
