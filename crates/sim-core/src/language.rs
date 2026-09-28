use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Distinct everyday concepts a settlement lexicon names.
/// Small on purpose: divergence must stay legible, not a vocabulary simulator.
pub const CONCEPTS: u8 = 12;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Lexicon {
    pub words: BTreeMap<u8, u16>,
}

fn stream(seed: u64, salt: u64) -> u64 {
    let mut x = seed ^ salt.wrapping_mul(0x9E3779B97F4A7C15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58476D1CE4E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D049BB133111EB);
    x ^= x >> 31;
    x
}

fn unit01(seed: u64, salt: u64) -> f32 {
    (stream(seed, salt) as f64 / u64::MAX as f64) as f32
}

impl Lexicon {
    pub fn word(&self, concept: u8) -> u16 {
        *self.words.get(&concept).unwrap_or(&0)
    }
    /// Deterministic founding vocabulary so daughter settlements start alike, then diverge.
    pub fn seed_from(id: u64) -> Self {
        let mut words = BTreeMap::new();
        for c in 0..CONCEPTS {
            words.insert(c, (stream(id, c as u64 * 7919 + 13) & 0x3FF) as u16);
        }
        Lexicon { words }
    }
    /// Independent sound drift: each concept word may flip one phoneme bit per call.
    pub fn drift(&mut self, seed: u64, salt: u64, rate: f32) {
        for c in 0..CONCEPTS {
            if unit01(seed, salt ^ (c as u64).wrapping_mul(104729)) < rate {
                let bit = 1u16 << (stream(seed, salt ^ (c as u64)) % 10);
                let w = self.word(c) ^ bit;
                self.words.insert(c, w);
            }
        }
    }
    /// Contact blends vocabularies: each concept adopts the other's word with some probability.
    pub fn blend_toward(&mut self, other: &Lexicon, weight: f32, seed: u64, salt: u64) {
        for c in 0..CONCEPTS {
            if unit01(seed, salt ^ (c as u64).wrapping_mul(1299709)) < weight {
                self.words.insert(c, other.word(c));
            }
        }
    }
    /// Fraction of concepts named differently. 0 = same tongue, 1 = mutually unintelligible.
    pub fn divergence(a: &Lexicon, b: &Lexicon) -> f32 {
        let mut diff = 0u32;
        for c in 0..CONCEPTS {
            if a.word(c) != b.word(c) {
                diff += 1;
            }
        }
        diff as f32 / CONCEPTS as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn daughters_start_alike() {
        let a = Lexicon::seed_from(7);
        let b = Lexicon::seed_from(7);
        assert_eq!(Lexicon::divergence(&a, &b), 0.0);
    }
    #[test]
    fn isolation_diverges_contact_reconverges() {
        let mut a = Lexicon::seed_from(7);
        let mut b = Lexicon::seed_from(7);
        for i in 0..400u64 {
            a.drift(1, i, 0.02);
            b.drift(2, i, 0.02);
        }
        let apart = Lexicon::divergence(&a, &b);
        assert!(apart > 0.0);
        for i in 0..50u64 {
            a.blend_toward(&b, 0.3, 3, i);
            b.blend_toward(&a, 0.3, 4, i);
        }
        assert!(Lexicon::divergence(&a, &b) < apart);
    }
    #[test]
    fn divergence_is_bounded() {
        let a = Lexicon::seed_from(1);
        let b = Lexicon::seed_from(999);
        let d = Lexicon::divergence(&a, &b);
        assert!(d >= 0.0 && d <= 1.0);
    }
}
