use crate::blueprints::MonsterBlueprint;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BlueprintLibrary {
    pub monsters:Vec<MonsterBlueprint>,
    pub capacity:usize,
}
impl BlueprintLibrary {
    pub fn save(&mut self,blueprint:MonsterBlueprint) {
        if self.capacity==0 {self.capacity=32;}
        if let Some(i)=self.monsters.iter().position(|b|b.id==blueprint.id){self.monsters[i]=blueprint;return;}
        self.monsters.push(blueprint);
        if self.monsters.len()>self.capacity {self.monsters.remove(0);}
    }
    pub fn get(&self,id:u64)->Option<&MonsterBlueprint>{self.monsters.iter().find(|b|b.id==id)}
    pub fn duplicate(&mut self,id:u64,new_id:u64,new_name:String)->Option<MonsterBlueprint>{
        let mut b=self.get(id)?.clone();b.id=new_id;b.name=new_name;self.save(b.clone());Some(b)
    }
}
