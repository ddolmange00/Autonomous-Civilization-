pub const STANDARD_GRAVITY: f64 = 9.80665;
pub const UNIVERSAL_GAS_CONSTANT: f64 = 8.314_462_618_153_24;

pub fn weight_kg(mass_kg:f64)->f64 { mass_kg*STANDARD_GRAVITY }
pub fn kinetic_energy_j(mass_kg:f64, velocity_m_s:f64)->f64 { 0.5*mass_kg*velocity_m_s*velocity_m_s }
pub fn momentum(mass_kg:f64, velocity_m_s:f64)->f64 { mass_kg*velocity_m_s }
pub fn stress_pa(force_n:f64, area_m2:f64)->f64 { force_n/area_m2.max(1e-12) }
pub fn elastic_strain(stress_pa:f64, youngs_modulus_pa:f64)->f64 { stress_pa/youngs_modulus_pa.max(1e-12) }
pub fn hydrostatic_pressure_pa(surface_pa:f64, density:f64, depth_m:f64)->f64 {
    surface_pa+density*STANDARD_GRAVITY*depth_m.max(0.0)
}
pub fn buoyancy_n(fluid_density:f64, displaced_volume_m3:f64)->f64 {
    fluid_density*STANDARD_GRAVITY*displaced_volume_m3.max(0.0)
}
pub fn drag_n(density:f64, velocity_m_s:f64, drag_coefficient:f64, area_m2:f64)->f64 {
    0.5*density*velocity_m_s*velocity_m_s*drag_coefficient.max(0.0)*area_m2.max(0.0)
}
pub fn ideal_gas_pressure_pa(moles:f64, temperature_k:f64, volume_m3:f64)->f64 {
    moles.max(0.0)*UNIVERSAL_GAS_CONSTANT*temperature_k.max(0.0)/volume_m3.max(1e-12)
}
pub fn heat_energy_j(mass_kg:f64, specific_heat_j_kg_k:f64, delta_t_k:f64)->f64 {
    mass_kg*specific_heat_j_kg_k*delta_t_k
}
pub fn manning_velocity_m_s(hydraulic_radius_m:f64, slope:f64, roughness_n:f64)->f64 {
    hydraulic_radius_m.max(0.0).powf(2.0/3.0)*slope.max(0.0).sqrt()/roughness_n.max(1e-6)
}
pub fn arrhenius_rate(pre_exponential:f64, activation_energy_j_mol:f64, temperature_k:f64)->f64 {
    pre_exponential*(-activation_energy_j_mol/(UNIVERSAL_GAS_CONSTANT*temperature_k.max(1e-9))).exp()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn water_buoyancy_is_about_9810_n_per_cubic_meter() {
        assert!((buoyancy_n(1000.0,1.0)-9806.65).abs()<0.1);
    }
    #[test] fn drag_scales_with_velocity_squared() {
        let a=drag_n(1.2,10.,1.,1.); let b=drag_n(1.2,20.,1.,1.);
        assert!((b/a-4.0).abs()<1e-10);
    }
    #[test] fn rougher_channel_flows_slower() {
        assert!(manning_velocity_m_s(1.,.01,.03)>manning_velocity_m_s(1.,.01,.06));
    }
}
