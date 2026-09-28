use crate::diplomacy::{Contact, Stance};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A civilization is a derived grouping, never an assigned archetype:
/// settlements joined by fission lineage or sustained friendly contact.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Civilization {
    pub id: u64,
    pub settlement_ids: Vec<u64>,
    pub formed_year: f64,
}

fn find(parents: &mut BTreeMap<u64, u64>, x: u64) -> u64 {
    let mut root = x;
    while let Some(&p) = parents.get(&root) {
        if p == root {
            break;
        }
        root = p;
    }
    root
}

/// Group settlements by fission parentage and friendly contact ledgers.
pub fn derive_civilizations(
    settlements: &[(u64, Option<u64>)],
    contacts: &[Contact],
    year: f64,
    next_id: &mut u64,
) -> Vec<Civilization> {
    let mut parents: BTreeMap<u64, u64> = BTreeMap::new();
    for (id, _) in settlements {
        parents.insert(*id, *id);
    }
    let mut link = |a: u64, b: u64| {
        if !parents.contains_key(&a) || !parents.contains_key(&b) {
            return;
        }
        let ra = find(&mut parents, a);
        let rb = find(&mut parents, b);
        if ra != rb {
            parents.insert(ra, rb);
        }
    };
    for (id, parent) in settlements {
        if let Some(p) = parent {
            link(*id, *p);
        }
    }
    for c in contacts {
        if c.stance() == Stance::Friendly {
            link(c.a, c.b);
        }
    }
    let mut groups: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    let ids: Vec<u64> = settlements.iter().map(|(id, _)| *id).collect();
    for id in ids {
        let r = find(&mut parents, id);
        groups.entry(r).or_default().push(id);
    }
    groups
        .into_values()
        .map(|mut members| {
            members.sort_unstable();
            let id = *next_id;
            *next_id += 1;
            Civilization { id, settlement_ids: members, formed_year: year }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fission_links_settlements() {
        let settlements = vec![(1u64, None), (2u64, Some(1u64)), (3u64, None)];
        let mut n = 100u64;
        let civs = derive_civilizations(&settlements, &[], 5.0, &mut n);
        assert_eq!(civs.len(), 2);
        let big = civs.iter().find(|c| c.settlement_ids.len() == 2).unwrap();
        assert!(big.settlement_ids.contains(&1) && big.settlement_ids.contains(&2));
    }
    #[test]
    fn friendship_merges_hostility_splits() {
        let settlements = vec![(1u64, None), (2u64, None)];
        let mut n = 100u64;
        let mut friendly = Contact { a: 1, b: 2, trust: 0.8, exchanges: 5, conflicts: 0, first_year: 1.0 };
        assert_eq!(derive_civilizations(&settlements, &[friendly.clone()], 5.0, &mut n).len(), 1);
        friendly.trust = -0.9;
        friendly.conflicts = 6;
        assert_eq!(derive_civilizations(&settlements, &[friendly], 5.0, &mut n).len(), 2);
    }
}
