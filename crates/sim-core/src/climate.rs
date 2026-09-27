use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ClimateState {
    pub air_temperature_c: f32,
    pub relative_humidity: f32,
    pub wind_m_s: f32,
    pub solar_w_m2: f32,
    pub precipitation_mm_day: f32,
}

pub fn saturation_vapor_pressure_kpa(temp_c:f32)->f32 {
    0.6108*((17.27*temp_c)/(temp_c+237.3)).exp()
}

pub fn vapor_pressure_deficit_kpa(c:ClimateState)->f32 {
    saturation_vapor_pressure_kpa(c.air_temperature_c)*(1.0-c.relative_humidity.clamp(0.,1.))
}

pub fn reference_et0_mm_day(c:ClimateState)->f32 {
    // Lightweight FAO Penman-Monteith-inspired game kernel; calibrated, not a replacement for FAO-56.
    let t=c.air_temperature_c;
    let es=saturation_vapor_pressure_kpa(t);
    let delta=4098.0*es/(t+237.3).powi(2);
    let gamma=0.066;
    let rn=(c.solar_w_m2.max(0.)*0.0864*0.77).max(0.); // MJ/m2/day effective net radiation
    let vpd=vapor_pressure_deficit_kpa(c);
    let u2=c.wind_m_s.max(0.05);
    ((0.408*delta*rn + gamma*(900.0/(t+273.0))*u2*vpd)/(delta+gamma*(1.0+0.34*u2))).max(0.)
}
