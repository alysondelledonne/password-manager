//! The optional memorable phrase (architecture doc, section 4.4).
//!
//! Decisions (2026-10-19):
//! - Transformed mode only swaps letters for **symbols** (`a→@`, `s→$`, `i→!`),
//!   never digits, so the exact digit count (R5) stays exact.
//! - A phrase containing digits is rejected.
//! - A phrase that breaks any rule is rejected with a message telling the user
//!   what to change. The app never edits the phrase silently.
//!
//! The phrase gets zero strength credit: assume an attacker can guess it.

use std::fmt;

use rand::{CryptoRng, RngExt};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::generator::charsets::is_lookalike;
use crate::generator::error::PhraseError;
use crate::generator::policy::PasswordPolicy;
use crate::generator::rules;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PhraseMode {
    /// Random letter case plus some letter-to-symbol swaps.
    Transformed,
    /// Used exactly as typed.
    AsIs,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PhraseOptions {
    /// Wiped from memory when dropped.
    pub phrase: Zeroizing<String>,
    pub mode: PhraseMode,
}

impl fmt::Debug for PhraseOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhraseOptions")
            .field("phrase", &"[REDACTED]")
            .field("mode", &self.mode)
            .finish()
    }
}

/// Letter → symbol swaps for transformed mode. A swap is only used if the
/// symbol is in the policy's allowed set.
pub const SWAPS: &[(char, char)] = &[('a', '@'), ('s', '$'), ('i', '!')];

/// Rejects a phrase that breaks a rule, saying what to change.
pub fn validate(
    phrase: &str,
    policy: &PasswordPolicy,
    username: Option<&str>,
) -> Result<(), PhraseError> {
    if phrase.trim().is_empty() {
        return Err(PhraseError::Empty);
    }
    for c in phrase.chars() {
        if c.is_control() {
            return Err(PhraseError::InvalidCharacter);
        }
        if c.is_numeric() {
            return Err(PhraseError::ContainsDigits);
        }
        if c.is_whitespace() {
            if policy.no_spaces {
                return Err(PhraseError::ContainsSpaces);
            }
            continue;
        }
        if policy.ascii_only && !c.is_ascii() {
            return Err(PhraseError::NonAscii(c));
        }
        if !c.is_alphanumeric() && !policy.allowed_symbols.contains(&c) {
            return Err(PhraseError::DisallowedSymbol(c));
        }
        if policy.exclude_lookalikes && is_lookalike(c) {
            return Err(PhraseError::ContainsLookalike(c));
        }
    }
    if let Some(limit) = policy.max_repeat
        && rules::longest_run(phrase) > limit
    {
        return Err(PhraseError::TooManyRepeats { limit });
    }
    if policy.no_sequences
        && let Some(seq) = rules::find_sequence(phrase)
    {
        return Err(PhraseError::ContainsSequence(seq));
    }
    if let Some(name) = username
        && rules::contains_username(phrase, name)
    {
        return Err(PhraseError::ContainsUsername);
    }
    Ok(())
}

/// Returns the phrase as it will appear in the password. Call [`validate`] first.
/// Transformed output has the same number of characters as the input.
pub fn prepare<R: CryptoRng + ?Sized>(
    opts: &PhraseOptions,
    policy: &PasswordPolicy,
    rng: &mut R,
) -> Zeroizing<String> {
    match opts.mode {
        PhraseMode::AsIs => Zeroizing::new(opts.phrase.to_string()),
        PhraseMode::Transformed => transform(&opts.phrase, policy, rng),
    }
}

fn transform<R: CryptoRng + ?Sized>(
    phrase: &str,
    policy: &PasswordPolicy,
    rng: &mut R,
) -> Zeroizing<String> {
    let mut out = Zeroizing::new(String::with_capacity(phrase.len()));
    for c in phrase.chars() {
        if !c.is_ascii_alphabetic() {
            out.push(c);
            continue;
        }
        let lower = c.to_ascii_lowercase();
        let mut options = vec![lower, c.to_ascii_uppercase()];
        if let Some(&(_, symbol)) = SWAPS.iter().find(|(letter, _)| *letter == lower)
            && policy.allowed_symbols.contains(&symbol)
        {
            options.push(symbol);
        }
        // Never introduce a look-alike (e.g. 'i' → 'I') when they're excluded.
        options.retain(|o| !(policy.exclude_lookalikes && is_lookalike(*o)));
        let pick = if options.is_empty() {
            c
        } else {
            options[rng.random_range(0..options.len())]
        };
        out.push(pick);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    fn policy() -> PasswordPolicy {
        PasswordPolicy::default()
    }

    #[test]
    fn rejects_each_rule_break_with_a_specific_error() {
        let p = policy();
        assert_eq!(validate("   ", &p, None), Err(PhraseError::Empty));
        assert_eq!(
            validate("summer2019", &p, None),
            Err(PhraseError::ContainsDigits)
        );
        assert_eq!(
            validate("blue heron", &p, None),
            Err(PhraseError::ContainsSpaces)
        );
        assert_eq!(validate("café", &p, None), Err(PhraseError::NonAscii('é')));
        assert_eq!(
            validate("blue~heron", &p, None),
            Err(PhraseError::DisallowedSymbol('~'))
        );
        assert_eq!(
            validate("baaad", &p, None),
            Err(PhraseError::TooManyRepeats { limit: 2 })
        );
        assert_eq!(
            validate("first", &p, None),
            Err(PhraseError::ContainsSequence("rst".into()))
        );
        assert_eq!(
            validate("bluealyson", &p, Some("alyson@example.com")),
            Err(PhraseError::ContainsUsername)
        );
    }

    #[test]
    fn lookalikes_rejected_only_when_excluded() {
        let mut p = policy();
        assert_eq!(validate("blueheron", &p, None), Ok(()));
        p.exclude_lookalikes = true;
        assert_eq!(
            validate("blueheron", &p, None),
            Err(PhraseError::ContainsLookalike('l'))
        );
    }

    #[test]
    fn spaces_allowed_when_the_rule_is_off() {
        let p = PasswordPolicy {
            no_spaces: false,
            ..policy()
        };
        assert_eq!(validate("blue heron", &p, None), Ok(()));
    }

    #[test]
    fn transformed_keeps_length_and_never_adds_digits() {
        let p = policy();
        let mut rng = ChaCha20Rng::seed_from_u64(7);
        for _ in 0..200 {
            let opts = PhraseOptions {
                phrase: Zeroizing::new("blueheronsail".into()),
                mode: PhraseMode::Transformed,
            };
            let out = prepare(&opts, &p, &mut rng);
            assert_eq!(out.chars().count(), 13);
            assert!(!out.chars().any(|c| c.is_ascii_digit()));
            for (orig, new) in "blueheronsail".chars().zip(out.chars()) {
                let swapped = SWAPS.iter().any(|&(l, s)| l == orig && s == new);
                assert!(
                    new.eq_ignore_ascii_case(&orig) || swapped,
                    "{orig} became {new}"
                );
            }
        }
    }

    #[test]
    fn transformed_skips_swaps_for_symbols_that_are_not_allowed() {
        let mut p = policy();
        p.allowed_symbols.retain(|c| *c != '@');
        let mut rng = ChaCha20Rng::seed_from_u64(1);
        let opts = PhraseOptions {
            phrase: Zeroizing::new("aaaa".to_string()),
            mode: PhraseMode::Transformed,
        };
        for _ in 0..100 {
            assert!(!prepare(&opts, &p, &mut rng).contains('@'));
        }
    }

    #[test]
    fn as_is_returns_the_phrase_unchanged() {
        let opts = PhraseOptions {
            phrase: Zeroizing::new("BlueHeron".into()),
            mode: PhraseMode::AsIs,
        };
        let mut rng = ChaCha20Rng::seed_from_u64(0);
        assert_eq!(prepare(&opts, &policy(), &mut rng).as_str(), "BlueHeron");
    }

    #[test]
    fn debug_output_hides_the_phrase() {
        let opts = PhraseOptions {
            phrase: Zeroizing::new("secretword".into()),
            mode: PhraseMode::AsIs,
        };
        assert!(!format!("{opts:?}").contains("secretword"));
    }
}
