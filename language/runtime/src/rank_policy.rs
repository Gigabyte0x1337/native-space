// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Exact rational policy shared by rank-selection adapters.
//!
//! The policy does not know what a channel or an output means. A domain
//! adapter ranks its own channels, retains the requested fraction, and reports
//! how many fixed comparisons still agree. This separation keeps rank control
//! reusable without pretending that sequence rows and context-table channels
//! have the same semantics.

use std::str::FromStr as _;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One as _, ToPrimitive as _, Zero as _};

use crate::core::{Diagnostic, LanguageError};

/// One exact rank target and its required finite agreement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RankPolicy {
    target_rank: BigRational,
    minimum_agreement: BigRational,
}

impl RankPolicy {
    /// Parse exact rational values such as `1`, `3/4`, or `99/100`.
    ///
    /// # Errors
    ///
    /// Returns `NSR003` when rank is outside `(0, 1]` or agreement is outside
    /// `[0, 1]`.
    pub fn parse(
        target_rank: &str,
        minimum_agreement: &str,
        source_name: &str,
    ) -> Result<Self, LanguageError> {
        let target_rank = parse_ratio(target_rank).ok_or_else(|| {
            policy_error(
                "target rank must be an exact number greater than zero through one",
                source_name,
            )
        })?;
        if target_rank <= BigRational::zero() || target_rank > BigRational::one() {
            return Err(policy_error(
                "target rank must be an exact number greater than zero through one",
                source_name,
            ));
        }
        let minimum_agreement = parse_ratio(minimum_agreement).ok_or_else(|| {
            policy_error(
                "minimum agreement must be an exact number from zero through one",
                source_name,
            )
        })?;
        if minimum_agreement < BigRational::zero() || minimum_agreement > BigRational::one() {
            return Err(policy_error(
                "minimum agreement must be an exact number from zero through one",
                source_name,
            ));
        }
        Ok(Self {
            target_rank,
            minimum_agreement,
        })
    }

    /// Return the canonical exact target rank.
    #[must_use]
    pub fn target_rank(&self) -> String {
        rational_source(&self.target_rank)
    }

    /// Return the canonical exact minimum agreement.
    #[must_use]
    pub fn minimum_agreement(&self) -> String {
        rational_source(&self.minimum_agreement)
    }

    /// Return whether this policy requires every comparison to match.
    #[must_use]
    pub fn is_exact(&self) -> bool {
        self.minimum_agreement == BigRational::one()
    }

    /// Return `exact` or `lossy` for user-facing reports.
    #[must_use]
    pub fn mode(&self) -> &'static str {
        if self.is_exact() { "exact" } else { "lossy" }
    }

    /// Return the ceiling of `total_channels * target_rank`.
    #[must_use]
    pub fn retained_count(&self, total_channels: usize) -> usize {
        if total_channels == 0 {
            return 0;
        }
        let scaled = BigInt::from(total_channels) * self.target_rank.numer();
        let denominator = self.target_rank.denom();
        let retained = (scaled + denominator - BigInt::one()) / denominator;
        retained.to_usize().unwrap_or(total_channels)
    }

    /// Decide a finite comparison without floating-point rounding.
    ///
    /// An empty comparison never passes: it contains no evidence that the
    /// candidate preserves the reference behavior.
    #[must_use]
    pub fn accepts(&self, matching: usize, compared: usize) -> bool {
        if compared == 0 || matching > compared {
            return false;
        }
        BigInt::from(matching) * self.minimum_agreement.denom()
            >= BigInt::from(compared) * self.minimum_agreement.numer()
    }
}

fn parse_ratio(source: &str) -> Option<BigRational> {
    BigRational::from_str(source).ok()
}

fn rational_source(value: &BigRational) -> String {
    if value.denom() == &BigInt::one() {
        value.numer().to_string()
    } else {
        format!("{}/{}", value.numer(), value.denom())
    }
}

fn policy_error(message: &str, source_name: &str) -> LanguageError {
    LanguageError(Diagnostic {
        code: "NSR003".into(),
        message: message.into(),
        source_name: source_name.into(),
        span: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_count_uses_an_exact_ceiling() {
        let policy = RankPolicy::parse("1/3", "1", "test").unwrap();

        assert_eq!(policy.retained_count(10), 4);
        assert_eq!(policy.retained_count(0), 0);
    }

    #[test]
    fn agreement_is_compared_without_float_rounding() {
        let policy = RankPolicy::parse("1/2", "2/3", "test").unwrap();

        assert!(policy.accepts(2, 3));
        assert!(!policy.accepts(1, 3));
        assert!(!policy.accepts(0, 0));
    }

    #[test]
    fn exact_policy_requires_every_comparison() {
        let policy = RankPolicy::parse("1", "1", "test").unwrap();

        assert!(policy.is_exact());
        assert!(policy.accepts(8, 8));
        assert!(!policy.accepts(7, 8));
    }

    #[test]
    fn invalid_bounds_are_rejected() {
        RankPolicy::parse("0", "1", "test").unwrap_err();
        RankPolicy::parse("2", "1", "test").unwrap_err();
        RankPolicy::parse("1", "-1/2", "test").unwrap_err();
        RankPolicy::parse("1", "3/2", "test").unwrap_err();
    }
}
