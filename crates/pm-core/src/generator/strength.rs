//! Strength estimate (architecture doc, section 4.5).

use serde::Serialize;

use crate::generator::charsets::Pools;
use crate::generator::policy::PasswordPolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Strength {
    /// Under 60 bits.
    Weak,
    /// 60 to 80 bits.
    Fair,
    /// 80 to 100 bits. 80 is the proposed minimum.
    Strong,
    /// 100 bits or more.
    VeryStrong,
}

impl Strength {
    pub fn from_bits(bits: f64) -> Self {
        match bits {
            b if b < 60.0 => Self::Weak,
            b if b < 80.0 => Self::Fair,
            b if b < 100.0 => Self::Strong,
            _ => Self::VeryStrong,
        }
    }
}

/// Estimated entropy of the random part, in bits.
///
/// Each random character adds log₂(size of the pool it was drawn from). If
/// there's a phrase, its position adds log₂(number of places it could go).
/// The phrase itself adds nothing.
///
/// This is an estimate: shuffling adds a little and throwing away candidates
/// that break a rule removes a little. Both effects are small next to the total.
pub fn entropy_bits(
    policy: &PasswordPolicy,
    pools: &Pools,
    random_len: usize,
    has_phrase: bool,
) -> f64 {
    let fill_slots = random_len.saturating_sub(policy.required_chars());
    let mut bits = class_bits(policy.min_upper, pools.upper.len())
        + class_bits(policy.min_lower, pools.lower.len())
        + class_bits(policy.exact_digits, pools.digits.len())
        + class_bits(policy.min_symbols, pools.symbols.len())
        + class_bits(fill_slots, pools.fill.len());
    if has_phrase {
        bits += (random_len as f64 + 1.0).log2();
    }
    bits
}

fn class_bits(count: usize, pool_len: usize) -> f64 {
    if count == 0 || pool_len == 0 {
        0.0
    } else {
        count as f64 * (pool_len as f64).log2()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rating_thresholds() {
        assert_eq!(Strength::from_bits(59.9), Strength::Weak);
        assert_eq!(Strength::from_bits(60.0), Strength::Fair);
        assert_eq!(Strength::from_bits(80.0), Strength::Strong);
        assert_eq!(Strength::from_bits(100.0), Strength::VeryStrong);
    }

    #[test]
    fn default_policy_entropy_matches_hand_calculation() {
        let p = PasswordPolicy::default();
        let pools = Pools::for_policy(&p);
        // 1 upper (26), 1 lower (26), 3 digits (10), 1 symbol (13), 10 fill (26+26+13 = 65)
        let expected = 2.0 * 26f64.log2() + 3.0 * 10f64.log2() + 13f64.log2() + 10.0 * 65f64.log2();
        let got = entropy_bits(&p, &pools, 16, false);
        assert!((got - expected).abs() < 1e-9, "{got} vs {expected}");
        assert_eq!(Strength::from_bits(got), Strength::Strong);
    }

    #[test]
    fn empty_pools_never_produce_nan() {
        let p = PasswordPolicy {
            min_symbols: 0,
            allowed_symbols: Default::default(),
            ..PasswordPolicy::default()
        };
        let pools = Pools::for_policy(&p);
        assert!(entropy_bits(&p, &pools, 16, true).is_finite());
    }
}
