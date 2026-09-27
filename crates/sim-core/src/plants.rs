use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PlantTraits {
    pub min_temp_c:f32, pub optimum_temp_c:f32, pub max_temp_c:f32,
    pub water_demand:f32, pub light_demand:f32,
    pub max_growth_per_day:f32, pub edible_fraction:f32,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct PlantPatch { pub biomass_kg_m2:f32, pub maturity:f32 }

fn triangular(x:f32,lo:f32,opt:f32,hi:f32)->f32 {
    if x<=lo || x>=hi {0.} else if x<=opt {(x-lo)/(opt-lo).max(1e-5)} else {(hi-x)/(hi-opt).max(1e-5)}
}

pub fn step_day(mut p:PlantPatch,t:PlantTraits,temp_c:f32,available_water:f32,light:f32)->PlantPatch {
    let tf=triangular(temp_c,t.min_temp_c,t.optimum_temp_c,t.max_temp_c);
    let wf=(available_water/t.water_demand.max(1e-5)).clamp(0.,1.);
    let lf=(light/t.light_demand.max(1e-5)).clamp(0.,1.);
    let limiting=tf.min(wf).min(lf);
    p.biomass_kg_m2=(p.biomass_kg_m2+t.max_growth_per_day*limiting*(1.0-p.maturity*0.35)).max(0.);
    p.maturity=(p.maturity+0.003*limiting).clamp(0.,1.);
    p
}
