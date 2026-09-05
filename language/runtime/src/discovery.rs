// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Discovers deterministic continuations or exact relationship frequencies.
//!
//! An exact recurrence is returned only when it regenerates every held-out
//! observation. Otherwise every earlier-to-later observation pair becomes a
//! relationship channel identified by its two exact symbols and positive
//! relative distance. Rank one retains all channels; a lower exact rank keeps
//! the strongest ceiling of that fraction. Relationship generation is also
//! deterministic: matching retained channels vote for the next symbol, exact
//! counts are vote weights, and first-occurrence symbol order breaks ties.

use std::{collections::BTreeMap, fmt, str::FromStr as _, sync::Arc};

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One as _, ToPrimitive as _, Zero as _};

use crate::{
    continuation::Continuation,
    core::{
        Diagnostic, Expr, LanguageError, MultiIndex, NativeScalar, NativeState, Span,
        expression_source, state_expression,
    },
};

/// Maximum pair visits in one exact relationship discovery.
///
/// Distance-preserving discovery is quadratic. This bound allows roughly two
/// thousand observations while preventing accidental unbounded source output.
/// Raising it requires a benchmark and a reviewed memory budget.
const MAX_RELATIONSHIP_PAIRS: usize = 2_000_000;

/// Maximum rows generated in one finite relationship replay.
///
/// Generation retains the complete result and scans a bounded observed
/// distance window for every new row. Raising this limit requires benchmarks
/// for both peak memory and the largest retained channel set.
const MAX_GENERATED_ROWS: usize = 1_000_000;

/// One retained ordered relationship-frequency channel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipChannel {
    left_symbol: u64,
    right_symbol: u64,
    distance: u64,
    count: u64,
}

impl RelationshipChannel {
    /// Return the earlier symbol identifier.
    #[must_use]
    pub const fn left_symbol(&self) -> u64 {
        self.left_symbol
    }

    /// Return the later symbol identifier.
    #[must_use]
    pub const fn right_symbol(&self) -> u64 {
        self.right_symbol
    }

    /// Return the positive observation distance.
    #[must_use]
    pub const fn distance(&self) -> u64 {
        self.distance
    }

    /// Return the exact occurrence count.
    #[must_use]
    pub const fn count(&self) -> u64 {
        self.count
    }
}

/// One exact relationship-frequency pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipPattern {
    observation_count: usize,
    symbols: Vec<NativeState>,
    symbol_counts: Vec<u64>,
    total_channel_count: usize,
    rank: String,
    channels: Arc<Vec<RelationshipChannel>>,
    retained_channel_count: usize,
    directions: RelationshipDirections,
    state: NativeState,
    source: String,
}

impl RelationshipPattern {
    /// Return the number of complete supplied observations.
    #[must_use]
    pub const fn observation_count(&self) -> usize {
        self.observation_count
    }

    /// Return the exact symbol dictionary in identifier order.
    #[must_use]
    pub fn symbols(&self) -> &[NativeState] {
        &self.symbols
    }

    /// Return exact observation counts in symbol identifier order.
    #[must_use]
    pub fn symbol_counts(&self) -> &[u64] {
        &self.symbol_counts
    }

    /// Return the number of distinct channels before ranking.
    #[must_use]
    pub const fn total_channel_count(&self) -> usize {
        self.total_channel_count
    }

    /// Return the canonical requested rank fraction.
    #[must_use]
    pub fn rank(&self) -> &str {
        &self.rank
    }

    /// Return retained channels in deterministic rank order.
    #[must_use]
    pub fn channels(&self) -> &[RelationshipChannel] {
        &self.channels[..self.retained_channel_count]
    }

    /// Return the complete exact relationship state.
    #[must_use]
    pub fn state(&self) -> &NativeState {
        &self.state
    }

    /// Return the complete runnable source document.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Retain a different exact fraction of the already counted channels.
    ///
    /// This reuses the complete rank-one channel order and exact symbol
    /// dictionary. It performs no recurrence search and no pair recount.
    ///
    /// # Errors
    ///
    /// Returns `NSU012` for a rank outside zero through one, or `NSU013` if
    /// the reranked native state cannot be represented.
    pub fn rerank(&self, rank: &str, source_name: &str) -> Result<Self, LanguageError> {
        let rank = parse_rank(rank, source_name, None)?;
        let retained_count =
            retained_channel_count(self.total_channel_count, &rank, source_name, None)?;
        let channels = &self.channels[..retained_count];
        let state = relationship_state(
            &self.symbols,
            &self.symbol_counts,
            channels,
            self.directions,
            source_name,
            None,
        )?;
        let rank = rational_source(&rank);
        let source = relationship_source(
            self.observation_count,
            self.symbols.len(),
            self.total_channel_count,
            &rank,
            channels,
            self.directions,
            &state,
        );
        Ok(Self {
            observation_count: self.observation_count,
            symbols: self.symbols.clone(),
            symbol_counts: self.symbol_counts.clone(),
            total_channel_count: self.total_channel_count,
            rank,
            channels: Arc::clone(&self.channels),
            retained_channel_count: retained_count,
            directions: self.directions,
            state,
            source,
        })
    }

    /// Generate a finite canonical replay from retained relationships.
    ///
    /// The first observed symbol is the initial row. At every later position,
    /// each retained channel whose left symbol occurs at its recorded distance
    /// votes for its right symbol using the channel's exact count. The greatest
    /// vote wins; symbol identifier order breaks ties. If no channel applies,
    /// the most frequent observed symbol wins with the same tie rule.
    ///
    /// # Errors
    ///
    /// Returns `NSU014` if `row_count` exceeds the finite replay budget or an
    /// exact vote total exceeds `u64`.
    pub fn generate(
        &self,
        row_count: usize,
        source_name: &str,
    ) -> Result<Vec<NativeState>, LanguageError> {
        if row_count > MAX_GENERATED_ROWS {
            return Err(discovery_error(
                "NSU014",
                format!("relationship replay supports at most {MAX_GENERATED_ROWS} rows"),
                source_name,
                None,
            ));
        }
        if row_count == 0 {
            return Ok(Vec::new());
        }

        let mut channels = BTreeMap::<(usize, u64), Vec<(u64, u64)>>::new();
        let mut maximum_distance = 0_usize;
        for channel in self.channels() {
            let distance = usize::try_from(channel.distance).map_err(|_capacity_error| {
                discovery_error(
                    "NSU014",
                    "relationship distance exceeds the platform size",
                    source_name,
                    None,
                )
            })?;
            maximum_distance = maximum_distance.max(distance);
            channels
                .entry((distance, channel.left_symbol))
                .or_default()
                .push((channel.right_symbol, channel.count));
        }

        let fallback = most_frequent_symbol(&self.symbol_counts);
        let mut generated_ids = Vec::with_capacity(row_count);
        generated_ids.push(1_u64);
        while generated_ids.len() < row_count {
            let history = generated_ids.len().min(maximum_distance);
            let mut votes = BTreeMap::<u64, u64>::new();
            for distance in 1..=history {
                let left_symbol = generated_ids[generated_ids.len() - distance];
                if let Some(matches) = channels.get(&(distance, left_symbol)) {
                    for &(right_symbol, count) in matches {
                        let vote = votes.entry(right_symbol).or_default();
                        *vote = vote.checked_add(count).ok_or_else(|| {
                            discovery_error(
                                "NSU014",
                                "relationship vote total exceeds u64",
                                source_name,
                                None,
                            )
                        })?;
                    }
                }
            }
            generated_ids.push(winning_symbol(&votes).unwrap_or(fallback));
        }

        generated_ids
            .into_iter()
            .map(|symbol| {
                let position = usize::try_from(symbol - 1).map_err(|_capacity_error| {
                    discovery_error(
                        "NSU014",
                        "symbol identifier exceeds the platform size",
                        source_name,
                        None,
                    )
                })?;
                self.symbols.get(position).cloned().ok_or_else(|| {
                    discovery_error(
                        "NSU014",
                        "relationship channel refers to an absent symbol",
                        source_name,
                        None,
                    )
                })
            })
            .collect()
    }
}

/// A discovered deterministic or relationship pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiscoveredPattern {
    /// Every held-out observation follows one exact recurrence.
    Deterministic(Continuation),
    /// No supported exact recurrence exists; relationships are counted.
    Relationships(RelationshipPattern),
}

impl DiscoveredPattern {
    /// Return a stable human-readable mode name.
    #[must_use]
    pub const fn mode(&self) -> &'static str {
        match self {
            Self::Deterministic(_) => "deterministic",
            Self::Relationships(_) => "relationships",
        }
    }

    /// Return the generated runnable source document.
    #[must_use]
    pub fn source(&self) -> &str {
        match self {
            Self::Deterministic(pattern) => pattern.source(),
            Self::Relationships(pattern) => pattern.source(),
        }
    }

    /// Return the generated native pattern state.
    ///
    /// # Errors
    ///
    /// Returns a language diagnostic if a deterministic operation strand
    /// cannot be interpreted as an exact state.
    pub fn pattern_state(&self) -> Result<NativeState, LanguageError> {
        match self {
            Self::Deterministic(pattern) => pattern.pattern_state(),
            Self::Relationships(pattern) => Ok(pattern.state().clone()),
        }
    }

    /// Return a compact CSV representation.
    ///
    /// # Errors
    ///
    /// Returns `NSU009` if exact pattern serialization fails.
    pub fn pattern_csv(&self) -> Result<String, LanguageError> {
        match self {
            Self::Deterministic(pattern) => pattern.pattern_csv(),
            Self::Relationships(pattern) => relationship_csv(pattern),
        }
    }

    /// Generate a finite canonical replay from this pattern.
    ///
    /// Deterministic mode executes its exact recurrence. Relationship mode uses
    /// the channel-voting rule documented by [`RelationshipPattern::generate`].
    ///
    /// # Errors
    ///
    /// Returns `NSU014` when relationship replay exceeds its finite budget or
    /// encounters an unrepresentable exact vote total.
    pub fn generate(
        &self,
        row_count: usize,
        source_name: &str,
    ) -> Result<Vec<NativeState>, LanguageError> {
        match self {
            Self::Deterministic(pattern) => Ok(pattern.sequence().take(row_count).collect()),
            Self::Relationships(pattern) => pattern.generate(row_count, source_name),
        }
    }

    pub(crate) fn expression(
        &self,
        source_name: &str,
        span: Option<Span>,
    ) -> Result<Expr, LanguageError> {
        match self {
            Self::Deterministic(pattern) => pattern.strand_expression(source_name, span),
            Self::Relationships(pattern) => Ok(state_expression(pattern.state())),
        }
    }
}

impl fmt::Display for DiscoveredPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.mode())
    }
}

/// Discover a pattern from one indexed native observation state.
///
/// Rank is ignored in deterministic mode. Relationship mode retains the
/// strongest ceiling of the exact rank fraction. Language syntax defaults to
/// rank one, preserving every distinct relationship channel.
///
/// # Errors
///
/// Returns an `NSU` diagnostic for invalid observations, invalid rank,
/// exhausted relationship capacity, or an unrepresentable generated state.
pub fn discover(
    state: &NativeState,
    rank: &str,
    source_name: &str,
    span: Option<Span>,
) -> Result<DiscoveredPattern, LanguageError> {
    let rank = parse_rank(rank, source_name, span)?;
    match crate::continuation::synthesize(state, source_name, span) {
        Ok(continuation) => Ok(DiscoveredPattern::Deterministic(continuation)),
        Err(error) if error.0.code == "NSU004" => {
            let (_first_index, values) =
                crate::continuation::indexed_observations(state, source_name, span)?;
            relationship_pattern(&values, &rank, source_name, span)
                .map(DiscoveredPattern::Relationships)
        }
        Err(error) => Err(error),
    }
}

/// Discover a pattern over ordered complete native states.
///
/// Complete states remain exact symbols. Deterministic discovery uses one
/// shared recurrence over every coordinate. Relationship fallback assigns one
/// exact identifier to each distinct complete state.
///
/// # Errors
///
/// Returns an `NSU` diagnostic for empty input, invalid rank, exhausted
/// relationship capacity, or an unrepresentable generated state.
pub fn discover_states(
    values: &[NativeState],
    rank: &str,
    source_name: &str,
) -> Result<DiscoveredPattern, LanguageError> {
    let rank = parse_rank(rank, source_name, None)?;
    match crate::continuation::synthesize_states(values, source_name) {
        Ok(continuation) => Ok(DiscoveredPattern::Deterministic(continuation)),
        Err(error) if error.0.code == "NSU004" => {
            relationship_pattern(values, &rank, source_name, None)
                .map(DiscoveredPattern::Relationships)
        }
        Err(error) => Err(error),
    }
}

fn parse_rank(
    source: &str,
    source_name: &str,
    span: Option<Span>,
) -> Result<BigRational, LanguageError> {
    let invalid = || {
        discovery_error(
            "NSU012",
            "untrace rank must be an exact number from 0 through 1",
            source_name,
            span,
        )
    };
    let ratio = BigRational::from_str(source).map_err(|_parse_error| invalid())?;
    if ratio < BigRational::zero() || ratio > BigRational::one() {
        return Err(invalid());
    }
    Ok(ratio)
}

fn relationship_pattern(
    values: &[NativeState],
    rank: &BigRational,
    source_name: &str,
    span: Option<Span>,
) -> Result<RelationshipPattern, LanguageError> {
    if values.is_empty() {
        return Err(discovery_error(
            "NSU001",
            "untrace requires at least one observation",
            source_name,
            span,
        ));
    }
    let dictionary = symbol_dictionary(values, source_name, span)?;
    let counts = count_relationships(&dictionary.ids, source_name, span)?;
    let symbols = dictionary.symbols;
    let symbol_counts = dictionary.counts;

    let total_channel_count = counts.len();
    let retained_count = retained_channel_count(total_channel_count, rank, source_name, span)?;
    let mut channels = counts
        .into_iter()
        .map(
            |((left_symbol, right_symbol, distance), count)| RelationshipChannel {
                left_symbol,
                right_symbol,
                distance,
                count,
            },
        )
        .collect::<Vec<_>>();
    channels.sort_by(|left, right| {
        right.count.cmp(&left.count).then_with(|| {
            (left.left_symbol, left.right_symbol, left.distance).cmp(&(
                right.left_symbol,
                right.right_symbol,
                right.distance,
            ))
        })
    });
    let directions = relationship_directions(values, source_name, span)?;
    let state = relationship_state(
        &symbols,
        &symbol_counts,
        &channels[..retained_count],
        directions,
        source_name,
        span,
    )?;
    let rank = rational_source(rank);
    let source = relationship_source(
        values.len(),
        symbols.len(),
        total_channel_count,
        &rank,
        &channels[..retained_count],
        directions,
        &state,
    );
    Ok(RelationshipPattern {
        observation_count: values.len(),
        symbols,
        symbol_counts,
        total_channel_count,
        rank,
        channels: Arc::new(channels),
        retained_channel_count: retained_count,
        directions,
        state,
        source,
    })
}

fn count_relationships(
    symbol_ids: &[u64],
    source_name: &str,
    span: Option<Span>,
) -> Result<BTreeMap<(u64, u64, u64), u64>, LanguageError> {
    let pair_count = symbol_ids
        .len()
        .checked_mul(symbol_ids.len().saturating_sub(1))
        .map(|count| count / 2)
        .ok_or_else(|| {
            discovery_error(
                "NSU013",
                "relationship pair count exceeds the platform size",
                source_name,
                span,
            )
        })?;
    if pair_count > MAX_RELATIONSHIP_PAIRS {
        return Err(discovery_error(
            "NSU013",
            format!(
                "relationship mode supports at most {MAX_RELATIONSHIP_PAIRS} ordered pair visits"
            ),
            source_name,
            span,
        ));
    }
    let mut counts = BTreeMap::<(u64, u64, u64), u64>::new();
    for left_position in 0..symbol_ids.len() {
        for right_position in (left_position + 1)..symbol_ids.len() {
            let distance =
                u64::try_from(right_position - left_position).map_err(|_capacity_error| {
                    discovery_error(
                        "NSU013",
                        "relationship distance exceeds u64",
                        source_name,
                        span,
                    )
                })?;
            let key = (
                symbol_ids[left_position],
                symbol_ids[right_position],
                distance,
            );
            let count = counts.entry(key).or_default();
            *count = count.checked_add(1).ok_or_else(|| {
                discovery_error(
                    "NSU013",
                    "relationship count exceeds u64",
                    source_name,
                    span,
                )
            })?;
        }
    }
    Ok(counts)
}

#[derive(Debug)]
struct SymbolDictionary {
    symbols: Vec<NativeState>,
    ids: Vec<u64>,
    counts: Vec<u64>,
}

fn symbol_dictionary(
    values: &[NativeState],
    source_name: &str,
    span: Option<Span>,
) -> Result<SymbolDictionary, LanguageError> {
    let mut by_state = BTreeMap::<String, u64>::new();
    let mut symbols = Vec::new();
    let mut symbol_counts = Vec::<u64>::new();
    let mut ids = Vec::with_capacity(values.len());
    for value in values {
        let key = value.to_data().to_string();
        let id = if let Some(id) = by_state.get(&key) {
            *id
        } else {
            let id = u64::try_from(symbols.len() + 1).map_err(|_capacity_error| {
                discovery_error("NSU013", "symbol count exceeds u64", source_name, span)
            })?;
            by_state.insert(key, id);
            symbols.push(value.clone());
            symbol_counts.push(0);
            id
        };
        let count = symbol_counts
            .get_mut(usize::try_from(id - 1).map_err(|_capacity_error| {
                discovery_error(
                    "NSU013",
                    "symbol identifier exceeds usize",
                    source_name,
                    span,
                )
            })?)
            .ok_or_else(|| {
                discovery_error(
                    "NSU013",
                    "symbol identifier has no dictionary count",
                    source_name,
                    span,
                )
            })?;
        *count = count.checked_add(1).ok_or_else(|| {
            discovery_error("NSU013", "symbol count exceeds u64", source_name, span)
        })?;
        ids.push(id);
    }
    Ok(SymbolDictionary {
        symbols,
        ids,
        counts: symbol_counts,
    })
}

fn most_frequent_symbol(counts: &[u64]) -> u64 {
    counts
        .iter()
        .enumerate()
        .max_by(|(left_index, left_count), (right_index, right_count)| {
            left_count
                .cmp(right_count)
                .then_with(|| right_index.cmp(left_index))
        })
        .map_or(1, |(index, _count)| {
            u64::try_from(index + 1).expect("the existing symbol identifier fits u64")
        })
}

fn winning_symbol(votes: &BTreeMap<u64, u64>) -> Option<u64> {
    votes
        .iter()
        .max_by(|(left_symbol, left_vote), (right_symbol, right_vote)| {
            left_vote
                .cmp(right_vote)
                .then_with(|| right_symbol.cmp(left_symbol))
        })
        .map(|(symbol, _vote)| *symbol)
}

fn retained_channel_count(
    channel_count: usize,
    rank: &BigRational,
    source_name: &str,
    span: Option<Span>,
) -> Result<usize, LanguageError> {
    if channel_count == 0 || rank.is_zero() {
        return Ok(0);
    }
    let retained =
        (BigInt::from(channel_count) * rank.numer() + rank.denom() - BigInt::one()) / rank.denom();
    retained.to_usize().ok_or_else(|| {
        discovery_error(
            "NSU013",
            "retained relationship count exceeds usize",
            source_name,
            span,
        )
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RelationshipDirections {
    dictionary_entry: u64,
    dictionary_data: u64,
    relationship: u64,
    left_symbol: u64,
    right_symbol: u64,
    distance: u64,
}

fn relationship_directions(
    values: &[NativeState],
    source_name: &str,
    span: Option<Span>,
) -> Result<RelationshipDirections, LanguageError> {
    let maximum = values
        .iter()
        .flat_map(|state| state.0.keys())
        .flat_map(|index| index.0.keys().copied())
        .max()
        .unwrap_or(0);
    let direction = |offset| {
        maximum.checked_add(offset).ok_or_else(|| {
            discovery_error(
                "NSU013",
                "relationship field directions exceed u64",
                source_name,
                span,
            )
        })
    };
    Ok(RelationshipDirections {
        dictionary_entry: direction(1)?,
        dictionary_data: direction(2)?,
        relationship: direction(3)?,
        left_symbol: direction(4)?,
        right_symbol: direction(5)?,
        distance: direction(6)?,
    })
}

fn relationship_state(
    symbols: &[NativeState],
    symbol_counts: &[u64],
    channels: &[RelationshipChannel],
    directions: RelationshipDirections,
    source_name: &str,
    span: Option<Span>,
) -> Result<NativeState, LanguageError> {
    let mut terms = Vec::new();
    for (offset, (symbol, count)) in symbols.iter().zip(symbol_counts).enumerate() {
        let id = u64::try_from(offset + 1).map_err(|_capacity_error| {
            discovery_error("NSU013", "symbol identifier exceeds u64", source_name, span)
        })?;
        terms.push((
            MultiIndex::from_depths([(directions.dictionary_entry, id)])
                .map_err(|message| discovery_error("NSU013", message, source_name, span))?,
            NativeScalar::from_text(&count.to_string(), "0")
                .map_err(|message| discovery_error("NSU013", message, source_name, span))?,
        ));
        for (index, coefficient) in &symbol.0 {
            terms.push((
                index
                    .shift_by(directions.dictionary_data, id)
                    .map_err(|message| discovery_error("NSU013", message, source_name, span))?,
                coefficient.clone(),
            ));
        }
    }
    for channel in channels {
        let index = MultiIndex::from_depths([
            (directions.relationship, 1),
            (directions.left_symbol, channel.left_symbol),
            (directions.right_symbol, channel.right_symbol),
            (directions.distance, channel.distance),
        ])
        .map_err(|message| discovery_error("NSU013", message, source_name, span))?;
        let coefficient = NativeScalar::from_text(&channel.count.to_string(), "0")
            .map_err(|message| discovery_error("NSU013", message, source_name, span))?;
        terms.push((index, coefficient));
    }
    Ok(NativeState::from_terms(terms))
}

fn relationship_source(
    observation_count: usize,
    symbol_count: usize,
    total_channel_count: usize,
    rank: &str,
    channels: &[RelationshipChannel],
    directions: RelationshipDirections,
    state: &NativeState,
) -> String {
    format!(
        "# Generated by untrace.\n\
# Mode: relationships.\n\
# No supported exact deterministic continuation matched every held-out observation.\n\
# Observations: {observation_count}. Exact symbols: {symbol_count}.\n\
# Distinct relationship channels: {total_channel_count}. Retained: {} at rank {rank}.\n\
# Every channel is (earlier symbol, later symbol, positive distance, exact count).\n\
# Dictionary entry/data directions: {}/{}.\n\
# Relationship/left/right/distance directions: {}/{}/{}/{}.\n\n\
output {} as pattern\n",
        channels.len(),
        directions.dictionary_entry,
        directions.dictionary_data,
        directions.relationship,
        directions.left_symbol,
        directions.right_symbol,
        directions.distance,
        expression_source(&state_expression(state)),
    )
}

fn relationship_csv(pattern: &RelationshipPattern) -> Result<String, LanguageError> {
    let source_name = "generated-untrace-relationships";
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer
        .write_record([
            "part", "symbol", "left", "right", "distance", "count", "value",
        ])
        .map_err(|csv_error| discovery_error("NSU009", csv_error.to_string(), source_name, None))?;
    for (offset, (symbol, count)) in pattern
        .symbols
        .iter()
        .zip(&pattern.symbol_counts)
        .enumerate()
    {
        writer
            .write_record([
                "symbol",
                &(offset + 1).to_string(),
                "",
                "",
                "",
                &count.to_string(),
                &symbol.to_data().to_string(),
            ])
            .map_err(|csv_error| {
                discovery_error("NSU009", csv_error.to_string(), source_name, None)
            })?;
    }
    for channel in pattern.channels() {
        writer
            .write_record([
                "relationship",
                "",
                &channel.left_symbol.to_string(),
                &channel.right_symbol.to_string(),
                &channel.distance.to_string(),
                &channel.count.to_string(),
                "",
            ])
            .map_err(|csv_error| {
                discovery_error("NSU009", csv_error.to_string(), source_name, None)
            })?;
    }
    let bytes = writer
        .into_inner()
        .map_err(|csv_error| discovery_error("NSU009", csv_error.to_string(), source_name, None))?;
    String::from_utf8(bytes)
        .map_err(|utf8_error| discovery_error("NSU009", utf8_error.to_string(), source_name, None))
}

fn rational_source(value: &BigRational) -> String {
    if value.denom().is_one() {
        value.numer().to_string()
    } else {
        format!("{}/{}", value.numer(), value.denom())
    }
}

fn discovery_error(
    code: &str,
    message: impl Into<String>,
    source_name: &str,
    span: Option<Span>,
) -> LanguageError {
    LanguageError(Diagnostic {
        code: code.into(),
        message: message.into(),
        source_name: source_name.into(),
        span,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scalar(value: i64) -> NativeState {
        NativeState::scalar(NativeScalar::from_text(&value.to_string(), "0").unwrap())
    }

    #[test]
    fn exact_recurrence_selects_deterministic_mode() {
        let values = [1, 1, 2, 3, 5, 8, 13].map(scalar);
        let pattern = discover_states(&values, "1/4", "fibonacci.json").unwrap();

        let DiscoveredPattern::Deterministic(continuation) = pattern else {
            panic!("Fibonacci must select deterministic mode");
        };
        assert_eq!(continuation.recurrence_order(), 2);
        assert_eq!(continuation.next_value(), &scalar(21));
    }

    #[test]
    fn unsupported_recurrence_counts_every_ordered_relationship() {
        let values = [2, 3, 5, 7, 11, 13].map(scalar);
        let pattern = discover_states(&values, "1", "primes.json").unwrap();

        let DiscoveredPattern::Relationships(pattern) = pattern else {
            panic!("prime observations must select relationship mode");
        };
        assert_eq!(pattern.observation_count(), 6);
        assert_eq!(pattern.total_channel_count(), 15);
        assert_eq!(pattern.channels().len(), 15);
        assert_eq!(pattern.rank(), "1");
    }

    #[test]
    fn fractional_rank_keeps_the_strongest_ceiling() {
        let values = [1, 2, 1, 2].map(scalar);
        let pattern = relationship_pattern(
            &values,
            &BigRational::new(BigInt::from(2), BigInt::from(5)),
            "symbols.json",
            None,
        )
        .unwrap();

        assert_eq!(pattern.total_channel_count(), 5);
        assert_eq!(pattern.channels().len(), 2);
        assert_eq!(pattern.channels()[0].count(), 2);
    }

    #[test]
    fn relationship_source_round_trips_exactly() {
        let values = [2, 3, 5, 7, 11, 13].map(scalar);
        let pattern =
            relationship_pattern(&values, &BigRational::one(), "primes.json", None).unwrap();
        let program = crate::core::parse(pattern.source(), "generated.ns").unwrap();

        assert_eq!(crate::core::interpret(&program).unwrap(), *pattern.state());
        assert!(relationship_csv(&pattern).unwrap().contains("relationship"));
    }

    #[test]
    fn relationship_generation_uses_counts_and_exact_distance_votes() {
        let values = [1, 2, 1, 2].map(scalar);
        let pattern =
            relationship_pattern(&values, &BigRational::one(), "alternating.json", None).unwrap();

        assert_eq!(pattern.symbol_counts(), &[2, 2]);
        assert_eq!(pattern.generate(4, "alternating.json").unwrap(), values);
    }

    #[test]
    fn rerank_reuses_complete_counts_and_changes_only_retained_prefix() {
        let values = [2, 3, 5, 7, 11, 13].map(scalar);
        let full = relationship_pattern(&values, &BigRational::one(), "primes.json", None).unwrap();
        let quarter = full.rerank("1/4", "primes.json").unwrap();

        assert_eq!(quarter.total_channel_count(), 15);
        assert_eq!(quarter.channels().len(), 4);
        assert_eq!(quarter.symbols(), full.symbols());
        assert_eq!(quarter.symbol_counts(), full.symbol_counts());
        assert_eq!(
            crate::core::interpret(&crate::core::parse(quarter.source(), "quarter.ns").unwrap())
                .unwrap(),
            *quarter.state()
        );
    }

    #[test]
    fn invalid_rank_is_rejected() {
        let error = discover_states(&[scalar(1)], "3/2", "invalid.json").unwrap_err();

        assert_eq!(error.0.code, "NSU012");
    }
}
