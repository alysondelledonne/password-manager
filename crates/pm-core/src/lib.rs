//! Core logic for the password manager.
//!
//! v0.1 contains the password generators: random characters (`generator`)
//! and memorable words (`passphrase`). The vault, encryption, profiles
//! and sync described in `docs/architecture.md` arrive in later stages.
//!
//! This crate has no UI or Tauri code, so it can be tested on its own.

pub mod generator;
pub mod passphrase;

pub use generator::{
    GenerateError, GenerateRequest, GeneratedPassword, PasswordPolicy, PhraseError, PhraseMode,
    PhraseOptions, Strength, generate, generate_with_os_rng,
};
