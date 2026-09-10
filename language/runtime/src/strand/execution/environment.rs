// SPDX-License-Identifier: AGPL-3.0-or-later

//! Collision-free Native binding records with explicit presence.
//!
//! Argument indices are stored as record data rather than overlaid on metadata.
//! This prevents an arbitrary indexed argument from cancelling a presence flag
//! or merging with another parameter. Cached values retain their original graphs;
//! all slot selection uses the same canonical REFLECT rule as source programs.

use super::{Value, diagnostic};
use crate::{
    core::{LanguageError, MultiIndex, NativeScalar, NativeState},
    retained::State,
    value_reflection::Rule,
};
use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};
use std::collections::BTreeMap;

// Low field directions are distinct from both the strand namespace and slots.
pub(super) const SLOT_START: u64 = 1024;
const PRESENT: u64 = 1;
const TERM: u64 = 2;
const FIELD: u64 = 3;
const POSITION: u64 = 4;
const PAYLOAD_KIND: u64 = 6;
const COEFFICIENT: u64 = 1;
const DIRECTION: u64 = 2;
const DEPTH: u64 = 3;

#[derive(Clone, Debug)]
pub(super) struct Environment {
    native: State,
    selection: NativeState,
    values: Vec<Value>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            native: State::zero(),
            selection: NativeState::zero(),
            values: Vec::new(),
        }
    }
}

impl Environment {
    pub(super) fn len(&self) -> usize {
        self.values.len()
    }
    pub(super) fn native(&self) -> &State {
        &self.native
    }

    pub(super) fn append(&self, values: Vec<Value>) -> Result<Self, LanguageError> {
        let mut result = self.clone();
        for value in values {
            // Slots contain Native references to shared inputs. Binding must not
            // project an argument; zero products and deferred reads stay lazy.
            let encoded = NativeState::one()
                .index_power(PRESENT, 1)
                .expect("positive schema")
                .add(
                    &NativeState::scalar(integer(BigUint::from(result.values.len() + 1)))
                        .index_power(FIELD, 1)
                        .expect("positive schema"),
                );
            let direction = SLOT_START
                + u64::try_from(result.values.len())
                    .map_err(|_error| diagnostic("too many bindings", "<bindings>", None))?;
            let slot = encoded
                .index_power(direction, 1)
                .map_err(|message| diagnostic(message, "<bindings>", None))?;
            result.selection = result.selection.add(&slot);
            result.native = result
                .native
                .add(&State::from_projection(&slot).retaining(&value.dependencies()));
            result.values.push(value);
        }
        Ok(result)
    }

    pub(super) fn portable(&self, depth: usize) -> Result<State, LanguageError> {
        let mut result = State::zero();
        for (position, value) in self.values.iter().enumerate() {
            // A function already is program-record data. Encoding the arithmetic
            // used to store those records would needlessly wrap a graph in a graph.
            let (kind, graph, dependencies) = match value {
                Value::Function(function) => (2_u8, function.native(depth)?, value.dependencies()),
                Value::State(state) => {
                    (1_u8, super::state_data::encode(state)?, vec![state.clone()])
                }
                Value::Pack(_) => {
                    return Err(diagnostic(
                        "argument pack must be spread",
                        "<bindings>",
                        None,
                    ));
                }
            };
            let payload = encode(graph.project()).add(
                &NativeState::scalar(integer(kind.into()))
                    .index_power(PAYLOAD_KIND, 1)
                    .expect("positive schema"),
            );
            let encoded = State::from_projection(&payload);
            let slot = encoded
                .index_power(SLOT_START + position as u64, 1)
                .expect("positive schema");
            result = result.add(&slot.retaining(&dependencies));
        }
        Ok(result)
    }

    pub(super) fn get(&self, position: usize) -> Result<Value, LanguageError> {
        let direction = SLOT_START
            + u64::try_from(position)
                .map_err(|_error| diagnostic("invalid binding position", "<bindings>", None))?;
        let selected = Rule::select(direction).apply(&self.selection);
        let marker = MultiIndex::from_depths([(PRESENT, 1)]).expect("positive schema direction");
        if selected.0.get(&marker) != Some(&NativeScalar::one()) {
            return Err(diagnostic("unbound parameter", "<bindings>", None));
        }
        self.values
            .get(position)
            .cloned()
            .ok_or_else(|| diagnostic("unbound parameter", "<bindings>", None))
    }

    pub(super) fn load(native: &State, depth: usize) -> Result<Self, LanguageError> {
        let mut slots = BTreeMap::<u64, Vec<_>>::new();
        for (index, coefficient) in &native.project().0 {
            let mut fields = index.0.clone();
            let slot = fields
                .keys()
                .copied()
                .filter(|direction| *direction >= SLOT_START)
                .collect::<Vec<_>>();
            if slot.len() != 1 || fields.remove(&slot[0]) != Some(BigUint::one()) {
                return Err(diagnostic("invalid binding slot", "<bindings>", None));
            }
            slots
                .entry(slot[0])
                .or_default()
                .push((MultiIndex(fields), coefficient.clone()));
        }
        let mut values = Vec::new();
        for (expected, (slot, terms)) in slots.into_iter().enumerate() {
            if slot != SLOT_START + expected as u64 {
                return Err(diagnostic(
                    "binding slots must form a contiguous prefix",
                    "<bindings>",
                    None,
                ));
            }
            let mut payload = NativeState::from_terms(terms);
            let marker = MultiIndex::from_depths([(PAYLOAD_KIND, 1)]).expect("positive schema");
            let kind = payload
                .0
                .remove(&marker)
                .ok_or_else(|| diagnostic("missing binding payload kind", "<bindings>", None))?;
            let value = decode(&payload)?;
            match natural(&kind)?.to_u8() {
                Some(1) => values.push(Value::State(super::state_data::decode(&value)?)),
                Some(2) => values.push(Value::Function(super::FunctionValue::load_nested(
                    &State::from_projection(&value),
                    "<bindings>",
                    depth,
                )?)),
                _ => {
                    return Err(diagnostic(
                        "invalid binding payload kind",
                        "<bindings>",
                        None,
                    ));
                }
            }
        }
        Self::default().append(values)
    }
}

fn integer(value: BigUint) -> NativeScalar {
    NativeScalar {
        real: num_rational::BigRational::from_integer(value.into()),
        imag: num_rational::BigRational::zero(),
    }
}

fn encode(value: &NativeState) -> NativeState {
    let mut terms = vec![(
        MultiIndex::from_depths([(PRESENT, 1)]).expect("positive schema"),
        NativeScalar::one(),
    )];
    for (term, (index, coefficient)) in value.0.iter().enumerate() {
        let term = term as u64 + 1;
        terms.push((
            MultiIndex::from_depths([(TERM, term), (FIELD, COEFFICIENT)]).expect("positive schema"),
            coefficient.clone(),
        ));
        for (position, (direction, depth)) in index.0.iter().enumerate() {
            let position = position as u64 + 1;
            terms.push((
                MultiIndex::from_depths([(TERM, term), (FIELD, DIRECTION), (POSITION, position)])
                    .expect("positive schema"),
                integer(BigUint::from(*direction)),
            ));
            terms.push((
                MultiIndex::from_depths([(TERM, term), (FIELD, DEPTH), (POSITION, position)])
                    .expect("positive schema"),
                integer(depth.clone()),
            ));
        }
    }
    NativeState::from_terms(terms)
}

#[derive(Default)]
struct Term {
    coefficient: Option<NativeScalar>,
    directions: BTreeMap<u64, u64>,
    depths: BTreeMap<u64, BigUint>,
}

fn natural(value: &NativeScalar) -> Result<BigUint, LanguageError> {
    if !value.imag.is_zero() || !value.real.is_integer() {
        return Err(diagnostic(
            "binding metadata must be an exact natural number",
            "<bindings>",
            None,
        ));
    }
    value
        .real
        .to_integer()
        .to_biguint()
        .ok_or_else(|| diagnostic("negative binding metadata", "<bindings>", None))
}

fn decode(value: &NativeState) -> Result<NativeState, LanguageError> {
    let invalid = || diagnostic("invalid Native binding record", "<bindings>", None);
    let marker = MultiIndex::from_depths([(PRESENT, 1)]).expect("positive schema");
    if value.0.get(&marker) != Some(&NativeScalar::one()) {
        return Err(invalid());
    }
    let mut terms = BTreeMap::<u64, Term>::new();
    for (index, coefficient) in &value.0 {
        if index == &marker {
            continue;
        }
        let mut fields = index.0.clone();
        let term = fields
            .remove(&TERM)
            .and_then(|value| value.to_u64())
            .ok_or_else(invalid)?;
        let field = fields
            .remove(&FIELD)
            .and_then(|value| value.to_u64())
            .ok_or_else(invalid)?;
        let position = fields.remove(&POSITION).and_then(|value| value.to_u64());
        if !fields.is_empty() || term == 0 {
            return Err(invalid());
        }
        let term = terms.entry(term).or_default();
        match (field, position) {
            (COEFFICIENT, None) => {
                term.coefficient = Some(coefficient.clone());
            }
            (DIRECTION, Some(position)) if position > 0 => {
                term.directions.insert(
                    position,
                    natural(coefficient)?
                        .to_u64()
                        .filter(|value| *value > 0)
                        .ok_or_else(invalid)?,
                );
            }
            (DEPTH, Some(position)) if position > 0 => {
                term.depths.insert(position, natural(coefficient)?);
            }
            _ => return Err(invalid()),
        }
    }
    let mut result = Vec::new();
    for (expected, (address, term)) in terms.into_iter().enumerate() {
        if address != expected as u64 + 1 || term.directions.len() != term.depths.len() {
            return Err(invalid());
        }
        let mut indices = BTreeMap::new();
        for (expected, (position, direction)) in term.directions.into_iter().enumerate() {
            if position != expected as u64 + 1 {
                return Err(invalid());
            }
            let depth = term.depths.get(&position).ok_or_else(invalid)?;
            if depth.is_zero() || indices.insert(direction, depth.clone()).is_some() {
                return Err(invalid());
            }
        }
        result.push((MultiIndex(indices), term.coefficient.ok_or_else(invalid)?));
    }
    Ok(NativeState::from_terms(result))
}
