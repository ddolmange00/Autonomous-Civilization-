use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stance {
    Hostile,
    Wary,
    Neutral,
    Friendly,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contact {
    pub a: u64,
    pub b: u64,
    pub trust: f32,
    pub exchanges: u32,
    pub conflicts: u32,
    pub first_year: f64,
}

impl Contact {
    pub fn stance(&self) -> Stance {
        if self.trust <= -0.45 || self.conflicts >= self.exchanges.saturating_add(3) {
            Stance::Hostile
        } else if self.trust < -0.10 {
            Stance::Wary
        } else if self.trust > 0.45 && self.exchanges >= 3 {
            Stance::Friendly
        } else {
            Stance::Neutral
        }
    }
    pub fn note_exchange(&mut self, amount: f32) {
        self.exchanges += 1;
        self.trust = (self.trust + 0.03 + amount.min(20.0) * 0.004).clamp(-1.0, 1.0);
    }
    pub fn note_conflict(&mut self, severity: f32) {
        self.conflicts += 1;
        self.trust = (self.trust - 0.10 - severity.clamp(0.0, 1.0) * 0.25).clamp(-1.0, 1.0);
    }
}

/// Order-independent pair key so (a,b) and (b,a) share one ledger.
fn ordered(x: u64, y: u64) -> (u64, u64) {
    if x <= y {
        (x, y)
    } else {
        (y, x)
    }
}

/// Fetch the contact ledger for a settlement pair, recording first contact.
pub fn contact_between<'a>(contacts: &'a mut Vec<Contact>, a: u64, b: u64, year: f64) -> &'a mut Contact {
    let (x, y) = ordered(a, b);
    let pos = contacts.iter().position(|c| c.a == x && c.b == y);
    match pos {
        Some(i) => &mut contacts[i],
        None => {
            contacts.push(Contact { a: x, b: y, trust: 0.0, exchanges: 0, conflicts: 0, first_year: year });
            contacts.last_mut().unwrap()
        }
    }
}

pub fn mean_trust(contacts: &[Contact]) -> f32 {
    if contacts.is_empty() {
        return 0.0;
    }
    contacts.iter().map(|c| c.trust).sum::<f32>() / contacts.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stance_is_derived_not_assigned() {
        let mut c = Contact { a: 1, b: 2, trust: 0.0, exchanges: 0, conflicts: 0, first_year: 0.0 };
        assert_eq!(c.stance(), Stance::Neutral);
        for _ in 0..8 {
            c.note_exchange(10.0);
        }
        assert_eq!(c.stance(), Stance::Friendly);
        c.note_conflict(1.0);
        c.note_conflict(1.0);
        assert!(matches!(c.stance(), Stance::Wary | Stance::Hostile));
    }
    #[test]
    fn pair_key_is_symmetric() {
        let mut v = Vec::new();
        contact_between(&mut v, 5, 3, 1.0).note_exchange(4.0);
        assert_eq!(v.len(), 1);
        contact_between(&mut v, 3, 5, 2.0).note_conflict(0.2);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].exchanges, 1);
        assert_eq!(v[0].conflicts, 1);
    }
}
