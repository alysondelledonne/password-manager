//! Property tests for word-based passwords: for any settings, a generated
//! password follows every rule, and an error is only returned for a reason
//! the settings explain.

use std::collections::BTreeSet;

use pm_core::generator::charsets::DEFAULT_SYMBOLS;
use pm_core::passphrase::{
    PassphraseError, PassphrasePolicy, PassphraseRequest, WordSource, generate_passphrase, wordlist,
};
use proptest::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use zeroize::Zeroizing;

const MY_WORDS: &[&str] = &[
    "maple", "otter", "lantern", "fig", "harbor", "quilt", "ember",
];

fn arb_policy() -> impl Strategy<Value = PassphrasePolicy> {
    (
        1usize..=40,
        0usize..=30,
        0usize..=5,
        (any::<bool>(), any::<bool>()),
        any::<bool>(),
        proptest::sample::subsequence(DEFAULT_SYMBOLS.chars().collect::<Vec<_>>(), 1..=13),
    )
        .prop_map(
            |(min, extra, digits, (upper, lower), symbols, allowed)| PassphrasePolicy {
                min_length: min,
                max_length: min + extra,
                digits,
                uppercase: upper,
                lowercase: lower,
                symbols_required: symbols,
                allowed_symbols: allowed.into_iter().collect(),
            },
        )
}

fn arb_source() -> impl Strategy<Value = (WordSource, Option<Vec<&'static str>>)> {
    prop_oneof![
        Just((WordSource::Random, None)),
        proptest::sample::subsequence(MY_WORDS.to_vec(), 2..=MY_WORDS.len()).prop_map(|words| {
            let source = WordSource::Mine(
                words
                    .iter()
                    .map(|w| Zeroizing::new(w.to_string()))
                    .collect(),
            );
            (source, Some(words))
        }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    #[test]
    fn every_password_follows_every_rule(
        policy in arb_policy(),
        (source, mine) in arb_source(),
        seed in any::<u64>(),
    ) {
        let request = PassphraseRequest { policy: policy.clone(), words: source };
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        match generate_passphrase(&request, &mut rng) {
            Ok(out) => {
                let pw = out.password.as_str();
                let len = pw.chars().count();
                prop_assert!((policy.min_length..=policy.max_length).contains(&len), "length {len}: {pw}");
                prop_assert_eq!(pw.chars().filter(char::is_ascii_digit).count(), policy.digits, "digits: {}", pw);
                prop_assert!(!pw.chars().any(char::is_whitespace), "no spaces");

                let symbols: Vec<char> = pw.chars().filter(|c| !c.is_alphanumeric()).collect();
                if policy.symbols_required {
                    prop_assert!(!symbols.is_empty(), "at least one symbol: {}", pw);
                    prop_assert!(symbols.iter().all(|c| policy.allowed_symbols.contains(c)), "only selected symbols: {}", pw);
                } else {
                    prop_assert!(symbols.is_empty(), "no symbols when off: {}", pw);
                }

                let words: Vec<&str> = pw.split(|c: char| !c.is_alphabetic()).filter(|s| !s.is_empty()).collect();
                prop_assert_eq!(words.len(), out.word_count);
                prop_assert!(out.word_count == 2 || out.word_count == 3);
                prop_assert!((1..=5).contains(&out.guess_stars));
                prop_assert!((1..=5).contains(&out.memory_stars));
                prop_assert_eq!(out.guess_stars, pm_core::passphrase::guess_stars(out.entropy_bits));
                // Easy-to-remember stars match the password's actual shape.
                let symbol_seps = symbols.len();
                let digit_seps = (out.word_count - 1) - symbol_seps;
                let shape = pm_core::passphrase::Shape {
                    words: out.word_count,
                    own_words: mine.is_some(),
                    symbol_separators: symbol_seps,
                    digit_separators: digit_seps,
                    trailing_digits: policy.digits - digit_seps,
                };
                prop_assert_eq!(out.memory_stars, shape.memory_stars(), "{}", pw);
                for w in &words {
                    let lower = w.to_lowercase();
                    match &mine {
                        Some(list) => prop_assert!(list.contains(&lower.as_str()), "{} is one of mine", w),
                        None => prop_assert!(wordlist::eff_short().contains(&lower.as_str()), "{} is on the list", w),
                    }
                    match (policy.uppercase, policy.lowercase) {
                        (true, true) => {
                            let mut chars = w.chars();
                            prop_assert!(chars.next().unwrap().is_uppercase() && chars.all(char::is_lowercase), "capitalized: {}", w);
                        }
                        (true, false) => prop_assert!(w.chars().all(char::is_uppercase), "all caps: {}", w),
                        _ => prop_assert!(w.chars().all(char::is_lowercase), "all lowercase: {}", w),
                    }
                }
                if mine.is_some() {
                    let unique: BTreeSet<_> = words.iter().map(|w| w.to_lowercase()).collect();
                    prop_assert_eq!(unique.len(), words.len(), "own words not repeated");
                }
            }
            Err(err) => {
                // Errors must be explained by the settings.
                let explained = match err {
                    PassphraseError::NoLetters => !policy.uppercase && !policy.lowercase,
                    PassphraseError::NeedsSeparator => !policy.symbols_required && policy.digits == 0,
                    PassphraseError::DoesNotFit { .. } => true,
                    _ => false,
                };
                prop_assert!(explained, "unexpected error {:?} for {:?}", err, policy);
            }
        }
    }
}

/// With a generous range, defaults and random words always work, using 3 words.
#[test]
fn defaults_with_random_words_use_three_words() {
    let request = PassphraseRequest {
        policy: PassphrasePolicy {
            max_length: 40,
            ..Default::default()
        },
        words: WordSource::Random,
    };
    let mut rng = ChaCha20Rng::seed_from_u64(1);
    for _ in 0..200 {
        assert_eq!(
            generate_passphrase(&request, &mut rng).unwrap().word_count,
            3
        );
    }
}
