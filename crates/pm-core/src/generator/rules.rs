//! Checks that run on every finished candidate (rules R9, R11–R14).
//!
//! Lengths and character-class counts are guaranteed by how the engine builds
//! a candidate, so they aren't re-checked here. The engine asserts them in debug builds.

use std::collections::HashSet;
use std::sync::OnceLock;

use crate::generator::policy::PasswordPolicy;

/// Why a candidate was thrown away. Internal: the engine just tries again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Violation {
    StartsWithNonLetter,
    TooManyRepeats,
    Sequence(String),
    ContainsUsername,
    CommonPassword,
}

/// Runs of 3 or more in any of these, forwards or backwards, count as a sequence.
const SEQUENCES: &[&str] = &[
    "abcdefghijklmnopqrstuvwxyz",
    "0123456789",
    "qwertyuiop",
    "asdfghjkl",
    "zxcvbnm",
];

const COMMON_PASSWORDS: &str = include_str!("../../data/common-passwords.txt");

pub fn check(
    password: &str,
    policy: &PasswordPolicy,
    username: Option<&str>,
) -> Result<(), Violation> {
    if policy.start_with_letter && !password.chars().next().is_some_and(char::is_alphabetic) {
        return Err(Violation::StartsWithNonLetter);
    }
    if let Some(limit) = policy.max_repeat
        && longest_run(password) > limit
    {
        return Err(Violation::TooManyRepeats);
    }
    if policy.no_sequences
        && let Some(seq) = find_sequence(password)
    {
        return Err(Violation::Sequence(seq));
    }
    if let Some(name) = username
        && contains_username(password, name)
    {
        return Err(Violation::ContainsUsername);
    }
    if is_common_password(password) {
        return Err(Violation::CommonPassword);
    }
    Ok(())
}

/// Length of the longest run of the exact same character.
pub fn longest_run(s: &str) -> usize {
    let mut longest = 0;
    let mut current = 0;
    let mut prev = None;
    for c in s.chars() {
        current = if Some(c) == prev { current + 1 } else { 1 };
        longest = longest.max(current);
        prev = Some(c);
    }
    longest
}

/// The first 3-character sequence found (case-insensitive), if any.
pub fn find_sequence(s: &str) -> Option<String> {
    let lower: Vec<char> = s.to_lowercase().chars().collect();
    lower.windows(3).find_map(|w| {
        let window: String = w.iter().collect();
        let reversed: String = w.iter().rev().collect();
        SEQUENCES
            .iter()
            .any(|seq| seq.contains(window.as_str()) || seq.contains(reversed.as_str()))
            .then_some(window)
    })
}

/// True if the password contains the username, or the part of an email before the `@`.
/// Names shorter than 3 characters are ignored, since they'd match by chance.
pub fn contains_username(password: &str, username: &str) -> bool {
    let password = password.to_lowercase();
    let username = username.trim().to_lowercase();
    let local_part = username.split('@').next().unwrap_or_default();
    [username.as_str(), local_part]
        .iter()
        .any(|name| name.chars().count() >= 3 && password.contains(name))
}

/// True if the password (case-insensitive) is on the bundled common-password list.
pub fn is_common_password(password: &str) -> bool {
    static LIST: OnceLock<HashSet<String>> = OnceLock::new();
    let list = LIST.get_or_init(|| {
        COMMON_PASSWORDS
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_lowercase)
            .collect()
    });
    list.contains(&password.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longest_run_counts_identical_characters() {
        assert_eq!(longest_run(""), 0);
        assert_eq!(longest_run("abc"), 1);
        assert_eq!(longest_run("aab"), 2);
        assert_eq!(longest_run("xaaay"), 3);
        assert_eq!(longest_run("aA"), 1, "case matters for repeats");
    }

    #[test]
    fn finds_alphabet_digit_and_keyboard_sequences_both_ways() {
        assert_eq!(find_sequence("xxABCxx").as_deref(), Some("abc"));
        assert_eq!(find_sequence("x321x").as_deref(), Some("321"));
        assert_eq!(find_sequence("Qwe!").as_deref(), Some("qwe"));
        assert_eq!(find_sequence("mnb").as_deref(), Some("mnb"));
        assert_eq!(find_sequence("Kp7!x"), None);
    }

    #[test]
    fn username_and_email_local_part_are_detected() {
        assert!(contains_username("xxALYSONxx", "alyson"));
        assert!(contains_username("Q!alyson9", "alyson@example.com"));
        assert!(
            !contains_username("Q!aly9", "al"),
            "short names are ignored"
        );
    }

    #[test]
    fn common_passwords_are_detected_case_insensitively() {
        assert!(is_common_password("Password1"));
        assert!(!is_common_password("Kp7!Blu#Hr4xQ"));
    }

    #[test]
    fn start_with_letter_rule() {
        let p = PasswordPolicy {
            start_with_letter: true,
            ..PasswordPolicy::default()
        };
        assert_eq!(check("7abc", &p, None), Err(Violation::StartsWithNonLetter));
    }
}
