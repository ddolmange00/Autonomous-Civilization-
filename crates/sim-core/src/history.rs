use crate::causal_log::CausalNode;
use serde::{Deserialize, Serialize};

/// Compacted era: routine history aggregates into counts so deep time stays
/// affordable. Significant nodes are counted, not kept verbatim.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EraSummary {
    pub start_year: f64,
    pub end_year: f64,
    pub births: u32,
    pub deaths: u32,
    pub battles: u32,
    pub trades: u32,
    pub built: u32,
    pub contacts: u32,
    pub other: u32,
}

fn classify(node: &CausalNode, s: &mut EraSummary) {
    match node {
        CausalNode::Outcome { label, .. } => {
            if label.starts_with("born to") {
                s.births += 1;
            } else if label.starts_with("died at age") {
                s.deaths += 1;
            } else if label.contains("warband") || label.contains("repulsed") {
                s.battles += 1;
            } else if label.contains("traded") {
                s.trades += 1;
            } else if label.contains("construction") && label.contains("completed") {
                s.built += 1;
            } else if label.contains("contact") {
                s.contacts += 1;
            } else {
                s.other += 1;
            }
        }
        _ => {
            s.other += 1;
        }
    }
}

/// Fold drained nodes into one era summary.
pub fn compact(nodes: &[(f64, CausalNode)]) -> EraSummary {
    let mut s = EraSummary::default();
    if let Some((first, _)) = nodes.first() {
        s.start_year = *first;
    }
    if let Some((last, _)) = nodes.last() {
        s.end_year = *last;
    }
    for (_, n) in nodes {
        classify(n, &mut s);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compaction_counts_without_keeping_verbatim() {
        let nodes = vec![
            (1.0, CausalNode::Outcome { resident_id: Some(1), label: "born to 2 and 3".into(), value: 1.0 }),
            (1.5, CausalNode::Outcome { resident_id: Some(2), label: "died at age 41.2".into(), value: -1.0 }),
            (2.0, CausalNode::Decision { resident_id: 1, action: crate::agency::ActionPrimitive::Gather, score: 0.0 }),
        ];
        let era = compact(&nodes);
        assert_eq!(era.births, 1);
        assert_eq!(era.deaths, 1);
        assert_eq!(era.other, 1);
        assert_eq!(era.start_year, 1.0);
        assert_eq!(era.end_year, 2.0);
    }
}
