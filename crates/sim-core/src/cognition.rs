use crate::materials::Belief;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProblemKind { WaterBarrier, CliffBarrier, Cutting, Mining, ShelterFailure, MonsterThreat, FoodShortage }

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProblemMemory {
    pub pressure: BTreeMap<ProblemKind, f32>,
    pub failed_attempts: BTreeMap<ProblemKind, u32>,
}

impl ProblemMemory {
    pub fn encounter(&mut self, problem: ProblemKind, severity: f32) {
        *self.pressure.entry(problem).or_default() += severity.max(0.0);
    }
    pub fn failure(&mut self, problem: ProblemKind) {
        *self.failed_attempts.entry(problem).or_default() += 1;
        *self.pressure.entry(problem).or_default() += 0.2;
    }
    pub fn resolve(&mut self, problem: ProblemKind, effectiveness: f32) {
        let p = self.pressure.entry(problem).or_default();
        *p = (*p - effectiveness.max(0.0)).max(0.0);
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MaterialKnowledge {
    pub properties: BTreeMap<String, Belief>,
}
