use serde::{Deserialize, Serialize};

/// A shared causal story: an unseen-agent explanation for suffering nobody
/// confidently observed. Narratives comfort (small fear relief in gatherings)
/// but never change physical truth.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Narrative {
    pub settlement_id: u64,
    pub label: String,
    pub believers: u32,
    pub origin_year: f64,
    pub comfort: f32,
}

/// Misattribution mass: affected members who lack a confident true-cause report.
/// Returns a 0..1 weight for how strongly the unexplained presses toward story-making.
pub fn misattribution(affected: usize, confident: usize) -> f32 {
    if affected == 0 {
        return 0.0;
    }
    ((affected.saturating_sub(confident)) as f32 / affected as f32).clamp(0.0, 1.0)
}

pub fn reinforce(narratives: &mut Vec<Narrative>, settlement_id: u64, label: String, mass: f32, members: u32, year: f64) {
    if mass <= 0.05 {
        return;
    }
    match narratives.iter_mut().find(|n| n.settlement_id == settlement_id && n.label == label) {
        Some(n) => {
            n.believers = (n.believers + (mass * members as f32 * 0.2) as u32).min(members);
            n.comfort = (n.comfort + 0.02).min(1.0);
        }
        None => {
            narratives.push(Narrative {
                settlement_id,
                label,
                believers: (mass * members as f32 * 0.3) as u32,
                origin_year: year,
                comfort: 0.2,
            });
        }
    }
}

/// Ritual calming: gathering under a shared story eases fear a little.
pub fn ritual_relief(narratives: &[Narrative], settlement_id: u64) -> f32 {
    narratives
        .iter()
        .filter(|n| n.settlement_id == settlement_id)
        .map(|n| 0.03 * n.comfort)
        .fold(0.0_f32, f32::max)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explained_events_make_no_stories() {
        assert_eq!(misattribution(10, 10), 0.0);
        assert_eq!(misattribution(0, 0), 0.0);
    }
    #[test]
    fn unexplained_suffering_seeds_belief() {
        let mut v = Vec::new();
        let mass = misattribution(10, 2);
        assert!(mass > 0.5);
        reinforce(&mut v, 4, "spirits shook the ground".into(), mass, 10, 6.0);
        assert_eq!(v.len(), 1);
        assert!(v[0].believers > 0);
        assert!(ritual_relief(&v, 4) > 0.0);
        assert_eq!(ritual_relief(&v, 5), 0.0);
    }
}
