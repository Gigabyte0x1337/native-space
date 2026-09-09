// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Shares source evaluation rules between native states and the independent classical camera.
//!
//! The two carriers deliberately differ in retention, not source-language
//! semantics. Keeping this interface private avoids exposing a generic runtime
//! framework while preventing calls, packs, and feedback from drifting apart.

use super::State;
use crate::core::{NativeScalar, NativeState, Span};

pub(crate) trait Evaluated: Clone {
    fn zero() -> Self;
    fn one() -> Self;
    fn scalar(value: NativeScalar) -> Self;
    fn add(&self, other: &Self) -> Self;
    fn multiply(&self, other: &Self) -> Self;
    fn phase(&self, turns: i64) -> Self;
    fn index_power(&self, direction: u64, depth: u64) -> Result<Self, String>;
    fn camera(&self, from: u64, to: u64) -> Self;
    fn project(&self) -> &NativeState;
    fn lift(value: &NativeState) -> Self;
    fn retaining(&self, inputs: &[Self]) -> Self;
    fn at_span(self, span: Option<Span>) -> Self;
}

// Both implementations use their carrier's inherent operations. Only the
// classical carrier deliberately forgets scope dependencies.
macro_rules! operations {
    () => {
        fn zero() -> Self {
            Self::zero()
        }
        fn one() -> Self {
            Self::one()
        }
        fn scalar(value: NativeScalar) -> Self {
            Self::scalar(value)
        }
        fn add(&self, other: &Self) -> Self {
            self.add(other)
        }
        fn multiply(&self, other: &Self) -> Self {
            self.multiply(other)
        }
        fn phase(&self, turns: i64) -> Self {
            self.phase(turns)
        }
        fn index_power(&self, direction: u64, depth: u64) -> Result<Self, String> {
            self.index_power(direction, depth)
        }
        fn camera(&self, from: u64, to: u64) -> Self {
            self.camera(from, to)
        }
    };
}

impl Evaluated for NativeState {
    operations!();
    fn project(&self) -> &NativeState {
        self
    }
    fn lift(value: &NativeState) -> Self {
        value.clone()
    }
    fn retaining(&self, _inputs: &[Self]) -> Self {
        self.clone()
    }
    fn at_span(self, _span: Option<Span>) -> Self {
        self
    }
}

impl Evaluated for State {
    operations!();
    fn project(&self) -> &NativeState {
        self.project()
    }
    fn lift(value: &NativeState) -> Self {
        Self::from_projection(value)
    }
    fn retaining(&self, inputs: &[Self]) -> Self {
        self.retaining(inputs)
    }
    fn at_span(self, span: Option<Span>) -> Self {
        self.at_span(span)
    }
}
