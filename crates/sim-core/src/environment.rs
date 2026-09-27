use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct EnvironmentCell {
    pub elevation_m: f32,
    pub slope: f32,
    pub temperature_c: f32,
    pub humidity: f32,
    pub rainfall_mm_year: f32,
    pub wind_m_s: f32,
    pub soil_depth_m: f32,
    pub soil_fertility: f32,
    pub water_depth_m: f32,
    pub current_m_s: f32,
    pub salinity: f32,
    pub vegetation: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Barrier { None, DeepWater, Cliff, LethalTemperature }

impl EnvironmentCell {
    pub fn barrier(self) -> Barrier {
        if self.water_depth_m > 1.3 { return Barrier::DeepWater; }
        if self.slope > 0.78 { return Barrier::Cliff; }
        if self.temperature_c < -55.0 || self.temperature_c > 58.0 { return Barrier::LethalTemperature; }
        Barrier::None
    }
}
