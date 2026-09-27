use crate::{agency::Traits, relationships::Relation};

pub fn spend_time(relation:&mut Relation, hours:f32, a:Traits, b:Traits) {
    let h=(hours/24.0).clamp(0.0,1.0);
    let similarity=1.0-((a.aggression-b.aggression).abs()+(a.curiosity-b.curiosity).abs()+(a.risk_tolerance-b.risk_tolerance).abs())/3.0;
    relation.familiarity=(relation.familiarity+0.08*h).clamp(0.0,1.0);
    relation.trust=(relation.trust+(0.03+0.04*similarity)*h).clamp(-1.0,1.0);
    relation.affection=(relation.affection+(0.015+0.04*similarity)*h).clamp(-1.0,1.0);
}

pub fn partnership_affinity(r:Relation,a:Traits,b:Traits)->f32 {
    let social=(r.trust*0.45+r.affection*0.40+r.familiarity*0.15).clamp(-1.0,1.0);
    let compatibility=1.0-((a.social_trust-b.social_trust).abs()+(a.planning_horizon-b.planning_horizon).abs())*0.5;
    (social*0.7+compatibility*0.3).clamp(-1.0,1.0)
}
