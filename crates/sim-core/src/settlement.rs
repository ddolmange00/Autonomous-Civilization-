use crate::{
    agency::ActionPrimitive,
    awareness::{Awareness, SituationKind},
    culture::CulturalField,
    observability::{summarize, SettlementPulse},
    sandbox::Resident,
    world::Position,
};

#[derive(Clone, Debug)]
pub struct SettlementView {
    pub center:Position,
    pub radius:f32,
    pub pulse:SettlementPulse,
    pub alive:usize,
    pub average_health:f32,
}

pub fn inspect_settlement(
    center:Position,
    radius:f32,
    residents:&[Resident],
    awareness:&[(u64,Awareness)],
    culture:&CulturalField,
)->SettlementView {
    let inside:Vec<&Resident>=residents.iter().filter(|r|{
        let dx=r.position.x-center.x; let dy=r.position.y-center.y;
        dx*dx+dy*dy<=radius*radius
    }).collect();
    let alive=inside.iter().filter(|r|r.health>0.0).count();
    let avg=if alive==0 {0.0} else {inside.iter().filter(|r|r.health>0.0).map(|r|r.health).sum::<f32>()/alive as f32};
    let ids:std::collections::BTreeSet<u64>=inside.iter().map(|r|r.id).collect();
    let aware:Vec<&Awareness>=awareness.iter().filter(|(id,_)|ids.contains(id)).map(|(_,a)|a).collect();
    let actions=inside.iter().filter(|r|r.health>0.0).map(|r|r.current_action);
    SettlementView{center,radius,pulse:summarize(alive,aware.into_iter(),actions,culture),alive,average_health:avg}
}

pub fn top_actions(p:&SettlementPulse,n:usize)->Vec<(ActionPrimitive,f32)> {
    let mut v:Vec<_>=p.action_fraction.iter().map(|(&a,&x)|(a,x)).collect();
    v.sort_by(|a,b|b.1.total_cmp(&a.1));v.truncate(n);v
}
pub fn threat_awareness(p:&SettlementPulse)->f32 {
    p.aware_fraction.get(&SituationKind::CreatureThreat).copied().unwrap_or(0.0)
}
