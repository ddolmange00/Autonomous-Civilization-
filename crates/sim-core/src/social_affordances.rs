use crate::{
    agency::{ActionPrimitive,Affordance,ExpectedOutcome},
    life_history::LifeStage,
    relationships::Relation,
};

#[derive(Clone, Copy, Debug)]
pub struct SocialTarget {
    pub id:u64,
    pub distance_m:f32,
    pub stage:LifeStage,
    pub health:f32,
    pub hunger:f32,
    pub safety_need:f32,
    pub relation:Relation,
}

pub fn generate_social(targets:&[SocialTarget])->Vec<Affordance> {
    let mut out=Vec::new();
    for t in targets {
        if t.distance_m>28.0 {continue;}
        out.push(Affordance{
            action:ActionPrimitive::Communicate,target:Some(t.id),
            expected:ExpectedOutcome{belonging:0.18,knowledge:0.08,care:(t.relation.affection.max(0.0))*0.08,effort:0.04,..Default::default()},
            uncertainty:0.08,local_norm:0.0,
        });
        let dependency=match t.stage {LifeStage::Infant=>1.0,LifeStage::Child=>0.55,LifeStage::Elder=>0.35,_=>0.10};
        let need=((1.0-t.health)*0.35+t.hunger*0.35+t.safety_need*0.20+dependency*0.30).clamp(0.0,1.0);
        if need>0.18 {
            out.push(Affordance{
                action:ActionPrimitive::Assist,target:Some(t.id),
                expected:ExpectedOutcome{
                    care:need*(0.45+0.35*t.relation.affection.max(0.0)+0.20*t.relation.obligation),
                    belonging:0.08,status:0.04,physical_risk:t.safety_need*0.08,effort:0.12+need*0.10,..Default::default()
                },
                uncertainty:0.10,local_norm:0.0,
            });
        }
        out.push(Affordance{
            action:ActionPrimitive::Observe,target:Some(t.id),
            expected:ExpectedOutcome{knowledge:0.08+0.08*t.relation.prestige.max(0.0),belonging:0.03,effort:0.02,..Default::default()},
            uncertainty:0.12,local_norm:0.0,
        });
    }
    out
}
