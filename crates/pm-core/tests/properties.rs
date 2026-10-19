//! Property tests: for any valid rule set, every generated password follows
//! every enabled rule. proptest tries many random rule sets and, if one fails,
//! shrinks it to the smallest failing example.

use std::collections::BTreeSet;

use pm_core::generator::charsets::{DEFAULT_SYMBOLS, is_lookalike};
use pm_core::generator::rules;
use pm_core::{GenerateRequest, PasswordPolicy, PhraseMode, PhraseOptions, generate};
use proptest::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use zeroize::Zeroizing;

fn arb_policy() -> impl Strategy<Value = PasswordPolicy> {
    (
        12usize..=64,                                     // target length
        (0usize..=3, 0usize..=3, 0usize..=4, 0usize..=3), // min upper, lower, digits, symbols
        proptest::sample::subsequence(DEFAULT_SYMBOLS.chars().collect::<Vec<_>>(), 1..=13),
        (any::<bool>(), any::<bool>(), any::<bool>()), // start with letter, exclude look-alikes, no sequences
        proptest::option::of(2usize..=3),              // max repeat
    )
        .prop_map(
            |(
                target,
                (upper, lower, digits, symbols),
                allowed,
                (start, exclude, no_seq),
                repeat,
            )| {
                PasswordPolicy {
                    min_length: 8,
                    max_length: 64,
                    target_length: target,
                    min_upper: upper,
                    min_lower: lower,
                    exact_digits: digits,
                    min_symbols: symbols,
                    allowed_symbols: allowed.into_iter().collect::<BTreeSet<_>>(),
                    no_spaces: true,
                    start_with_letter: start,
                    exclude_lookalikes: exclude,
                    max_repeat: repeat,
                    no_sequences: no_seq,
                    ascii_only: true,
                }
            },
        )
}

fn assert_follows_policy(pw: &str, p: &PasswordPolicy) {
    assert_eq!(pw.chars().count(), p.target_length, "length");
    assert_eq!(
        pw.chars().filter(char::is_ascii_digit).count(),
        p.exact_digits,
        "exact digits"
    );
    assert!(
        pw.chars().filter(char::is_ascii_uppercase).count() >= p.min_upper,
        "uppercase"
    );
    assert!(
        pw.chars().filter(char::is_ascii_lowercase).count() >= p.min_lower,
        "lowercase"
    );
    let symbols = pw
        .chars()
        .filter(|c| !c.is_ascii_alphanumeric())
        .collect::<Vec<_>>();
    assert!(symbols.len() >= p.min_symbols, "symbols");
    assert!(
        symbols.iter().all(|c| p.allowed_symbols.contains(c)),
        "only allowed symbols"
    );
    assert!(pw.is_ascii(), "ascii only");
    assert!(!pw.contains(' '), "no spaces");
    if p.start_with_letter {
        assert!(
            pw.starts_with(|c: char| c.is_ascii_alphabetic()),
            "starts with a letter"
        );
    }
    if p.exclude_lookalikes {
        assert!(!pw.chars().any(is_lookalike), "no look-alikes");
    }
    if let Some(limit) = p.max_repeat {
        assert!(rules::longest_run(pw) <= limit, "repeat limit");
    }
    if p.no_sequences {
        assert_eq!(rules::find_sequence(pw), None, "no sequences");
    }
    assert!(!rules::is_common_password(pw), "not a common password");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn every_password_follows_every_rule(policy in arb_policy(), seed in any::<u64>()) {
        let req = GenerateRequest { policy: policy.clone(), ..Default::default() };
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let out = generate(&req, &mut rng).expect("valid policy should generate");
        assert_follows_policy(&out.password, &policy);
    }

    #[test]
    fn rules_still_hold_with_a_phrase(
        policy in arb_policy(),
        word in prop::sample::select(vec!["garden", "chair", "thunder", "Garden"]),
        transformed in any::<bool>(),
        seed in any::<u64>(),
    ) {
        let mode = if transformed { PhraseMode::Transformed } else { PhraseMode::AsIs };
        let req = GenerateRequest {
            policy: PasswordPolicy { target_length: policy.target_length.max(24), ..policy.clone() },
            phrase: Some(PhraseOptions { phrase: Zeroizing::new(word.into()), mode }),
            username: None,
        };
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let out = generate(&req, &mut rng).expect("valid policy and phrase should generate");
        assert_follows_policy(&out.password, &req.policy);
        if mode == PhraseMode::AsIs {
            prop_assert!(out.password.contains(word));
        }
    }
}

/// With no required classes, every fill character should come up about equally often.
/// A biased picker (e.g. modulo bias) would fail this.
#[test]
fn fill_characters_are_evenly_distributed() {
    let policy = PasswordPolicy {
        min_length: 32,
        target_length: 32,
        min_upper: 0,
        min_lower: 0,
        exact_digits: 0,
        min_symbols: 0,
        max_repeat: None,
        no_sequences: false,
        ..Default::default()
    };
    let req = GenerateRequest {
        policy,
        ..Default::default()
    };
    let mut rng = ChaCha20Rng::seed_from_u64(2026);
    let mut counts = std::collections::HashMap::<char, usize>::new();
    let runs = 4000;
    for _ in 0..runs {
        for c in generate(&req, &mut rng).unwrap().password.chars() {
            *counts.entry(c).or_default() += 1;
        }
    }
    let pool_size = 26 + 26 + DEFAULT_SYMBOLS.len();
    assert_eq!(
        counts.len(),
        pool_size,
        "every character in the pool appears"
    );
    let expected = (runs * 32) as f64 / pool_size as f64;
    for (c, n) in counts {
        let off = (n as f64 - expected).abs() / expected;
        assert!(
            off < 0.15,
            "'{c}' appeared {n} times, expected about {expected:.0}"
        );
    }
}
