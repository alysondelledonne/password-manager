//! Core logic for the password manager.
//!
//! v0.1 contains only the password generator. The vault, encryption, profiles
//! and sync described in `docs/architecture.md` arrive in later stages.
//!
//! This crate has no UI or Tauri code, so it can be tested on its own.

pub mod generator;

pub use generator::{
    GenerateError, GenerateRequest, GeneratedPassword, PasswordPolicy, PhraseError, PhraseMode,
    PhraseOptions, Strength, generate, generate_with_os_rng,
};
