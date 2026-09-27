use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Soil {
    pub sand: f32,
    pub silt: f32,
    pub clay: f32,
    pub organic_matter: f32,
    pub depth_m: f32,
    pub porosity: f32,
    pub field_capacity: f32,
    pub wilting_point: f32,
    pub saturation: f32,
    pub hydraulic_conductivity_m_day: f32,
    pub cohesion_kpa: f32,
    pub friction_angle_deg: f32,
}

impl Soil {
    pub fn normalize_texture(&mut self) {
        let sum=(self.sand+self.silt+self.clay).max(1e-6);
        self.sand/=sum; self.silt/=sum; self.clay/=sum;
    }
    pub fn plant_available_water(&self)->f32 {
        (self.saturation.clamp(self.wilting_point,self.field_capacity)-self.wilting_point).max(0.0)
    }
    pub fn infiltration_capacity_mm_day(&self)->f32 {
        (self.hydraulic_conductivity_m_day.max(0.0)*1000.0*(0.35+0.65*(1.0-self.saturation.clamp(0.0,1.0)))).max(0.0)
    }
    pub fn slope_safety_factor(&self, slope_rad:f32, unit_weight_kn_m3:f32)->f32 {
        let z=self.depth_m.max(0.05); let theta=slope_rad.clamp(0.001,1.45);
        let normal=unit_weight_kn_m3*z*theta.cos()*theta.cos();
        let driving=unit_weight_kn_m3*z*theta.sin()*theta.cos();
        let pore=self.saturation.clamp(0.,1.)*9.81*z*theta.cos()*theta.cos();
        let resisting=self.cohesion_kpa+(normal-pore).max(0.)*self.friction_angle_deg.to_radians().tan();
        resisting/driving.max(0.001)
    }
}
