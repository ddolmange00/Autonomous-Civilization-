use crate::agency::ActionPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct PracticeStat {
    pub repetitions:f32,
    pub skill:f32,
    pub last_year:f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PracticeProfile {
    pub actions:BTreeMap<ActionPrimitive,PracticeStat>,
}

impl PracticeProfile {
    pub fn practice(&mut self,action:ActionPrimitive,value:f32,days:f32,year:f64) {
        let s=self.actions.entry(action).or_default();
        let amount=(days.max(0.0)*(0.2+value.max(0.0))).min(20.0);
        s.repetitions+=amount;
        let target=(1.0-(-s.repetitions/180.0).exp()).clamp(0.0,1.0);
        s.skill=(s.skill+(target-s.skill)*0.08).clamp(0.0,1.0);
        s.last_year=year;
    }
    pub fn skill(&self,action:ActionPrimitive)->f32 {self.actions.get(&action).map(|s|s.skill).unwrap_or(0.0)}
    pub fn dominant(&self,n:usize)->Vec<(ActionPrimitive,f32)> {
        let mut v:Vec<_>=self.actions.iter().map(|(&a,s)|(a,s.skill)).collect();
        v.sort_by(|a,b|b.1.total_cmp(&a.1));v.truncate(n);v
    }
}
