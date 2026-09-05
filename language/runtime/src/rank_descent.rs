// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Finds finite replays while relationship rank decreases.
//!
//! Rank one first generates one fixed finite reference. Every candidate is
//! rediscovered from that same reference and attempts to regenerate all of it.
//! Adaptive and linear candidates succeed only when every row matches exactly.
//! A static candidate uses one exact rational agreement policy; thresholds
//! below one are explicitly lossy. Every candidate reports both its longest
//! exact prefix and its total matching rows.
//!
//! Adaptive descent starts at one half. Success moves toward lower rank;
//! failure moves toward higher rank. It stops when the tested relationship
//! channel counts become adjacent. Linear descent tests every positive exact
//! step below one. The adaptive search relies only on tested outcomes and does
//! not claim that success is monotone between untested ranks.

use std::str::FromStr as _;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One as _, Zero as _};
use serde_json::{Value, json};

use crate::{
    core::{Diagnostic, LanguageError, NativeState},
    discovery::{DiscoveredPattern, discover_states},
    rank_policy::RankPolicy,
};

/// One finite rank-search strategy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RankStrategy {
    /// Branch lower after exact success and higher after a mismatch.
    Adaptive,
    /// Subtract one exact positive step and test every resulting rank.
    Linear { step: String },
    /// Test one exact target rank against one finite agreement threshold.
    Static {
        target_rank: String,
        minimum_agreement: String,
    },
}

impl RankStrategy {
    /// Create the default adaptive half-interval search.
    #[must_use]
    pub const fn adaptive() -> Self {
        Self::Adaptive
    }

    /// Create a linear schedule from one exact rank step.
    #[must_use]
    pub fn linear(step: impl Into<String>) -> Self {
        Self::Linear { step: step.into() }
    }

    /// Create one static rank target. Agreement `1` is exact; lower values are
    /// explicitly lossy.
    #[must_use]
    pub fn static_target(
        target_rank: impl Into<String>,
        minimum_agreement: impl Into<String>,
    ) -> Self {
        Self::Static {
            target_rank: target_rank.into(),
            minimum_agreement: minimum_agreement.into(),
        }
    }

    const fn name(&self) -> &'static str {
        match self {
            Self::Adaptive => "adaptive",
            Self::Linear { .. } => "linear",
            Self::Static { .. } => "static",
        }
    }
}

/// One measured candidate in a finite rank search.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RankLevel {
    rank: String,
    mode: &'static str,
    attempted_rows: usize,
    verified_rows: usize,
    matching_rows: usize,
    first_mismatch: Option<usize>,
    total_channels: Option<usize>,
    retained_channels: Option<usize>,
    selected: bool,
}

impl RankLevel {
    /// Return the exact canonical rank fraction.
    #[must_use]
    pub fn rank(&self) -> &str {
        &self.rank
    }

    /// Return `deterministic` or `relationships`.
    #[must_use]
    pub const fn mode(&self) -> &'static str {
        self.mode
    }

    /// Return how many fixed reference rows this candidate attempted.
    #[must_use]
    pub const fn attempted_rows(&self) -> usize {
        self.attempted_rows
    }

    /// Return the exact matching prefix length.
    #[must_use]
    pub const fn verified_rows(&self) -> usize {
        self.verified_rows
    }

    /// Return how many rows match, including matches after an earlier mismatch.
    #[must_use]
    pub const fn matching_rows(&self) -> usize {
        self.matching_rows
    }

    /// Return the first one-based mismatching row, when present.
    #[must_use]
    pub const fn first_mismatch(&self) -> Option<usize> {
        self.first_mismatch
    }

    /// Return distinct relationship channels before rank selection.
    #[must_use]
    pub const fn total_channels(&self) -> Option<usize> {
        self.total_channels
    }

    /// Return retained relationship channels at this rank.
    #[must_use]
    pub const fn retained_channels(&self) -> Option<usize> {
        self.retained_channels
    }

    /// Return whether this candidate met the declared selection policy.
    #[must_use]
    pub const fn selected(&self) -> bool {
        self.selected
    }

    fn to_data(&self) -> Value {
        json!({
            "rank": self.rank,
            "mode": self.mode,
            "attempted_rows": self.attempted_rows,
            "verified_rows": self.verified_rows,
            "matching_rows": self.matching_rows,
            "first_mismatch": self.first_mismatch,
            "total_channels": self.total_channels,
            "retained_channels": self.retained_channels,
            "selected": self.selected,
        })
    }
}

/// Complete finite rank-search result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RankDescent {
    strategy: &'static str,
    requested_rows: usize,
    levels: Vec<RankLevel>,
    selected_rank: String,
    minimum_agreement: String,
    stop_reason: &'static str,
    final_pattern: DiscoveredPattern,
}

impl RankDescent {
    /// Return the stable strategy name.
    #[must_use]
    pub const fn strategy(&self) -> &'static str {
        self.strategy
    }

    /// Return the requested rank-one reference length.
    #[must_use]
    pub const fn requested_rows(&self) -> usize {
        self.requested_rows
    }

    /// Return candidates in actual test order.
    #[must_use]
    pub fn levels(&self) -> &[RankLevel] {
        &self.levels
    }

    /// Return the final rank selected by the declared strategy and policy.
    #[must_use]
    pub fn selected_rank(&self) -> &str {
        &self.selected_rank
    }

    /// Return the exact agreement threshold used by this search.
    #[must_use]
    pub fn minimum_agreement(&self) -> &str {
        &self.minimum_agreement
    }

    /// Return why finite search stopped.
    #[must_use]
    pub const fn stop_reason(&self) -> &'static str {
        self.stop_reason
    }

    /// Return the selected native pattern.
    #[must_use]
    pub const fn final_pattern(&self) -> &DiscoveredPattern {
        &self.final_pattern
    }

    /// Return a stable JSON report.
    #[must_use]
    pub fn to_data(&self) -> Value {
        let selected = self.levels.iter().rev().find(|level| level.selected);
        json!({
            "schema": "native-space-rank-descent",
            "version": 1,
            "strategy": self.strategy,
            "requested_rows": self.requested_rows,
            "levels": self.levels.iter().map(RankLevel::to_data).collect::<Vec<_>>(),
            "selected_rank": self.selected_rank,
            "minimum_agreement": self.minimum_agreement,
            "agreement_mode": if self.minimum_agreement == "1" { "exact" } else { "lossy" },
            "verified_rows": selected.map_or(0, |level| level.verified_rows),
            "matching_rows": selected.map_or(0, |level| level.matching_rows),
            "stop_reason": self.stop_reason,
        })
    }
}

/// Select a pattern under an adaptive, linear, or static rank search.
///
/// Rank one discovered from `observations` generates the fixed
/// `requested_rows` reference. Every lower candidate is discovered from and
/// compared with that complete reference. Linear schedules require an exact
/// positive step smaller than one. Static search tests one target and uses its
/// declared exact rational agreement threshold.
///
/// # Errors
///
/// Returns `NSR001` for no requested rows or an invalid linear step and
/// `NSR003` for an invalid static policy. Discovery and generation errors
/// retain their original `NSU` diagnostics.
pub fn descend_states(
    observations: &[NativeState],
    requested_rows: usize,
    strategy: &RankStrategy,
    source_name: &str,
) -> Result<RankDescent, LanguageError> {
    if requested_rows == 0 {
        return Err(rank_error(
            "NSR001",
            "rank search requires at least one generated row",
            source_name,
        ));
    }
    let linear_step = parse_linear_step(strategy, source_name)?;
    let static_policy = parse_static_policy(strategy, source_name)?;
    let baseline_pattern = discover_states(observations, "1", source_name)?;
    let reference = baseline_pattern.generate(requested_rows, source_name)?;
    let reference_pattern = discover_states(&reference, "1", source_name)?;
    let reference_replay = reference_pattern.generate(requested_rows, source_name)?;
    let reference_verified = matching_prefix(&reference, &reference_replay);
    let search_pattern = reference_pattern.clone();

    let (mut final_pattern, mut selected_rank, rank_one_pattern) =
        if reference_verified == requested_rows {
            (reference_pattern.clone(), "1".to_owned(), reference_pattern)
        } else {
            (baseline_pattern.clone(), "1".to_owned(), baseline_pattern)
        };
    let mut levels = vec![rank_level(
        &BigRational::one(),
        &rank_one_pattern,
        requested_rows,
        requested_rows,
        requested_rows,
        true,
    )];

    let stop_reason = match strategy {
        RankStrategy::Adaptive => adaptive_search(
            &reference,
            &search_pattern,
            source_name,
            &mut levels,
            &mut final_pattern,
            &mut selected_rank,
        )?,
        RankStrategy::Linear { .. } => {
            let Some(linear_step) = linear_step.as_ref() else {
                return Err(rank_error(
                    "NSR001",
                    "linear rank search has no validated step",
                    source_name,
                ));
            };
            linear_search(
                &reference,
                &search_pattern,
                linear_step,
                source_name,
                &mut levels,
                &mut final_pattern,
                &mut selected_rank,
            )?
        }
        RankStrategy::Static { .. } => {
            let Some(policy) = static_policy.as_ref() else {
                return Err(rank_error(
                    "NSR003",
                    "static rank search has no validated policy",
                    source_name,
                ));
            };
            static_search(
                &reference,
                &search_pattern,
                policy,
                source_name,
                &mut levels,
                &mut final_pattern,
                &mut selected_rank,
            )?
        }
    };

    Ok(RankDescent {
        strategy: strategy.name(),
        requested_rows,
        levels,
        selected_rank,
        minimum_agreement: static_policy
            .as_ref()
            .map_or_else(|| "1".to_owned(), RankPolicy::minimum_agreement),
        stop_reason,
        final_pattern,
    })
}

/// Replay one selected pattern at a positive one-based position.
///
/// Position one is the pattern's first seed or first observed symbol. Positions
/// beyond the rank-search reference are generated deterministically but were
/// not validated by that finite search.
///
/// # Errors
///
/// Returns `NSR002` for position zero or a position exceeding the platform
/// size. Relationship replay errors retain their original `NSU` diagnostics.
pub fn replay_at(
    pattern: &DiscoveredPattern,
    position: u64,
    source_name: &str,
) -> Result<NativeState, LanguageError> {
    if position == 0 {
        return Err(rank_error(
            "NSR002",
            "pattern position must be positive",
            source_name,
        ));
    }
    let length = usize::try_from(position).map_err(|_capacity_error| {
        rank_error(
            "NSR002",
            "pattern position exceeds the platform size",
            source_name,
        )
    })?;
    pattern
        .generate(length, source_name)?
        .pop()
        .ok_or_else(|| rank_error("NSR002", "pattern replay produced no row", source_name))
}

fn adaptive_search(
    reference: &[NativeState],
    search_pattern: &DiscoveredPattern,
    source_name: &str,
    levels: &mut Vec<RankLevel>,
    final_pattern: &mut DiscoveredPattern,
    selected_rank: &mut String,
) -> Result<&'static str, LanguageError> {
    let DiscoveredPattern::Relationships(full_relationships) = search_pattern else {
        let half = BigRational::new(BigInt::one(), BigInt::from(2));
        let candidate = search_pattern.clone();
        let generated = candidate.generate(reference.len(), source_name)?;
        let verified_rows = matching_prefix(reference, &generated);
        let matching_rows = matching_count(reference, &generated);
        let success = verified_rows == reference.len();
        levels.push(rank_level(
            &half,
            &candidate,
            reference.len(),
            verified_rows,
            matching_rows,
            success,
        ));
        if success {
            *final_pattern = candidate;
            "1/2".clone_into(selected_rank);
        }
        return Ok("an exact deterministic continuation superseded rank selection");
    };
    let channel_count = full_relationships.total_channel_count();
    if channel_count <= 1 {
        return Ok("at most one relationship channel exists");
    }

    let mut lower_rank = BigRational::zero();
    let mut lower_channels = 0_usize;
    let mut upper_rank = BigRational::one();
    let mut upper_channels = channel_count;
    while upper_channels.saturating_sub(lower_channels) > 1 {
        let candidate_rank = (&lower_rank + &upper_rank) / BigInt::from(2);
        let candidate_source = rational_source(&candidate_rank);
        let candidate = DiscoveredPattern::Relationships(
            full_relationships.rerank(&candidate_source, source_name)?,
        );
        let generated = candidate.generate(reference.len(), source_name)?;
        let verified_rows = matching_prefix(reference, &generated);
        let matching_rows = matching_count(reference, &generated);
        let success = verified_rows == reference.len();
        let retained_channels = relationship_channel_count(&candidate).unwrap_or(channel_count);
        levels.push(rank_level(
            &candidate_rank,
            &candidate,
            reference.len(),
            verified_rows,
            matching_rows,
            success,
        ));

        if candidate.mode() == "deterministic" {
            if success {
                *final_pattern = candidate;
                *selected_rank = candidate_source;
            }
            return Ok("an exact deterministic continuation superseded rank selection");
        }
        if success {
            upper_rank = candidate_rank;
            upper_channels = retained_channels;
            *final_pattern = candidate;
            *selected_rank = candidate_source;
        } else {
            lower_rank = candidate_rank;
            lower_channels = retained_channels;
        }
    }
    Ok("adjacent retained-channel counts bracket the tested exact boundary")
}

fn linear_search(
    reference: &[NativeState],
    search_pattern: &DiscoveredPattern,
    step: &BigRational,
    source_name: &str,
    levels: &mut Vec<RankLevel>,
    final_pattern: &mut DiscoveredPattern,
    selected_rank: &mut String,
) -> Result<&'static str, LanguageError> {
    let mut rank = BigRational::one() - step;
    while rank > BigRational::zero() {
        let rank_source = rational_source(&rank);
        let candidate = match search_pattern {
            DiscoveredPattern::Deterministic(pattern) => {
                DiscoveredPattern::Deterministic(pattern.clone())
            }
            DiscoveredPattern::Relationships(pattern) => {
                DiscoveredPattern::Relationships(pattern.rerank(&rank_source, source_name)?)
            }
        };
        let generated = candidate.generate(reference.len(), source_name)?;
        let verified_rows = matching_prefix(reference, &generated);
        let matching_rows = matching_count(reference, &generated);
        let success = verified_rows == reference.len();
        levels.push(rank_level(
            &rank,
            &candidate,
            reference.len(),
            verified_rows,
            matching_rows,
            success,
        ));
        if success {
            *final_pattern = candidate.clone();
            *selected_rank = rank_source;
        }
        if candidate.mode() == "deterministic" {
            return Ok("an exact deterministic continuation superseded rank selection");
        }
        rank -= step;
    }
    Ok("every positive linear rank was tested")
}

fn static_search(
    reference: &[NativeState],
    search_pattern: &DiscoveredPattern,
    policy: &RankPolicy,
    source_name: &str,
    levels: &mut Vec<RankLevel>,
    final_pattern: &mut DiscoveredPattern,
    selected_rank: &mut String,
) -> Result<&'static str, LanguageError> {
    let rank_source = policy.target_rank();
    let rank = BigRational::from_str(&rank_source).expect("validated rank remains parseable");
    let candidate = match search_pattern {
        DiscoveredPattern::Deterministic(pattern) => {
            DiscoveredPattern::Deterministic(pattern.clone())
        }
        DiscoveredPattern::Relationships(pattern) => {
            DiscoveredPattern::Relationships(pattern.rerank(&rank_source, source_name)?)
        }
    };
    let generated = candidate.generate(reference.len(), source_name)?;
    let verified_rows = matching_prefix(reference, &generated);
    let matching_rows = matching_count(reference, &generated);
    let accepted = policy.accepts(matching_rows, reference.len());
    levels.push(rank_level(
        &rank,
        &candidate,
        reference.len(),
        verified_rows,
        matching_rows,
        accepted,
    ));
    if accepted {
        *final_pattern = candidate;
        *selected_rank = rank_source;
        Ok(if policy.is_exact() {
            "the static candidate matched every fixed reference row"
        } else {
            "the static candidate met the declared lossy agreement threshold"
        })
    } else {
        Ok(
            "the static candidate missed the declared agreement threshold; rank one remains selected",
        )
    }
}

fn parse_linear_step(
    strategy: &RankStrategy,
    source_name: &str,
) -> Result<Option<BigRational>, LanguageError> {
    let RankStrategy::Linear { step } = strategy else {
        return Ok(None);
    };
    let value = BigRational::from_str(step).map_err(|_parse_error| {
        rank_error(
            "NSR001",
            "linear rank step must be exact, greater than zero, and less than one",
            source_name,
        )
    })?;
    if value <= BigRational::zero() || value >= BigRational::one() {
        return Err(rank_error(
            "NSR001",
            "linear rank step must be exact, greater than zero, and less than one",
            source_name,
        ));
    }
    Ok(Some(value))
}

fn parse_static_policy(
    strategy: &RankStrategy,
    source_name: &str,
) -> Result<Option<RankPolicy>, LanguageError> {
    let RankStrategy::Static {
        target_rank,
        minimum_agreement,
    } = strategy
    else {
        return Ok(None);
    };
    RankPolicy::parse(target_rank, minimum_agreement, source_name).map(Some)
}

fn matching_prefix(reference: &[NativeState], generated: &[NativeState]) -> usize {
    reference
        .iter()
        .zip(generated)
        .take_while(|(expected, actual)| expected == actual)
        .count()
}

fn matching_count(reference: &[NativeState], generated: &[NativeState]) -> usize {
    reference
        .iter()
        .zip(generated)
        .filter(|(expected, actual)| expected == actual)
        .count()
}

fn relationship_channel_count(pattern: &DiscoveredPattern) -> Option<usize> {
    match pattern {
        DiscoveredPattern::Deterministic(_) => None,
        DiscoveredPattern::Relationships(relationships) => Some(relationships.channels().len()),
    }
}

fn rank_level(
    rank: &BigRational,
    pattern: &DiscoveredPattern,
    attempted_rows: usize,
    verified_rows: usize,
    matching_rows: usize,
    selected: bool,
) -> RankLevel {
    let (total_channels, retained_channels) = match pattern {
        DiscoveredPattern::Deterministic(_) => (None, None),
        DiscoveredPattern::Relationships(relationships) => (
            Some(relationships.total_channel_count()),
            Some(relationships.channels().len()),
        ),
    };
    RankLevel {
        rank: rational_source(rank),
        mode: pattern.mode(),
        attempted_rows,
        verified_rows,
        matching_rows,
        first_mismatch: (verified_rows < attempted_rows).then_some(verified_rows + 1),
        total_channels,
        retained_channels,
        selected,
    }
}

fn rational_source(value: &BigRational) -> String {
    if value.denom() == &BigInt::one() {
        value.numer().to_string()
    } else {
        format!("{}/{}", value.numer(), value.denom())
    }
}

fn rank_error(code: &str, message: impl Into<String>, source_name: &str) -> LanguageError {
    LanguageError(Diagnostic {
        code: code.into(),
        message: message.into(),
        source_name: source_name.into(),
        span: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::NativeScalar;

    fn scalar(value: i64) -> NativeState {
        NativeState::scalar(NativeScalar::from_text(&value.to_string(), "0").unwrap())
    }

    #[test]
    fn adaptive_direction_moves_lower_after_success() {
        let lower = BigRational::zero();
        let mut upper = BigRational::one();
        let half = (&lower + &upper) / BigInt::from(2);
        upper = half;
        let next = (&lower + &upper) / BigInt::from(2);

        assert_eq!(rational_source(&next), "1/4");
    }

    #[test]
    fn adaptive_direction_moves_higher_after_failure() {
        let mut lower = BigRational::zero();
        let upper = BigRational::one();
        let half = (&lower + &upper) / BigInt::from(2);
        lower = half;
        let next = (&lower + &upper) / BigInt::from(2);

        assert_eq!(rational_source(&next), "3/4");
    }

    #[test]
    fn linear_quarters_test_three_quarters_first() {
        let observations = [2, 3, 5, 7, 11, 13, 17, 19, 23].map(scalar);
        let result = descend_states(
            &observations,
            64,
            &RankStrategy::linear("1/4"),
            "primes.json",
        )
        .unwrap();
        let ranks = result
            .levels()
            .iter()
            .map(RankLevel::rank)
            .collect::<Vec<_>>();

        assert_eq!(ranks.first(), Some(&"1"));
        if ranks.len() > 1 {
            assert_eq!(ranks.get(1), Some(&"3/4"));
        }
    }

    #[test]
    fn static_exact_rank_keeps_rank_one_after_any_mismatch() {
        let observations = [
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97, 101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173,
        ]
        .map(scalar);
        let result = descend_states(
            &observations,
            observations.len(),
            &RankStrategy::static_target("1/32", "1"),
            "primes.json",
        )
        .unwrap();

        assert_eq!(result.selected_rank(), "1");
        assert_eq!(result.minimum_agreement(), "1");
        assert!(!result.levels().last().unwrap().selected());
        assert!(result.levels().last().unwrap().matching_rows() < observations.len());
    }

    #[test]
    fn static_lossy_rank_can_select_the_declared_candidate() {
        let observations = [
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97, 101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173,
        ]
        .map(scalar);
        let result = descend_states(
            &observations,
            observations.len(),
            &RankStrategy::static_target("1/32", "0"),
            "primes.json",
        )
        .unwrap();

        assert_eq!(result.selected_rank(), "1/32");
        assert_eq!(result.minimum_agreement(), "0");
        assert!(result.levels().last().unwrap().selected());
    }

    #[test]
    fn selected_pattern_always_matches_every_requested_row() {
        let observations = [1, 2, 1, 3, 2, 5, 3, 8, 5].map(scalar);
        let result =
            descend_states(&observations, 64, &RankStrategy::adaptive(), "mixed.json").unwrap();
        let baseline = discover_states(&observations, "1", "mixed.json")
            .unwrap()
            .generate(64, "mixed.json")
            .unwrap();
        let selected = result.final_pattern().generate(64, "mixed.json").unwrap();

        assert_eq!(selected, baseline);
        assert!(result.levels().iter().any(RankLevel::selected));
    }

    #[test]
    fn replay_at_uses_one_based_positions() {
        let observations = [1, 1, 2, 3, 5, 8, 13].map(scalar);
        let result = descend_states(
            &observations,
            observations.len(),
            &RankStrategy::adaptive(),
            "fibonacci.json",
        )
        .unwrap();

        assert_eq!(
            replay_at(result.final_pattern(), 8, "fibonacci.json").unwrap(),
            scalar(21)
        );
        assert_eq!(
            replay_at(result.final_pattern(), 0, "fibonacci.json")
                .unwrap_err()
                .0
                .code,
            "NSR002"
        );
    }

    #[test]
    fn invalid_linear_step_is_located() {
        let error = descend_states(&[scalar(1)], 10, &RankStrategy::linear("0"), "invalid.json")
            .unwrap_err();

        assert_eq!(error.0.code, "NSR001");
        assert_eq!(error.0.source_name, "invalid.json");
    }
}
