use crate::materials::MaterialProperties;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Function { Cut, Impact, Pierce, Float, Climb, Shelter }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DesignGenome {
    pub id: u64,
    pub parent: Option<u64>,
    pub generation: u32,
    pub function: Function,
    pub length_m: f32,
    pub width_m: f32,
    pub thickness_m: f32,
    pub curvature: f32,
    pub edge_fraction: f32,
    pub binding_quality: f32,
    pub material_fraction: Vec<(u32, f32)>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Performance {
    pub structural: f32,
    pub cutting: f32,
    pub impact: f32,
    pub piercing: f32,
    pub flotation: f32,
    pub climbing: f32,
}

pub fn evaluate(g: &DesignGenome, p: MaterialProperties) -> Performance {
    let volume = (g.length_m * g.width_m * g.thickness_m).max(0.0001);
    let mass = volume * p.density.max(0.01);
    let structural = (p.fracture_toughness * 0.35 + p.tensile_strength * 0.3 + p.compressive_strength * 0.2 + p.elastic_modulus * 0.15)
        * (0.6 + g.binding_quality * 0.4);
    Performance {
        structural,
        cutting: p.hardness * g.edge_fraction * p.fracture_toughness.sqrt(),
        impact: mass.sqrt() * p.fracture_toughness * (0.5 + g.length_m),
        piercing: p.hardness * (0.4 + g.length_m) / g.width_m.max(0.03),
        flotation: p.buoyancy_factor * volume / mass.max(0.01) * (0.5 + g.width_m),
        climbing: p.friction * g.binding_quality * (0.4 + p.flexibility),
    }
}
