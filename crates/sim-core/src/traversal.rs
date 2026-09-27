use crate::environment::Barrier;

#[derive(Clone, Copy, Debug, Default)]
pub struct Capabilities {
    pub flotation: f32,
    pub propulsion: f32,
    pub steering: f32,
    pub anchoring: f32,
    pub climbing: f32,
    pub thermal_protection: f32,
}

pub fn can_cross(barrier: Barrier, c: Capabilities, severity: f32) -> bool {
    match barrier {
        Barrier::None => true,
        Barrier::DeepWater => c.flotation >= severity && c.propulsion >= severity * 0.45 && c.steering >= severity * 0.25,
        Barrier::Cliff => c.climbing >= severity && c.anchoring >= severity * 0.55,
        Barrier::LethalTemperature => c.thermal_protection >= severity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn deep_water_requires_real_capability() {
        assert!(!can_cross(Barrier::DeepWater, Capabilities::default(), 0.5));
        assert!(can_cross(Barrier::DeepWater, Capabilities { flotation:0.8, propulsion:0.5, steering:0.3, ..Default::default() }, 0.5));
    }
    #[test] fn cliff_requires_climbing_and_anchor() {
        assert!(!can_cross(Barrier::Cliff, Capabilities { climbing:1.0, ..Default::default() }, 0.6));
        assert!(can_cross(Barrier::Cliff, Capabilities { climbing:0.8, anchoring:0.5, ..Default::default() }, 0.6));
    }
}
