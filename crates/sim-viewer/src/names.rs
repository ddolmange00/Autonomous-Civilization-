//! Names shown to the player. Village names are spoken in the village's own
//! lexicon (sim-core `language`), so daughter villages sound like their
//! parents and names drift a sound at a time as the language drifts.
//! Resident names are stable per person; the family syllable comes first.
use sim_core::language::Lexicon;

// Hangul jamo indices (Unicode composition order) chosen to sound soft and readable.
const INITIALS: [u32; 13] = [0, 2, 3, 5, 6, 7, 9, 11, 12, 14, 15, 16, 18]; // ㄱㄴㄷㄹㅁㅂㅅㅇㅈㅊㅋㅌㅎ
const VOWELS: [u32; 12] = [0, 4, 8, 13, 18, 20, 1, 5, 2, 6, 12, 17]; // ㅏㅓㅗㅜㅡㅣㅐㅔㅑㅕㅛㅠ
const FINALS: [u32; 4] = [0, 0, 4, 8]; // none, none, ㄴ, ㄹ

/// One Hangul syllable from a 10-bit sound value.
pub fn syllable(sound: u16) -> char {
    let s = sound as u32;
    let i = INITIALS[(s & 0xF) as usize % INITIALS.len()];
    let v = VOWELS[((s >> 4) & 0xF) as usize % VOWELS.len()];
    let f = FINALS[((s >> 8) & 0x3) as usize];
    char::from_u32(0xAC00 + (i * 21 + v) * 28 + f).unwrap_or('?')
}

fn mix(x: u64) -> u64 {
    let mut x = x.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 29; x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 32;
    x
}

/// A village's name: its words for "home" and "people".
pub fn village(lexicon: &Lexicon) -> String {
    [syllable(lexicon.word(0)), syllable(lexicon.word(1))].iter().collect()
}

/// A resident's name: family syllable, then a two-syllable given name.
pub fn person(id: u64, family: u64) -> String {
    let f = mix(family ^ 0x5A17);
    let g = mix(id ^ 0x9131);
    [syllable((f & 0x3FF) as u16), syllable((g & 0x3FF) as u16), syllable(((g >> 12) & 0x3FF) as u16)].iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syllables_are_hangul_and_vary() {
        let all: std::collections::HashSet<char> = (0..1024u16).map(syllable).collect();
        assert!(all.iter().all(|c| ('\u{AC00}'..='\u{D7A3}').contains(c)));
        assert!(all.len() > 200);
    }

    #[test]
    fn families_share_a_surname_and_villages_follow_their_language() {
        let a = person(1, 77);
        let b = person(2, 77);
        assert_eq!(a.chars().next(), b.chars().next());
        assert_ne!(a, b);
        let home = Lexicon::seed_from(5);
        let mut drifted = home.clone();
        drifted.words.insert(0, home.word(0) ^ 0x10);
        let (n1, n2) = (village(&home), village(&drifted));
        assert_ne!(n1, n2);
        assert_eq!(n1.chars().nth(1), n2.chars().nth(1), "one drifted word changes one syllable");
    }
}
