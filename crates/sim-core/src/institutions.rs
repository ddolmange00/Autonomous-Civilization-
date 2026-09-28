use crate::agency::ActionPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A recurring coordination pattern that crystallized into a named role.
/// Institutions never force behavior; they make coordinated action cheaper.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Institution {
    pub settlement_id: u64,
    pub action: ActionPrimitive,
    pub strength: f32,
    pub since_year: f64,
}

/// Minimum members acting alike in one step before a pattern counts as coordination.
fn quorum(members: usize) -> usize {
    (members / 3).max(3)
}

pub fn update_institutions(
    settlement_id: u64,
    action_counts: &BTreeMap<ActionPrimitive, u32>,
    members: usize,
    existing: &mut Vec<Institution>,
    year: f64,
) {
    let q = quorum(members) as u32;
    for (&action, &n) in action_counts {
        if n < q {
            continue;
        }
        match existing.iter_mut().find(|i| i.settlement_id == settlement_id && i.action == action) {
            Some(inst) => {
                inst.strength = (inst.strength + 0.05).min(1.0);
            }
            None => {
                existing.push(Institution { settlement_id, action, strength: 0.2, since_year: year });
            }
        }
    }
    // Unused institutions fade but are not forgotten instantly.
    for inst in existing.iter_mut().filter(|i| i.settlement_id == settlement_id) {
        if action_counts.get(&inst.action).copied().unwrap_or(0) < q {
            inst.strength = (inst.strength - 0.002).max(0.0);
        }
    }
    existing.retain(|i| i.strength > 0.0 || i.settlement_id != settlement_id);
}

/// Coordination bonus for a resident acting with an institution behind them.
pub fn coordination_bonus(institutions: &[Institution], settlement_id: u64, action: ActionPrimitive) -> f32 {
    institutions
        .iter()
        .find(|i| i.settlement_id == settlement_id && i.action == action)
        .map(|i| 0.25 * i.strength)
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repeated_coordination_crystallizes() {
        let mut v = Vec::new();
        let mut counts = BTreeMap::new();
        counts.insert(ActionPrimitive::Raise, 4);
        update_institutions(9, &counts, 9, &mut v, 3.0);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].action, ActionPrimitive::Raise);
        update_institutions(9, &counts, 9, &mut v, 3.1);
        assert!(v[0].strength > 0.2);
    }
    #[test]
    fn lone_actors_form_nothing() {
        let mut v = Vec::new();
        let mut counts = BTreeMap::new();
        counts.insert(ActionPrimitive::Raise, 1);
        update_institutions(9, &counts, 9, &mut v, 3.0);
        assert!(v.is_empty());
    }
    #[test]
    fn bonus_scales_with_strength() {
        let v = vec![Institution { settlement_id: 9, action: ActionPrimitive::Raise, strength: 0.8, since_year: 1.0 }];
        assert!((coordination_bonus(&v, 9, ActionPrimitive::Raise) - 0.2).abs() < 1e-5);
        assert_eq!(coordination_bonus(&v, 9, ActionPrimitive::Dig), 0.0);
        assert_eq!(coordination_bonus(&v, 10, ActionPrimitive::Raise), 0.0);
    }
}
