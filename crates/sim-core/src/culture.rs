use crate::agency::ActionPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CulturalField {
    pub norms: BTreeMap<ActionPrimitive,f32>,
    pub observations: u64,
}

impl CulturalField {
    pub fn observe(&mut self, action:ActionPrimitive, success:f32, prestige:f32, visibility:f32) {
        let signal=(success*0.55+prestige*0.30+visibility*0.15).clamp(-1.,1.);
        let old=*self.norms.get(&action).unwrap_or(&0.0);
        self.norms.insert(action,(old*.97+signal*.03).clamp(-1.,1.));
        self.observations+=1;
    }
    pub fn norm(&self, action:ActionPrimitive)->f32 { *self.norms.get(&action).unwrap_or(&0.0) }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct EmergentProfile {
    pub confrontation: f32,
    pub avoidance: f32,
    pub experimentation: f32,
    pub cooperation: f32,
    pub construction: f32,
}

impl CulturalField {
    pub fn profile(&self)->EmergentProfile {
        let n=|a| self.norm(a);
        EmergentProfile{
            confrontation:n(ActionPrimitive::Attack),
            avoidance:(n(ActionPrimitive::Avoid)+n(ActionPrimitive::Hide))*.5,
            experimentation:n(ActionPrimitive::Experiment),
            cooperation:(n(ActionPrimitive::Assist)+n(ActionPrimitive::Communicate))*.5,
            construction:(n(ActionPrimitive::Bind)+n(ActionPrimitive::Dig)+n(ActionPrimitive::Raise))/3.,
        }
    }
}
