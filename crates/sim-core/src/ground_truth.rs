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
