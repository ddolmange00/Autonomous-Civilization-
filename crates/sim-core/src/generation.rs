use crate::{agency::Traits, life_history::{LifeHistory,LifeStage,Sex}, relationships::Relation};

#[derive(Clone, Copy, Debug)]
pub struct ReproductionContext {
    pub stage:LifeStage,
    pub sex:Sex,
    pub health:f32,
    pub hunger:f32,
    pub safety_need:f32,
    pub care_trait:f32,
    pub household_pressure:f32,
    pub partner_relation:Relation,
}

pub fn conception_propensity(c:ReproductionContext)->f32 {
    if c.stage!=LifeStage::Adult || c.sex!=Sex::Female {return 0.0;}
    let relationship=((c.partner_relation.trust+1.0)*0.5*0.45+(c.partner_relation.affection+1.0)*0.5*0.55).clamp(0.0,1.0);
    let body=c.health.clamp(0.0,1.0)*(1.0-c.hunger.clamp(0.0,1.0));
    let environment=(1.0-c.safety_need.clamp(0.0,1.0)*0.6)*(1.0-c.household_pressure.clamp(0.0,1.0)*0.75);
    (relationship*body*environment*(0.35+0.65*c.care_trait.clamp(0.0,1.0))*0.25).clamp(0.0,0.25)
}

pub fn annual_mortality_risk(life:&LifeHistory,year:f64,health:f32)->f32 {
    let age=life.age(year);
    let age_risk=if age<1.0{0.025}else if age<15.0{0.002}else if age<50.0{0.003}else if age<70.0{0.015+(age-50.0)*0.002}else{0.055+(age-70.0)*0.012};
    (age_risk+(1.0-health.clamp(0.0,1.0))*0.20).clamp(0.0,0.95)
}

pub fn child_imitation_weight(child:Traits,relation:Relation,prestige:f32)->f32 {
    (0.15+child.conformity*0.30+child.social_trust*0.20+relation.trust.max(0.0)*0.20+prestige.clamp(0.0,1.0)*0.15).clamp(0.0,1.0)
}
