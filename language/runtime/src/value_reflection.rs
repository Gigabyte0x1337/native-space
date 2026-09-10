// SPDX-License-Identifier: AGPL-3.0-or-later

//! Compiled matching and replacement on canonical indexed values.
//!
//! A pattern is a nonzero monomial times one capture. Each canonical term
//! supplies at most one binding. This deliberately avoids inventing solutions
//! to ambiguous equations such as add(a, b) or multiply(a, a).
//! Replacement instructions are compiled once, not generated from observations.
//! INDEX depth binders consume a complete direction and retain its `BigUint`.
//! They are template metadata, never an additional state operation. A binder
//! cannot overlap a fixed depth or another binder on the same direction: that
//! would require choosing an arbitrary partition of a canonical exponent.

use std::collections::BTreeMap;

use num_bigint::BigUint;
use num_traits::Zero;
use serde::{Deserialize, Serialize};

use crate::core::{Expr, MultiIndex, NativeScalar, NativeState};
use crate::retained::{Scalar, ScalarData};

// Bound template depth and work before recursive source traversal or decoding.
const MAX_DEPTH: usize = 128;
const MAX_INSTRUCTIONS: usize = 4096;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
enum Instruction {
    Capture,
    Depth {
        name: String,
    },
    IndexCapture {
        input: usize,
        direction: u64,
        name: String,
    },
    Scalar {
        value: ScalarData,
    },
    Add {
        left: usize,
        right: usize,
    },
    Multiply {
        left: usize,
        right: usize,
    },
    Phase {
        input: usize,
        turns: i64,
    },
    Index {
        input: usize,
        direction: u64,
        depth: u64,
    },
}

/// A finite, validated canonical-value matching rule.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    indices: Vec<(u64, BigUint)>,
    depths: Vec<(u64, String)>,
    inverse: ScalarData,
    instructions: Vec<Instruction>,
}

struct Bindings {
    remaining: MultiIndex,
    depths: BTreeMap<String, BigUint>,
}

impl Rule {
    /// Host coordinate routing uses the same full-depth rule as Native source.
    pub(crate) fn route(from: u64, to: u64) -> Self {
        assert!(from > 0, "source direction must be positive");
        let mut instructions = vec![Instruction::Capture];
        if to != 0 {
            instructions.push(Instruction::IndexCapture {
                input: 0,
                direction: to,
                name: "depth".into(),
            });
        }
        Self {
            indices: Vec::new(),
            depths: vec![(from, "depth".into())],
            inverse: Scalar::from_classical(&NativeScalar::one()).to_data(),
            instructions,
        }
    }

    /// Construct the same one-layer INDEX matcher used for environment slots.
    pub(crate) fn select(direction: u64) -> Self {
        assert!(direction > 0, "environment directions are positive");
        Self {
            indices: vec![(direction, BigUint::from(1_u8))],
            depths: Vec::new(),
            inverse: Scalar::from_classical(&NativeScalar::one()).to_data(),
            instructions: vec![Instruction::Capture],
        }
    }
    pub(crate) fn compile(pattern: &Expr, replacement: &Expr) -> Result<Self, String> {
        Self::compile_view(pattern, replacement)
    }

    pub(crate) fn compile_view<T: Template>(pattern: T, replacement: T) -> Result<Self, String> {
        let mut capture = None;
        let mut depths = BTreeMap::new();
        let (indices, factor) = pattern_factor(pattern, &mut capture, &mut depths, 0)?;
        let capture = capture.ok_or("reflect pattern must contain one capture")?;
        if depths.values().any(|name| name == &capture) {
            return Err("reflect value and depth captures must have different names".into());
        }
        if depths
            .keys()
            .any(|direction| indices.0.contains_key(direction))
        {
            return Err(
                "reflect cannot both subtract and capture depth on the same direction".into(),
            );
        }
        let norm = &factor.real * &factor.real + &factor.imag * &factor.imag;
        if norm.is_zero() {
            return Err("reflect pattern cannot erase its capture by multiplying by zero".into());
        }
        let inverse = Scalar::from_classical(&NativeScalar {
            real: factor.real / &norm,
            imag: -factor.imag / norm,
        })
        .to_data();
        let mut instructions = Vec::new();
        compile_replacement(replacement, &capture, &depths, &mut instructions, 0)?;
        let rule = Self {
            indices: indices.0.into_iter().collect(),
            depths: depths.into_iter().collect(),
            inverse,
            instructions,
        };
        rule.validate()?;
        Ok(rule)
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.instructions.is_empty() || self.instructions.len() > MAX_INSTRUCTIONS {
            return Err("reflect replacement has an invalid instruction count".into());
        }
        if self
            .indices
            .iter()
            .any(|(direction, depth)| *direction == 0 || depth.is_zero())
        {
            return Err("reflect pattern has an invalid INDEX".into());
        }
        if self.indices.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
            return Err("reflect INDEX directions must be unique and sorted".into());
        }
        if self.depths.iter().any(|(direction, name)| {
            *direction == 0
                || name.is_empty()
                || self.indices.iter().any(|(fixed, _)| fixed == direction)
        }) || self.depths.windows(2).any(|pair| pair[0].0 >= pair[1].0)
        {
            return Err(
                "reflect depth captures must have distinct positive directions and valid names"
                    .into(),
            );
        }
        let check_depth = |name: &str| -> Result<(), String> {
            if self.depths.iter().any(|(_, bound)| bound == name) {
                Ok(())
            } else {
                Err(format!("unbound reflect depth capture {name:?}"))
            }
        };
        if Scalar::from_data(&self.inverse)?.project().is_zero() {
            return Err("reflect pattern inverse must be nonzero".into());
        }
        let mut used = vec![false; self.instructions.len()];
        used[self.instructions.len() - 1] = true;
        for (position, instruction) in self.instructions.iter().enumerate().rev() {
            if !used[position] {
                return Err("reflect replacement contains an unreachable instruction".into());
            }
            let mut edge = |input: usize| -> Result<(), String> {
                if input >= position {
                    return Err("reflect replacement edges must point backward".into());
                }
                used[input] = true;
                Ok(())
            };
            match instruction {
                Instruction::Capture => {}
                Instruction::Depth { name } => check_depth(name)?,
                Instruction::IndexCapture {
                    input,
                    direction,
                    name,
                } => {
                    check_depth(name)?;
                    if *direction == 0 {
                        return Err("reflect INDEX direction must be positive".into());
                    }
                    edge(*input)?;
                }
                Instruction::Scalar { value } => {
                    Scalar::from_data(value)?;
                }
                Instruction::Add { left, right } | Instruction::Multiply { left, right } => {
                    edge(*left)?;
                    edge(*right)?;
                }
                Instruction::Phase { input, turns } => {
                    if !crate::core::is_canonical_phase(*turns) {
                        return Err("reflect phase must be from 0 through 3".into());
                    }
                    edge(*input)?;
                }
                Instruction::Index {
                    input, direction, ..
                } => {
                    if *direction == 0 {
                        return Err("reflect INDEX direction must be positive".into());
                    }
                    edge(*input)?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn apply(&self, subject: &NativeState) -> NativeState {
        let field = subject
            .0
            .iter()
            .map(|(index, value)| (index.clone(), Scalar::from_classical(value)))
            .collect();
        let result = self.apply_field(&field).expect("validated exact rule");
        NativeState::from_terms(
            result
                .into_iter()
                .map(|(index, value)| (index, value.project())),
        )
    }

    pub(crate) fn apply_field<C: crate::retained::field::Coefficient>(
        &self,
        subject: &crate::retained::field::Field<C>,
    ) -> Result<crate::retained::field::Field<C>, String> {
        use crate::retained::{Operation, field};
        let inverse = C::scalar(&self.inverse)?;
        let mut result = BTreeMap::new();
        for (index, coefficient) in subject {
            // Canonical values have no zero contribution. Provenance remains on
            // the subject edge; it does not create a second match semantics.
            if coefficient.is_zero() {
                continue;
            }
            let Some(Bindings { remaining, depths }) = self.bind(index) else {
                continue;
            };
            let capture = BTreeMap::from([(remaining, coefficient.multiply(&inverse)?)]);
            let mut values = Vec::with_capacity(self.instructions.len());
            for instruction in &self.instructions {
                let (operation, inputs) = match instruction {
                    Instruction::Capture => {
                        values.push(capture.clone());
                        continue;
                    }
                    Instruction::Depth { name } => {
                        let scalar = NativeScalar {
                            real: num_rational::BigRational::from_integer(
                                depths[name.as_str()].clone().into(),
                            ),
                            imag: num_rational::BigRational::zero(),
                        };
                        values.push(BTreeMap::from([(
                            MultiIndex::default(),
                            C::scalar(&Scalar::from_classical(&scalar).to_data())?,
                        )]));
                        continue;
                    }
                    Instruction::IndexCapture {
                        input,
                        direction,
                        name,
                    } => {
                        let shift = MultiIndex(BTreeMap::from([(
                            *direction,
                            depths[name.as_str()].clone(),
                        )]));
                        values.push(
                            values[*input]
                                .iter()
                                .map(|(index, value)| (index.compose(&shift), value.clone()))
                                .collect(),
                        );
                        continue;
                    }
                    Instruction::Scalar { value } => (
                        Operation::Scalar {
                            coordinates: value.clone(),
                        },
                        vec![],
                    ),
                    Instruction::Add { left, right } => (Operation::Add, vec![*left, *right]),
                    Instruction::Multiply { left, right } => {
                        (Operation::Multiply, vec![*left, *right])
                    }
                    Instruction::Phase { input, turns } => {
                        (Operation::Phase { turns: *turns }, vec![*input])
                    }
                    Instruction::Index {
                        input,
                        direction,
                        depth,
                    } => (
                        Operation::Index {
                            direction: *direction,
                            depth: *depth,
                        },
                        vec![*input],
                    ),
                };
                values.push(field::evaluate_step(&operation, &inputs, &values)?);
            }
            for (index, value) in values.pop().expect("validated nonempty rule") {
                field::accumulate(&mut result, index, value)?;
            }
        }
        result.retain(|_, value| !value.is_zero());
        Ok(result)
    }

    fn bind(&self, index: &MultiIndex) -> Option<Bindings> {
        let mut remaining = index.clone();
        for (direction, depth) in &self.indices {
            let actual = remaining.0.get_mut(direction)?;
            if *actual < *depth {
                return None;
            }
            *actual -= depth;
        }
        remaining.0.retain(|_, depth| !depth.is_zero());
        let mut depths = BTreeMap::new();
        for (direction, name) in &self.depths {
            let depth = remaining.0.remove(direction)?;
            if let Some(previous) = depths.insert(name.clone(), depth.clone()) {
                if previous != depth {
                    return None;
                }
            }
        }
        Some(Bindings { remaining, depths })
    }
}

/// A borrowed source or graph-record view, never a reconstructed expression tree.
pub(crate) trait Template: Copy {
    fn node(self) -> Result<TemplateNode<Self>, String>;
}

pub(crate) enum TemplateNode<T> {
    Reference(String),
    Scalar(NativeScalar),
    Index(u64, u64, T),
    Depth(u64, String, T),
    Phase(i64, T),
    Add(Vec<T>),
    Multiply(Vec<T>),
}

impl Template for &Expr {
    fn node(self) -> Result<TemplateNode<Self>, String> {
        Ok(match self {
            Expr::Reference { name, .. } => TemplateNode::Reference(name.clone()),
            Expr::Literal { real, imag, .. } => {
                TemplateNode::Scalar(NativeScalar::from_text(real, imag)?)
            }
            Expr::Index {
                direction,
                multiplicity,
                value,
                ..
            } => TemplateNode::Index(*direction, *multiplicity, value.as_ref()),
            Expr::IndexCapture {
                direction,
                depth,
                value,
                ..
            } => TemplateNode::Depth(*direction, depth.clone(), value.as_ref()),
            Expr::Phase { turns, value, .. } => TemplateNode::Phase(*turns, value.as_ref()),
            Expr::Add { operands, .. } => TemplateNode::Add(operands.iter().collect()),
            Expr::Multiply { operands, .. } => TemplateNode::Multiply(operands.iter().collect()),
            _ => {
                return Err(
                    "reflect templates use captures and ADD, MULTIPLY, PHASE, INDEX".into(),
                );
            }
        })
    }
}

fn pattern_factor<T: Template>(
    expr: T,
    capture: &mut Option<String>,
    depths: &mut BTreeMap<u64, String>,
    depth: usize,
) -> Result<(MultiIndex, NativeScalar), String> {
    if depth > MAX_DEPTH {
        return Err("reflect pattern is too deep".into());
    }
    match expr.node()? {
        TemplateNode::Reference(name) => {
            if capture.is_some() { return Err("reflect pattern must have exactly one capture occurrence; splitting values is ambiguous".into()); }
            *capture = Some(name);
            Ok((MultiIndex::default(), NativeScalar::one()))
        }
        TemplateNode::Scalar(value) => Ok((MultiIndex::default(), value)),
        TemplateNode::Index(direction, multiplicity, value) => {
            let (index, factor) = pattern_factor(value, capture, depths, depth + 1)?;
            Ok((index.shift_by(direction, multiplicity)?, factor))
        }
        TemplateNode::Depth(direction, name, value) => {
            if direction == 0 || depths.insert(direction, name).is_some() { return Err("reflect can capture a direction's complete depth only once".into()); }
            pattern_factor(value, capture, depths, depth + 1)
        }
        TemplateNode::Phase(turns, value) if crate::core::is_canonical_phase(turns) => {
            let (index, factor) = pattern_factor(value, capture, depths, depth + 1)?;
            Ok((index, factor.phase(turns)))
        }
        TemplateNode::Multiply(operands) => {
            let mut index = MultiIndex::default();
            let mut factor = NativeScalar::one();
            for operand in operands {
                let (other, scalar) = pattern_factor(operand, capture, depths, depth + 1)?;
                index = index.compose(&other);
                factor = factor.multiply(&scalar);
            }
            Ok((index, factor))
        }
        _ => Err("reflect pattern must be a scalar/PHASE/INDEX monomial with one capture; additive splitting is ambiguous".into()),
    }
}

fn compile_replacement<T: Template>(
    expr: T,
    capture: &str,
    depths: &BTreeMap<u64, String>,
    instructions: &mut Vec<Instruction>,
    depth: usize,
) -> Result<usize, String> {
    if depth > MAX_DEPTH || instructions.len() >= MAX_INSTRUCTIONS {
        return Err("reflect replacement exceeds the template limit".into());
    }
    let node = expr.node()?;
    let addition = matches!(&node, TemplateNode::Add(_));
    let instruction = match node {
        TemplateNode::Reference(name) if name == capture => Instruction::Capture,
        TemplateNode::Reference(name) if depths.values().any(|bound| bound == &name) => {
            Instruction::Depth { name }
        }
        TemplateNode::Reference(name) => return Err(format!("unbound reflect capture {name:?}")),
        TemplateNode::Scalar(value) => Instruction::Scalar {
            value: Scalar::from_classical(&value).to_data(),
        },
        TemplateNode::Index(direction, multiplicity, value) => Instruction::Index {
            input: compile_replacement(value, capture, depths, instructions, depth + 1)?,
            direction,
            depth: multiplicity,
        },
        TemplateNode::Depth(direction, name, value) => {
            if !depths.values().any(|bound| bound == &name) {
                return Err(format!("unbound reflect depth capture {name:?}"));
            }
            Instruction::IndexCapture {
                input: compile_replacement(value, capture, depths, instructions, depth + 1)?,
                direction,
                name,
            }
        }
        TemplateNode::Phase(turns, value) => Instruction::Phase {
            input: compile_replacement(value, capture, depths, instructions, depth + 1)?,
            turns,
        },
        TemplateNode::Add(operands) | TemplateNode::Multiply(operands) => {
            let mut operands = operands.into_iter();
            let first = operands
                .next()
                .ok_or("empty reflect replacement operation")?;
            let mut left = compile_replacement(first, capture, depths, instructions, depth + 1)?;
            for operand in operands {
                let right = compile_replacement(operand, capture, depths, instructions, depth + 1)?;
                if instructions.len() >= MAX_INSTRUCTIONS {
                    return Err("reflect replacement exceeds the template limit".into());
                }
                instructions.push(if addition {
                    Instruction::Add { left, right }
                } else {
                    Instruction::Multiply { left, right }
                });
                left = instructions.len() - 1;
            }
            return Ok(left);
        }
    };
    instructions.push(instruction);
    Ok(instructions.len() - 1)
}
