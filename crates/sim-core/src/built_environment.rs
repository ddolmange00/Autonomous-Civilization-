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
    pub started_year:f64,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct StructureCapabilities {
    pub shelter:f32,
    pub storage:f32,
    pub workspace:f32,
    pub defense:f32,
    pub observation:f32,
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

impl BuiltStructure {
    pub fn capabilities(&self)->StructureCapabilities {
        capabilities_from(&self.design,self.integrity)
    }
}

pub fn capabilities_from(design:&DesignGenome,integrity:f32)->StructureCapabilities {
    let i=integrity.clamp(0.0,1.0);
    let footprint=(design.length_m*design.width_m).max(0.1);
    let area=(footprint/18.0).clamp(0.0,1.0);
    let thickness=(design.thickness_m/0.35).clamp(0.0,1.0);
    let binding=design.binding_quality.clamp(0.0,1.0);
    let openness=(1.0-design.curvature*0.35).clamp(0.35,1.0);
    StructureCapabilities{
        shelter:(i*(0.40*area+0.40*binding+0.20*thickness)).clamp(0.0,1.0),
        storage:(i*(0.60*area+0.25*binding+0.15*thickness)).clamp(0.0,1.0),
        workspace:(i*(0.65*area+0.20*openness+0.15*binding)).clamp(0.0,1.0),
        defense:(i*(0.55*thickness+0.35*binding+0.10*area)).clamp(0.0,1.0),
        observation:(i*(0.45*(design.length_m/6.0).clamp(0.0,1.0)+0.30*openness+0.25*binding)).clamp(0.0,1.0),
    }
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

pub fn evolve_shelter_design(parent:&DesignGenome,id:u64,skill:f32,material:f32,variation:f32)->DesignGenome {
    let v=variation.clamp(-1.0,1.0);
    let skill=skill.clamp(0.0,1.0);
    DesignGenome{
        id,parent:Some(parent.id),generation:parent.generation+1,function:Function::Shelter,
        length_m:(parent.length_m*(1.0+v*0.08)).clamp(1.2,8.0),
        width_m:(parent.width_m*(1.0-v*0.06)).clamp(1.2,7.0),
        thickness_m:(parent.thickness_m*(1.0+0.10*(1.0-skill)-v*0.05)).clamp(0.05,0.60),
        curvature:(parent.curvature+v*0.08).clamp(0.0,1.0),
        edge_fraction:0.0,
        binding_quality:(parent.binding_quality*0.72+(0.18+0.78*skill)*0.28+v*0.03).clamp(0.0,1.0),
        material_fraction:vec![(1,material.max(0.1))],
    }
}

/// Wall-like defensive design: thick, well-bound, low and straight.
/// Proposed only under real threat pressure, never by script.
pub fn seed_wall_design(id:u64,skill:f32,material:f32)->DesignGenome {
    let s=skill.clamp(0.0,1.0);
    DesignGenome{
        id,parent:None,generation:0,function:Function::Shelter,
        length_m:4.0+2.0*s,
        width_m:0.6+0.4*s,
        thickness_m:0.35+0.20*s,
        curvature:0.0,
        edge_fraction:0.0,
        binding_quality:(0.35+0.60*s).clamp(0.0,1.0),
        material_fraction:vec![(2,material.max(0.1))],
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
    base*1.4*(0.35+skill.clamp(0.0,1.0)*0.9)*days.max(0.0)
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
    #[test] fn stronger_structure_has_more_useful_capabilities() {
        let d=seed_shelter_design(1,None,0,0.8,25.0);
        let good=capabilities_from(&d,0.9);
        let bad=capabilities_from(&d,0.2);
        assert!(good.shelter>bad.shelter);assert!(good.defense>bad.defense);
    }
    #[test] fn descendants_keep_design_lineage() {
        let p=seed_shelter_design(10,None,0,0.4,20.0);
        let c=evolve_shelter_design(&p,11,0.7,25.0,0.2);
        assert_eq!(c.parent,Some(10));assert_eq!(c.generation,1);
    }
    #[test] fn practiced_work_advances_faster() {
        assert!(work_value(ActionPrimitive::Raise,0.9,1.0)>work_value(ActionPrimitive::Raise,0.1,1.0));
    }
    #[test] fn walls_defend_better_than_huts() {
        let hut=seed_shelter_design(1,None,0,0.7,25.0);
        let wall=seed_wall_design(2,0.7,25.0);
        assert!(capabilities_from(&wall,0.8).defense>capabilities_from(&hut,0.8).defense);
    }
}
