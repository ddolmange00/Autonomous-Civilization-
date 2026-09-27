use crate::{design::{evaluate, DesignGenome, Performance}, materials::{Belief, MaterialProperties}};

#[derive(Clone, Copy, Debug)]
pub struct ExperimentResult {
    pub predicted: f32,
    pub actual: f32,
    pub success: bool,
    pub surprise: f32,
}

pub fn test_design(genome:&DesignGenome, believed:MaterialProperties, truth:MaterialProperties, threshold:f32)->(ExperimentResult,Performance) {
    let predicted_perf=evaluate(genome,believed);
    let actual_perf=evaluate(genome,truth);
    let score=|p:Performance| match genome.function {
        crate::design::Function::Cut=>p.cutting,
        crate::design::Function::Impact=>p.impact,
        crate::design::Function::Pierce=>p.piercing,
        crate::design::Function::Float=>p.flotation,
        crate::design::Function::Climb=>p.climbing,
        crate::design::Function::Shelter=>p.structural,
    };
    let predicted=score(predicted_perf);
    let actual=score(actual_perf);
    (ExperimentResult{predicted,actual,success:actual>=threshold,surprise:(predicted-actual).abs()},actual_perf)
}

pub fn learn_scalar(belief:&mut Belief, actual:f32, result:ExperimentResult) {
    let reliability=(0.2+result.surprise.min(1.0)*0.55).clamp(0.05,0.9);
    belief.observe(actual,reliability);
}
