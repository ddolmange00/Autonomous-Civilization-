use crate::{materials::MaterialProperties, provenance::PropertyDatum};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialGroundTruth {
    pub material_id:u32,
    pub canonical_name:String,
    pub composition_note:String,
    pub properties:MaterialProperties,
    pub provenance:Vec<PropertyDatum>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GroundTruthRegistry {
    pub materials:Vec<MaterialGroundTruth>,
}

impl GroundTruthRegistry {
    pub fn get(&self,id:u32)->Option<&MaterialGroundTruth> {
        self.materials.iter().find(|m|m.material_id==id)
    }
}

fn datum(property:&str,value:f32)->PropertyDatum {
    use crate::provenance::{EvidenceKind, SourceMetadata};
    PropertyDatum{
        property:property.into(),value_si:value as f64,
        lower_si:Some(value as f64*0.8),upper_si:Some(value as f64*1.2),
        original_unit:"game-normalized 0..1".into(),temperature_k:None,pressure_pa:None,
        moisture_fraction:None,process_condition:None,
        source:SourceMetadata{source_id:"canonical-game-truth".into(),
            citation:"Autonomous Civilization canonical material table".into(),
            url_or_doi:"".into(),version_or_access_date:"v08".into(),
            license_note:"MIT (project-owned)".into(),evidence_kind:EvidenceKind::GameCalibrated},
        transformation_note:Some("normalized for design evaluation".into()),fidelity_tier:3,
    }
}

fn entry(id:u32,name:&str,note:&str,p:MaterialProperties)->MaterialGroundTruth {
    let props=[
        ("density",p.density),("tensile_strength",p.tensile_strength),
        ("compressive_strength",p.compressive_strength),("fracture_toughness",p.fracture_toughness),
        ("hardness",p.hardness),("elastic_modulus",p.elastic_modulus),
        ("flexibility",p.flexibility),("friction",p.friction),
        ("buoyancy_factor",p.buoyancy_factor),("workability",p.workability),
    ];
    MaterialGroundTruth{material_id:id,canonical_name:name.into(),composition_note:note.into(),
        properties:p,provenance:props.iter().map(|(k,v)|datum(k,*v)).collect()}
}

/// Canonical world truth: timber, stone, fiber. Hidden from civilizations;
/// knowledge about them is local, uncertain and revisable (constitution #3).
pub fn canonical_registry()->GroundTruthRegistry {
    let timber=MaterialProperties{density:0.6,tensile_strength:0.55,compressive_strength:0.45,
        fracture_toughness:0.5,hardness:0.3,elastic_modulus:0.5,flexibility:0.5,friction:0.6,
        thermal_conductivity:0.3,ignition_temperature:0.6,corrosion_resistance:0.5,
        water_absorption:0.5,permeability:0.3,buoyancy_factor:0.9,workability:0.8};
    let stone=MaterialProperties{density:0.9,tensile_strength:0.2,compressive_strength:0.9,
        fracture_toughness:0.6,hardness:0.85,elastic_modulus:0.8,flexibility:0.05,friction:0.7,
        thermal_conductivity:0.5,ignition_temperature:1.0,corrosion_resistance:0.9,
        water_absorption:0.1,permeability:0.05,buoyancy_factor:0.05,workability:0.25};
    let fiber=MaterialProperties{density:0.2,tensile_strength:0.5,compressive_strength:0.1,
        fracture_toughness:0.3,hardness:0.1,elastic_modulus:0.3,flexibility:0.9,friction:0.5,
        thermal_conductivity:0.4,ignition_temperature:0.4,corrosion_resistance:0.4,
        water_absorption:0.7,permeability:0.6,buoyancy_factor:0.6,workability:0.9};
    GroundTruthRegistry{materials:vec![
        entry(1,"timber","structural wood; floats, works easily",timber),
        entry(2,"stone","hard rock; cuts well, sinks",stone),
        entry(3,"fiber","cordage and binding; flexible",fiber),
    ]}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn registry_holds_three_canonical_materials() {
        let r=canonical_registry();
        assert_eq!(r.materials.len(),3);
        assert!(r.get(1).is_some()&&r.get(2).is_some()&&r.get(3).is_some());
        assert!(r.get(1).unwrap().properties.buoyancy_factor>0.5);
        assert!(r.get(2).unwrap().properties.hardness>0.5);
    }
}
