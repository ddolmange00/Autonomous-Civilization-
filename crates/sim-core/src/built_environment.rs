use crate::{agency::ActionPrimitive, design::{DesignGenome,Function}, world::Position};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConstructionProject {
    pub id:u64,
    pub household_id:u64,
    pub position:Position,
    pub design:DesignGenome,
    pub progress:f32,
    pub required_work:f32,
    pub material_committed:f32,
    pub material_required:f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuiltStructure {
    pub id:u64,
    pub household_id:u64,
    pub position:Position,
    pub design:DesignGenome,
    pub integrity:f32,
    pub completed_year:f64,
    pub material_invested:f32,
}

pub fn proposal_strength(safety_need:f32,rest_need:f32,build_skill:f32,material:f32)->f32 {
    let need=(safety_need*0.65+rest_need*0.35).clamp(0.0,1.0);
    let capability=(0.15+build_skill*0.85).clamp(0.0,1.0);
    let supply=(material/30.0).clamp(0.0,1.0);
    need*capability*supply
}

pub fn seed_shelter_design(id:u64,parent:Option<u64>,generation:u32,skill:f32,material:f32)->DesignGenome {
    let sophistication=skill.clamp(0.0,1.0);
    DesignGenome{
        id,parent,generation,function:Function::Shelter,
        length_m:2.0+2.8*sophistication,
        width_m:1.8+2.2*sophistication,
        thickness_m:0.08+0.22*(1.0-sophistication*0.4),
        curvature:0.05+0.35*sophistication,
        edge_fraction:0.0,
        binding_quality:(0.15+0.75*sophistication).clamp(0.0,1.0),
        material_fraction:vec![(1,material.max(0.1))],
    }
}

pub fn work_value(action:ActionPrimitive,skill:f32,days:f32)->f32 {
    let base=match action {
        ActionPrimitive::Bind=>1.0,
        ActionPrimitive::Raise=>1.15,
        ActionPrimitive::Dig=>0.75,
        ActionPrimitive::Carry=>0.55,
        ActionPrimitive::Experiment=>0.30,
        _=>0.0,
    };
    base*(0.35+skill.clamp(0.0,1.0)*0.9)*days.max(0.0)
}

pub fn integrity_from(design:&DesignGenome,build_skill:f32,material_ratio:f32)->f32 {
    let geometry=(0.35+design.binding_quality*0.35+(design.thickness_m/0.30).clamp(0.0,1.0)*0.30).clamp(0.0,1.0);
    (geometry*0.45+build_skill.clamp(0.0,1.0)*0.35+material_ratio.clamp(0.0,1.0)*0.20).clamp(0.05,1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn skilled_supplied_household_is_more_likely_to_build() {
        assert!(proposal_strength(0.8,0.7,0.8,40.0)>proposal_strength(0.8,0.7,0.1,3.0));
    }
    #[test] fn practiced_work_advances_faster() {
        assert!(work_value(ActionPrimitive::Raise,0.9,1.0)>work_value(ActionPrimitive::Raise,0.1,1.0));
    }
}
