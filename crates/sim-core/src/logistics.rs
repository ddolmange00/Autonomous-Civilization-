/// Campaign logistics: what a body can haul and what distance costs.
/// Supply lines are physics, not interface: far fighters go hungry.
#[derive(Clone, Copy, Debug)]
pub struct CarryCapacity {
    pub food: f32,
    pub material: f32,
}

/// Hauling scales with stature and practiced carrying.
pub fn carry_capacity(stature: f32, carry_skill: f32) -> CarryCapacity {
    let base = 3.0 + stature.clamp(0.0, 1.0) * 4.0 + carry_skill.clamp(0.0, 1.0) * 6.0;
    CarryCapacity { food: base, material: base * 0.8 }
}

/// Extra hunger per day for operating far from home supply.
pub fn supply_drain(distance_from_home: f32, days: f32) -> f32 {
    let over = (distance_from_home - 60.0).max(0.0);
    (over / 60.0).min(2.0) * 0.004 * days
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn skill_hauls_more() {
        assert!(carry_capacity(0.5, 0.9).food > carry_capacity(0.5, 0.0).food);
    }
    #[test]
    fn home_ground_costs_nothing() {
        assert_eq!(supply_drain(30.0, 5.0), 0.0);
        assert!(supply_drain(180.0, 5.0) > 0.0);
    }
}
