use crate::affordances::{FeatureKind,PerceivedFeature};

#[derive(Clone, Copy, Debug)]
pub struct WorldFeature {
    pub id:u64,
    pub kind:FeatureKind,
    pub distance_m:f32,
    pub true_danger:f32,
    pub true_food:f32,
    pub true_material_value:f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Senses {
    pub range_m:f32,
    pub acuity:f32,
    pub danger_attention:f32,
}

pub fn perceive(features:&[WorldFeature],s:Senses,noise:&[f32])->Vec<PerceivedFeature> {
    features.iter().enumerate().filter(|(_,f)|f.distance_m<=s.range_m.max(0.))
        .map(|(i,f)|{
            let distance_loss=(f.distance_m/s.range_m.max(.001)).clamp(0.,1.);
            let precision=(s.acuity*(1.-distance_loss*.7)).clamp(.02,1.);
            let n=noise.get(i).copied().unwrap_or(0.).clamp(-1.,1.);
            let err=(1.-precision)*n;
            PerceivedFeature{
                id:f.id,kind:f.kind,distance_m:f.distance_m,
                danger:(f.true_danger*(1.+err*(1.-s.danger_attention*.5))).clamp(0.,2.),
                food_hint:(f.true_food*(1.+err)).max(0.),
                material_hint:(f.true_material_value*(1.+err)).max(0.),
                uncertainty:(1.-precision).clamp(0.,1.),
            }
        }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn out_of_range_feature_is_not_known() {
        let f=WorldFeature{id:1,kind:FeatureKind::Creature,distance_m:100.,true_danger:1.,true_food:0.,true_material_value:0.};
        assert!(perceive(&[f],Senses{range_m:10.,acuity:1.,danger_attention:1.},&[0.]).is_empty());
    }
}
