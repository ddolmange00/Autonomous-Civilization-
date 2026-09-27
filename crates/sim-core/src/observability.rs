use crate::{agency::ActionPrimitive, awareness::{Awareness,SituationKind}, culture::CulturalField};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct SettlementPulse {
    pub population:u64,
    pub aware_fraction:BTreeMap<SituationKind,f32>,
    pub action_fraction:BTreeMap<ActionPrimitive,f32>,
    pub dominant_norms:Vec<(ActionPrimitive,f32)>,
}

pub fn summarize<'a>(
    population:usize,
    awareness:impl Iterator<Item=&'a Awareness>,
    actions:impl Iterator<Item=ActionPrimitive>,
    culture:&CulturalField,
)->SettlementPulse {
    let mut p=SettlementPulse{population:population as u64,..Default::default()};
    let denom=population.max(1) as f32;
    for a in awareness {
        for (&k,r) in &a.reports {
            if r.confidence>=0.25 {*p.aware_fraction.entry(k).or_default()+=1.0/denom;}
        }
    }
    for a in actions {*p.action_fraction.entry(a).or_default()+=1.0/denom;}
    let mut norms:Vec<_>=culture.norms.iter().map(|(&a,&v)|(a,v)).collect();
    norms.sort_by(|a,b|b.1.abs().total_cmp(&a.1.abs()));
    p.dominant_norms=norms.into_iter().take(5).collect();
    p
}
