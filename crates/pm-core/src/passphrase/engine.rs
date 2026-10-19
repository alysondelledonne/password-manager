//! Builds a word-based password.
//!
//! For each word count (3 first, then 2) and each way of splitting the digits
//! between separators and the end (`k` = digits used as separators), the engine
//! works out the letter budget the words must fit in and **counts** how many
//! word combinations fit. Any split with at least one fitting combination is a
//! valid choice. It then picks a split, a combination and the separators
//! uniformly at random. Counting first, instead of guessing and retrying, means
//! tight length ranges are handled exactly and the strength figure is exact.

use std::collections::BTreeMap;
use std::fmt;

use rand::rand_core::UnwrapErr;
use rand::rngs::SysRng;
use rand::seq::{IndexedRandom, SliceRandom};
use rand::{CryptoRng, RngExt};
use serde::Serialize;
use zeroize::Zeroizing;

use super::error::PassphraseError;
use super::rating::{Shape, guess_stars};
use super::{PassphrasePolicy, PassphraseRequest, WordSource, prepare_own_words, wordlist};
use crate::generator::rules::is_common_password;

const DIGITS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedPassphrase {
    /// Wiped from memory when dropped.
    pub password: Zeroizing<String>,
    /// 2 or 3.
    pub word_count: usize,
    /// Randomness in bits for these settings (the lowest over the valid digit
    /// splits, so it doesn't change between regenerations). For the person's
    /// own words, only the random choice and order of words counts, as if
    /// someone knew all of them.
    pub entropy_bits: f64,
    /// "Hard to guess", 1 to 5 stars (see `rating`).
    pub guess_stars: u8,
    /// "Easy to remember", 1 to 5 stars, for this particular password (see `rating`).
    pub memory_stars: u8,
}

impl fmt::Debug for GeneratedPassphrase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeneratedPassphrase")
            .field("password", &"[REDACTED]")
            .field("word_count", &self.word_count)
            .field("entropy_bits", &self.entropy_bits)
            .field("guess_stars", &self.guess_stars)
            .field("memory_stars", &self.memory_stars)
            .finish()
    }
}

/// Generates with the operating system's secure random number generator.
///
/// # Panics
/// If the OS can't provide randomness (see `generate_with_os_rng`).
pub fn generate_passphrase_with_os_rng(
    request: &PassphraseRequest,
) -> Result<GeneratedPassphrase, PassphraseError> {
    generate_passphrase(request, &mut UnwrapErr(SysRng))
}

pub fn generate_passphrase<R: CryptoRng + ?Sized>(
    request: &PassphraseRequest,
    rng: &mut R,
) -> Result<GeneratedPassphrase, PassphraseError> {
    let policy = &request.policy;
    policy.validate()?;

    let own_words;
    let (words, own): (Vec<&str>, bool) = match &request.words {
        WordSource::Random => (wordlist::eff_short().to_vec(), false),
        WordSource::Mine(list) => {
            own_words = prepare_own_words(list)?;
            (own_words.iter().map(|w| w.as_str()).collect(), true)
        }
    };
    let symbols: Vec<char> = policy.allowed_symbols.iter().copied().collect();

    for word_count in [3, 2] {
        if word_count > words.len() {
            continue;
        }
        let pool = WordPool::new(&words, own, word_count);
        let splits = valid_splits(policy, &pool, word_count);
        if splits.is_empty() {
            continue;
        }
        let entropy_bits = splits
            .iter()
            .map(|s| s.entropy_bits(policy, symbols.len()))
            .fold(f64::INFINITY, f64::min);

        loop {
            let split = splits.choose(rng).expect("splits is not empty");
            let chosen = pool.sample(split.letters_min, split.letters_max, rng);
            let password = assemble(policy, &chosen, split, &symbols, rng);
            // Practically never true for 2-3 words with separators, but checked anyway.
            if !is_common_password(&password) {
                return Ok(GeneratedPassphrase {
                    password,
                    word_count,
                    entropy_bits,
                    guess_stars: guess_stars(entropy_bits),
                    memory_stars: Shape {
                        words: word_count,
                        own_words: own,
                        symbol_separators: split.separators - split.digit_separators,
                        digit_separators: split.digit_separators,
                        trailing_digits: policy.digits - split.digit_separators,
                    }
                    .memory_stars(),
                });
            }
        }
    }

    Err(PassphraseError::DoesNotFit {
        min: policy.min_length,
        max: policy.max_length,
        digits: policy.digits,
        hint: if own {
            ", or enter words of different lengths"
        } else {
            ""
        },
    })
}

/// One valid way to split the digits: `digit_separators` of the separators are
/// digits, the rest are symbols, and `policy.digits - digit_separators` digits go at the end.
#[derive(Debug, Clone, PartialEq)]
struct Split {
    separators: usize,
    digit_separators: usize,
    letters_min: usize,
    letters_max: usize,
    /// How many word combinations fit this split's letter budget.
    combinations: u128,
}

impl Split {
    fn entropy_bits(&self, policy: &PassphrasePolicy, symbol_count: usize) -> f64 {
        let k = self.digit_separators;
        let symbol_separators = self.separators - k;
        let suffix_digits = policy.digits - k;
        // Which words are used, and in what order, is chosen at random. For the
        // person's own words that is the only credit they get: the words
        // themselves are treated as known to an attacker.
        let words = (self.combinations as f64).log2();
        let positions = (binomial(self.separators, k) as f64).log2();
        let digits = (k + suffix_digits) as f64 * 10f64.log2();
        let symbols = if symbol_separators == 0 {
            0.0
        } else {
            symbol_separators as f64 * (symbol_count as f64).log2()
        };
        words + positions + digits + symbols
    }
}

fn valid_splits(policy: &PassphrasePolicy, pool: &WordPool, word_count: usize) -> Vec<Split> {
    let separators = word_count - 1;
    let digit_separator_options: Vec<usize> = if policy.symbols_required {
        // At least one separator must be a symbol.
        (0..=policy.digits.min(separators - 1)).collect()
    } else if policy.digits >= separators {
        // No symbols: every separator is a digit.
        vec![separators]
    } else {
        vec![]
    };

    digit_separator_options
        .into_iter()
        .filter_map(|k| {
            let overhead = separators + (policy.digits - k);
            let letters_max = policy.max_length.checked_sub(overhead)?;
            let letters_min = policy.min_length.saturating_sub(overhead);
            let combinations = pool.count(letters_min, letters_max);
            (combinations > 0).then_some(Split {
                separators,
                digit_separators: k,
                letters_min,
                letters_max,
                combinations,
            })
        })
        .collect()
}

/// The words to choose from, for a fixed word count.
enum WordPool<'a> {
    /// Random words, chosen independently (repeats allowed, as in Diceware),
    /// grouped by length so combinations can be counted without listing them.
    Random {
        by_length: BTreeMap<usize, Vec<&'a str>>,
        word_count: usize,
    },
    /// The person's own words: every ordered choice of distinct words.
    Own(Vec<Vec<&'a str>>),
}

impl<'a> WordPool<'a> {
    fn new(words: &[&'a str], own: bool, word_count: usize) -> Self {
        if own {
            let mut tuples = Vec::new();
            let indices: Vec<usize> = (0..words.len()).collect();
            permutations(&indices, word_count, &mut Vec::new(), &mut tuples);
            WordPool::Own(
                tuples
                    .into_iter()
                    .map(|t| t.into_iter().map(|i| words[i]).collect())
                    .collect(),
            )
        } else {
            let mut by_length: BTreeMap<usize, Vec<&str>> = BTreeMap::new();
            for w in words {
                by_length.entry(w.chars().count()).or_default().push(w);
            }
            WordPool::Random {
                by_length,
                word_count,
            }
        }
    }

    /// Length tuples (one length per word) whose total is in range, with how
    /// many word combinations each allows.
    fn length_tuples(&self, min: usize, max: usize) -> Vec<(Vec<usize>, u128)> {
        let WordPool::Random {
            by_length,
            word_count,
        } = self
        else {
            return vec![];
        };
        let lengths: Vec<(usize, u128)> = by_length
            .iter()
            .map(|(l, w)| (*l, w.len() as u128))
            .collect();
        let mut walk = LengthWalk {
            lengths: &lengths,
            min,
            max,
            current: Vec::new(),
            out: Vec::new(),
        };
        walk.visit(*word_count, 0, 1);
        walk.out
    }

    fn count(&self, min: usize, max: usize) -> u128 {
        match self {
            WordPool::Random { .. } => self.length_tuples(min, max).iter().map(|(_, w)| w).sum(),
            WordPool::Own(tuples) => tuples.iter().filter(|t| fits(t, min, max)).count() as u128,
        }
    }

    /// Picks one fitting combination uniformly at random.
    fn sample<R: CryptoRng + ?Sized>(&self, min: usize, max: usize, rng: &mut R) -> Vec<&'a str> {
        match self {
            WordPool::Random { by_length, .. } => {
                let tuples = self.length_tuples(min, max);
                let total: u128 = tuples.iter().map(|(_, w)| w).sum();
                let mut pick = rng.random_range(0..total);
                let (lengths, _) = tuples
                    .iter()
                    .find(|(_, weight)| {
                        if pick < *weight {
                            true
                        } else {
                            pick -= weight;
                            false
                        }
                    })
                    .expect("pick is below the total weight");
                lengths
                    .iter()
                    .map(|len| {
                        *by_length[len]
                            .choose(rng)
                            .expect("length buckets are not empty")
                    })
                    .collect()
            }
            WordPool::Own(tuples) => {
                let fitting: Vec<&Vec<&str>> =
                    tuples.iter().filter(|t| fits(t, min, max)).collect();
                fitting
                    .choose(rng)
                    .expect("count was checked to be > 0")
                    .to_vec()
            }
        }
    }
}

/// Lists every tuple of word lengths whose total is within `min..=max`,
/// with how many word combinations each tuple allows.
struct LengthWalk<'l> {
    /// (word length, how many words have that length)
    lengths: &'l [(usize, u128)],
    min: usize,
    max: usize,
    current: Vec<usize>,
    out: Vec<(Vec<usize>, u128)>,
}

impl LengthWalk<'_> {
    fn visit(&mut self, remaining: usize, total: usize, weight: u128) {
        if remaining == 0 {
            if (self.min..=self.max).contains(&total) {
                self.out.push((self.current.clone(), weight));
            }
            return;
        }
        for &(len, count) in self.lengths {
            if total + len > self.max {
                continue;
            }
            self.current.push(len);
            self.visit(remaining - 1, total + len, weight * count);
            self.current.pop();
        }
    }
}

fn fits(words: &[&str], min: usize, max: usize) -> bool {
    let total: usize = words.iter().map(|w| w.chars().count()).sum();
    (min..=max).contains(&total)
}

fn permutations(items: &[usize], k: usize, current: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
    if current.len() == k {
        out.push(current.clone());
        return;
    }
    for &i in items {
        if !current.contains(&i) {
            current.push(i);
            permutations(items, k, current, out);
            current.pop();
        }
    }
}

fn binomial(n: usize, k: usize) -> u128 {
    (0..k).fold(1u128, |acc, i| acc * (n - i) as u128 / (i as u128 + 1))
}

fn assemble<R: CryptoRng + ?Sized>(
    policy: &PassphrasePolicy,
    words: &[&str],
    split: &Split,
    symbols: &[char],
    rng: &mut R,
) -> Zeroizing<String> {
    // Which separators are digits.
    let mut positions: Vec<usize> = (0..split.separators).collect();
    positions.shuffle(rng);
    let digit_positions = &positions[..split.digit_separators];

    let mut out = Zeroizing::new(String::new());
    for (i, word) in words.iter().enumerate() {
        out.push_str(&policy.apply_case(word));
        if i < split.separators {
            let sep = if digit_positions.contains(&i) {
                *DIGITS.choose(rng).expect("digits")
            } else {
                *symbols
                    .choose(rng)
                    .expect("symbols are required, so not empty")
            };
            out.push(sep);
        }
    }
    for _ in 0..(policy.digits - split.digit_separators) {
        out.push(*DIGITS.choose(rng).expect("digits"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    fn rng() -> ChaCha20Rng {
        ChaCha20Rng::seed_from_u64(7)
    }

    fn mine(list: &[&str]) -> WordSource {
        WordSource::Mine(list.iter().map(|w| Zeroizing::new(w.to_string())).collect())
    }

    fn request(policy: PassphrasePolicy, words: WordSource) -> PassphraseRequest {
        PassphraseRequest { policy, words }
    }

    /// Splits a password into its letter runs (the words).
    fn word_runs(pw: &str) -> Vec<String> {
        pw.split(|c: char| !c.is_alphabetic())
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn binomial_values() {
        assert_eq!(binomial(2, 0), 1);
        assert_eq!(binomial(2, 1), 2);
        assert_eq!(binomial(5, 2), 10);
    }

    #[test]
    fn own_words_use_two_or_three_of_mine_capitalized() {
        let req = request(
            PassphrasePolicy::default(),
            mine(&["maple", "otter", "wagon", "lantern"]),
        );
        let mut rng = rng();
        for _ in 0..200 {
            let out = generate_passphrase(&req, &mut rng).unwrap();
            let runs = word_runs(&out.password);
            assert_eq!(runs.len(), out.word_count);
            assert_eq!(out.word_count, 3, "3 words fit the default range");
            for w in &runs {
                assert!(
                    ["Maple", "Otter", "Wagon", "Lantern"].contains(&w.as_str()),
                    "{w}"
                );
            }
            let unique: std::collections::BTreeSet<_> = runs.iter().collect();
            assert_eq!(unique.len(), runs.len(), "no word repeated");
        }
    }

    #[test]
    fn two_words_when_only_two_given() {
        let req = request(PassphrasePolicy::default(), mine(&["maple", "otter"]));
        let out = generate_passphrase(&req, &mut rng()).unwrap();
        assert_eq!(out.word_count, 2);
    }

    #[test]
    fn falls_back_to_two_words_when_three_cannot_fit() {
        // maple+otter+wagon = 15 letters + 2 separators + 2 digits = 19 > 16.
        let policy = PassphrasePolicy {
            min_length: 8,
            max_length: 16,
            ..Default::default()
        };
        let req = request(policy, mine(&["maple", "otter", "wagon"]));
        let out = generate_passphrase(&req, &mut rng()).unwrap();
        assert_eq!(out.word_count, 2);
        assert!((8..=16).contains(&out.password.chars().count()));
    }

    #[test]
    fn exact_digits_and_at_least_one_symbol() {
        let policy = PassphrasePolicy {
            digits: 4,
            ..Default::default()
        };
        let req = request(policy.clone(), mine(&["maple", "otter", "wagon"]));
        let mut rng = rng();
        for _ in 0..300 {
            let out = generate_passphrase(&req, &mut rng).unwrap();
            let pw = out.password.as_str();
            assert_eq!(pw.chars().filter(char::is_ascii_digit).count(), 4, "{pw}");
            assert!(
                pw.chars().any(|c| policy.allowed_symbols.contains(&c)),
                "{pw}"
            );
            assert!(!pw.contains(' '));
        }
    }

    #[test]
    fn symbols_off_means_digit_separators_only() {
        let policy = PassphrasePolicy {
            symbols_required: false,
            digits: 3,
            ..Default::default()
        };
        let req = request(policy, mine(&["maple", "otter", "wagon"]));
        let mut rng = rng();
        for _ in 0..100 {
            let pw = generate_passphrase(&req, &mut rng).unwrap().password;
            assert!(pw.chars().all(|c| c.is_alphanumeric()), "{}", pw.as_str());
            assert_eq!(pw.chars().filter(char::is_ascii_digit).count(), 3);
        }
    }

    #[test]
    fn uppercase_only_and_lowercase_only() {
        let upper = PassphrasePolicy {
            lowercase: false,
            ..Default::default()
        };
        let pw = generate_passphrase(&request(upper, mine(&["maple", "otter"])), &mut rng())
            .unwrap()
            .password;
        assert!(
            word_runs(&pw)
                .iter()
                .all(|w| w.chars().all(char::is_uppercase))
        );

        let lower = PassphrasePolicy {
            uppercase: false,
            ..Default::default()
        };
        let pw = generate_passphrase(&request(lower, mine(&["maple", "otter"])), &mut rng())
            .unwrap()
            .password;
        assert!(
            word_runs(&pw)
                .iter()
                .all(|w| w.chars().all(char::is_lowercase))
        );
    }

    #[test]
    fn reports_when_nothing_fits() {
        let policy = PassphrasePolicy {
            min_length: 1,
            max_length: 6,
            ..Default::default()
        };
        let err = generate_passphrase(&request(policy, mine(&["maple", "otter"])), &mut rng())
            .unwrap_err();
        assert_eq!(err.code(), "does_not_fit");
        assert_eq!(err.field(), Some("length"));
    }

    #[test]
    fn own_words_count_only_their_random_order() {
        // 2 own words, 1 separator that must be a symbol, 0 digits. Random parts:
        // the order of the 2 words (1 bit) and the symbol.
        let policy = PassphrasePolicy {
            digits: 0,
            ..Default::default()
        };
        let symbols = policy.allowed_symbols.len() as f64;
        let out =
            generate_passphrase(&request(policy, mine(&["lantern", "otter"])), &mut rng()).unwrap();
        assert!(
            (out.entropy_bits - (1.0 + symbols.log2())).abs() < 1e-9,
            "{}",
            out.entropy_bits
        );
        assert_eq!(out.guess_stars, 1);
        assert_eq!(out.memory_stars, 5, "2 own words and 1 symbol: easy");
    }

    #[test]
    fn random_words_count_combinations_exactly() {
        let words = ["aa", "bb", "ccc"];
        let pool = WordPool::new(&words, false, 2);
        // Pairs with 4-5 letters: (aa,aa) (aa,bb) (bb,aa) (bb,bb) = 4 with 4; (aa|bb, ccc) and (ccc, aa|bb) = 4 with 5.
        assert_eq!(pool.count(4, 5), 8);
        assert_eq!(pool.count(6, 6), 1);
        assert_eq!(pool.count(7, 9), 0);
    }

    #[test]
    fn random_sampling_is_uniform_over_fitting_combinations() {
        let words = ["aa", "bb", "ccc"];
        let pool = WordPool::new(&words, false, 2);
        let mut rng = rng();
        let mut counts = std::collections::HashMap::new();
        let runs = 40_000;
        for _ in 0..runs {
            *counts
                .entry(pool.sample(4, 5, &mut rng).join("+"))
                .or_insert(0usize) += 1;
        }
        assert_eq!(counts.len(), 8);
        let expected = runs as f64 / 8.0;
        for (combo, n) in counts {
            assert!(
                (n as f64 - expected).abs() / expected < 0.06,
                "{combo}: {n}"
            );
        }
    }

    #[test]
    fn real_word_list_star_ratings() {
        // Defaults: 3 random words, 2 digits, 13 symbols: about 42 bits.
        let req = request(PassphrasePolicy::default(), WordSource::Random);
        let out = generate_passphrase(&req, &mut rng()).unwrap();
        assert!(
            (40.0..45.0).contains(&out.entropy_bits),
            "{}",
            out.entropy_bits
        );
        assert_eq!(out.guess_stars, 3);
        assert!(
            out.memory_stars >= 3,
            "random words, defaults: about 6 chunks"
        );

        // 4 digits and room for them: about 49 bits.
        let policy = PassphrasePolicy {
            digits: 4,
            max_length: 30,
            ..Default::default()
        };
        let out = generate_passphrase(&request(policy, WordSource::Random), &mut rng()).unwrap();
        assert_eq!(out.guess_stars, 4, "{} bits", out.entropy_bits);
        assert_eq!(
            out.memory_stars, 2,
            "4 extra digits make it harder to remember"
        );
    }

    #[test]
    fn same_seed_same_password() {
        let req = request(PassphrasePolicy::default(), WordSource::Random);
        let a = generate_passphrase(&req, &mut ChaCha20Rng::seed_from_u64(9)).unwrap();
        let b = generate_passphrase(&req, &mut ChaCha20Rng::seed_from_u64(9)).unwrap();
        assert_eq!(a.password, b.password);
    }

    #[test]
    fn os_rng_works_and_regenerates() {
        let req = request(PassphrasePolicy::default(), WordSource::Random);
        let a = generate_passphrase_with_os_rng(&req).unwrap();
        let b = generate_passphrase_with_os_rng(&req).unwrap();
        assert_ne!(a.password, b.password);
    }

    #[test]
    fn debug_hides_the_password() {
        let out = generate_passphrase(
            &request(PassphrasePolicy::default(), WordSource::Random),
            &mut rng(),
        )
        .unwrap();
        assert!(!format!("{out:?}").contains(out.password.as_str()));
    }
}
