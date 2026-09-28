use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TradeRecord {
    pub year: f64,
    pub from_settlement: u64,
    pub to_settlement: u64,
    pub food: f32,
    pub material: f32,
}

/// Surplus above a per-member reserve. Negative means deficit.
pub fn surplus(stock: f32, members: usize, reserve_per_member: f32) -> f32 {
    stock - members.max(1) as f32 * reserve_per_member
}

/// Exchange amount from giver surplus toward taker deficit.
/// Nothing moves without a real differential: no scripted trade routes.
pub fn exchange_amount(giver_surplus: f32, taker_deficit: f32, carry_limit: f32) -> f32 {
    if giver_surplus <= 0.0 || taker_deficit <= 0.0 {
        return 0.0;
    }
    giver_surplus.min(taker_deficit).min(carry_limit.max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn no_differential_no_trade() {
        assert_eq!(exchange_amount(10.0, 0.0, 5.0), 0.0);
        assert_eq!(exchange_amount(0.0, 10.0, 5.0), 0.0);
        assert_eq!(exchange_amount(-3.0, 10.0, 5.0), 0.0);
    }
    #[test]
    fn trade_is_capped_by_need_and_carry() {
        assert_eq!(exchange_amount(100.0, 4.0, 50.0), 4.0);
        assert_eq!(exchange_amount(100.0, 40.0, 5.0), 5.0);
        assert_eq!(exchange_amount(3.0, 40.0, 50.0), 3.0);
    }
    #[test]
    fn surplus_accounts_for_mouths() {
        assert!(surplus(100.0, 10, 20.0) < 0.0);
        assert!(surplus(300.0, 10, 20.0) > 0.0);
    }
}
