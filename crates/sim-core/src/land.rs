use crate::{
    climate::{reference_et0_mm_day, ClimateState},
    hydrology::{step_bucket, WaterBalance, WaterFlux},
    plants::{step_day as step_plant, PlantPatch, PlantTraits},
    soil::Soil,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LandPatch {
    pub soil: Soil,
    pub water: WaterBalance,
    pub vegetation: PlantPatch,
}

impl LandPatch {
    pub fn step_day(&mut self, climate:ClimateState, plant:PlantTraits) {
        let storage_capacity_mm=(self.soil.porosity.max(0.01)*self.soil.depth_m.max(0.05)*1000.0) as f64;
        self.soil.saturation=(self.water.soil_water_mm/storage_capacity_mm).clamp(0.0,1.0) as f32;
        let infiltration=self.soil.infiltration_capacity_mm_day() as f64;
        let et0=reference_et0_mm_day(climate) as f64;
        self.water=step_bucket(self.water,WaterFlux{
            rainfall_mm:climate.precipitation_mm_day.max(0.0) as f64,
            infiltration_capacity_mm:infiltration,
            evapotranspiration_mm:et0,
            drainage_fraction:(0.01+self.soil.sand*0.12).clamp(0.0,0.25) as f64,
        });
        self.soil.saturation=(self.water.soil_water_mm/storage_capacity_mm).clamp(0.0,1.0) as f32;
        let available=self.soil.plant_available_water()*self.soil.depth_m.max(0.05)*1000.0;
        self.vegetation=step_plant(self.vegetation,plant,climate.air_temperature_c,available,climate.solar_w_m2.max(0.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn patch()->LandPatch {
        LandPatch{
            soil:Soil{sand:0.4,silt:0.4,clay:0.2,organic_matter:0.03,depth_m:1.0,porosity:0.45,
                field_capacity:0.30,wilting_point:0.12,saturation:0.2,hydraulic_conductivity_m_day:0.08,
                cohesion_kpa:8.0,friction_angle_deg:30.0},
            water:WaterBalance{surface_water_mm:0.0,soil_water_mm:80.0,groundwater_mm:20.0},
            vegetation:PlantPatch{biomass_kg_m2:0.5,maturity:0.1},
        }
    }
    fn plant()->PlantTraits {
        PlantTraits{min_temp_c:4.0,optimum_temp_c:22.0,max_temp_c:40.0,water_demand:120.0,
            light_demand:300.0,max_growth_per_day:0.03,edible_fraction:0.45}
    }
    #[test] fn rain_wets_soil() {
        let mut p=patch(); let before=p.water.soil_water_mm;
        p.step_day(ClimateState{air_temperature_c:18.0,relative_humidity:0.8,wind_m_s:1.0,solar_w_m2:100.0,precipitation_mm_day:40.0},plant());
        assert!(p.water.soil_water_mm>before);
    }
    #[test] fn vegetation_growth_is_environment_limited() {
        let mut good=patch(); let mut frozen=patch();
        good.step_day(ClimateState{air_temperature_c:22.0,relative_humidity:0.6,wind_m_s:1.0,solar_w_m2:500.0,precipitation_mm_day:2.0},plant());
        frozen.step_day(ClimateState{air_temperature_c:-10.0,relative_humidity:0.6,wind_m_s:1.0,solar_w_m2:500.0,precipitation_mm_day:2.0},plant());
        assert!(good.vegetation.biomass_kg_m2>frozen.vegetation.biomass_kg_m2);
    }
}
