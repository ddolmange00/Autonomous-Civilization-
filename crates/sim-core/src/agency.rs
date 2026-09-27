use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ActionPrimitive {
    Move, Observe, Gather, Carry, Combine, Separate, Bind, Dig, Raise,
    Heat, Cool, Strike, Pierce, Cut, Attack, Avoid, Hide, Assist,
    Communicate, Consume, Rest, Experiment,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Traits {
    pub threat_sensitivity: f32,
    pub aggression: f32,
    pub curiosity: f32,
    pub empathy: f32,
    pub conformity: f32,
    pub persistence: f32,
    pub risk_tolerance: f32,
    pub novelty_seeking: f32,
    pub social_trust: f32,
    pub planning_horizon: f32,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Needs {
    pub hunger: f32,
    pub safety: f32,
    pub rest: f32,
    pub belonging: f32,
    pub status: f32,
    pub curiosity: f32,
    pub care: f32,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct ExpectedOutcome {
    pub food: f32,
    pub safety: f32,
    pub rest: f32,
    pub belonging: f32,
    pub status: f32,
    pub knowledge: f32,
    pub care: f32,
    pub physical_risk: f32,
    pub effort: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Affordance {
    pub action: ActionPrimitive,
    pub target: Option<u64>,
    pub expected: ExpectedOutcome,
    pub uncertainty: f32,
    pub local_norm: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AgentMind {
    pub traits: Traits,
    pub needs: Needs,
    pub learned_action_value: BTreeMap<ActionPrimitive, f32>,
}

impl AgentMind {
    pub fn score(&self, a:&Affordance, idiosyncratic_noise:f32)->f32 {
        let n=self.needs; let t=self.traits; let o=a.expected;
        let survival = o.food*n.hunger + o.safety*n.safety*(0.55+0.9*t.threat_sensitivity) + o.rest*n.rest;
        let social = o.belonging*n.belonging*(0.4+0.8*t.social_trust)
            + o.care*n.care*(0.35+0.9*t.empathy) + o.status*n.status*(0.45+0.65*t.aggression);
        let learning = o.knowledge*n.curiosity*(0.35+t.curiosity+t.novelty_seeking*0.4);
        let risk = o.physical_risk*(1.35-t.risk_tolerance.clamp(0.,1.));
        let effort = o.effort*(1.15-t.persistence.clamp(0.,1.)*.45);
        let norm = a.local_norm*t.conformity;
        let learned=*self.learned_action_value.get(&a.action).unwrap_or(&0.0);
        let uncertainty_cost=a.uncertainty*(0.45-t.risk_tolerance*.25-t.curiosity*.15);
        survival+social+learning+norm+learned-risk-effort-uncertainty_cost+idiosyncratic_noise
    }

    pub fn choose<'a>(&self, affordances:&'a [Affordance], noise:&[f32])->Option<&'a Affordance> {
        affordances.iter().enumerate().max_by(|(ia,a),(ib,b)| {
            self.score(a,*noise.get(*ia).unwrap_or(&0.0))
                .total_cmp(&self.score(b,*noise.get(*ib).unwrap_or(&0.0)))
        }).map(|(_,a)|a)
    }

    pub fn learn_action(&mut self, action:ActionPrimitive, experienced_value:f32, rate:f32) {
        let old=*self.learned_action_value.get(&action).unwrap_or(&0.0);
        self.learned_action_value.insert(action,old+(experienced_value-old)*rate.clamp(0.,1.));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn aff(action:ActionPrimitive,safety:f32,status:f32,risk:f32)->Affordance {
        Affordance{action,target:None,expected:ExpectedOutcome{safety,status,physical_risk:risk,..Default::default()},uncertainty:.1,local_norm:0.}
    }
    #[test] fn personalities_can_choose_different_actions_from_same_world() {
        let options=[aff(ActionPrimitive::Hide,1.,0.,.05),aff(ActionPrimitive::Attack,.2,1.,.8)];
        let mut cautious=AgentMind::default(); cautious.needs.safety=1.; cautious.needs.status=.4; cautious.traits.threat_sensitivity=1.; cautious.traits.risk_tolerance=.05;
        let mut bold=AgentMind::default(); bold.needs.safety=.4; bold.needs.status=1.; bold.traits.aggression=1.; bold.traits.risk_tolerance=1.;
        assert_eq!(cautious.choose(&options,&[0.,0.]).unwrap().action,ActionPrimitive::Hide);
        assert_eq!(bold.choose(&options,&[0.,0.]).unwrap().action,ActionPrimitive::Attack);
    }
}
