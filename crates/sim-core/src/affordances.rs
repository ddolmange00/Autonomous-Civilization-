use crate::agency::{ActionPrimitive, Affordance, ExpectedOutcome};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeatureKind {
    Vegetation, LooseMaterial, Soil, RockFace, ShallowWater, DeepWater,
    HeatSource, Creature, Shelter, ConstructedObject,
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
        push(&mut out,ActionPrimitive::Observe,f,ExpectedOutcome{knowledge:.35,physical_risk:f.danger*.15,..Default::default()});
        if !reachable { continue; }
        match f.kind {
            FeatureKind::Vegetation => {
                push(&mut out,ActionPrimitive::Gather,f,ExpectedOutcome{food:f.food_hint,knowledge:.05,effort:.12,..Default::default()});
                push(&mut out,ActionPrimitive::Cut,f,ExpectedOutcome{knowledge:.08,status:.02,physical_risk:.08/(0.2+cap.cutting),effort:.32/(0.2+cap.cutting),..Default::default()});
                push(&mut out,ActionPrimitive::Bind,f,ExpectedOutcome{knowledge:.14,effort:.18,..Default::default()});
            }
            FeatureKind::LooseMaterial => {
                push(&mut out,ActionPrimitive::Gather,f,ExpectedOutcome{knowledge:.05,status:.02,effort:.12,..Default::default()});
                if cap.carrying>0.05 { push(&mut out,ActionPrimitive::Carry,f,ExpectedOutcome{effort:.18/(0.2+cap.carrying),..Default::default()}); }
                push(&mut out,ActionPrimitive::Strike,f,ExpectedOutcome{knowledge:.12,physical_risk:.06,effort:.15,..Default::default()});
            }
            FeatureKind::Soil => {
                push(&mut out,ActionPrimitive::Dig,f,ExpectedOutcome{knowledge:.10,effort:.28/(0.2+cap.digging),..Default::default()});
                push(&mut out,ActionPrimitive::Experiment,f,ExpectedOutcome{knowledge:.24,effort:.12,..Default::default()});
            }
            FeatureKind::RockFace => {
                push(&mut out,ActionPrimitive::Strike,f,ExpectedOutcome{knowledge:.12,physical_risk:.14,effort:.32,..Default::default()});
                push(&mut out,ActionPrimitive::Dig,f,ExpectedOutcome{knowledge:.15,physical_risk:.18,effort:.38/(0.2+cap.digging),..Default::default()});
            }
            FeatureKind::ShallowWater => {
                push(&mut out,ActionPrimitive::Consume,f,ExpectedOutcome{food:.08,physical_risk:f.danger,..Default::default()});
                push(&mut out,ActionPrimitive::Experiment,f,ExpectedOutcome{knowledge:.22,physical_risk:f.danger*.35,..Default::default()});
            }
            FeatureKind::DeepWater => {
                push(&mut out,ActionPrimitive::Experiment,f,ExpectedOutcome{knowledge:.30,physical_risk:.35+f.danger,..Default::default()});
                push(&mut out,ActionPrimitive::Avoid,f,ExpectedOutcome{safety:.30,..Default::default()});
            }
            FeatureKind::HeatSource => {
                let risk=(f.danger-cap.heat_tolerance*.3).max(0.);
                push(&mut out,ActionPrimitive::Heat,f,ExpectedOutcome{knowledge:.30,physical_risk:risk,effort:.08,..Default::default()});
                push(&mut out,ActionPrimitive::Avoid,f,ExpectedOutcome{safety:risk,..Default::default()});
            }
            FeatureKind::Creature => {
                push(&mut out,ActionPrimitive::Avoid,f,ExpectedOutcome{safety:f.danger,..Default::default()});
                push(&mut out,ActionPrimitive::Communicate,f,ExpectedOutcome{belonging:.12,care:.10,safety:f.danger*.15,..Default::default()});
                push(&mut out,ActionPrimitive::Attack,f,ExpectedOutcome{status:.18,physical_risk:f.danger,knowledge:.05,..Default::default()});
                push(&mut out,ActionPrimitive::Observe,f,ExpectedOutcome{knowledge:.25,physical_risk:f.danger*.2,..Default::default()});
            }
            FeatureKind::Shelter | FeatureKind::ConstructedObject => {
                push(&mut out,ActionPrimitive::Hide,f,ExpectedOutcome{safety:.25,rest:.08,..Default::default()});
                push(&mut out,ActionPrimitive::Experiment,f,ExpectedOutcome{knowledge:.15,..Default::default()});
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn deep_water_offers_no_magic_cross_action() {
        let f=PerceivedFeature{id:1,kind:FeatureKind::DeepWater,distance_m:1.,danger:.4,food_hint:0.,material_hint:0.,uncertainty:.5};
        let a=generate(&[f],LocalCapabilities{reach_m:2.,..Default::default()});
        assert!(a.iter().any(|x|x.action==ActionPrimitive::Experiment));
        assert!(!a.iter().any(|x|x.action==ActionPrimitive::Move));
    }
    #[test] fn same_creature_exposes_multiple_possible_responses() {
        let f=PerceivedFeature{id:9,kind:FeatureKind::Creature,distance_m:1.,danger:.8,food_hint:0.,material_hint:0.,uncertainty:.4};
        let a=generate(&[f],LocalCapabilities{reach_m:2.,..Default::default()});
        for x in [ActionPrimitive::Avoid,ActionPrimitive::Communicate,ActionPrimitive::Attack,ActionPrimitive::Observe] {
            assert!(a.iter().any(|q|q.action==x));
        }
    }
}
