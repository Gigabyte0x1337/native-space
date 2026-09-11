// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pattern fields reuse the retained-state and function codecs.
//!
//! These are data-schema addresses, never operation codes. Only fields at this
//! record level are interpreted. Payload directions become record coefficients,
//! so even these exact addresses inside a seed cannot cancel outer metadata.

use super::{Pattern, error};
use crate::{
    core::{LanguageError, MultiIndex, NativeScalar, NativeState},
    retained::State,
    strand::execution::{FunctionValue, Value, state_data},
    value_reflection::Rule,
};
use num_bigint::BigUint;
use num_traits::{One, Zero};

pub(super) const KIND: u64 = 100;
pub(super) const SEED: u64 = 101;
pub(super) const STEP: u64 = 102;
pub(super) const INDEX: u64 = 103;
pub(super) const PATTERN: u64 = 104;
const OBSERVATION_KIND: u64 = 105;
// Within the index field: one explicit presence flag and an exact scalar k.
const PRESENT: u64 = 1;
const VALUE: u64 = 6;

fn integer(n: BigUint) -> NativeState {
    NativeState::scalar(NativeScalar {
        real: num_rational::BigRational::from_integer(n.into()),
        imag: num_rational::BigRational::zero(),
    })
}

fn field(direction: u64, value: &NativeState) -> NativeState {
    value
        .index_power(direction, 1)
        .expect("positive schema direction")
}

pub(super) fn select(state: &NativeState, direction: u64) -> NativeState {
    Rule::select(direction).apply(state)
}

fn record(fields: &[(u64, NativeState)]) -> State {
    State::from_projection(&NativeState::from_terms(
        fields
            .iter()
            .flat_map(|(direction, value)| field(*direction, value).0),
    ))
}

pub(super) fn pattern(seed: &State, step: &FunctionValue) -> Result<State, LanguageError> {
    Ok(record(&[
        (KIND, integer(1_u32.into())),
        (SEED, state_data::encode(seed)?.project().clone()),
        (
            STEP,
            Value::Function(step.clone()).native()?.project().clone(),
        ),
    ]))
}

pub(super) fn observation(pattern: &Pattern, index: &BigUint) -> State {
    // Share the existing Native generator node. Do not flatten/copy its records
    // on every selection or retain the previous observation's index wrapper.
    let metadata = record(&[
        (OBSERVATION_KIND, integer(2_u32.into())),
        (
            INDEX,
            field(PRESENT, &NativeState::one()).add(&field(VALUE, &integer(index.clone()))),
        ),
    ]);
    pattern
        .native()
        .index_power(PATTERN, 1)
        .expect("positive schema")
        .add(&metadata)
}

fn validate_fields(
    state: &NativeState,
    kind: u32,
    directions: &[u64],
) -> Result<(), LanguageError> {
    let marker = MultiIndex::from_depths([(directions[0], 1)]).expect("positive schema");
    let expected = integer(kind.into());
    if state.0.get(&marker) != expected.0.values().next() {
        return Err(error("invalid Native Pattern record kind"));
    }
    // Exact reconstruction rejects unknown fields, overlapping tags, and depths >1.
    let fields = directions
        .iter()
        .map(|d| (*d, select(state, *d)))
        .collect::<Vec<_>>();
    let rebuilt = record(&fields);
    if rebuilt.project() != state
        || state.0.keys().any(|indices| {
            let tags = directions
                .iter()
                .filter(|d| indices.0.contains_key(d))
                .collect::<Vec<_>>();
            tags.len() != 1 || indices.0.get(tags[0]) != Some(&BigUint::one())
        })
    {
        return Err(error("invalid or overlapping Native Pattern fields"));
    }
    Ok(())
}

pub(super) fn decode_pattern(native: &State) -> Result<(State, FunctionValue), LanguageError> {
    let state = native.project();
    validate_fields(state, 1, &[KIND, SEED, STEP])?;
    let seed = select(state, SEED);
    let step = select(state, STEP);
    let decoded_seed = state_data::decode(&seed)?;
    let decoded_step = FunctionValue::from_native(&State::from_projection(&step))?;
    Ok((decoded_seed, decoded_step))
}

pub(super) fn decode_observation(native: &State) -> Result<(Pattern, BigUint), LanguageError> {
    let state = native.project();
    validate_fields(state, 2, &[OBSERVATION_KIND, PATTERN, INDEX])?;
    let pattern = Pattern::from_native(&State::from_projection(&select(state, PATTERN)))?;
    let index = select(state, INDEX);
    let presence = MultiIndex::from_depths([(PRESENT, 1)]).expect("positive schema");
    let value = MultiIndex::from_depths([(VALUE, 1)]).expect("positive schema");
    if index.0.get(&presence) != Some(&NativeScalar::one())
        || index.0.keys().any(|key| key != &presence && key != &value)
    {
        return Err(error("invalid observation INDEX presence or fields"));
    }
    let coefficient = index
        .0
        .get(&value)
        .cloned()
        .unwrap_or_else(NativeScalar::zero);
    if !coefficient.imag.is_zero() || !coefficient.real.is_integer() {
        return Err(error("observation INDEX must be an exact natural number"));
    }
    let k = coefficient
        .real
        .to_integer()
        .to_biguint()
        .ok_or_else(|| error("observation INDEX must be nonnegative"))?;
    Ok((pattern, k))
}
