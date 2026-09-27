use crate::{agency::{ActionPrimitive,Affordance},life_history::LifeStage};

pub fn action_allowed(stage:LifeStage,action:ActionPrimitive)->bool {
    match stage {
        LifeStage::Infant=>matches!(action,ActionPrimitive::Observe|ActionPrimitive::Communicate|ActionPrimitive::Consume|ActionPrimitive::Rest|ActionPrimitive::Hide),
        LifeStage::Child=>!matches!(action,ActionPrimitive::Attack|ActionPrimitive::Heat|ActionPrimitive::Pierce|ActionPrimitive::Strike),
        LifeStage::Adolescent=>true,
        LifeStage::Adult=>true,
        LifeStage::Elder=>true,
    }
}

pub fn filter_affordances(stage:LifeStage,affordances:&mut Vec<Affordance>) {
    affordances.retain(|a|action_allowed(stage,a.action));
}

pub fn mobility_factor(stage:LifeStage)->f32 {
    match stage {LifeStage::Infant=>0.10,LifeStage::Child=>0.65,LifeStage::Adolescent=>0.95,LifeStage::Adult=>1.0,LifeStage::Elder=>0.72}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn infants_cannot_attack(){assert!(!action_allowed(LifeStage::Infant,ActionPrimitive::Attack));}
    #[test] fn adults_can_experiment(){assert!(action_allowed(LifeStage::Adult,ActionPrimitive::Experiment));}
}
