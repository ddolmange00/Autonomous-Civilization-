use crate::agency::ActionPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Episode {
    pub year:f64,
    pub action:ActionPrimitive,
    pub target:Option<u64>,
    pub value:f32,
    pub surprise:f32,
    pub danger:f32,
    pub social_visibility:f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EpisodicMemory {
    pub episodes:Vec<Episode>,
    pub capacity:usize,
}

impl EpisodicMemory {
    pub fn remember(&mut self,e:Episode) {
        if self.capacity==0 { self.capacity=64; }
        self.episodes.push(e);
        if self.episodes.len()>self.capacity {
            let remove=self.episodes.len()-self.capacity;
            self.episodes.drain(0..remove);
        }
    }
    pub fn recalled_value(&self,action:ActionPrimitive,current_year:f64)->f32 {
        let mut weighted=0.; let mut total=0.;
        for e in self.episodes.iter().filter(|e|e.action==action) {
            let age=(current_year-e.year).max(0.) as f32;
            let recency=(-age/20.).exp();
            let salience=1.+e.surprise*.7+e.danger*.5+e.social_visibility*.2;
            let w=recency*salience;
            weighted+=e.value*w; total+=w;
        }
        if total>0. {weighted/total} else {0.}
    }
}
