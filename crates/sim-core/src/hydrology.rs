use crate::physics_laws::manning_velocity_m_s;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct WaterBalance {
    pub surface_water_mm: f64,
    pub soil_water_mm: f64,
    pub groundwater_mm: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct WaterFlux {
    pub rainfall_mm: f64,
    pub infiltration_capacity_mm: f64,
    pub evapotranspiration_mm: f64,
    pub drainage_fraction: f64,
}

pub fn step_bucket(mut s:WaterBalance, f:WaterFlux)->WaterBalance {
    let rain=f.rainfall_mm.max(0.0);
    let infiltration=rain.min(f.infiltration_capacity_mm.max(0.0));
    s.surface_water_mm+=(rain-infiltration).max(0.0);
    s.soil_water_mm+=infiltration;
    let et=f.evapotranspiration_mm.max(0.0).min(s.soil_water_mm);
    s.soil_water_mm-=et;
    let drainage=(s.soil_water_mm*f.drainage_fraction.clamp(0.0,1.0)).max(0.0);
    s.soil_water_mm-=drainage;
    s.groundwater_mm+=drainage;
    s
}

pub fn open_channel_discharge_m3_s(area_m2:f64, hydraulic_radius_m:f64, slope:f64, roughness_n:f64)->f64 {
    area_m2.max(0.0)*manning_velocity_m_s(hydraulic_radius_m,slope,roughness_n)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn bucket_conserves_rain_without_et() {
        let a=WaterBalance::default();
        let b=step_bucket(a,WaterFlux{rainfall_mm:100.,infiltration_capacity_mm:40.,evapotranspiration_mm:0.,drainage_fraction:0.25});
        let total=b.surface_water_mm+b.soil_water_mm+b.groundwater_mm;
        assert!((total-100.).abs()<1e-9);
    }
}
