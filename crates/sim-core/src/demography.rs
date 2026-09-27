use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HeritableTraits {
    pub stature:f32,
    pub body_mass:f32,
    pub cold_tolerance:f32,
    pub heat_tolerance:f32,
    pub pigmentation:f32,
    pub disease_resistance:f32,
}

pub fn inherit(a:HeritableTraits,b:HeritableTraits,variation:[f32;6])->HeritableTraits {
    let mix=|x:f32,y:f32,n:f32|(0.5*(x+y)+n.clamp(-1.0,1.0)*0.04).clamp(0.0,1.0);
    HeritableTraits{
        stature:mix(a.stature,b.stature,variation[0]),
        body_mass:mix(a.body_mass,b.body_mass,variation[1]),
        cold_tolerance:mix(a.cold_tolerance,b.cold_tolerance,variation[2]),
        heat_tolerance:mix(a.heat_tolerance,b.heat_tolerance,variation[3]),
        pigmentation:mix(a.pigmentation,b.pigmentation,variation[4]),
        disease_resistance:mix(a.disease_resistance,b.disease_resistance,variation[5]),
    }
}

/// These biological traits describe continuous physical variation.
/// Culture, intelligence, morality, aggression and technology are intentionally not racial stats.
