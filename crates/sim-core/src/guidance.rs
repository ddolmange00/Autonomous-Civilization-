use crate::agency::ActionPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GuidanceSignal {
    pub action_bias:BTreeMap<ActionPrimitive,f32>,
    pub source_trust:f32,
    pub urgency:f32,
}

impl GuidanceSignal {
    pub fn bias_for(&self,action:ActionPrimitive,conformity:f32)->f32 {
        self.action_bias.get(&action).copied().unwrap_or(0.)
            * self.source_trust.clamp(0.,1.)
            * self.urgency.clamp(0.,1.)
            * (0.25+0.75*conformity.clamp(0.,1.))
    }
}
