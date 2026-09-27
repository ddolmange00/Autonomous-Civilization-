use std::f64::consts::PI;

pub fn axial_stress_pa(force_n:f64,area_m2:f64)->f64 { force_n/area_m2.max(1e-12) }
pub fn bending_stress_pa(moment_n_m:f64,outer_distance_m:f64,second_moment_m4:f64)->f64 {
    moment_n_m*outer_distance_m/second_moment_m4.max(1e-18)
}
pub fn euler_buckling_load_n(youngs_modulus_pa:f64,second_moment_m4:f64,effective_length_m:f64)->f64 {
    PI*PI*youngs_modulus_pa*second_moment_m4/effective_length_m.max(1e-9).powi(2)
}
pub fn safety_factor(capacity:f64,demand:f64)->f64 { capacity.max(0.)/demand.max(1e-12) }

#[derive(Clone, Copy, Debug, Default)]
pub struct FatigueState { pub damage:f64 }

impl FatigueState {
    pub fn apply_cycles(&mut self,cycles:f64,cycles_to_failure:f64) {
        self.damage+=(cycles.max(0.)/cycles_to_failure.max(1.)).max(0.);
    }
    pub fn failed(&self)->bool { self.damage>=1.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn longer_column_buckles_more_easily() {
        let short=euler_buckling_load_n(10e9,1e-5,1.);
        let long=euler_buckling_load_n(10e9,1e-5,2.);
        assert!((short/long-4.).abs()<1e-9);
    }
    #[test] fn miners_rule_accumulates_damage() {
        let mut f=FatigueState::default(); f.apply_cycles(500.,1000.); f.apply_cycles(500.,1000.);
        assert!(f.failed());
    }
}
