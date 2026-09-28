use crate::agency::{ActionPrimitive, Affordance, ExpectedOutcome};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeatureKind {
    Vegetation, LooseMaterial, Soil, RockFace, ShallowWater, DeepWater,
    HeatSource, Creature, Shelter, ConstructedObject, ConstructionSite,
}

#[derive(Clone, Copy, Debug)]
pub struct PerceivedFeature {
    pub id:u64,
    pub kind:FeatureKind,
    pub distance_m:f32,
    pub danger:f32,
    pub food_hint:f32,
    pub material_hint:f32,
    pub uncertainty:f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LocalCapabilities {
    pub reach_m:f32,
    pub cutting:f32,
    pub digging:f32,
    pub carrying:f32,
    pub heat_tolerance:f32,
}

fn push(out:&mut Vec<Affordance>, action:ActionPrimitive, f:PerceivedFeature, expected:ExpectedOutcome) {
    out.push(Affordance{action,target:Some(f.id),expected,uncertainty:f.uncertainty.clamp(0.,1.),local_norm:0.});
}

pub fn generate(features:&[PerceivedFeature], cap:LocalCapabilities)->Vec<Affordance> {
    let mut out=Vec::new();
    for &f in features {
        let reachable=f.distance_m<=cap.reach_m.max(0.5);
        push(&mut out,ActionPrimitive::Observe,f,ExpectedOutcome{knowledge:0.15,effort:0.05,physical_risk:f.danger*0.15,..Default::default()});
        if !reachable {
            match f.kind {
                FeatureKind::Vegetation=>push(&mut out,ActionPrimitive::Move,f,ExpectedOutcome{food:f.food_hint*0.45,material:f.material_hint*0.25,effort:0.08,..Default::default()}),
                FeatureKind::LooseMaterial=>push(&mut out,ActionPrimitive::Move,f,ExpectedOutcome{material:f.material_hint*0.55,knowledge:0.03,effort:0.08,..Default::default()}),
                FeatureKind::Soil|FeatureKind::RockFace=>push(&mut out,ActionPrimitive::Move,f,ExpectedOutcome{material:f.material_hint*0.18,knowledge:0.07,effort:0.09,..Default::default()}),
                FeatureKind::ShallowWater=>push(&mut out,ActionPrimitive::Move,f,ExpectedOutcome{food:0.04,knowledge:0.05,effort:0.07,..Default::default()}),
                FeatureKind::Shelter|FeatureKind::ConstructedObject=>push(&mut out,ActionPrimitive::Move,f,ExpectedOutcome{safety:0.12,rest:0.05,effort:0.06,..Default::default()}),
                FeatureKind::ConstructionSite=>push(&mut out,ActionPrimitive::Move,f,ExpectedOutcome{safety:0.08,rest:0.05,material:0.12,status:0.02,effort:0.07,..Default::default()}),
                FeatureKind::Creature=>{
                    push(&mut out,ActionPrimitive::Avoid,f,ExpectedOutcome{safety:f.danger*0.85,effort:0.04,..Default::default()});
                    push(&mut out,ActionPrimitive::Communicate,f,ExpectedOutcome{belonging:0.08,care:0.08,safety:f.danger*0.10,..Default::default()});
                }
                FeatureKind::DeepWater=> {
                    push(&mut out,ActionPrimitive::Experiment,f,ExpectedOutcome{knowledge:0.20,physical_risk:f.danger*0.15,..Default::default()});
                    push(&mut out,ActionPrimitive::Avoid,f,ExpectedOutcome{safety:0.16,..Default::default()});
                }
                FeatureKind::HeatSource=>push(&mut out,ActionPrimitive::Avoid,f,ExpectedOutcome{safety:f.danger*0.65,..Default::default()}),
            }
            continue;
        }
        match f.kind {
            FeatureKind::Vegetation => {
                push(&mut out,ActionPrimitive::Gather,f,ExpectedOutcome{food:f.food_hint,material:f.material_hint*0.35,knowledge:0.05,effort:0.12,..Default::default()});
                push(&mut out,ActionPrimitive::Cut,f,ExpectedOutcome{knowledge:0.08,status:0.02,physical_risk:0.08/(0.2+cap.cutting),effort:0.32/(0.2+cap.cutting),..Default::default()});
                push(&mut out,ActionPrimitive::Bind,f,ExpectedOutcome{knowledge:0.14,effort:0.18,..Default::default()});
            }
            FeatureKind::LooseMaterial => {
                push(&mut out,ActionPrimitive::Gather,f,ExpectedOutcome{material:f.material_hint,knowledge:0.05,status:0.02,effort:0.12,..Default::default()});
                if cap.carrying>0.05 { push(&mut out,ActionPrimitive::Carry,f,ExpectedOutcome{material:f.material_hint*0.35,effort:0.18/(0.2+cap.carrying),..Default::default()}); }
                push(&mut out,ActionPrimitive::Strike,f,ExpectedOutcome{material:f.material_hint*0.20,knowledge:0.12,physical_risk:0.06,effort:0.15,..Default::default()});
            }
            FeatureKind::Soil => {
                push(&mut out,ActionPrimitive::Dig,f,ExpectedOutcome{knowledge:0.10,effort:0.28/(0.2+cap.digging),..Default::default()});
                push(&mut out,ActionPrimitive::Experiment,f,ExpectedOutcome{knowledge:0.24,effort:0.12,..Default::default()});
            }
            FeatureKind::RockFace => {
                push(&mut out,ActionPrimitive::Strike,f,ExpectedOutcome{knowledge:0.12,physical_risk:0.14,effort:0.32,..Default::default()});
                push(&mut out,ActionPrimitive::Dig,f,ExpectedOutcome{knowledge:0.15,physical_risk:0.18,effort:0.38/(0.2+cap.digging),..Default::default()});
            }
            FeatureKind::ShallowWater => {
                push(&mut out,ActionPrimitive::Consume,f,ExpectedOutcome{food:0.08,physical_risk:f.danger,..Default::default()});
                push(&mut out,ActionPrimitive::Experiment,f,ExpectedOutcome{knowledge:0.22,physical_risk:f.danger*0.35,..Default::default()});
            }
            FeatureKind::DeepWater => {
                push(&mut out,ActionPrimitive::Experiment,f,ExpectedOutcome{knowledge:0.30,physical_risk:0.35+f.danger,..Default::default()});
                push(&mut out,ActionPrimitive::Avoid,f,ExpectedOutcome{safety:0.30,..Default::default()});
            }
            FeatureKind::HeatSource => {
                let risk=(f.danger-cap.heat_tolerance*0.3).max(0.);
                push(&mut out,ActionPrimitive::Heat,f,ExpectedOutcome{knowledge:0.30,physical_risk:risk,effort:0.08,..Default::default()});
                push(&mut out,ActionPrimitive::Avoid,f,ExpectedOutcome{safety:risk,..Default::default()});
            }
            FeatureKind::Creature => {
                push(&mut out,ActionPrimitive::Avoid,f,ExpectedOutcome{safety:f.danger,..Default::default()});
                push(&mut out,ActionPrimitive::Communicate,f,ExpectedOutcome{belonging:0.12,care:0.10,safety:f.danger*0.15,..Default::default()});
                push(&mut out,ActionPrimitive::Attack,f,ExpectedOutcome{status:0.18,physical_risk:f.danger,knowledge:0.05,..Default::default()});
                push(&mut out,ActionPrimitive::Observe,f,ExpectedOutcome{knowledge:0.25,physical_risk:f.danger*0.2,..Default::default()});
            }
            FeatureKind::Shelter | FeatureKind::ConstructedObject => {
                push(&mut out,ActionPrimitive::Hide,f,ExpectedOutcome{safety:0.25,rest:0.08,..Default::default()});
                push(&mut out,ActionPrimitive::Experiment,f,ExpectedOutcome{knowledge:0.15,..Default::default()});
                if f.danger>0.04 {
                    push(&mut out,ActionPrimitive::Bind,f,ExpectedOutcome{safety:0.12,knowledge:0.05,status:0.03,effort:0.12,..Default::default()});
                    push(&mut out,ActionPrimitive::Raise,f,ExpectedOutcome{safety:0.14,knowledge:0.05,status:0.03,physical_risk:f.danger*0.25,effort:0.16,..Default::default()});
                }
            }
            FeatureKind::ConstructionSite => {
                // Shelter work pays safety and rest: a roof is protection, not decoration.
                push(&mut out,ActionPrimitive::Bind,f,ExpectedOutcome{safety:0.34,rest:0.14,knowledge:0.08,status:0.04,effort:0.16,..Default::default()});
                push(&mut out,ActionPrimitive::Raise,f,ExpectedOutcome{safety:0.42,rest:0.18,knowledge:0.09,status:0.05,physical_risk:0.05,effort:0.22,..Default::default()});
                push(&mut out,ActionPrimitive::Dig,f,ExpectedOutcome{safety:0.08,knowledge:0.05,effort:0.20/(0.2+cap.digging),..Default::default()});
                if cap.carrying>0.05 {push(&mut out,ActionPrimitive::Carry,f,ExpectedOutcome{safety:0.08,material:0.55,status:0.02,effort:0.16/(0.2+cap.carrying),..Default::default()});}
                push(&mut out,ActionPrimitive::Experiment,f,ExpectedOutcome{knowledge:0.16,effort:0.08,..Default::default()});
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn deep_water_offers_no_magic_cross_action() {
        let f=PerceivedFeature{id:1,kind:FeatureKind::DeepWater,distance_m:1.,danger:0.4,food_hint:0.,material_hint:0.,uncertainty:0.5};
        let a=generate(&[f],LocalCapabilities{reach_m:2.,..Default::default()});
        assert!(a.iter().any(|x|x.action==ActionPrimitive::Experiment));
        assert!(!a.iter().any(|x|x.action==ActionPrimitive::Move));
    }
    #[test] fn distant_useful_resource_offers_approach_without_magic_water_crossing() {
        let f=PerceivedFeature{id:2,kind:FeatureKind::LooseMaterial,distance_m:20.,danger:0.,food_hint:0.,material_hint:0.9,uncertainty:0.2};
        let a=generate(&[f],LocalCapabilities{reach_m:2.,..Default::default()});
        assert!(a.iter().any(|x|x.action==ActionPrimitive::Move));
    }
    #[test] fn distant_creature_can_trigger_avoidance() {
        let f=PerceivedFeature{id:3,kind:FeatureKind::Creature,distance_m:30.,danger:0.8,food_hint:0.,material_hint:0.,uncertainty:0.2};
        let a=generate(&[f],LocalCapabilities{reach_m:2.,..Default::default()});
        assert!(a.iter().any(|x|x.action==ActionPrimitive::Avoid));
    }
    #[test] fn same_creature_exposes_multiple_possible_responses() {
        let f=PerceivedFeature{id:9,kind:FeatureKind::Creature,distance_m:1.,danger:0.8,food_hint:0.,material_hint:0.,uncertainty:0.4};
        let a=generate(&[f],LocalCapabilities{reach_m:2.,..Default::default()});
        for x in [ActionPrimitive::Avoid,ActionPrimitive::Communicate,ActionPrimitive::Attack,ActionPrimitive::Observe] {
            assert!(a.iter().any(|q|q.action==x));
        }
    }
}
