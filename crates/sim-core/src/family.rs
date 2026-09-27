use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Kinship {
    pub parents:Vec<u64>,
    pub children:Vec<u64>,
    pub partners:Vec<u64>,
    pub household:Option<u64>,
}

pub fn relatedness(a:&Kinship,b_id:u64)->f32 {
    if a.parents.contains(&b_id)||a.children.contains(&b_id){0.5}
    else if a.partners.contains(&b_id){0.35}
    else {0.0}
}
