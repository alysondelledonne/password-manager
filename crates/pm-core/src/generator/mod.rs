//! Password generator (architecture doc, section 4).
//!
//! - [`policy`]: the rule set the user picks.
//! - [`charsets`]: the character pools the rules draw from.
//! - [`phrase`]: checking and transforming the optional memorable phrase.
//! - [`rules`]: checks that run on every finished candidate.
//! - [`strength`]: entropy estimate and strength rating.
//! - [`engine`]: puts it together: build, check, retry.

pub mod charsets;
pub mod engine;
pub mod error;
pub mod phrase;
pub mod policy;
pub mod rules;
pub mod strength;

pub use engine::{GenerateRequest, GeneratedPassword, generate, generate_with_os_rng};
pub use error::{GenerateError, PhraseError};
pub use phrase::{PhraseMode, PhraseOptions};
pub use policy::PasswordPolicy;
pub use strength::Strength;
