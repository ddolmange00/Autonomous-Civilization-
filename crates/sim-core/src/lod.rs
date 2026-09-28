use serde::{Deserialize, Serialize};

/// Off-screen population: residents the simulation does not step individually.
/// Cohorts eat from the same pools, grow logistically with plenty, and starve
/// with famine. They are statistics with appetites, not decoration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cohort {
    pub settlement_id: u64,
    pub count: f32,
}

impl Cohort {
    /// Granary-coupled carrying capacity: stored surplus supports multitudes.
    pub fn capacity(storage_sum: f32) -> f32 {
        150.0 * (1.0 + storage_sum.max(0.0) * 6.0)
    }
    /// Yearly update. Plenty grows toward capacity, famine shrinks.
    /// Returns net food consumed (positive) from the settlement pool.
    pub fn step_year(&mut self, food_per_capita: f32, storage_sum: f32) -> f32 {
        let cap = Self::capacity(storage_sum).max(1.0);
        let plenty = (food_per_capita / 20.0).clamp(0.0, 1.5);
        let growth = self.count * 0.06 * plenty * (1.0 - self.count / cap);
        let famine = if food_per_capita < 4.0 { self.count * 0.10 * (1.0 - food_per_capita / 4.0) } else { 0.0 };
        self.count = (self.count + growth - famine).max(0.0).min(cap * 1.2);
        // Mouths eat: ~0.9 food-units per head per year against a 20-unit reserve scale.
        self.count * 0.9 / 365.0
    }
}

pub fn represented_population(simulated_alive: usize, cohorts: &[Cohort]) -> f64 {
    simulated_alive as f64 + cohorts.iter().map(|c| c.count as f64).sum::<f64>()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plenty_grows_famine_shrinks() {
        let mut fat = Cohort { settlement_id: 1, count: 100.0 };
        fat.step_year(30.0, 1.0);
        assert!(fat.count > 100.0);
        let mut thin = Cohort { settlement_id: 2, count: 100.0 };
        thin.step_year(0.0, 0.0);
        assert!(thin.count < 100.0);
    }
    #[test]
    fn granaries_raise_the_ceiling() {
        assert!(Cohort::capacity(2.0) > Cohort::capacity(0.0));
        let mut c = Cohort { settlement_id: 1, count: 140.0 };
        for _ in 0..40 {
            c.step_year(40.0, 0.0);
        }
        assert!(c.count <= Cohort::capacity(0.0) * 1.2 + 1.0);
    }
    #[test]
    fn mouths_eat() {
        let mut c = Cohort { settlement_id: 1, count: 100.0 };
        assert!(c.step_year(30.0, 1.0) > 0.0);
    }
}
