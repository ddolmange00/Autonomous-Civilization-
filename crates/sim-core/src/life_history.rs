use crate::{agency::Traits, demography::HeritableTraits, family::Kinship, relationships::SocialMemory};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifeStage { Infant, Child, Adolescent, Adult, Elder }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LifeHistory {
    pub birth_year:f64,
    pub biological:HeritableTraits,
    pub kinship:Kinship,
    pub social:SocialMemory,
}

impl LifeHistory {
    pub fn age(&self,year:f64)->f32 {(year-self.birth_year).max(0.0) as f32}
    pub fn stage(&self,year:f64)->LifeStage {
        match self.age(year) {a if a<3.0=>LifeStage::Infant,a if a<12.0=>LifeStage::Child,a if a<18.0=>LifeStage::Adolescent,a if a<58.0=>LifeStage::Adult,_=>LifeStage::Elder}
    }
}

pub fn inherit_personality(a:Traits,b:Traits,variation:[f32;10])->Traits {
    let m=|x:f32,y:f32,n:f32|(0.5*(x+y)+n.clamp(-1.0,1.0)*0.10).clamp(0.0,1.0);
    Traits{threat_sensitivity:m(a.threat_sensitivity,b.threat_sensitivity,variation[0]),aggression:m(a.aggression,b.aggression,variation[1]),
        curiosity:m(a.curiosity,b.curiosity,variation[2]),empathy:m(a.empathy,b.empathy,variation[3]),conformity:m(a.conformity,b.conformity,variation[4]),
        persistence:m(a.persistence,b.persistence,variation[5]),risk_tolerance:m(a.risk_tolerance,b.risk_tolerance,variation[6]),
        novelty_seeking:m(a.novelty_seeking,b.novelty_seeking,variation[7]),social_trust:m(a.social_trust,b.social_trust,variation[8]),
        planning_horizon:m(a.planning_horizon,b.planning_horizon,variation[9])}
}
