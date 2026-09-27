use sim_core::{culture::CulturalField,settlement_identity::{nearest_identity,SettlementIdentity},world::Position};
use std::collections::BTreeMap;

fn village(id:u64,x:f32)->SettlementIdentity {
    SettlementIdentity{id,center:Position{x,y:0.0},founded_year:0.0,last_seen_year:0.0,parent_id:None,members:vec![],
        culture:CulturalField::default(),shared_food:0.0,shared_material:0.0,knowledge_items:0,specialization:BTreeMap::new()}
}

#[test]
fn small_center_drift_keeps_same_identity() {
    let v=vec![village(10,0.0),village(20,200.0)];
    assert_eq!(nearest_identity(Position{x:22.0,y:8.0},&v,90.0),Some(0));
}
#[test]
fn distant_fission_can_become_new_identity() {
    let v=vec![village(10,0.0)];
    assert_eq!(nearest_identity(Position{x:160.0,y:0.0},&v,90.0),None);
}
