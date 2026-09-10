// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Shared indexed routing for exact coordinates and explicitly rounded execution.
//!
//! Zero coefficients stay in this view so INDEX labels and boundary rays can
//! survive cancellation. Classical projection separately combines/removes zeros.

use std::collections::BTreeMap;

use super::{Operation, Scalar, ScalarData, State};
use crate::core::{MultiIndex, NativeScalar};

pub(crate) type Field<C> = BTreeMap<MultiIndex, C>;

pub(crate) trait Coefficient: Clone {
    fn is_zero(&self) -> bool;
    fn scalar(data: &ScalarData) -> Result<Self, String>;
    fn add(&self, right: &Self) -> Result<Self, String>;
    fn multiply(&self, right: &Self) -> Result<Self, String>;
    fn phase(&self, turns: i64) -> Result<Self, String>;
}

impl Coefficient for Scalar {
    fn is_zero(&self) -> bool {
        self.project().is_zero()
    }
    fn scalar(data: &ScalarData) -> Result<Self, String> {
        Self::from_data(data)
    }
    fn add(&self, right: &Self) -> Result<Self, String> {
        Ok(self.add(right))
    }
    fn multiply(&self, right: &Self) -> Result<Self, String> {
        Ok(self.multiply(right))
    }
    fn phase(&self, turns: i64) -> Result<Self, String> {
        Ok(self.multiply(&Self::from_classical(&NativeScalar::one().phase(turns))))
    }
}

pub(crate) fn accumulate<C: Coefficient>(
    field: &mut Field<C>,
    index: MultiIndex,
    value: C,
) -> Result<(), String> {
    if let Some(previous) = field.get_mut(&index) {
        *previous = previous.add(&value)?;
    } else {
        field.insert(index, value);
    }
    Ok(())
}

pub(super) fn evaluate<C: Coefficient>(
    state: &State,
    all_branches: bool,
) -> Result<Vec<Field<C>>, String> {
    let plan = state.plan();
    let mut needed = vec![all_branches; plan.len()];
    let mut pending = vec![plan.len() - 1];
    while let Some(address) = pending.pop() {
        needed[address] = true;
        pending.extend(
            plan[address]
                .inputs
                .iter()
                .copied()
                .filter(|index| !needed[*index]),
        );
    }
    let mut fields = Vec::new();
    for (address, step) in plan.iter().enumerate() {
        if !needed[address] {
            fields.push(Field::new());
            continue;
        }
        let result = evaluate_step(&step.operator, &step.inputs, &fields).map_err(|message| {
            let location = step.span.map_or_else(String::new, |span| {
                format!(" at line {}, column {}", span.start_line, span.start_column)
            });
            format!("{message} (operation {}{location})", address + 1)
        })?;
        fields.push(result);
    }
    Ok(fields)
}

pub(crate) fn evaluate_step<C: Coefficient>(
    operation: &Operation,
    inputs: &[usize],
    fields: &[Field<C>],
) -> Result<Field<C>, String> {
    let mut result = Field::new();
    match operation {
        Operation::Reflect { rule } => {
            return rule.apply_field(&fields[inputs[0]]);
        }
        Operation::Scalar { coordinates } => {
            result.insert(MultiIndex::default(), C::scalar(coordinates)?);
        }
        Operation::Add => {
            result = fields[inputs[0]].clone();
            for (index, value) in &fields[inputs[1]] {
                accumulate(&mut result, index.clone(), value.clone())?;
            }
        }
        Operation::Multiply => {
            for (left_index, left) in &fields[inputs[0]] {
                for (right_index, right) in &fields[inputs[1]] {
                    accumulate(
                        &mut result,
                        left_index.compose(right_index),
                        left.multiply(right)?,
                    )?;
                }
            }
        }
        Operation::Phase { turns } => {
            for (index, value) in &fields[inputs[0]] {
                result.insert(index.clone(), value.phase(*turns)?);
            }
        }
        Operation::Index { direction, depth } => {
            for (index, value) in &fields[inputs[0]] {
                result.insert(index.shift_by(*direction, *depth)?, value.clone());
            }
        }
    }
    Ok(result)
}
