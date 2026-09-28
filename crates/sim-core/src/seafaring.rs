use crate::{
    design::{evaluate, DesignGenome, Function},
    materials::MaterialProperties,
    traversal::{can_cross, Capabilities},
    environment::Barrier,
};

/// A watercraft as understood by its builders: flotation first,
/// propulsion and steering as fractions of it. No hull, no voyage.
#[derive(Clone, Copy, Debug, Default)]
pub struct Craft {
    pub flotation: f32,
    pub propulsion: f32,
    pub steering: f32,
}

impl Craft {
    pub fn capabilities(&self) -> Capabilities {
        Capabilities {
            flotation: self.flotation,
            propulsion: self.propulsion,
            steering: self.steering,
            anchoring: 0.0,
            climbing: 0.0,
            thermal_protection: 0.0,
        }
    }
}

/// Only Float-function designs count as craft. Rafts are not assigned;
/// they must exist in the repertoire first.
pub fn craft_from_designs<'a>(designs: impl Iterator<Item = &'a DesignGenome>, truth: &MaterialProperties) -> Option<Craft> {
    let best = designs
        .filter(|d| matches!(d.function, Function::Float))
        .map(|d| evaluate(d, *truth).flotation)
        .fold(0.0_f32, f32::max);
    if best <= 0.05 {
        return None;
    }
    Some(Craft { flotation: best, propulsion: best * 0.5, steering: best * 0.3 })
}

pub fn can_embark(craft: Option<Craft>, severity: f32) -> bool {
    match craft {
        Some(c) => can_cross(Barrier::DeepWater, c.capabilities(), severity),
        None => false,
    }
}

/// Straight-line water crossing check between two points.
/// Returns the worst barrier severity found, if any water intervenes.
pub fn crosses_water(from: (f32, f32), to: (f32, f32), waters: &[(f32, f32, f32)]) -> Option<f32> {
    let mut worst: Option<f32> = None;
    for &(wx, wy, wr) in waters {
        let seg = ((to.0 - from.0).powi(2) + (to.1 - from.1).powi(2)).sqrt().max(0.001);
        let t = (((wx - from.0) * (to.0 - from.0) + (wy - from.1) * (to.1 - from.1)) / (seg * seg)).clamp(0.0, 1.0);
        let px = from.0 + (to.0 - from.0) * t;
        let py = from.1 + (to.1 - from.1) * t;
        let d = ((wx - px).powi(2) + (wy - py).powi(2)).sqrt();
        if d < wr {
            let severity = (1.0 - d / wr).clamp(0.2, 1.0);
            worst = Some(worst.map(|w: f32| w.max(severity)).unwrap_or(severity));
        }
    }
    worst
}

#[cfg(test)]
mod tests {
    use super::*;
    fn timber() -> MaterialProperties {
        MaterialProperties {
            density: 0.6,
            tensile_strength: 0.55,
            compressive_strength: 0.45,
            fracture_toughness: 0.5,
            hardness: 0.3,
            elastic_modulus: 0.5,
            flexibility: 0.5,
            friction: 0.6,
            thermal_conductivity: 0.3,
            ignition_temperature: 0.6,
            corrosion_resistance: 0.5,
            water_absorption: 0.5,
            permeability: 0.3,
            buoyancy_factor: 0.9,
            workability: 0.8,
        }
    }
    fn raft() -> DesignGenome {
        DesignGenome {
            id: 1,
            parent: None,
            generation: 0,
            function: Function::Float,
            length_m: 3.0,
            width_m: 2.0,
            thickness_m: 0.15,
            curvature: 0.1,
            edge_fraction: 0.0,
            binding_quality: 0.7,
            material_fraction: vec![],
        }
    }
    #[test]
    fn no_float_design_no_voyage() {
        assert!(craft_from_designs([].iter(), &timber()).is_none());
        assert!(!can_embark(None, 0.5));
    }
    #[test]
    fn real_raft_crosses_calm_water() {
        let craft = craft_from_designs([raft()].iter(), &timber());
        assert!(craft.is_some());
        assert!(can_embark(craft, 0.4));
    }
    #[test]
    fn crossing_detects_intervening_water() {
        let waters = vec![(0.0, 0.0, 20.0)];
        assert!(crosses_water((-50.0, 0.0), (50.0, 0.0), &waters).is_some());
        assert!(crosses_water((-50.0, 0.0), (-30.0, 0.0), &waters).is_none());
    }
}
