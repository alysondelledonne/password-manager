//! Errors the generator can return.
//!
//! Every message is written for the person using the app, because the UI shows
//! it as-is. [`GenerateError::code`] gives a stable machine-readable code so the
//! UI can highlight the right field.

use thiserror::Error;

/// Something is wrong with the rule set, or no password could satisfy it.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GenerateError {
    #[error("Length must be at least 1 character.")]
    LengthTooSmall,

    #[error("Maximum length can't be more than {max} characters.")]
    LengthTooLarge { max: usize },

    #[error("Minimum length ({min}) can't be more than maximum length ({max}).")]
    MinAboveMax { min: usize, max: usize },

    #[error("Password length ({target}) must be between the minimum ({min}) and maximum ({max}).")]
    TargetOutOfRange {
        target: usize,
        min: usize,
        max: usize,
    },

    #[error(
        "The required characters and phrase need {required} characters, \
         but the password is only {target} long. Increase the length or lower a requirement."
    )]
    RequiredExceedsLength { required: usize, target: usize },

    #[error(
        "Special characters are required, but none are allowed. Add some allowed special characters."
    )]
    NoSymbolsAvailable,

    #[error("'{0}' can't be used as a special character. Use punctuation such as ! @ # $.")]
    InvalidSymbol(char),

    #[error("The repeat limit must be at least 1.")]
    InvalidRepeatLimit,

    #[error(transparent)]
    Phrase(#[from] PhraseError),

    #[error(
        "Couldn't build a password that meets all the rules after {0} tries. \
         Loosen a rule or increase the length."
    )]
    TooStrict(usize),
}

/// The memorable phrase breaks a rule. The user is asked to change the phrase;
/// the app never edits it silently.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PhraseError {
    #[error("The phrase is empty. Enter a phrase or turn the phrase option off.")]
    Empty,

    #[error("The phrase contains numbers. Remove them; numbers are added separately.")]
    ContainsDigits,

    #[error(
        "The phrase contains spaces, but spaces aren't allowed. Remove the spaces or allow them."
    )]
    ContainsSpaces,

    #[error("The phrase contains '{0}', but only plain English letters (ASCII) are allowed.")]
    NonAscii(char),

    #[error("The phrase contains a character that can't be used in a password.")]
    InvalidCharacter,

    #[error("The phrase contains '{0}', which isn't in your allowed special characters.")]
    DisallowedSymbol(char),

    #[error("The phrase contains '{0}', a look-alike character you chose to exclude.")]
    ContainsLookalike(char),

    #[error("The phrase repeats a character more than {limit} times in a row.")]
    TooManyRepeats { limit: usize },

    #[error("The phrase contains the sequence \"{0}\", and sequences aren't allowed.")]
    ContainsSequence(String),

    #[error("The phrase contains your username or email.")]
    ContainsUsername,
}

impl GenerateError {
    /// A stable, machine-readable code for the UI.
    pub fn code(&self) -> &'static str {
        match self {
            Self::LengthTooSmall => "length_too_small",
            Self::LengthTooLarge { .. } => "length_too_large",
            Self::MinAboveMax { .. } => "min_above_max",
            Self::TargetOutOfRange { .. } => "target_out_of_range",
            Self::RequiredExceedsLength { .. } => "required_exceeds_length",
            Self::NoSymbolsAvailable => "no_symbols_available",
            Self::InvalidSymbol(_) => "invalid_symbol",
            Self::InvalidRepeatLimit => "invalid_repeat_limit",
            Self::Phrase(_) => "phrase",
            Self::TooStrict(_) => "too_strict",
        }
    }
}
