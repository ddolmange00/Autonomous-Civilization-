use crate::{
    design::{DesignGenome, Function},
    experiments::{learn_scalar, test_design, ExperimentResult},
    materials::{Belief, MaterialProperties},
};

/// A drafted prototype: a genome plus the test verdict.
/// Failure is evidence, not nothing (simulation constitution #4).
pub struct Draft {
    pub genome: DesignGenome,
    pub result: ExperimentResult,
    pub believed: MaterialProperties,
}

/// Seed a tool prototype from practiced skill. Geometry is a guess;
/// physics decides whether it works.
pub fn seed_tool_design(id: u64, function: Function, skill: f32, variation: f32) -> DesignGenome {
    let s = skill.clamp(0.0, 1.0);
    let v = variation.clamp(-1.0, 1.0);
    DesignGenome {
        id,
        parent: None,
        generation: 0,
        function,
        length_m: (0.4 + 0.9 * s + v * 0.1).clamp(0.2, 2.0),
        width_m: (0.08 + 0.15 * s + v * 0.03).clamp(0.03, 0.5),
        thickness_m: (0.03 + 0.08 * s).clamp(0.02, 0.2),
        curvature: (0.1 + v * 0.05).clamp(0.0, 0.5),
        edge_fraction: (0.3 + 0.6 * s).clamp(0.0, 1.0),
        binding_quality: (0.2 + 0.7 * s).clamp(0.0, 1.0),
        material_fraction: vec![(1, 1.0)],
    }
}

/// Seed a float prototype: wide, buoyant, bound.
pub fn seed_float_design(id: u64, skill: f32, variation: f32) -> DesignGenome {
    let s = skill.clamp(0.0, 1.0);
    let v = variation.clamp(-1.0, 1.0);
    DesignGenome {
        id,
        parent: None,
        generation: 0,
        function: Function::Float,
        length_m: (2.0 + 1.6 * s + v * 0.2).clamp(1.2, 5.0),
        width_m: (1.2 + 1.2 * s).clamp(0.8, 3.0),
        thickness_m: (0.08 + 0.10 * s).clamp(0.05, 0.3),
        curvature: (0.05 + v * 0.05).clamp(0.0, 0.4),
        edge_fraction: 0.0,
        binding_quality: (0.25 + 0.65 * s).clamp(0.0, 1.0),
        material_fraction: vec![(1, 1.0)],
    }
}

/// Prototype test: skilled guessers predict closer to truth.
/// Returns the draft; the caller records successes and learns from failures.
pub fn test_prototype(genome: &DesignGenome, truth: MaterialProperties, skill: f32) -> Draft {
    let closeness = 0.4 + 0.6 * skill.clamp(0.0, 1.0);
    let believed = MaterialProperties {
        density: truth.density * closeness + 0.5 * (1.0 - closeness),
        tensile_strength: truth.tensile_strength * closeness + 0.5 * (1.0 - closeness),
        compressive_strength: truth.compressive_strength * closeness + 0.5 * (1.0 - closeness),
        fracture_toughness: truth.fracture_toughness * closeness + 0.5 * (1.0 - closeness),
        hardness: truth.hardness * closeness + 0.5 * (1.0 - closeness),
        elastic_modulus: truth.elastic_modulus * closeness + 0.5 * (1.0 - closeness),
        flexibility: truth.flexibility * closeness + 0.5 * (1.0 - closeness),
        friction: truth.friction * closeness + 0.5 * (1.0 - closeness),
        thermal_conductivity: truth.thermal_conductivity,
        ignition_temperature: truth.ignition_temperature,
        corrosion_resistance: truth.corrosion_resistance,
        water_absorption: truth.water_absorption,
        permeability: truth.permeability,
        buoyancy_factor: truth.buoyancy_factor * closeness + 0.5 * (1.0 - closeness),
        workability: truth.workability,
    };
    let (result, _) = test_design(genome, believed, truth, 0.15);
    Draft { genome: genome.clone(), result, believed }
}

/// Fold a test outcome back into a scalar belief.
pub fn learn_from_draft(belief: &mut Belief, draft: &Draft) {
    learn_scalar(belief, draft.result.actual, draft.result);
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
    #[test]
    fn skilled_prototypes_beat_bare_hands() {
        let good = seed_tool_design(1, Function::Cut, 0.9, 0.1);
        let draft = test_prototype(&good, timber(), 0.9);
        assert!(draft.result.success);
        let poor = seed_tool_design(2, Function::Cut, 0.0, -0.5);
        let draft2 = test_prototype(&poor, timber(), 0.0);
        assert!(draft2.result.actual < draft.result.actual);
    }
    #[test]
    fn rafts_float_on_timber_truth() {
        let raft = seed_float_design(3, 0.8, 0.1);
        let draft = test_prototype(&raft, timber(), 0.8);
        assert!(draft.result.success);
    }
    #[test]
    fn failure_teaches() {
        let mut belief = Belief::default();
        let bad = seed_tool_design(4, Function::Cut, 0.0, -0.9);
        let draft = test_prototype(&bad, timber(), 0.0);
        learn_from_draft(&mut belief, &draft);
        assert!(belief.observations > 0);
    }
}
