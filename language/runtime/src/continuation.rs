// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Synthesizes exact continuation programs from finite ordered observations.
//!
//! The first synthesis grammar is intentionally narrow and auditable: a
//! constant-coefficient linear recurrence over exact native states. Recurrence
//! coefficients remain exact native scalars, so the same operation applies to
//! every retained coordinate without flattening it. Every
//! candidate is determined from a training prefix, must then regenerate at
//! least one held-out supplied observation, and may differ at no more than the
//! declared ratio of held-out positions. The selected candidate first minimizes
//! its exact mismatch ratio and then seed nodes plus recurrence-expression nodes;
//! source length and recurrence order break remaining ties deterministically.

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::Path,
    str::FromStr as _,
};

use num_bigint::{BigInt, BigUint};
use num_rational::BigRational;
use num_traits::{One as _, ToPrimitive as _, Zero as _};

use crate::core::{
    Diagnostic, Expr, Function, LanguageError, MultiIndex, NativeScalar, NativeState, Span,
};

/// Maximum recurrence order considered by exact synthesis.
///
/// Candidate solving is cubic in its order. The complete input remains in
/// memory and participates in validation; this limit bounds model complexity,
/// not the amount of evidence examined. Raising it requires a benchmark.
const MAX_RECURRENCE_ORDER: usize = 32;

/// Maximum dense span reconstructed from one sparse in-language state.
///
/// File inputs already arrive as a dense ordered list and have no equivalent
/// observation-count limit. This protects a tiny sparse state from naming an
/// enormous absent interval that would otherwise require a matching allocation.
const MAX_INDEXED_SPAN: usize = 1_000_000;

/// Maximum INDEX nesting emitted into one generated source document.
///
/// Native source spells multiplicity by nesting `index`, so this bound keeps
/// hostile flat-stack inputs from requesting an impractically large document.
const MAX_SOURCE_INDEX_STEPS: usize = 1_000_000;

const POSITION_PARAMETER: &str = "position";
const NEXT_FUNCTION: &str = "next_value";
const CONTINUATION_FUNCTION: &str = "continuation";
const START_FUNCTION: &str = "start_continuation";

/// One exact synthesized continuation program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Continuation {
    first_index: u64,
    last_index: u64,
    next_index: u64,
    observation_count: usize,
    recurrence_order: usize,
    validated_steps: usize,
    maximum_error_ratio: String,
    error_indexes: Vec<u64>,
    description_nodes: usize,
    primitive_steps: usize,
    next_value: NativeState,
    seeds: Vec<NativeState>,
    coefficients: Vec<NativeScalar>,
    source: String,
    functions: Vec<Function>,
}

/// An unbounded exact execution of one synthesized recurrence.
///
/// The supplied seed values are yielded first. Every later value is computed
/// from the same coefficients that `untrace` selected and validated. The
/// cursor retains only one recurrence window, so continued execution does not
/// grow with the number of generated values.
#[derive(Clone, Debug)]
pub struct ContinuationSequence {
    prefix: std::vec::IntoIter<NativeState>,
    window: VecDeque<NativeState>,
    coefficients: Vec<NativeScalar>,
}

impl Iterator for ContinuationSequence {
    type Item = NativeState;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(seed) = self.prefix.next() {
            return Some(seed);
        }
        let window = self.window.iter().cloned().collect::<Vec<_>>();
        let next = apply_recurrence(&self.coefficients, &window);
        self.window.pop_front();
        self.window.push_back(next.clone());
        Some(next)
    }
}

impl Continuation {
    /// Return the first supplied observation index.
    #[must_use]
    pub const fn first_index(&self) -> u64 {
        self.first_index
    }

    /// Return the final supplied observation index.
    #[must_use]
    pub const fn last_index(&self) -> u64 {
        self.last_index
    }

    /// Return the number of supplied index positions, including internal zeros.
    #[must_use]
    pub const fn observation_count(&self) -> usize {
        self.observation_count
    }

    /// Return the number of prior values consumed by one continuation step.
    #[must_use]
    pub const fn recurrence_order(&self) -> usize {
        self.recurrence_order
    }

    /// Return how many held-out supplied states matched the recurrence.
    #[must_use]
    pub const fn validated_steps(&self) -> usize {
        self.validated_steps
    }

    /// Return the maximum accepted exact ratio of mismatching held-out indexes.
    #[must_use]
    pub fn maximum_error_ratio(&self) -> &str {
        &self.maximum_error_ratio
    }

    /// Return every supplied index where the generated continuation disagrees.
    #[must_use]
    pub fn error_indexes(&self) -> &[u64] {
        &self.error_indexes
    }

    /// Return the complete seed-plus-expression node count used for selection.
    #[must_use]
    pub const fn description_nodes(&self) -> usize {
        self.description_nodes
    }

    /// Return the primitive operations performed by one continuation step.
    #[must_use]
    pub const fn primitive_steps(&self) -> usize {
        self.primitive_steps
    }

    /// Return the first value predicted beyond the supplied observations.
    #[must_use]
    pub const fn next_value(&self) -> &NativeState {
        &self.next_value
    }

    /// Start exact execution at the first seed used by this continuation.
    #[must_use]
    pub fn sequence(&self) -> ContinuationSequence {
        ContinuationSequence {
            prefix: self.seeds.clone().into_iter(),
            window: self.seeds.clone().into(),
            coefficients: self.coefficients.clone(),
        }
    }

    /// Return a complete runnable source document for the recursive pattern.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Return the generated continuation as its exact nested operation state.
    ///
    /// # Errors
    ///
    /// Returns a language diagnostic if the internally generated strand cannot
    /// be interpreted as an exact state.
    pub fn pattern_state(&self) -> Result<NativeState, LanguageError> {
        let result = self.strand_expression("generated-untrace-pattern", None)?;
        crate::core::interpret(&crate::core::Program {
            functions: Vec::new(),
            bindings: Vec::new(),
            goal: crate::core::Goal::Emit,
            output_kind: crate::core::OutputKind::Pattern,
            result,
            source_name: "generated-untrace-pattern".into(),
            span: None,
        })
    }

    /// Return the discovered continuation as a compact CSV pattern table.
    ///
    /// # Errors
    ///
    /// Returns `NSU009` if CSV serialization fails.
    pub fn pattern_csv(&self) -> Result<String, LanguageError> {
        let source_name = "generated-untrace-pattern";
        let mut writer = csv::Writer::from_writer(Vec::new());
        writer
            .write_record(["part", "index", "real", "imag"])
            .map_err(|csv_error| error("NSU009", csv_error.to_string(), source_name, None))?;
        for (offset, seed) in self.seeds.iter().enumerate() {
            let offset = u64::try_from(offset).map_err(|_capacity_error| {
                error("NSU009", "seed index exceeds u64", source_name, None)
            })?;
            let index = self
                .first_index
                .checked_add(offset)
                .ok_or_else(|| error("NSU009", "seed index exceeds u64", source_name, None))?;
            let seed = scalar_state(seed).ok_or_else(|| {
                error(
                    "NSU009",
                    "pattern CSV supports scalar continuations; use source output for indexed states",
                    source_name,
                    None,
                )
            })?;
            write_pattern_row(&mut writer, "seed", index, &seed, source_name)?;
        }
        for (offset, coefficient) in self.coefficients.iter().enumerate() {
            let lag = u64::try_from(offset + 1).map_err(|_capacity_error| {
                error("NSU009", "coefficient lag exceeds u64", source_name, None)
            })?;
            write_pattern_row(&mut writer, "coefficient", lag, coefficient, source_name)?;
        }
        for index in &self.error_indexes {
            writer
                .write_record(["mismatch", &index.to_string(), "", ""])
                .map_err(|csv_error| error("NSU009", csv_error.to_string(), source_name, None))?;
        }
        let next_value = scalar_state(&self.next_value).ok_or_else(|| {
            error(
                "NSU009",
                "pattern CSV supports scalar continuations; use source output for indexed states",
                source_name,
                None,
            )
        })?;
        write_pattern_row(
            &mut writer,
            "prediction",
            self.next_index,
            &next_value,
            source_name,
        )?;
        let bytes = writer
            .into_inner()
            .map_err(|csv_error| error("NSU009", csv_error.to_string(), source_name, None))?;
        String::from_utf8(bytes)
            .map_err(|utf8_error| error("NSU009", utf8_error.to_string(), source_name, None))
    }

    pub(crate) fn strand_expression(
        &self,
        diagnostic_source: &str,
        span: Option<Span>,
    ) -> Result<Expr, LanguageError> {
        let catalog = self
            .functions
            .iter()
            .map(|function| (function.name.clone(), function))
            .collect::<BTreeMap<_, _>>();
        crate::strand::operation_strand(
            START_FUNCTION,
            &catalog,
            &self.source,
            diagnostic_source,
            span,
        )
    }
}

/// Read exact indexed observations from a CSV file.
///
/// The header must contain `index` and either `value` or `real`; optional
/// `imag` values default to zero. Every index must be unique and positive.
///
/// # Errors
///
/// Returns `NSU008` for an unreadable or malformed observation table.
pub fn read_observations_csv(path: impl AsRef<Path>) -> Result<NativeState, LanguageError> {
    let path = path.as_ref();
    let source_name = path.display().to_string();
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)
        .map_err(|csv_error| {
            error(
                "NSU008",
                format!("could not read observation CSV: {csv_error}"),
                &source_name,
                None,
            )
        })?;
    let headers = reader
        .headers()
        .map_err(|csv_error| {
            error(
                "NSU008",
                format!("invalid observation CSV header: {csv_error}"),
                &source_name,
                None,
            )
        })?
        .clone();
    let columns = observation_columns(&headers, &source_name)?;
    let mut seen = BTreeSet::new();
    let terms = reader
        .records()
        .enumerate()
        .map(|(row, record)| {
            let row = row + 2;
            let record = record.map_err(|csv_error| {
                error(
                    "NSU008",
                    format!("invalid observation CSV row {row}: {csv_error}"),
                    &source_name,
                    None,
                )
            })?;
            observation_term(&record, columns, row, &mut seen, &source_name)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(NativeState::from_terms(terms))
}

#[derive(Clone, Copy, Debug)]
struct ObservationColumns {
    index: usize,
    real: usize,
    imag: Option<usize>,
}

fn observation_columns(
    headers: &csv::StringRecord,
    source_name: &str,
) -> Result<ObservationColumns, LanguageError> {
    let index = column(headers, "index").ok_or_else(|| {
        error(
            "NSU008",
            "observation CSV needs an index column",
            source_name,
            None,
        )
    })?;
    let real = column(headers, "value")
        .or_else(|| column(headers, "real"))
        .ok_or_else(|| {
            error(
                "NSU008",
                "observation CSV needs a value or real column",
                source_name,
                None,
            )
        })?;
    Ok(ObservationColumns {
        index,
        real,
        imag: column(headers, "imag"),
    })
}

fn observation_term(
    record: &csv::StringRecord,
    columns: ObservationColumns,
    row: usize,
    seen: &mut BTreeSet<u64>,
    source_name: &str,
) -> Result<(crate::core::MultiIndex, NativeScalar), LanguageError> {
    let index = record
        .get(columns.index)
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|index| *index > 0)
        .ok_or_else(|| {
            error(
                "NSU008",
                format!("observation CSV row {row} has no positive integer index"),
                source_name,
                None,
            )
        })?;
    if !seen.insert(index) {
        return Err(error(
            "NSU008",
            format!("observation CSV repeats index {index}"),
            source_name,
            None,
        ));
    }
    let real = record.get(columns.real).ok_or_else(|| {
        error(
            "NSU008",
            format!("observation CSV row {row} has no value"),
            source_name,
            None,
        )
    })?;
    let imag = columns
        .imag
        .and_then(|column| record.get(column))
        .unwrap_or("0");
    let scalar = NativeScalar::from_text(real, imag).map_err(|message| {
        error(
            "NSU008",
            format!("observation CSV row {row}: {message}"),
            source_name,
            None,
        )
    })?;
    let index = crate::core::MultiIndex::from_depths([(index, 1)])
        .map_err(|message| error("NSU008", message, source_name, None))?;
    Ok((index, scalar))
}

fn column(headers: &csv::StringRecord, name: &str) -> Option<usize> {
    headers.iter().position(|header| header == name)
}

fn write_pattern_row(
    writer: &mut csv::Writer<Vec<u8>>,
    part: &str,
    index: u64,
    scalar: &NativeScalar,
    source_name: &str,
) -> Result<(), LanguageError> {
    let index = index.to_string();
    let real = rational_source(&scalar.real);
    let imag = rational_source(&scalar.imag);
    writer
        .write_record([part, &index, &real, &imag])
        .map_err(|csv_error| error("NSU009", csv_error.to_string(), source_name, None))
}

fn scalar_state(state: &NativeState) -> Option<NativeScalar> {
    if state.0.is_empty() {
        return Some(NativeScalar::zero());
    }
    let (index, scalar) = state.0.first_key_value()?;
    (state.0.len() == 1 && index.0.is_empty()).then(|| scalar.clone())
}

#[derive(Debug)]
struct Candidate {
    order: usize,
    coefficients: Vec<NativeScalar>,
    body: Expr,
    description_nodes: usize,
    primitive_steps: usize,
    source_bytes: usize,
    body_source: String,
    error_offsets: Vec<usize>,
    next_value: NativeState,
    held_out: usize,
}

/// Find the most repeatable recurrence in the Language 1.0 synthesis grammar.
///
/// Legacy scalar terms are ordered by INDEX direction. Array-style terms use
/// direction one as the sequence axis and its depth as the observation index;
/// every remaining INDEX coordinate stays inside that observation state.
/// Missing sequence positions inside the observed endpoints are exact zero.
/// `maximum_error_ratio` bounds the exact fraction of held-out indexes where
/// the recursively generated value may differ from the supplied observation.
///
/// # Errors
///
/// Returns `NSU001` for an empty or incompatible coordinate layout, `NSU002`
/// when the indexed span exceeds the bounded exact-search budget, `NSU003`
/// when no next index exists, `NSU004` when no supported recurrence meets the
/// error ratio, or `NSU005` when the ratio is not between zero and one.
pub fn synthesize(
    state: &NativeState,
    maximum_error_ratio: &str,
    source_name: &str,
    span: Option<Span>,
) -> Result<Continuation, LanguageError> {
    let (first_index, values) = observations(state, source_name, span)?;
    synthesize_values(&values, first_index, maximum_error_ratio, source_name, span)
}

/// Find one recurrence over ordered complete native states.
///
/// Each input is one observation. Scalar coefficients act on every retained
/// coordinate of that state, so INDEX structure remains intact.
///
/// # Errors
///
/// Returns `NSU001` for no observations or an unrepresentable source state,
/// `NSU003` when no next index exists, `NSU004` when no supported recurrence
/// meets the error ratio, or `NSU005` when the ratio is not between zero and
/// one.
pub fn synthesize_states(
    values: &[NativeState],
    maximum_error_ratio: &str,
    source_name: &str,
) -> Result<Continuation, LanguageError> {
    synthesize_values(values, 1, maximum_error_ratio, source_name, None)
}

/// Find one exact recurrence over a compact sequence of unsigned symbols.
///
/// This is mathematically the scalar case of [`synthesize_states`], but it
/// avoids allocating one map-backed native state per symbol. The iterator must
/// be cloneable because each bounded recurrence order is solved and validated
/// against the same complete sequence. Compact symbol input currently accepts
/// only an exact zero error ratio; retaining potentially billions of mismatch
/// indexes would defeat the compact representation.
///
/// # Errors
///
/// Returns `NSU001` for no observations, `NSU003` when no next index exists,
/// `NSU004` when no supported recurrence matches the complete sequence,
/// `NSU005` when the ratio is invalid, or `NSU011` when it is nonzero.
pub fn synthesize_symbols<I>(
    symbols: &I,
    maximum_error_ratio: &str,
    source_name: &str,
) -> Result<Continuation, LanguageError>
where
    I: Clone + ExactSizeIterator<Item = u16>,
{
    let observation_count = symbols.len();
    if observation_count == 0 {
        return Err(error(
            "NSU001",
            "untrace requires at least one observation",
            source_name,
            None,
        ));
    }
    let maximum_error_ratio = parse_error_ratio(maximum_error_ratio, source_name, None)?;
    if !maximum_error_ratio.is_zero() {
        return Err(error(
            "NSU011",
            "compact symbol untrace requires an exact maximum error ratio of 0",
            source_name,
            None,
        ));
    }
    let (last_index, next_index) = continuation_indexes(1, observation_count, source_name, None)?;
    let maximum_order = ((observation_count - 1) / 2).min(MAX_RECURRENCE_ORDER);
    let candidate = (1..=maximum_order)
        .filter_map(|order| symbol_candidate(symbols.clone(), order))
        .min_by(|left, right| {
            (
                left.description_nodes,
                left.source_bytes,
                left.order,
                &left.body_source,
            )
                .cmp(&(
                    right.description_nodes,
                    right.source_bytes,
                    right.order,
                    &right.body_source,
                ))
        })
        .ok_or_else(|| {
            error(
                "NSU004",
                "no supported continuation exactly matches the complete symbol sequence",
                source_name,
                None,
            )
        })?;
    let seeds = symbols
        .clone()
        .take(candidate.order)
        .map(symbol_state)
        .collect::<Vec<_>>();
    let functions =
        continuation_functions(1, &seeds, &candidate.coefficients, candidate.body.clone());
    let source = continuation_source(1, observation_count, &seeds, &candidate, "0", &[]);
    Ok(Continuation {
        first_index: 1,
        last_index,
        next_index,
        observation_count,
        recurrence_order: candidate.order,
        validated_steps: candidate.held_out,
        maximum_error_ratio: "0".into(),
        error_indexes: Vec::new(),
        description_nodes: candidate.description_nodes,
        primitive_steps: candidate.primitive_steps,
        next_value: candidate.next_value.clone(),
        seeds,
        coefficients: candidate.coefficients.clone(),
        source,
        functions,
    })
}

fn synthesize_values(
    values: &[NativeState],
    first_index: u64,
    maximum_error_ratio: &str,
    source_name: &str,
    span: Option<Span>,
) -> Result<Continuation, LanguageError> {
    if values.is_empty() {
        return Err(error(
            "NSU001",
            "untrace requires at least one observation",
            source_name,
            span,
        ));
    }
    validate_source_states(values, source_name, span)?;
    let maximum_error_ratio = parse_error_ratio(maximum_error_ratio, source_name, span)?;
    let maximum_error_ratio_source = rational_source(&maximum_error_ratio);
    let (last_index, next_index) =
        continuation_indexes(first_index, values.len(), source_name, span)?;

    let candidate = candidates(values, &maximum_error_ratio)
        .into_iter()
        .min_by(|left, right| {
            compare_error_ratio(left, right).then_with(|| {
                (
                    left.description_nodes,
                    left.source_bytes,
                    left.order,
                    &left.body_source,
                )
                    .cmp(&(
                        right.description_nodes,
                        right.source_bytes,
                        right.order,
                        &right.body_source,
                    ))
            })
        });
    let candidate = candidate.ok_or_else(|| {
        error(
            "NSU004",
            format!(
                "no supported continuation stays within a held-out index-error ratio of {maximum_error_ratio_source}"
            ),
            source_name,
            span,
        )
    })?;

    let error_indexes = candidate
        .error_offsets
        .iter()
        .map(|offset| {
            u64::try_from(*offset)
                .ok()
                .and_then(|offset| first_index.checked_add(offset))
                .ok_or_else(|| {
                    error(
                        "NSU003",
                        "an observation error index exceeds the native index space",
                        source_name,
                        span,
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let next_value = candidate.next_value.clone();
    let seeds = &values[..candidate.order];
    let functions = continuation_functions(
        first_index,
        seeds,
        &candidate.coefficients,
        candidate.body.clone(),
    );
    let source = continuation_source(
        first_index,
        values.len(),
        seeds,
        &candidate,
        &maximum_error_ratio_source,
        &error_indexes,
    );

    Ok(Continuation {
        first_index,
        last_index,
        next_index,
        observation_count: values.len(),
        recurrence_order: candidate.order,
        validated_steps: values
            .len()
            .saturating_sub(candidate.order * 2)
            .saturating_sub(error_indexes.len()),
        maximum_error_ratio: maximum_error_ratio_source,
        error_indexes,
        description_nodes: candidate.description_nodes,
        primitive_steps: candidate.primitive_steps,
        next_value,
        seeds: values[..candidate.order].to_vec(),
        coefficients: candidate.coefficients.clone(),
        source,
        functions,
    })
}

fn continuation_indexes(
    first_index: u64,
    observation_count: usize,
    source_name: &str,
    span: Option<Span>,
) -> Result<(u64, u64), LanguageError> {
    let last_offset = u64::try_from(observation_count - 1).map_err(|_capacity_error| {
        error(
            "NSU002",
            "the observation span exceeds the supported exact-search budget",
            source_name,
            span,
        )
    })?;
    let last_index = first_index.checked_add(last_offset).ok_or_else(|| {
        error(
            "NSU003",
            "the final observation index exceeds the native index space",
            source_name,
            span,
        )
    })?;
    let next_index = last_index.checked_add(1).ok_or_else(|| {
        error(
            "NSU003",
            "the observation has no representable continuation index",
            source_name,
            span,
        )
    })?;
    Ok((last_index, next_index))
}

fn observations(
    state: &NativeState,
    source_name: &str,
    span: Option<Span>,
) -> Result<(u64, Vec<NativeState>), LanguageError> {
    if state.0.is_empty() {
        return Err(error(
            "NSU001",
            "untrace requires at least one indexed observation",
            source_name,
            span,
        ));
    }
    let legacy_scalar_layout = state.0.keys().all(|index| {
        index.0.len() == 1
            && index
                .0
                .first_key_value()
                .is_some_and(|(_, depth)| depth.is_one())
    });
    if legacy_scalar_layout {
        let indexed = state
            .0
            .iter()
            .map(|(index, coefficient)| {
                let (&position, _) = index
                    .0
                    .first_key_value()
                    .expect("the legacy scalar layout has one index entry");
                (position, NativeState::scalar(coefficient.clone()))
            })
            .collect::<BTreeMap<_, _>>();
        return complete_observations(&indexed, source_name, span);
    }

    let mut indexed = BTreeMap::<u64, Vec<(MultiIndex, NativeScalar)>>::new();
    for (index, coefficient) in &state.0 {
        let position = index.0.get(&1).and_then(BigUint::to_u64).ok_or_else(|| {
            error(
                "NSU001",
                "array-style untrace expects a positive sequence depth on INDEX direction 1",
                source_name,
                span,
            )
        })?;
        let mut payload = index.0.clone();
        payload.remove(&1);
        indexed
            .entry(position)
            .or_default()
            .push((MultiIndex(payload), coefficient.clone()));
    }
    let observations = indexed
        .into_iter()
        .map(|(position, terms)| (position, NativeState::from_terms(terms)))
        .collect();
    complete_observations(&observations, source_name, span)
}

fn complete_observations(
    indexed: &BTreeMap<u64, NativeState>,
    source_name: &str,
    span: Option<Span>,
) -> Result<(u64, Vec<NativeState>), LanguageError> {
    let (&first, _) = indexed.first_key_value().ok_or_else(|| {
        error(
            "NSU001",
            "untrace requires at least one indexed observation",
            source_name,
            span,
        )
    })?;
    let (&last, _) = indexed
        .last_key_value()
        .expect("a checked nonempty observation map has a last entry");
    let span_size = last
        .checked_sub(first)
        .and_then(|difference| difference.checked_add(1))
        .and_then(|count| usize::try_from(count).ok())
        .ok_or_else(|| {
            error(
                "NSU002",
                "the indexed observation span is too large",
                source_name,
                span,
            )
        })?;
    if span_size > MAX_INDEXED_SPAN {
        return Err(error(
            "NSU002",
            format!(
                "one sparse untrace state supports at most {MAX_INDEXED_SPAN} consecutive indexed positions"
            ),
            source_name,
            span,
        ));
    }
    let values = (first..=last)
        .map(|position| {
            indexed
                .get(&position)
                .cloned()
                .unwrap_or_else(NativeState::zero)
        })
        .collect();
    Ok((first, values))
}

fn parse_error_ratio(
    source: &str,
    source_name: &str,
    span: Option<Span>,
) -> Result<BigRational, LanguageError> {
    let ratio = BigRational::from_str(source).map_err(|_parse_error| {
        error(
            "NSU005",
            "untrace maximum error ratio must be an exact number from 0 through 1",
            source_name,
            span,
        )
    })?;
    if ratio < BigRational::zero() || ratio > BigRational::one() {
        return Err(error(
            "NSU005",
            "untrace maximum error ratio must be an exact number from 0 through 1",
            source_name,
            span,
        ));
    }
    Ok(ratio)
}

fn candidates(values: &[NativeState], maximum_error_ratio: &BigRational) -> Vec<Candidate> {
    if values.len() == 1 {
        return Vec::new();
    }
    let maximum_order = ((values.len() - 1) / 2).min(MAX_RECURRENCE_ORDER);
    (1..=maximum_order)
        .filter_map(|order| {
            solve_recurrence(values, order).map(|coefficients| {
                let body = recurrence_expression(&coefficients);
                candidate(order, coefficients, body, values)
            })
        })
        .filter(|candidate| candidate_error_ratio(candidate) <= *maximum_error_ratio)
        .collect()
}

fn symbol_candidate<I>(symbols: I, order: usize) -> Option<Candidate>
where
    I: Clone + ExactSizeIterator<Item = u16>,
{
    let observation_count = symbols.len();
    let training_end = order.checked_mul(2)?;
    if training_end >= observation_count {
        return None;
    }
    let training_symbols = symbols.clone().take(training_end).collect::<Vec<_>>();
    let coefficients = solve_symbol_recurrence(&training_symbols);
    if coefficients.len() != order {
        return None;
    }
    let seeds = training_symbols[..order]
        .iter()
        .copied()
        .map(symbol_state)
        .collect::<Vec<_>>();
    let mut generated = symbols
        .clone()
        .skip(order)
        .take(order)
        .map(symbol_scalar)
        .collect::<Vec<_>>();
    for symbol in symbols.skip(training_end) {
        let predicted = apply_scalar_recurrence(&coefficients, &generated);
        if predicted != symbol_scalar(symbol) {
            return None;
        }
        generated.rotate_left(1);
        generated[order - 1] = predicted;
    }
    let next_value = NativeState::scalar(apply_scalar_recurrence(&coefficients, &generated));
    let body = recurrence_expression(&coefficients);
    let description_nodes = order + expression_nodes(&body);
    let primitive_steps = operation_steps(&body);
    let body_source = expression_source(&body);
    let source_bytes = body_source.len()
        + seeds
            .iter()
            .map(|value| state_source(value).len())
            .sum::<usize>();
    Some(Candidate {
        order,
        coefficients,
        body,
        description_nodes,
        primitive_steps,
        source_bytes,
        body_source,
        error_offsets: Vec::new(),
        next_value,
        held_out: observation_count - training_end,
    })
}

fn solve_symbol_recurrence(values: &[u16]) -> Vec<NativeScalar> {
    let sequence = values
        .iter()
        .map(|value| BigRational::from_integer(BigInt::from(*value)))
        .collect::<Vec<_>>();
    let mut connection = vec![BigRational::one()];
    let mut previous = vec![BigRational::one()];
    let mut order = 0_usize;
    let mut shift = 1_usize;
    let mut previous_discrepancy = BigRational::one();
    for position in 0..sequence.len() {
        let discrepancy = (1..=order).fold(sequence[position].clone(), |sum, lag| {
            sum + &connection[lag] * &sequence[position - lag]
        });
        if discrepancy.is_zero() {
            shift += 1;
            continue;
        }
        let snapshot = connection.clone();
        let factor = &discrepancy / &previous_discrepancy;
        let required = previous.len() + shift;
        if connection.len() < required {
            connection.resize(required, BigRational::zero());
        }
        for (index, coefficient) in previous.iter().enumerate() {
            connection[index + shift] -= &factor * coefficient;
        }
        if order * 2 <= position {
            order = position + 1 - order;
            previous = snapshot;
            previous_discrepancy = discrepancy;
            shift = 1;
        } else {
            shift += 1;
        }
    }
    if order == 0 {
        return vec![NativeScalar::zero()];
    }
    (1..=order)
        .map(|lag| NativeScalar {
            real: -&connection[lag],
            imag: BigRational::zero(),
        })
        .collect()
}

fn symbol_state(symbol: u16) -> NativeState {
    NativeState::scalar(symbol_scalar(symbol))
}

fn symbol_scalar(symbol: u16) -> NativeScalar {
    NativeScalar {
        real: BigRational::from_integer(BigInt::from(symbol)),
        imag: BigRational::zero(),
    }
}

fn apply_scalar_recurrence(
    coefficients: &[NativeScalar],
    oldest_to_newest: &[NativeScalar],
) -> NativeScalar {
    coefficients
        .iter()
        .zip(oldest_to_newest.iter().rev())
        .fold(NativeScalar::zero(), |sum, (coefficient, value)| {
            sum.add(&coefficient.multiply(value))
        })
}

fn candidate_error_ratio(candidate: &Candidate) -> BigRational {
    BigRational::new(
        BigInt::from(candidate.error_offsets.len()),
        BigInt::from(candidate.held_out),
    )
}

fn compare_error_ratio(left: &Candidate, right: &Candidate) -> Ordering {
    (left.error_offsets.len() * right.held_out).cmp(&(right.error_offsets.len() * left.held_out))
}

fn candidate(
    order: usize,
    coefficients: Vec<NativeScalar>,
    body: Expr,
    values: &[NativeState],
) -> Candidate {
    let generated = generate_values(&values[..order], &coefficients, values.len() + 1);
    let error_offsets = (order * 2..values.len())
        .filter(|position| generated[*position] != values[*position])
        .collect::<Vec<_>>();
    let next_value = generated[values.len()].clone();
    let held_out = values.len() - order * 2;
    let description_nodes = order + expression_nodes(&body);
    let primitive_steps = operation_steps(&body);
    let body_source = expression_source(&body);
    let source_bytes = body_source.len()
        + values[..order]
            .iter()
            .map(|value| state_source(value).len())
            .sum::<usize>();
    Candidate {
        order,
        coefficients,
        body,
        description_nodes,
        primitive_steps,
        source_bytes,
        body_source,
        error_offsets,
        next_value,
        held_out,
    }
}

fn recurrence_rows(
    values: &[NativeState],
    order: usize,
    training_end: usize,
) -> Vec<Vec<NativeScalar>> {
    let mut rows = Vec::new();
    for position in order..training_end {
        let mut coordinates = BTreeSet::new();
        for value in &values[position - order..=position] {
            coordinates.extend(value.0.keys().cloned());
        }
        for coordinate in coordinates {
            let mut row = (1..=order)
                .map(|lag| scalar_at(&values[position - lag], &coordinate))
                .collect::<Vec<_>>();
            row.push(scalar_at(&values[position], &coordinate));
            rows.push(row);
        }
    }
    rows
}

fn scalar_at(state: &NativeState, index: &MultiIndex) -> NativeScalar {
    state
        .0
        .get(index)
        .cloned()
        .unwrap_or_else(NativeScalar::zero)
}

fn solve_recurrence(values: &[NativeState], order: usize) -> Option<Vec<NativeScalar>> {
    let training_end = order.checked_mul(2)?;
    let solution = solve_recurrence_system(values, order)?;
    training_matches(values, &solution, training_end).then_some(solution)
}

fn solve_recurrence_system(values: &[NativeState], order: usize) -> Option<Vec<NativeScalar>> {
    let training_end = order.checked_mul(2)?;
    if training_end >= values.len() {
        return None;
    }
    let mut rows = recurrence_rows(values, order, training_end);
    let mut pivot_row = 0;
    let mut pivots = Vec::new();
    for column in 0..order {
        let pivot = (pivot_row..rows.len()).find(|&row| !rows[row][column].is_zero());
        let Some(pivot) = pivot else {
            continue;
        };
        rows.swap(pivot_row, pivot);
        let divisor = rows[pivot_row][column].clone();
        for entry in column..=order {
            rows[pivot_row][entry] = divide(&rows[pivot_row][entry], &divisor)?;
        }
        for row in 0..rows.len() {
            if row == pivot_row || rows[row][column].is_zero() {
                continue;
            }
            let factor = rows[row][column].clone();
            for entry in column..=order {
                let product = factor.multiply(&rows[pivot_row][entry]);
                rows[row][entry] = rows[row][entry].add(&product.negate());
            }
        }
        pivots.push((pivot_row, column));
        pivot_row += 1;
        if pivot_row == rows.len() {
            break;
        }
    }
    if rows
        .iter()
        .any(|row| row[..order].iter().all(NativeScalar::is_zero) && !row[order].is_zero())
    {
        return None;
    }
    let mut solution = vec![NativeScalar::zero(); order];
    for (row, column) in pivots {
        solution[column] = rows[row][order].clone();
    }
    Some(solution)
}

fn divide(numerator: &NativeScalar, denominator: &NativeScalar) -> Option<NativeScalar> {
    let norm = &denominator.real * &denominator.real + &denominator.imag * &denominator.imag;
    if norm.is_zero() {
        return None;
    }
    let conjugate = NativeScalar {
        real: denominator.real.clone(),
        imag: -&denominator.imag,
    };
    let product = numerator.multiply(&conjugate);
    Some(NativeScalar {
        real: product.real / &norm,
        imag: product.imag / norm,
    })
}

fn training_matches(
    values: &[NativeState],
    coefficients: &[NativeScalar],
    training_end: usize,
) -> bool {
    let order = coefficients.len();
    (order..training_end).all(|position| {
        let window = &values[position - order..position];
        apply_recurrence(coefficients, window) == values[position]
    })
}

fn generate_values(
    seeds: &[NativeState],
    coefficients: &[NativeScalar],
    length: usize,
) -> Vec<NativeState> {
    let mut generated = seeds.to_vec();
    while generated.len() < length {
        let start = generated.len() - coefficients.len();
        let next = apply_recurrence(coefficients, &generated[start..]);
        generated.push(next);
    }
    generated
}

fn apply_recurrence(
    coefficients: &[NativeScalar],
    oldest_to_newest: &[NativeState],
) -> NativeState {
    coefficients
        .iter()
        .zip(oldest_to_newest.iter().rev())
        .fold(NativeState::zero(), |sum, (coefficient, value)| {
            sum.add(&scale_state(value, coefficient))
        })
}

fn scale_state(value: &NativeState, coefficient: &NativeScalar) -> NativeState {
    NativeState::scalar(coefficient.clone()).multiply(value)
}

fn recurrence_expression(coefficients: &[NativeScalar]) -> Expr {
    let terms = coefficients
        .iter()
        .enumerate()
        .filter_map(|(offset, coefficient)| {
            if coefficient.is_zero() {
                return None;
            }
            let reference = Expr::Reference {
                name: previous_name(offset + 1),
                span: None,
            };
            if *coefficient == NativeScalar::one() {
                Some(reference)
            } else {
                Some(Expr::Multiply {
                    operands: vec![scalar_expression(coefficient), reference],
                    span: None,
                })
            }
        })
        .collect::<Vec<_>>();
    match terms.len() {
        0 => Expr::Zero { span: None },
        1 => terms.into_iter().next().expect("one term must exist"),
        _ => Expr::Add {
            operands: terms,
            span: None,
        },
    }
}

fn continuation_functions(
    first_index: u64,
    seeds: &[NativeState],
    coefficients: &[NativeScalar],
    next_body: Expr,
) -> Vec<Function> {
    let order = coefficients.len();
    let previous = (1..=order).map(previous_name).collect::<Vec<_>>();
    let next_call = Expr::Call {
        function: NEXT_FUNCTION.into(),
        arguments: previous
            .iter()
            .map(|name| Expr::Reference {
                name: name.clone(),
                span: None,
            })
            .collect(),
        span: None,
    };
    let mut recursive_arguments = vec![Expr::Add {
        operands: vec![
            Expr::Reference {
                name: POSITION_PARAMETER.into(),
                span: None,
            },
            Expr::One { span: None },
        ],
        span: None,
    }];
    recursive_arguments.push(next_call);
    recursive_arguments.extend(previous.iter().take(order.saturating_sub(1)).map(|name| {
        Expr::Reference {
            name: name.clone(),
            span: None,
        }
    }));
    let continuation = Function {
        name: CONTINUATION_FUNCTION.into(),
        parameters: std::iter::once(POSITION_PARAMETER.into())
            .chain(previous.iter().cloned())
            .collect(),
        body: Expr::Call {
            function: CONTINUATION_FUNCTION.into(),
            arguments: recursive_arguments,
            span: None,
        },
        span: None,
    };
    let first_generated =
        first_index + u64::try_from(order).expect("the bounded recurrence order fits u64");
    let mut start_arguments = vec![scalar_integer(first_generated)];
    start_arguments.extend(seeds.iter().rev().map(state_expression));
    vec![
        Function {
            name: NEXT_FUNCTION.into(),
            parameters: previous,
            body: next_body,
            span: None,
        },
        continuation,
        Function {
            name: START_FUNCTION.into(),
            parameters: Vec::new(),
            body: Expr::Call {
                function: CONTINUATION_FUNCTION.into(),
                arguments: start_arguments,
                span: None,
            },
            span: None,
        },
    ]
}

fn continuation_source(
    first_index: u64,
    observation_count: usize,
    seeds: &[NativeState],
    candidate: &Candidate,
    maximum_error_ratio: &str,
    error_indexes: &[u64],
) -> String {
    let (last_index, next_index) = continuation_indexes(
        first_index,
        observation_count,
        "generated-untrace-source",
        None,
    )
    .expect("validated continuation indexes remain representable");
    let previous = (1..=candidate.order).map(previous_name).collect::<Vec<_>>();
    let parameters = previous.join(", ");
    let next_arguments = parameters.clone();
    let shifted = previous
        .iter()
        .take(candidate.order.saturating_sub(1))
        .cloned()
        .collect::<Vec<_>>();
    let mut recursive_arguments = vec![
        "add(position, one)".to_owned(),
        format!("{NEXT_FUNCTION}({next_arguments})"),
    ];
    recursive_arguments.extend(shifted);
    let first_generated = first_index
        + u64::try_from(candidate.order).expect("the bounded recurrence order fits u64");
    let mut start_arguments = vec![first_generated.to_string()];
    start_arguments.extend(seeds.iter().rev().map(state_source));
    let held_out = observation_count.saturating_sub(candidate.order * 2);
    let matched = held_out.saturating_sub(error_indexes.len());
    let actual_error_ratio = rational_source(&candidate_error_ratio(candidate));
    let errors = if error_indexes.is_empty() {
        "none".to_owned()
    } else {
        error_indexes
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!(
        "# Generated by untrace from exact ordered observations {first_index} through {last_index}.\n\
# Grammar: homogeneous constant-coefficient linear recurrence.\n\
# Order: {}. Held-out matches: {matched} of {held_out}.\n\
# Maximum held-out index-error ratio: {maximum_error_ratio}. Actual: {actual_error_ratio}.\n\
# Mismatching indexes: {errors}.\n\
# First unseen prediction at index {next_index}: {}.\n\n\
let {NEXT_FUNCTION} = ({parameters}) =>\n{}\n\n\
let {CONTINUATION_FUNCTION} = (position, {parameters}) =>\n{CONTINUATION_FUNCTION}({})\n\n\
let {START_FUNCTION} = () =>\n{CONTINUATION_FUNCTION}({})\n\n\
output trace({START_FUNCTION}) as pattern\n",
        candidate.order,
        state_source(&candidate.next_value),
        candidate.body_source,
        recursive_arguments.join(", "),
        start_arguments.join(", "),
    )
}

fn previous_name(position: usize) -> String {
    format!("previous_{position}")
}

fn validate_source_states(
    values: &[NativeState],
    source_name: &str,
    span: Option<Span>,
) -> Result<(), LanguageError> {
    let source_steps = values
        .iter()
        .flat_map(|value| value.0.keys())
        .flat_map(|index| index.0.values())
        .try_fold(0_usize, |total, depth| total.checked_add(depth.to_usize()?));
    if source_steps.is_none_or(|steps| steps > MAX_SOURCE_INDEX_STEPS) {
        return Err(error(
            "NSU001",
            format!(
                "untrace generated source supports at most {MAX_SOURCE_INDEX_STEPS} total seed INDEX steps"
            ),
            source_name,
            span,
        ));
    }
    Ok(())
}

fn state_expression(value: &NativeState) -> Expr {
    let terms = value
        .0
        .iter()
        .map(|(index, coefficient)| {
            index.0.iter().fold(
                scalar_expression(coefficient),
                |expression, (&direction, depth)| Expr::Index {
                    direction,
                    multiplicity: depth
                        .to_u64()
                        .expect("validated continuation INDEX depth fits u64"),
                    value: Box::new(expression),
                    span: None,
                },
            )
        })
        .collect::<Vec<_>>();
    match terms.len() {
        0 => Expr::Zero { span: None },
        1 => terms.into_iter().next().expect("one state term must exist"),
        _ => Expr::Add {
            operands: terms,
            span: None,
        },
    }
}

fn state_source(value: &NativeState) -> String {
    expression_source(&state_expression(value))
}

fn scalar_expression(value: &NativeScalar) -> Expr {
    if value.is_zero() {
        Expr::Zero { span: None }
    } else if *value == NativeScalar::one() {
        Expr::One { span: None }
    } else {
        Expr::Scalar {
            real: rational_source(&value.real),
            imag: rational_source(&value.imag),
            span: None,
        }
    }
}

fn scalar_integer(value: u64) -> Expr {
    Expr::Scalar {
        real: value.to_string(),
        imag: "0".into(),
        span: None,
    }
}

fn rational_source(value: &BigRational) -> String {
    if value.denom().is_one() {
        value.numer().to_string()
    } else {
        format!("{}/{}", value.numer(), value.denom())
    }
}

fn expression_source(expression: &Expr) -> String {
    match expression {
        Expr::Zero { .. } => "zero".into(),
        Expr::One { .. } => "one".into(),
        Expr::Scalar { real, imag, .. } if imag == "0" => real.clone(),
        Expr::Scalar { real, imag, .. } => format!("scalar({real}, {imag})"),
        Expr::Reference { name, .. } => name.clone(),
        Expr::Add { operands, .. } => format!(
            "add({})",
            operands
                .iter()
                .map(expression_source)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::Multiply { operands, .. } => format!(
            "multiply({})",
            operands
                .iter()
                .map(expression_source)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::Call {
            function,
            arguments,
            ..
        } => format!(
            "{function}({})",
            arguments
                .iter()
                .map(expression_source)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::Orient { turns, value, .. } => {
            format!("orient({turns}, {})", expression_source(value))
        }
        Expr::Index {
            direction,
            multiplicity,
            value,
            ..
        } => {
            let multiplicity = usize::try_from(*multiplicity)
                .expect("validated continuation INDEX multiplicity fits usize");
            format!(
                "{}{}{}",
                format!("index({direction}, ").repeat(multiplicity),
                expression_source(value),
                ")".repeat(multiplicity)
            )
        }
        Expr::Trace { function, .. } => format!("trace({function})"),
        Expr::Untrace { value, .. } => format!("untrace({})", expression_source(value)),
    }
}

fn expression_nodes(expression: &Expr) -> usize {
    match expression {
        Expr::Add { operands, .. } | Expr::Multiply { operands, .. } => {
            1 + operands.iter().map(expression_nodes).sum::<usize>()
        }
        Expr::Call { arguments, .. } => 1 + arguments.iter().map(expression_nodes).sum::<usize>(),
        Expr::Orient { value, .. } | Expr::Index { value, .. } | Expr::Untrace { value, .. } => {
            1 + expression_nodes(value)
        }
        _ => 1,
    }
}

fn operation_steps(expression: &Expr) -> usize {
    match expression {
        Expr::Add { operands, .. } | Expr::Multiply { operands, .. } => {
            1 + operands.iter().map(operation_steps).sum::<usize>()
        }
        Expr::Orient { value, .. } | Expr::Index { value, .. } => 1 + operation_steps(value),
        Expr::Call { arguments, .. } => arguments.iter().map(operation_steps).sum(),
        Expr::Untrace { value, .. } => operation_steps(value),
        _ => 0,
    }
}

fn error(
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
    use crate::core::{interpret, parse};

    fn indexed(values: &[i64]) -> NativeState {
        NativeState::from_terms(values.iter().enumerate().map(|(offset, value)| {
            (
                MultiIndex::from_depths([(
                    u64::try_from(offset + 1).expect("test index fits u64"),
                    1,
                )])
                .unwrap(),
                NativeScalar::from_text(&value.to_string(), "0").unwrap(),
            )
        }))
    }

    fn vector(values: &[i64]) -> NativeState {
        NativeState::from_terms(values.iter().enumerate().map(|(offset, value)| {
            (
                MultiIndex::from_depths([(
                    1,
                    u64::try_from(offset + 1).expect("test vector position fits u64"),
                )])
                .unwrap(),
                NativeScalar::from_text(&value.to_string(), "0").unwrap(),
            )
        }))
    }

    #[test]
    fn constant_sequence_has_one_step_recurrence() {
        let continuation = synthesize(&indexed(&[7, 7, 7, 7]), "0", "constant.ns", None).unwrap();

        assert_eq!(continuation.recurrence_order(), 1);
        assert_eq!(continuation.primitive_steps(), 0);
        assert_eq!(
            continuation.next_value(),
            &NativeState::scalar(NativeScalar::from_text("7", "0").unwrap())
        );
        assert!(continuation.source().contains("next_value(previous_1)"));
    }

    #[test]
    fn fibonacci_finds_the_two_value_continuation() {
        let continuation =
            synthesize(&indexed(&[1, 1, 2, 3, 5, 8, 13]), "0", "fibonacci.ns", None).unwrap();

        assert_eq!(continuation.recurrence_order(), 2);
        assert_eq!(continuation.validated_steps(), 3);
        assert_eq!(
            continuation.next_value(),
            &NativeState::scalar(NativeScalar::from_text("21", "0").unwrap())
        );
        assert!(
            continuation
                .source()
                .contains("add(previous_1, previous_2)")
        );

        let program = parse(continuation.source(), "generated.ns").unwrap();
        assert!(crate::strand::is_operation_strand(
            &interpret(&program).unwrap()
        ));
        assert_eq!(
            continuation.pattern_csv().unwrap(),
            "part,index,real,imag\nseed,1,1,0\nseed,2,1,0\ncoefficient,1,1,0\ncoefficient,2,1,0\nprediction,8,21,0\n"
        );
    }

    #[test]
    fn continuation_sequence_executes_the_synthesized_recurrence() {
        let continuation =
            synthesize(&indexed(&[1, 1, 2, 3, 5, 8, 13]), "0", "fibonacci.ns", None).unwrap();

        let values = continuation
            .sequence()
            .take(9)
            .map(|state| scalar_state(&state).unwrap().real.to_integer())
            .collect::<Vec<_>>();

        assert_eq!(
            values,
            [1, 1, 2, 3, 5, 8, 13, 21, 34]
                .into_iter()
                .map(BigInt::from)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn error_ratio_selects_the_most_repeatable_continuation() {
        let observations = indexed(&[1, 1, 2, 3, 5, 8, 13, 21, 35]);

        let exact = synthesize(&observations, "1/6", "noisy-fibonacci.ns", None).unwrap_err();
        let continuation = synthesize(&observations, "1/5", "noisy-fibonacci.ns", None).unwrap();

        assert_eq!(exact.0.code, "NSU004");
        assert_eq!(continuation.recurrence_order(), 2);
        assert_eq!(continuation.maximum_error_ratio(), "1/5");
        assert_eq!(continuation.error_indexes(), &[9]);
        assert_eq!(continuation.validated_steps(), 4);
        assert_eq!(
            continuation.next_value(),
            &NativeState::scalar(NativeScalar::from_text("55", "0").unwrap())
        );
        assert!(continuation.source().contains("Mismatching indexes: 9"));
    }

    #[test]
    fn thirty_prime_observations_do_not_fake_a_held_out_continuation() {
        let primes = [
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97, 101, 103, 107, 109, 113,
        ];
        let state = NativeState::from_terms(primes.iter().enumerate().map(|(offset, prime)| {
            let birth = u64::try_from(offset + 1).expect("test birth index fits u64");
            (
                MultiIndex::from_depths([(birth, 1)]).unwrap(),
                NativeScalar::from_text(&prime.to_string(), "0")
                    .unwrap()
                    .orient(i64::try_from(birth).expect("test birth index fits i64")),
            )
        }));
        let error = synthesize(&state, "0", "prime30.ns", None).unwrap_err();

        assert_eq!(error.0.code, "NSU004");
    }

    #[test]
    fn complete_states_share_one_exact_recurrence_without_flattening() {
        let values = [
            vector(&[1, 2]),
            vector(&[1, 3]),
            vector(&[2, 5]),
            vector(&[3, 8]),
            vector(&[5, 13]),
            vector(&[8, 21]),
            vector(&[13, 34]),
        ];

        let continuation = synthesize_states(&values, "0", "vectors.json").unwrap();

        assert_eq!(continuation.recurrence_order(), 2);
        assert_eq!(continuation.validated_steps(), 3);
        assert_eq!(continuation.next_value(), &vector(&[21, 55]));
        assert!(
            continuation
                .source()
                .contains("add(previous_1, previous_2)")
        );
        assert!(continuation.source().contains("index(1, index(1, 2))"));
        let program = parse(continuation.source(), "vector-model.ns").unwrap();
        assert!(crate::strand::is_operation_strand(
            &interpret(&program).unwrap()
        ));
        assert_eq!(continuation.pattern_csv().unwrap_err().0.code, "NSU009");
    }

    #[test]
    fn complete_input_sequence_is_retained_beyond_sixty_four_observations() {
        let mut values = vec![vector(&[2, 3]); 100];

        let continuation = synthesize_states(&values, "0", "long-vectors.json").unwrap();

        assert_eq!(continuation.observation_count(), 100);
        assert_eq!(continuation.recurrence_order(), 1);
        assert_eq!(continuation.validated_steps(), 98);
        assert_eq!(continuation.next_value(), &vector(&[2, 3]));

        values[99] = vector(&[2, 4]);
        let error = synthesize_states(&values, "0", "long-vectors.json").unwrap_err();
        assert_eq!(error.0.code, "NSU004");
    }

    #[test]
    fn compact_symbols_use_the_same_exact_recurrence_grammar() {
        let symbols = [1_u16, 1, 2, 3, 5, 8, 13];

        let continuation = synthesize_symbols(&symbols.into_iter(), "0", "symbols").unwrap();

        assert_eq!(continuation.observation_count(), 7);
        assert_eq!(continuation.recurrence_order(), 2);
        assert_eq!(continuation.validated_steps(), 3);
        assert_eq!(
            continuation.next_value(),
            &NativeState::scalar(NativeScalar::from_text("21", "0").unwrap())
        );
    }

    #[test]
    fn compact_symbols_reject_an_unsupported_exact_text_pattern() {
        let symbols = "Native Space tests complete text rather than a shortened training preview."
            .bytes()
            .map(u16::from)
            .collect::<Vec<_>>();

        let error = synthesize_symbols(&symbols.into_iter(), "0", "text").unwrap_err();

        assert_eq!(error.0.code, "NSU004");
    }

    #[test]
    fn compact_symbols_do_not_hide_mismatches_in_large_error_lists() {
        let error = synthesize_symbols(&[1_u16, 2, 3].into_iter(), "1/2", "symbols").unwrap_err();

        assert_eq!(error.0.code, "NSU011");
    }

    #[test]
    fn array_style_sequence_axis_preserves_payload_coordinates() {
        let values = [
            vector(&[1, 2]),
            vector(&[1, 3]),
            vector(&[2, 5]),
            vector(&[3, 8]),
            vector(&[5, 13]),
            vector(&[8, 21]),
            vector(&[13, 34]),
        ];
        let state =
            NativeState::from_terms(values.iter().enumerate().flat_map(|(time_offset, value)| {
                value.0.iter().map(move |(payload, coefficient)| {
                    let time = MultiIndex::from_depths([(
                        1,
                        u64::try_from(time_offset + 1).expect("test time fits u64"),
                    )])
                    .unwrap();
                    (
                        time.compose(&MultiIndex(
                            payload
                                .0
                                .iter()
                                .map(|(&direction, depth)| (direction + 1, depth.clone()))
                                .collect(),
                        )),
                        coefficient.clone(),
                    )
                })
            }));

        let continuation = synthesize(&state, "0", "array-style.ns", None).unwrap();
        let expected =
            NativeState::from_terms(vector(&[21, 55]).0.into_iter().map(|(index, coefficient)| {
                (
                    MultiIndex(
                        index
                            .0
                            .into_iter()
                            .map(|(direction, depth)| (direction + 1, depth))
                            .collect(),
                    ),
                    coefficient,
                )
            }));

        assert_eq!(continuation.next_value(), &expected);
    }

    #[test]
    fn array_style_input_without_sequence_axis_is_rejected() {
        let state = NativeState::from_terms([(
            MultiIndex::from_depths([(2, 1), (3, 1)]).unwrap(),
            NativeScalar::one(),
        )]);
        let error = synthesize(&state, "0", "invalid.ns", None).unwrap_err();

        assert_eq!(error.0.code, "NSU001");
    }
}
