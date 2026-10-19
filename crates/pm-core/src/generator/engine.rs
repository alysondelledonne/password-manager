//! Builds a password (architecture doc, section 4.3).
//!
//! 1. Validate the policy and phrase.
//! 2. Fill the required slots (uppercase, lowercase, exact digits, symbols),
//!    then the rest from the fill pool, which never contains digits.
//! 3. Shuffle, then insert the phrase at a random position.
//! 4. Check the remaining rules. If any fails, throw the candidate away and
//!    start over. Rejecting instead of patching keeps every valid password
//!    equally likely.

use std::fmt;

use rand::rand_core::UnwrapErr;
use rand::rngs::SysRng;
use rand::seq::{IndexedRandom, SliceRandom};
use rand::{CryptoRng, RngExt};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::generator::charsets::Pools;
use crate::generator::error::GenerateError;
use crate::generator::phrase::{self, PhraseOptions};
use crate::generator::policy::PasswordPolicy;
use crate::generator::rules;
use crate::generator::strength::{self, Strength};

/// How many candidates to try before reporting that the rules are too strict.
pub const MAX_ATTEMPTS: usize = 1000;

/// Everything the UI sends to generate one password.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GenerateRequest {
    pub policy: PasswordPolicy,
    #[serde(default)]
    pub phrase: Option<PhraseOptions>,
    /// Optional. Used only for rule R13 (the password must not contain it).
    #[serde(default)]
    pub username: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedPassword {
    /// Wiped from memory when dropped.
    pub password: Zeroizing<String>,
    /// Estimated entropy of the random part (the phrase counts as zero).
    pub entropy_bits: f64,
    pub strength: Strength,
    /// How many candidates were built before one passed every rule.
    pub attempts: usize,
}

impl fmt::Debug for GeneratedPassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeneratedPassword")
            .field("password", &"[REDACTED]")
            .field("entropy_bits", &self.entropy_bits)
            .field("strength", &self.strength)
            .field("attempts", &self.attempts)
            .finish()
    }
}

/// Generates a password using the operating system's secure random number generator.
///
/// # Panics
/// If the OS can't provide randomness, which in practice only happens on a broken system.
/// Failing loudly is safer than falling back to weaker randomness.
pub fn generate_with_os_rng(request: &GenerateRequest) -> Result<GeneratedPassword, GenerateError> {
    let mut rng = UnwrapErr(SysRng);
    generate(request, &mut rng)
}

/// Generates a password with the given RNG. `CryptoRng` is required, so a
/// non-cryptographic RNG can't be passed in by mistake. Tests use a seeded
/// ChaCha20 RNG so results are repeatable.
pub fn generate<R: CryptoRng + ?Sized>(
    request: &GenerateRequest,
    rng: &mut R,
) -> Result<GeneratedPassword, GenerateError> {
    let policy = &request.policy;
    policy.validate()?;

    let username = request
        .username
        .as_deref()
        .map(str::trim)
        .filter(|u| !u.is_empty());

    let phrase_len = match &request.phrase {
        Some(opts) => {
            phrase::validate(&opts.phrase, policy, username)?;
            opts.phrase.chars().count()
        }
        None => 0,
    };

    let required = policy.required_chars() + phrase_len;
    if required > policy.target_length {
        return Err(GenerateError::RequiredExceedsLength {
            required,
            target: policy.target_length,
        });
    }

    let pools = Pools::for_policy(policy);
    let random_len = policy.target_length - phrase_len;
    let entropy_bits = strength::entropy_bits(policy, &pools, random_len, request.phrase.is_some());

    for attempt in 1..=MAX_ATTEMPTS {
        let random_part = build_random_part(policy, &pools, random_len, rng);
        let prepared_phrase = request
            .phrase
            .as_ref()
            .map(|opts| phrase::prepare(opts, policy, rng));
        let password = assemble(
            &random_part,
            prepared_phrase.as_deref().map(String::as_str),
            rng,
        );

        debug_assert_eq!(password.chars().count(), policy.target_length);
        debug_assert_eq!(
            password.chars().filter(char::is_ascii_digit).count(),
            policy.exact_digits
        );

        if rules::check(&password, policy, username).is_ok() {
            return Ok(GeneratedPassword {
                password,
                entropy_bits,
                strength: Strength::from_bits(entropy_bits),
                attempts: attempt,
            });
        }
    }
    Err(GenerateError::TooStrict(MAX_ATTEMPTS))
}

fn build_random_part<R: CryptoRng + ?Sized>(
    policy: &PasswordPolicy,
    pools: &Pools,
    len: usize,
    rng: &mut R,
) -> Zeroizing<Vec<char>> {
    let mut slots = Zeroizing::new(Vec::with_capacity(len));
    let required = [
        (&pools.upper, policy.min_upper),
        (&pools.lower, policy.min_lower),
        (&pools.digits, policy.exact_digits),
        (&pools.symbols, policy.min_symbols),
    ];
    for (pool, count) in required {
        for _ in 0..count {
            slots.push(pick(pool, rng));
        }
    }
    while slots.len() < len {
        slots.push(pick(&pools.fill, rng));
    }
    slots.shuffle(rng);
    slots
}

/// Picks one character uniformly at random.
fn pick<R: CryptoRng + ?Sized>(pool: &[char], rng: &mut R) -> char {
    // Pools are never empty when they're drawn from: letters and digits always
    // have characters left after exclusions, and `PasswordPolicy::validate`
    // rejects required symbols with an empty symbol pool.
    *pool.choose(rng).expect("character pool is never empty")
}

fn assemble<R: CryptoRng + ?Sized>(
    random: &[char],
    phrase: Option<&str>,
    rng: &mut R,
) -> Zeroizing<String> {
    let mut out = Zeroizing::new(String::with_capacity(
        random.len() * 4 + phrase.map_or(0, str::len),
    ));
    match phrase {
        None => out.extend(random),
        Some(p) => {
            let at = rng.random_range(0..=random.len());
            out.extend(&random[..at]);
            out.push_str(p);
            out.extend(&random[at..]);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::error::PhraseError;
    use crate::generator::phrase::PhraseMode;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    fn rng() -> ChaCha20Rng {
        ChaCha20Rng::seed_from_u64(42)
    }

    fn phrase(text: &str, mode: PhraseMode) -> Option<PhraseOptions> {
        Some(PhraseOptions {
            phrase: Zeroizing::new(text.into()),
            mode,
        })
    }

    #[test]
    fn default_policy_meets_every_requirement() {
        let req = GenerateRequest::default();
        let mut rng = rng();
        for _ in 0..500 {
            let out = generate(&req, &mut rng).unwrap();
            let pw = out.password.as_str();
            assert_eq!(pw.chars().count(), 16);
            assert_eq!(pw.chars().filter(char::is_ascii_digit).count(), 3);
            assert!(pw.chars().any(|c| c.is_ascii_uppercase()));
            assert!(pw.chars().any(|c| c.is_ascii_lowercase()));
            assert!(pw.chars().any(|c| req.policy.allowed_symbols.contains(&c)));
            assert!(rules::longest_run(pw) <= 2);
            assert_eq!(rules::find_sequence(pw), None);
            assert_eq!(out.strength, Strength::Strong);
        }
    }

    #[test]
    fn as_is_phrase_appears_verbatim() {
        let req = GenerateRequest {
            phrase: phrase("BlueHeron", PhraseMode::AsIs),
            ..Default::default()
        };
        let out = generate(&req, &mut rng()).unwrap();
        assert!(out.password.contains("BlueHeron"));
        assert_eq!(out.password.chars().count(), 16);
    }

    #[test]
    fn transformed_phrase_keeps_exact_digit_count() {
        let req = GenerateRequest {
            phrase: phrase("blueheronsail", PhraseMode::Transformed),
            policy: PasswordPolicy {
                target_length: 24,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut rng = rng();
        for _ in 0..200 {
            let out = generate(&req, &mut rng).unwrap();
            assert_eq!(out.password.chars().filter(char::is_ascii_digit).count(), 3);
        }
    }

    #[test]
    fn phrase_errors_reach_the_caller() {
        let req = GenerateRequest {
            phrase: phrase("summer2019", PhraseMode::AsIs),
            ..Default::default()
        };
        assert_eq!(
            generate(&req, &mut rng()).unwrap_err(),
            GenerateError::Phrase(PhraseError::ContainsDigits)
        );
    }

    #[test]
    fn phrase_too_long_for_the_length_is_rejected() {
        let req = GenerateRequest {
            phrase: phrase("averyveryverylongphrase", PhraseMode::AsIs),
            ..Default::default()
        };
        assert!(matches!(
            generate(&req, &mut rng()),
            Err(GenerateError::RequiredExceedsLength {
                required: 29,
                target: 16
            })
        ));
    }

    #[test]
    fn impossible_rules_report_too_strict() {
        // Every character must be a digit, but the password must start with a letter.
        let req = GenerateRequest {
            policy: PasswordPolicy {
                min_length: 4,
                target_length: 4,
                min_upper: 0,
                min_lower: 0,
                min_symbols: 0,
                exact_digits: 4,
                start_with_letter: true,
                no_sequences: false,
                max_repeat: None,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(matches!(
            generate(&req, &mut rng()),
            Err(GenerateError::TooStrict(MAX_ATTEMPTS))
        ));
    }

    #[test]
    fn username_never_appears() {
        let req = GenerateRequest {
            username: Some("abc@example.com".into()),
            policy: PasswordPolicy {
                min_length: 4,
                target_length: 4,
                min_upper: 0,
                min_symbols: 0,
                exact_digits: 0,
                no_sequences: false,
                allowed_symbols: Default::default(),
                ..Default::default()
            },
            ..Default::default()
        };
        let mut rng = rng();
        for _ in 0..2000 {
            let out = generate(&req, &mut rng).unwrap();
            assert!(!out.password.to_lowercase().contains("abc"));
        }
    }

    #[test]
    fn same_seed_gives_same_password_and_different_seeds_differ() {
        let req = GenerateRequest::default();
        let a = generate(&req, &mut ChaCha20Rng::seed_from_u64(1)).unwrap();
        let b = generate(&req, &mut ChaCha20Rng::seed_from_u64(1)).unwrap();
        let c = generate(&req, &mut ChaCha20Rng::seed_from_u64(2)).unwrap();
        assert_eq!(a.password, b.password);
        assert_ne!(a.password, c.password);
    }

    #[test]
    fn os_rng_works() {
        let a = generate_with_os_rng(&GenerateRequest::default()).unwrap();
        let b = generate_with_os_rng(&GenerateRequest::default()).unwrap();
        assert_ne!(a.password, b.password, "regenerate gives a fresh password");
    }

    #[test]
    fn debug_output_hides_the_password() {
        let out = generate(&GenerateRequest::default(), &mut rng()).unwrap();
        assert!(!format!("{out:?}").contains(out.password.as_str()));
    }
}
