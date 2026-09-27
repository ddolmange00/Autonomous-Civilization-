use crate::materials::MaterialProperties;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Composition {
    pub metallic: f32,
    pub silicate: f32,
    pub carbon: f32,
    pub sulfur: f32,
    pub oxide: f32,
    pub volatile: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ThermalProcess {
    pub peak_temperature_c: f32,
    pub hours: f32,
    pub reducing_strength: f32,
    pub flux_quality: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ProcessYield {
    pub recovered_fraction: f32,
    pub impurity_fraction: f32,
    pub properties: MaterialProperties,
}

pub fn smelt(ore: Composition, process: ThermalProcess, base: MaterialProperties) -> ProcessYield {
    let heat = ((process.peak_temperature_c - 450.0) / 1000.0).clamp(0.0, 1.0);
    let reduction = process.reducing_strength.clamp(0.0, 1.0);
    let flux = process.flux_quality.clamp(0.0, 1.0);
    let oxide_penalty = ore.oxide * (1.0 - reduction);
    let sulfur_penalty = ore.sulfur * (1.0 - 0.55 * flux);
    let recovered = (ore.metallic * heat * (0.35 + 0.65 * reduction) * (0.7 + 0.3 * flux)).clamp(0.0, 1.0);
    let impurity = (ore.silicate * (1.0 - flux * 0.7) + oxide_penalty + sulfur_penalty).clamp(0.0, 1.0);
    let quality = (recovered * (1.0 - impurity * 0.75)).clamp(0.02, 1.0);
    let mut p = base;
    p.tensile_strength *= 0.35 + 0.65 * quality;
    p.fracture_toughness *= 0.30 + 0.70 * quality;
    p.hardness *= 0.45 + 0.55 * quality;
    p.workability *= (0.45 + flux * 0.35 + heat * 0.20).clamp(0.1, 1.4);
    ProcessYield { recovered_fraction: recovered, impurity_fraction: impurity, properties: p }
}

pub fn blend(a: MaterialProperties, b: MaterialProperties, fraction_b: f32) -> MaterialProperties {
    let x = fraction_b.clamp(0.0, 1.0);
    let mix = |u:f32,v:f32| u*(1.0-x)+v*x;
    MaterialProperties {
        density: mix(a.density,b.density), tensile_strength: mix(a.tensile_strength,b.tensile_strength),
        compressive_strength: mix(a.compressive_strength,b.compressive_strength),
        fracture_toughness: mix(a.fracture_toughness,b.fracture_toughness),
        hardness: mix(a.hardness,b.hardness), elastic_modulus: mix(a.elastic_modulus,b.elastic_modulus),
        flexibility: mix(a.flexibility,b.flexibility), friction: mix(a.friction,b.friction),
        thermal_conductivity: mix(a.thermal_conductivity,b.thermal_conductivity),
        ignition_temperature: mix(a.ignition_temperature,b.ignition_temperature),
        corrosion_resistance: mix(a.corrosion_resistance,b.corrosion_resistance),
        water_absorption: mix(a.water_absorption,b.water_absorption),
        permeability: mix(a.permeability,b.permeability), buoyancy_factor: mix(a.buoyancy_factor,b.buoyancy_factor),
        workability: mix(a.workability,b.workability),
    }
}
