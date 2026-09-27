use crate::agency::ActionPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CausalNode {
    WorldEvent{event_id:u64,label:String},
    Perception{resident_id:u64,label:String},
    Decision{resident_id:u64,action:ActionPrimitive,score:f32},
    Outcome{resident_id:Option<u64>,label:String,value:f32},
    Transmission{from:u64,to:u64,label:String},
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CausalLog {
    pub nodes:Vec<(f64,CausalNode)>,
    pub capacity:usize,
}
impl CausalLog {
    pub fn push(&mut self,year:f64,node:CausalNode) {
        if self.capacity==0 {self.capacity=2048;}
        self.nodes.push((year,node));
        if self.nodes.len()>self.capacity {let n=self.nodes.len()-self.capacity;self.nodes.drain(0..n);}
    }
}
