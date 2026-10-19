//! Two 1–5 star ratings for word-based passwords, because the goal is a
//! password that is unique, hard to guess, **and** easy enough to remember
//! without a password manager. The two pull against each other, so both are shown.
//!
//! ## Hard to guess
//!
//! From the exact bits of randomness. The danger isn't someone typing guesses
//! at a login page (sites limit that) but a site being breached, after which
//! attackers can guess offline at high speed. More bits buy more time.
//!
//! | Stars | Bits     | Rough meaning                                   |
//! |-------|----------|-------------------------------------------------|
//! | 1     | < 20     | throwaway sign-ups                              |
//! | 2     | 20 – 35  | low-stakes accounts                             |
//! | 3     | 35 – 45  | everyday accounts                               |
//! | 4     | 45 – 60  | accounts with money or private details          |
//! | 5     | 60 +     | accounts that unlock other accounts             |
//!
//! ## Easy to remember
//!
//! From how many separate things a person has to recall ("chunks"). Working
//! memory holds about four chunks comfortably (Cowan, 2001), so fewer is easier.
//! This is a rule of thumb, not a measurement of any one person's memory.
//!
//! | Part of the password                       | Chunks            |
//! |--------------------------------------------|-------------------|
//! | each of your own words                     | 0.75 (meaningful to you) |
//! | each random word                           | 1                 |
//! | each special character between words       | 1                 |
//! | each digit between words                   | 1                 |
//! | digits together at the end                 | 1 per pair (`47` is one number) |
//!
//! | Stars | Chunks      |
//! |-------|-------------|
//! | 5     | up to 4.5   |
//! | 4     | up to 5.5   |
//! | 3     | up to 6.5   |
//! | 2     | up to 8     |
//! | 1     | more than 8 |

/// Bits needed for 2, 3, 4 and 5 "hard to guess" stars.
pub const GUESS_THRESHOLDS: [f64; 4] = [20.0, 35.0, 45.0, 60.0];

/// Most chunks for 5, 4, 3 and 2 "easy to remember" stars.
pub const MEMORY_THRESHOLDS: [f64; 4] = [4.5, 5.5, 6.5, 8.0];

/// "Hard to guess": 1 to 5 stars from bits of randomness.
pub fn guess_stars(bits: f64) -> u8 {
    1 + GUESS_THRESHOLDS.iter().filter(|&&t| bits >= t).count() as u8
}

/// The parts of a generated password that a person has to remember.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shape {
    pub words: usize,
    /// The person's own words (more memorable) rather than random ones.
    pub own_words: bool,
    pub symbol_separators: usize,
    pub digit_separators: usize,
    pub trailing_digits: usize,
}

impl Shape {
    /// How many separate things there are to remember.
    pub fn chunks(&self) -> f64 {
        let per_word = if self.own_words { 0.75 } else { 1.0 };
        self.words as f64 * per_word
            + self.symbol_separators as f64
            + self.digit_separators as f64
            + self.trailing_digits.div_ceil(2) as f64
    }

    /// "Easy to remember": 1 to 5 stars.
    pub fn memory_stars(&self) -> u8 {
        let chunks = self.chunks();
        5 - MEMORY_THRESHOLDS.iter().filter(|&&t| chunks > t).count() as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape(words: usize, own: bool, sym: usize, dig: usize, trailing: usize) -> Shape {
        Shape {
            words,
            own_words: own,
            symbol_separators: sym,
            digit_separators: dig,
            trailing_digits: trailing,
        }
    }

    #[test]
    fn guess_thresholds() {
        assert_eq!(guess_stars(0.0), 1);
        assert_eq!(guess_stars(19.9), 1);
        assert_eq!(guess_stars(20.0), 2);
        assert_eq!(guess_stars(35.0), 3);
        assert_eq!(guess_stars(45.0), 4);
        assert_eq!(guess_stars(59.9), 4);
        assert_eq!(guess_stars(60.0), 5);
        assert_eq!(
            guess_stars(f64::NAN),
            1,
            "NaN compares false: lowest rating"
        );
    }

    #[test]
    fn chunk_counting() {
        // Maple+Harbor82: 2 own words, 1 symbol, "82" at the end.
        assert_eq!(shape(2, true, 1, 0, 2).chunks(), 3.5);
        // Harbor4Maple+Ember7: 3 own words, 1 symbol, 1 digit between, 1 at the end.
        assert_eq!(shape(3, true, 1, 1, 1).chunks(), 5.25);
        // Vapor3Tidal*Quilt5: 3 random words, same shape.
        assert_eq!(shape(3, false, 1, 1, 1).chunks(), 6.0);
        // Three trailing digits are two chunks ("47" + "3").
        assert_eq!(shape(2, false, 1, 0, 3).chunks(), 5.0);
    }

    #[test]
    fn memory_stars_follow_the_table() {
        assert_eq!(shape(2, true, 1, 0, 2).memory_stars(), 5, "Maple+Harbor82");
        assert_eq!(
            shape(3, true, 1, 1, 1).memory_stars(),
            4,
            "Harbor4Maple+Ember7"
        );
        assert_eq!(
            shape(3, false, 1, 1, 1).memory_stars(),
            3,
            "random words, defaults"
        );
        assert_eq!(
            shape(3, false, 2, 0, 4).memory_stars(),
            2,
            "random words, 4 digits"
        );
        assert_eq!(
            shape(3, false, 2, 0, 8).memory_stars(),
            1,
            "random words, 8 digits"
        );
    }
}
