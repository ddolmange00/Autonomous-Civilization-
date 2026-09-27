use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Relation {
    pub familiarity:f32,
    pub trust:f32,
    pub affection:f32,
    pub obligation:f32,
    pub fear:f32,
    pub prestige:f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SocialMemory {
    pub relations:BTreeMap<u64,Relation>,
}

impl SocialMemory {
    pub fn relation_mut(&mut self,other:u64)->&mut Relation { self.relations.entry(other).or_default() }
    pub fn observe_help(&mut self,other:u64,magnitude:f32) {
        let r=self.relation_mut(other); let x=magnitude.clamp(0.0,1.0);
        r.trust=(r.trust+0.12*x).clamp(-1.0,1.0);
        r.affection=(r.affection+0.08*x).clamp(-1.0,1.0);
        r.obligation=(r.obligation+0.10*x).clamp(0.0,1.0);
    }
    pub fn observe_harm(&mut self,other:u64,magnitude:f32) {
        let r=self.relation_mut(other); let x=magnitude.clamp(0.0,1.0);
        r.trust=(r.trust-0.22*x).clamp(-1.0,1.0);
        r.fear=(r.fear+0.18*x).clamp(0.0,1.0);
    }
}
