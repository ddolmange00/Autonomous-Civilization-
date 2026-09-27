use crate::{agency::ActionPrimitive,culture::{CulturalField,EmergentProfile},settlement_detection::SettlementCluster,world::Position};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettlementIdentity {
    pub id:u64,
    pub center:Position,
    pub founded_year:f64,
    pub last_seen_year:f64,
    pub parent_id:Option<u64>,
    pub members:Vec<u64>,
    pub culture:CulturalField,
    pub shared_food:f32,
    pub shared_material:f32,
    pub knowledge_items:usize,
    pub specialization:BTreeMap<ActionPrimitive,f32>,
}

impl SettlementIdentity {
    pub fn profile(&self)->EmergentProfile {self.culture.profile()}
    pub fn top_specializations(&self,n:usize)->Vec<(ActionPrimitive,f32)> {
        let mut v:Vec<_>=self.specialization.iter().map(|(&a,&x)|(a,x)).collect();
        v.sort_by(|a,b|b.1.total_cmp(&a.1));v.truncate(n);v
    }
}

pub fn nearest_identity(center:Position,settlements:&[SettlementIdentity],max_distance:f32)->Option<usize> {
    settlements.iter().enumerate().filter_map(|(i,s)|{
        let dx=s.center.x-center.x;let dy=s.center.y-center.y;let d=(dx*dx+dy*dy).sqrt();
        (d<=max_distance).then_some((i,d))
    }).min_by(|a,b|a.1.total_cmp(&b.1)).map(|x|x.0)
}

pub fn cluster_member_ids(cluster:&SettlementCluster,resident_ids:&[u64])->Vec<u64> {
    cluster.member_indices.iter().filter_map(|&i|resident_ids.get(i).copied()).collect()
}
