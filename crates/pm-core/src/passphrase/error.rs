//! Errors for word-based passwords. Messages are shown to the person as-is.

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PassphraseError {
    #[error("The shortest length must be at least 1 character.")]
    LengthTooSmall,

    #[error("The longest length can't be more than {max} characters.")]
    LengthTooLarge { max: usize },

    #[error("The shortest length ({min}) can't be more than the longest length ({max}).")]
    MinAboveMax { min: usize, max: usize },

    #[error("Turn on uppercase or lowercase letters: the words are made of letters.")]
    NoLetters,

    #[error("Special characters are required, but none are selected. Select at least one.")]
    NoSymbolsSelected,

    #[error("'{0}' can't be used as a special character. Use punctuation such as ! @ # $.")]
    InvalidSymbol(char),

    #[error(
        "With special characters off, digits separate the words, so you need at least 1 digit. \
         Add a digit or turn special characters on."
    )]
    NeedsSeparator,

    #[error("Enter at least 2 different words, or switch to random words.")]
    TooFewWords,

    #[error("Enter at most {max} words.")]
    TooManyWords { max: usize },

    #[error("\"{0}\" contains a space. Enter one word per box.")]
    WordHasSpace(String),

    #[error("\"{0}\" contains a number. Remove it; digits are added separately.")]
    WordHasDigit(String),

    #[error(
        "\"{word}\" contains '{character}'. Use letters only; special characters are added separately."
    )]
    WordNotLetters { word: String, character: char },

    #[error("\"{word}\" is longer than {max} letters. Use a shorter word.")]
    WordTooLong { word: String, max: usize },

    #[error(
        "No 2- or 3-word password fits between {min} and {max} characters with {digits} digit(s). \
         Widen the length range{hint}."
    )]
    DoesNotFit {
        min: usize,
        max: usize,
        digits: usize,
        /// Extra advice, e.g. about the person's own words. Starts with ", " or is empty.
        hint: &'static str,
    },
}

impl PassphraseError {
    /// Stable machine-readable code for the UI.
    pub fn code(&self) -> &'static str {
        match self {
            Self::LengthTooSmall => "length_too_small",
            Self::LengthTooLarge { .. } => "length_too_large",
            Self::MinAboveMax { .. } => "min_above_max",
            Self::NoLetters => "no_letters",
            Self::NoSymbolsSelected => "no_symbols_selected",
            Self::InvalidSymbol(_) => "invalid_symbol",
            Self::NeedsSeparator => "needs_separator",
            Self::TooFewWords => "too_few_words",
            Self::TooManyWords { .. } => "too_many_words",
            Self::WordHasSpace(_) => "word_has_space",
            Self::WordHasDigit(_) => "word_has_digit",
            Self::WordNotLetters { .. } => "word_not_letters",
            Self::WordTooLong { .. } => "word_too_long",
            Self::DoesNotFit { .. } => "does_not_fit",
        }
    }

    /// Which part of the form the UI should highlight.
    pub fn field(&self) -> Option<&'static str> {
        match self {
            Self::LengthTooSmall
            | Self::LengthTooLarge { .. }
            | Self::MinAboveMax { .. }
            | Self::DoesNotFit { .. } => Some("length"),
            Self::NoLetters => Some("letters"),
            Self::NoSymbolsSelected | Self::InvalidSymbol(_) => Some("symbols"),
            Self::NeedsSeparator => Some("digits"),
            Self::TooFewWords
            | Self::TooManyWords { .. }
            | Self::WordHasSpace(_)
            | Self::WordHasDigit(_)
            | Self::WordNotLetters { .. }
            | Self::WordTooLong { .. } => Some("words"),
        }
    }
}
