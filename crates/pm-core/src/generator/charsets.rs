//! Character pools.

use crate::generator::policy::PasswordPolicy;

pub const UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
pub const DIGITS: &str = "0123456789";

/// Default allowed special characters (rule R7).
pub const DEFAULT_SYMBOLS: &str = "!@#$%^&*-_=+?";

/// Characters that are easy to confuse with each other (rule R10).
pub const LOOKALIKES: &[char] = &['0', 'O', 'o', '1', 'l', 'I', '|', '5', 'S', '2', 'Z'];

pub fn is_lookalike(c: char) -> bool {
    LOOKALIKES.contains(&c)
}

/// The pools a given policy draws from, after exclusions are applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pools {
    pub upper: Vec<char>,
    pub lower: Vec<char>,
    pub digits: Vec<char>,
    pub symbols: Vec<char>,
    /// Pool for slots that aren't reserved for a required class:
    /// letters plus allowed symbols. Never digits, so the digit count stays exact.
    pub fill: Vec<char>,
}

impl Pools {
    pub fn for_policy(policy: &PasswordPolicy) -> Self {
        let keep = |c: &char| !(policy.exclude_lookalikes && is_lookalike(*c));
        let upper: Vec<char> = UPPER.chars().filter(keep).collect();
        let lower: Vec<char> = LOWER.chars().filter(keep).collect();
        let digits: Vec<char> = DIGITS.chars().filter(keep).collect();
        let symbols: Vec<char> = policy
            .allowed_symbols
            .iter()
            .copied()
            .filter(keep)
            .collect();
        let fill = upper
            .iter()
            .chain(&lower)
            .chain(&symbols)
            .copied()
            .collect();
        Self {
            upper,
            lower,
            digits,
            symbols,
            fill,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookalikes_are_removed_from_every_pool() {
        let mut policy = PasswordPolicy {
            exclude_lookalikes: true,
            ..PasswordPolicy::default()
        };
        policy.allowed_symbols.insert('|');
        let pools = Pools::for_policy(&policy);
        for pool in [
            &pools.upper,
            &pools.lower,
            &pools.digits,
            &pools.symbols,
            &pools.fill,
        ] {
            assert!(
                pool.iter().all(|c| !is_lookalike(*c)),
                "pool still has a look-alike: {pool:?}"
            );
        }
        assert_eq!(pools.digits, vec!['3', '4', '6', '7', '8', '9']);
    }

    #[test]
    fn fill_pool_never_contains_digits() {
        let pools = Pools::for_policy(&PasswordPolicy::default());
        assert!(pools.fill.iter().all(|c| !c.is_ascii_digit()));
    }
}
