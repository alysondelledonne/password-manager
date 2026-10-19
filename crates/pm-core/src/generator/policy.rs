//! The rule set the user picks (architecture doc, section 4.1).

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::generator::charsets::{DEFAULT_SYMBOLS, Pools};
use crate::generator::error::GenerateError;

/// Longest password the generator will build. Guards against absurd input from the UI.
pub const MAX_SUPPORTED_LENGTH: usize = 128;

/// The rules a generated password must follow.
///
/// `min_length`/`max_length` are the site's limits; `target_length` is the
/// length the user chose within them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PasswordPolicy {
    /// R1
    pub min_length: usize,
    /// R2
    pub max_length: usize,
    pub target_length: usize,
    /// R3. 0 = not required (uppercase can still appear).
    pub min_upper: usize,
    /// R4. 0 = not required (lowercase can still appear).
    pub min_lower: usize,
    /// R5. Exactly this many digits, no more and no fewer.
    pub exact_digits: usize,
    /// R6. 0 = not required. Symbols can still appear if `allowed_symbols` isn't empty.
    pub min_symbols: usize,
    /// R7. Empty = no special characters at all.
    pub allowed_symbols: BTreeSet<char>,
    /// R8
    pub no_spaces: bool,
    /// R9
    pub start_with_letter: bool,
    /// R10
    pub exclude_lookalikes: bool,
    /// R11. `Some(n)`: no more than `n` identical characters in a row. `None`: off.
    pub max_repeat: Option<usize>,
    /// R12. No runs of 3+ like `abc`, `321` or `qwe`.
    pub no_sequences: bool,
    /// R15
    pub ascii_only: bool,
}

impl Default for PasswordPolicy {
    /// The proposed defaults from the architecture doc (section 4.1).
    fn default() -> Self {
        Self {
            min_length: 16,
            max_length: 64,
            target_length: 16,
            min_upper: 1,
            min_lower: 1,
            exact_digits: 3,
            min_symbols: 1,
            allowed_symbols: DEFAULT_SYMBOLS.chars().collect(),
            no_spaces: true,
            start_with_letter: false,
            exclude_lookalikes: false,
            max_repeat: Some(2),
            no_sequences: true,
            ascii_only: true,
        }
    }
}

impl PasswordPolicy {
    /// Characters reserved for required classes (uppercase, lowercase, digits, symbols).
    pub fn required_chars(&self) -> usize {
        self.min_upper + self.min_lower + self.exact_digits + self.min_symbols
    }

    /// Checks the rules make sense on their own, before any phrase is considered.
    pub fn validate(&self) -> Result<(), GenerateError> {
        if self.min_length == 0 || self.target_length == 0 {
            return Err(GenerateError::LengthTooSmall);
        }
        if self.max_length > MAX_SUPPORTED_LENGTH || self.target_length > MAX_SUPPORTED_LENGTH {
            return Err(GenerateError::LengthTooLarge {
                max: MAX_SUPPORTED_LENGTH,
            });
        }
        if self.min_length > self.max_length {
            return Err(GenerateError::MinAboveMax {
                min: self.min_length,
                max: self.max_length,
            });
        }
        if !(self.min_length..=self.max_length).contains(&self.target_length) {
            return Err(GenerateError::TargetOutOfRange {
                target: self.target_length,
                min: self.min_length,
                max: self.max_length,
            });
        }
        if let Some(bad) = self
            .allowed_symbols
            .iter()
            .find(|c| !c.is_ascii_punctuation())
        {
            return Err(GenerateError::InvalidSymbol(*bad));
        }
        if self.max_repeat == Some(0) {
            return Err(GenerateError::InvalidRepeatLimit);
        }
        if self.min_symbols > 0 && Pools::for_policy(self).symbols.is_empty() {
            return Err(GenerateError::NoSymbolsAvailable);
        }
        if self.required_chars() > self.target_length {
            return Err(GenerateError::RequiredExceedsLength {
                required: self.required_chars(),
                target: self.target_length,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        assert_eq!(PasswordPolicy::default().validate(), Ok(()));
    }

    #[test]
    fn target_must_be_inside_min_and_max() {
        let p = PasswordPolicy {
            target_length: 70,
            ..PasswordPolicy::default()
        };
        assert!(matches!(
            p.validate(),
            Err(GenerateError::TargetOutOfRange { .. })
        ));
    }

    #[test]
    fn required_characters_must_fit() {
        let p = PasswordPolicy {
            min_length: 8,
            target_length: 8,
            min_upper: 3,
            min_lower: 3,
            exact_digits: 3,
            ..PasswordPolicy::default()
        };
        assert_eq!(
            p.validate(),
            Err(GenerateError::RequiredExceedsLength {
                required: 10,
                target: 8
            })
        );
    }

    #[test]
    fn required_symbols_need_an_allowed_symbol() {
        let p = PasswordPolicy {
            allowed_symbols: BTreeSet::new(),
            ..PasswordPolicy::default()
        };
        assert_eq!(p.validate(), Err(GenerateError::NoSymbolsAvailable));
    }

    #[test]
    fn letters_and_spaces_are_not_symbols() {
        for bad in ['a', ' ', 'é'] {
            let mut p = PasswordPolicy::default();
            p.allowed_symbols.insert(bad);
            assert_eq!(p.validate(), Err(GenerateError::InvalidSymbol(bad)));
        }
    }

    #[test]
    fn deserializes_from_the_ui_json_shape() {
        let json = r#"{
            "minLength": 12, "maxLength": 20, "targetLength": 14,
            "minUpper": 1, "minLower": 1, "exactDigits": 2, "minSymbols": 1,
            "allowedSymbols": ["!", "@"], "noSpaces": true, "startWithLetter": true,
            "excludeLookalikes": false, "maxRepeat": null, "noSequences": false, "asciiOnly": true
        }"#;
        let p: PasswordPolicy = serde_json::from_str(json).unwrap();
        assert_eq!(p.target_length, 14);
        assert_eq!(p.max_repeat, None);
        assert!(p.allowed_symbols.contains(&'@'));
    }
}
