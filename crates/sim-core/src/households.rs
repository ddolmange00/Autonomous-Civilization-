use crate::{design::DesignGenome, world::Position};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Household {
    pub id:u64,
    pub members:Vec<u64>,
    pub home:Position,
    pub stored_food:f32,
    pub shared_material:f32,
    pub cohesion:f32,
    pub migration_goal:Option<Position>,
    pub construction_drive:f32,
    /// Tested design repertoire: observed or invented, capped so lineages stay legible.
    pub designs:Vec<DesignGenome>,
}

impl Household {
    pub fn pressure(&self)->f32 {
        let mouths=self.members.len().max(1) as f32;
        (1.0-self.stored_food/(mouths*20.0)).clamp(0.0,1.0)
    }
    pub fn remember_design(&mut self,design:DesignGenome) {
        if self.designs.iter().any(|d|d.id==design.id){return;}
        if self.designs.len()>=12{self.designs.remove(0);}
        self.designs.push(design);
    }
}
