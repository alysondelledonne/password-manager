//! Memorable word-based passwords (architecture doc, section 4.6).
//!
//! A password is 2 or 3 words joined by separators, e.g. `Otter7Wagon!Maple2`:
//!
//! - **Words** come from EFF's short word list (random) or from words the
//!   person typed (2 or 3 of them, picked at random).
//! - **Separators** between words are special characters or digits. Digits
//!   used as separators count toward the exact digit total; any digits left
//!   over go at the end.
//! - **Capitals:** the first letter of each word when both cases are on, all
//!   caps when only uppercase is on, all lowercase when only lowercase is on.
//! - **Word count:** 3 if any 3-word password fits the length range, else 2.
//!
//! Everything random is chosen uniformly among the passwords that fit, using
//! the OS random number generator.

pub mod engine;
pub mod error;
pub mod rating;
pub mod wordlist;

use std::collections::BTreeSet;
use std::fmt;

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::generator::charsets::DEFAULT_SYMBOLS;
use crate::generator::policy::MAX_SUPPORTED_LENGTH;

pub use engine::{GeneratedPassphrase, generate_passphrase, generate_passphrase_with_os_rng};
pub use error::PassphraseError;
pub use rating::{Shape, guess_stars};

/// Most words a person can enter.
pub const MAX_OWN_WORDS: usize = 10;
/// Longest word a person can enter.
pub const MAX_WORD_LENGTH: usize = 24;

/// The rules a word-based password must follow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PassphrasePolicy {
    /// Shortest password the site accepts.
    pub min_length: usize,
    /// Longest password the site accepts.
    pub max_length: usize,
    /// Exactly this many digits in the whole password.
    pub digits: usize,
    pub uppercase: bool,
    pub lowercase: bool,
    /// At least one separator is a special character from `allowed_symbols`.
    /// When off, no special characters are used and every separator is a digit.
    pub symbols_required: bool,
    pub allowed_symbols: BTreeSet<char>,
}

impl Default for PassphrasePolicy {
    fn default() -> Self {
        Self {
            min_length: 12,
            max_length: 24,
            digits: 2,
            uppercase: true,
            lowercase: true,
            symbols_required: true,
            allowed_symbols: DEFAULT_SYMBOLS.chars().collect(),
        }
    }
}

/// Where the words come from.
#[derive(Clone, Deserialize)]
#[serde(tag = "kind", content = "words", rename_all = "camelCase")]
pub enum WordSource {
    /// Random words from EFF's short word list.
    Random,
    /// Words the person typed. 2 or 3 of them are picked at random.
    Mine(Vec<Zeroizing<String>>),
}

impl fmt::Debug for WordSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Random => f.write_str("Random"),
            Self::Mine(words) => write!(f, "Mine([{} words, REDACTED])", words.len()),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PassphraseRequest {
    pub policy: PassphrasePolicy,
    pub words: WordSource,
}

impl PassphrasePolicy {
    /// Checks the rules make sense on their own, before any words are considered.
    pub fn validate(&self) -> Result<(), PassphraseError> {
        if self.min_length == 0 {
            return Err(PassphraseError::LengthTooSmall);
        }
        if self.max_length > MAX_SUPPORTED_LENGTH {
            return Err(PassphraseError::LengthTooLarge {
                max: MAX_SUPPORTED_LENGTH,
            });
        }
        if self.min_length > self.max_length {
            return Err(PassphraseError::MinAboveMax {
                min: self.min_length,
                max: self.max_length,
            });
        }
        if !self.uppercase && !self.lowercase {
            return Err(PassphraseError::NoLetters);
        }
        if let Some(bad) = self
            .allowed_symbols
            .iter()
            .find(|c| !c.is_ascii_punctuation())
        {
            return Err(PassphraseError::InvalidSymbol(*bad));
        }
        if self.symbols_required && self.allowed_symbols.is_empty() {
            return Err(PassphraseError::NoSymbolsSelected);
        }
        if !self.symbols_required && self.digits == 0 {
            return Err(PassphraseError::NeedsSeparator);
        }
        Ok(())
    }

    /// Applies the capitalization rule to one lowercase word.
    pub fn apply_case(&self, word: &str) -> String {
        match (self.uppercase, self.lowercase) {
            (true, true) => {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                    None => String::new(),
                }
            }
            (true, false) => word.to_uppercase(),
            _ => word.to_lowercase(),
        }
    }
}

/// Checks the person's words and returns them lowercased, without duplicates,
/// in the order entered. A word that breaks a rule is rejected with a message;
/// it is never edited silently.
pub fn prepare_own_words(
    words: &[Zeroizing<String>],
) -> Result<Vec<Zeroizing<String>>, PassphraseError> {
    if words.len() > MAX_OWN_WORDS {
        return Err(PassphraseError::TooManyWords { max: MAX_OWN_WORDS });
    }
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for word in words {
        let word = word.as_str();
        if word.is_empty() {
            continue;
        }
        if word.chars().any(char::is_whitespace) {
            return Err(PassphraseError::WordHasSpace(word.to_string()));
        }
        if word.chars().any(char::is_numeric) {
            return Err(PassphraseError::WordHasDigit(word.to_string()));
        }
        if let Some(c) = word.chars().find(|c| !c.is_alphabetic()) {
            return Err(PassphraseError::WordNotLetters {
                word: word.to_string(),
                character: c,
            });
        }
        if word.chars().count() > MAX_WORD_LENGTH {
            return Err(PassphraseError::WordTooLong {
                word: word.to_string(),
                max: MAX_WORD_LENGTH,
            });
        }
        let lower = Zeroizing::new(word.to_lowercase());
        if seen.insert(lower.to_string()) {
            out.push(lower);
        }
    }
    if out.len() < 2 {
        return Err(PassphraseError::TooFewWords);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(list: &[&str]) -> Vec<Zeroizing<String>> {
        list.iter().map(|w| Zeroizing::new(w.to_string())).collect()
    }

    #[test]
    fn defaults_are_valid() {
        assert_eq!(PassphrasePolicy::default().validate(), Ok(()));
    }

    #[test]
    fn policy_errors() {
        let p = PassphrasePolicy {
            uppercase: false,
            lowercase: false,
            ..Default::default()
        };
        assert_eq!(p.validate(), Err(PassphraseError::NoLetters));

        let p = PassphrasePolicy {
            allowed_symbols: BTreeSet::new(),
            ..Default::default()
        };
        assert_eq!(p.validate(), Err(PassphraseError::NoSymbolsSelected));

        let p = PassphrasePolicy {
            symbols_required: false,
            digits: 0,
            ..Default::default()
        };
        assert_eq!(p.validate(), Err(PassphraseError::NeedsSeparator));

        let p = PassphrasePolicy {
            min_length: 30,
            max_length: 20,
            ..Default::default()
        };
        assert_eq!(
            p.validate(),
            Err(PassphraseError::MinAboveMax { min: 30, max: 20 })
        );
    }

    #[test]
    fn capitalization_rules() {
        let both = PassphrasePolicy::default();
        assert_eq!(both.apply_case("otter"), "Otter");
        let upper = PassphrasePolicy {
            lowercase: false,
            ..Default::default()
        };
        assert_eq!(upper.apply_case("otter"), "OTTER");
        let lower = PassphrasePolicy {
            uppercase: false,
            ..Default::default()
        };
        assert_eq!(lower.apply_case("otter"), "otter");
    }

    #[test]
    fn own_words_are_checked_and_never_edited_silently() {
        assert_eq!(
            prepare_own_words(&words(&["blue heron", "maple"])).unwrap_err(),
            PassphraseError::WordHasSpace("blue heron".into())
        );
        assert_eq!(
            prepare_own_words(&words(&["summer2019", "maple"])).unwrap_err(),
            PassphraseError::WordHasDigit("summer2019".into())
        );
        assert_eq!(
            prepare_own_words(&words(&["rock'n", "maple"])).unwrap_err(),
            PassphraseError::WordNotLetters {
                word: "rock'n".into(),
                character: '\''
            }
        );
        assert_eq!(
            prepare_own_words(&words(&["maple"])).unwrap_err(),
            PassphraseError::TooFewWords
        );
    }

    #[test]
    fn own_words_are_lowercased_and_deduplicated() {
        let out = prepare_own_words(&words(&["Maple", "maple", "", "Otter"])).unwrap();
        let out: Vec<&str> = out.iter().map(|w| w.as_str()).collect();
        assert_eq!(out, ["maple", "otter"]);
        assert_eq!(
            prepare_own_words(&words(&["Maple", "MAPLE"])).unwrap_err(),
            PassphraseError::TooFewWords,
            "duplicates don't count as two words"
        );
    }

    #[test]
    fn debug_output_hides_own_words() {
        let source = WordSource::Mine(words(&["secretword", "other"]));
        assert!(!format!("{source:?}").contains("secretword"));
    }
}
