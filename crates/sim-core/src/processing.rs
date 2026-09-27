use crate::materials::MaterialProperties;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Process {
    Dry { intensity: f32 },
    Heat { temperature_c: f32, hours: f32 },
    Hammer { work: f32 },
    Quench { severity: f32 },
    Bind { quality: f32 },
}

pub fn apply_process(mut p: MaterialProperties, process: Process) -> MaterialProperties {
    match process {
        Process::Dry { intensity } => {
            let x = intensity.clamp(0.0, 1.0);
            p.water_absorption *= 1.0 - 0.45 * x;
            p.density *= 1.0 - 0.08 * x;
            p.workability *= 1.0 + 0.12 * x;
        }
        Process::Heat { temperature_c, hours } => {
            let dose = ((temperature_c - 100.0).max(0.0) / 900.0 * (hours / 8.0)).clamp(0.0, 1.5);
            p.hardness *= 1.0 + 0.22 * dose;
            p.workability *= (1.0 + 0.18 * dose).max(0.1);
            p.fracture_toughness *= (1.0 - 0.08 * dose).max(0.15);
        }
        Process::Hammer { work } => {
            let x = work.clamp(0.0, 1.0);
            p.tensile_strength *= 1.0 + 0.20 * x;
            p.hardness *= 1.0 + 0.16 * x;
            p.flexibility *= (1.0 - 0.08 * x).max(0.1);
        }
        Process::Quench { severity } => {
            let x = severity.clamp(0.0, 1.0);
            p.hardness *= 1.0 + 0.35 * x;
            p.fracture_toughness *= (1.0 - 0.22 * x).max(0.08);
        }
        Process::Bind { quality } => {
            let x = quality.clamp(0.0, 1.0);
            p.friction *= 1.0 + 0.25 * x;
            p.workability *= 1.0 + 0.10 * x;
        }
    }
    p
}
