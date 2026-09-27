use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct KnowledgeItem { pub confidence:f32, pub value:f32, pub evidence:u32 }

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KnowledgeStore { pub items:BTreeMap<String,KnowledgeItem> }

impl KnowledgeStore {
    pub fn learn(&mut self,key:String,value:f32,confidence:f32) {
        let k=self.items.entry(key).or_default();let w=confidence.clamp(0.01,1.0);
        k.value=if k.evidence==0{value}else{k.value*(1.0-w)+value*w};
        k.confidence=(k.confidence+(1.0-k.confidence)*w*0.35).clamp(0.0,1.0);k.evidence+=1;
    }
    pub fn transmit_to(&self,other:&mut KnowledgeStore,trust:f32,distortion:f32) {
        for (key,item) in &self.items {
            let value=item.value*(1.0+distortion.clamp(-0.5,0.5));
            other.learn(key.clone(),value,item.confidence*trust.clamp(0.0,1.0)*0.75);
        }
    }
}
