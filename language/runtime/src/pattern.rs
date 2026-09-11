// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Authoritative Native Pattern records with shared, disposable decoded views.
//!
//! A Pattern owns one seed and one reusable unary step graph, including curried
//! bindings. An Observation is that same Pattern plus an unwrapped repetition
//! index. Both relationships are Native records. Rust fields cache decoded views;
//! `native()`/`to_data()` contain everything needed to rebuild those views.
//! Neither selection nor successor executes the step or allocates a prefix.
//!
//! Explicit projection replays the step on retained Native states. Only the
//! current result is held, but that result may itself retain earlier inputs.
//! Dropping those inputs would change reflective generators. No replay prefix
//! is cached in the Pattern or Observation. Replay costs k step calls, not
//! constant time or necessarily constant temporary memory.
//! No spatial camera, radius, or instruction address defines these semantics.
//!
//! # Examples
//!
//! ```
//! use native_space_language::{core, pattern::Pattern, strand::execution::Graph};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let source = core::parse("let step = (x) => phase(1,x)\noutput 1", "cycle.ns")?;
//! let graph = Graph::compile(&source)?;
//! let pattern = Pattern::new(graph.run()?.native()?, graph.function("step")?)?;
//! let first = pattern.observe(1_u32.into());
//! let fifth = pattern.observe(5_u32.into());
//! assert!(!first.same_selection(&fifth));
//! assert_eq!(first.project(5)?, fifth.project(5)?);
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;

use num_bigint::BigUint;
use num_traits::ToPrimitive;
use serde_json::Value as Json;

mod encoding;

#[cfg(test)]
mod tests {
    #[test]
    fn pattern_adds_no_core_operation_or_machine_opcode() {
        use crate::core::{LANGUAGE_NAMESPACE, LanguageNameKind};
        let operations = LANGUAGE_NAMESPACE
            .iter()
            .filter(|(_, kind)| matches!(kind, LanguageNameKind::CoreOperation))
            .map(|(name, _)| name.to_ascii_lowercase())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            operations,
            ["add", "multiply", "phase", "index", "reflect"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        );
        for name in ["pattern", "observation", "observe", "successor"] {
            serde_json::from_value::<crate::bytecode::Opcode>(serde_json::json!(name)).unwrap_err();
            serde_json::from_value::<crate::retained::Operation>(
                serde_json::json!({"operation": name}),
            )
            .unwrap_err();
            let source =
                crate::core::parse(&format!("output {name}(1)"), "no-intrinsic.ns").unwrap();
            crate::strand::execution::Graph::compile(&source).unwrap_err();
        }
    }
}

use crate::{
    core::{Diagnostic, LanguageError, NativeState},
    retained::State,
    strand::execution::{FunctionValue, Value},
};

#[derive(Debug)]
struct Generator {
    native: State,
    seed: State,
    step: FunctionValue,
}

/// One seed and one shared reusable step, without an observation history.
#[derive(Clone, Debug)]
pub struct Pattern(Arc<Generator>);

/// One repetition of a Pattern, distinct even when its phase repeats.
#[derive(Clone, Debug)]
pub struct Observation {
    native: State,
    pattern: Pattern,
    index: BigUint,
}

/// A disposable playback cursor, separate from the immutable finite Pattern.
///
/// Retains only the current evaluation, which may itself contain prior inputs.
/// Forward seeks reuse it; backward seeks replay from the seed. This is an
/// execution aid, not a history cache or the representation of a Pattern.
#[derive(Clone, Debug)]
pub struct Cursor {
    pattern: Pattern,
    index: u64,
    current: State,
}

fn repetition_count(index: &BigUint, maximum_steps: u64) -> Result<u64, LanguageError> {
    index
        .to_u64()
        .filter(|k| *k <= maximum_steps)
        .ok_or_else(|| error("observation exceeds the repetition budget"))
}

fn repeat(pattern: &Pattern, mut current: State, count: u64) -> Result<State, LanguageError> {
    for _ in 0..count {
        let next = pattern.step().call(vec![Value::State(current)])?;
        let Value::State(next) = next else {
            return Err(error("pattern step must return a Native state"));
        };
        current = next;
    }
    Ok(current)
}

impl Cursor {
    /// Read the indexed observation reached by the last successful seek.
    ///
    /// This retains the same finite generator, not the current replay history.
    #[must_use]
    pub fn observation(&self) -> Observation {
        self.pattern.observe(self.index.into())
    }

    /// Seek without changing the cursor if evaluation fails.
    ///
    /// `maximum_steps` bounds the absolute selected index, not just new work.
    /// No canonicalization occurs between steps, including across seek calls.
    ///
    /// # Errors
    /// Rejects over-budget indices, failing steps and non-state results.
    pub fn seek(&mut self, index: &BigUint, maximum_steps: u64) -> Result<&State, LanguageError> {
        let target = repetition_count(index, maximum_steps)?;
        let (state, count) = if target >= self.index {
            (self.current.clone(), target - self.index)
        } else {
            (self.pattern.seed().clone(), target)
        };
        let next = repeat(&self.pattern, state, count)?;
        self.current = next;
        self.index = target;
        Ok(&self.current)
    }
}

fn error(message: &str) -> LanguageError {
    LanguageError(Diagnostic {
        code: "NSP001".into(),
        message: message.into(),
        source_name: "<pattern>".into(),
        span: None,
    })
}

impl Pattern {
    /// Return the authoritative Native record, including escaped seed and step.
    #[must_use]
    pub fn native(&self) -> &State {
        &self.0.native
    }

    /// Recover a Pattern and its callable cache from Native records, without source.
    ///
    /// # Errors
    /// Rejects malformed fields, retained seed graphs, function graphs or step arity.
    pub fn from_native(native: &State) -> Result<Self, LanguageError> {
        let (seed, step) = encoding::decode_pattern(native)?;
        if !step.has_exact_arity(1) {
            return Err(error(
                "pattern step must have exactly one unbound parameter",
            ));
        }
        Ok(Self(Arc::new(Generator {
            native: State::from_projection(native.project()),
            seed,
            step,
        })))
    }

    /// Serialize the authoritative Native Pattern, without decoded caches.
    #[must_use]
    pub fn to_data(&self) -> Json {
        self.native().native_data()
    }

    /// Reconstruct a Pattern from its ordinary Native-state serialization.
    ///
    /// # Errors
    /// Rejects malformed retained-state serialization or invalid Pattern records.
    pub fn from_data(data: &Json) -> Result<Self, LanguageError> {
        let native = State::from_data(data).map_err(|message| error(&message))?;
        Self::from_native(&native)
    }

    /// Pair a seed with a unary step without executing or recompiling it.
    ///
    /// # Errors
    /// Rejects steps other than fixed-arity functions with one remaining parameter.
    pub fn new(seed: State, step: FunctionValue) -> Result<Self, LanguageError> {
        if !step.has_exact_arity(1) {
            return Err(error(
                "pattern step must have exactly one unbound parameter",
            ));
        }
        let native = encoding::pattern(&seed, &step)?;
        Ok(Self(Arc::new(Generator { native, seed, step })))
    }

    /// Select a repetition without computing it or constructing preceding observations.
    #[must_use]
    pub fn observe(&self, index: BigUint) -> Observation {
        Observation {
            native: encoding::observation(self, &index),
            pattern: self.clone(),
            index,
        }
    }

    /// Start disposable sequential evaluation at the seed without executing a step.
    #[must_use]
    pub fn cursor(&self) -> Cursor {
        Cursor {
            pattern: self.clone(),
            index: 0,
            current: self.seed().clone(),
        }
    }

    /// Read the retained seed, including zero provenance.
    #[must_use]
    pub fn seed(&self) -> &State {
        &self.0.seed
    }

    /// Read the shared step with its existing curried bindings.
    #[must_use]
    pub fn step(&self) -> &FunctionValue {
        &self.0.step
    }

    /// Check cache allocation identity, not structural or extensional program equivalence.
    #[must_use]
    pub fn shares_generator(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Observation {
    /// Return the authoritative Native observation, with no host-only metadata.
    #[must_use]
    pub const fn native(&self) -> &State {
        &self.native
    }

    /// Recover the entire observation from Native data without source recompilation.
    ///
    /// # Errors
    /// Rejects invalid Pattern fields, step graphs and non-natural observation indices.
    pub fn from_native(native: &State) -> Result<Self, LanguageError> {
        let (pattern, index) = encoding::decode_observation(native)?;
        Ok(pattern.observe(index))
    }

    /// Load only a serialized Native state and reconstruct every typed cache.
    ///
    /// # Errors
    /// Rejects invalid retained-state serialization or malformed observation records.
    pub fn from_data(data: &Json) -> Result<Self, LanguageError> {
        let native = State::from_data(data).map_err(|message| error(&message))?;
        Self::from_native(&native)
    }

    /// Select the next unwrapped repetition without executing or expanding its graph.
    ///
    /// Only the arbitrary-precision index changes. Evaluation remains explicit
    /// and budgeted; this does not promise termination of an arbitrary step.
    #[must_use]
    pub fn successor(&self) -> Self {
        self.pattern.observe(&self.index + 1_u32)
    }

    /// Read the generator shared by every repetition.
    #[must_use]
    pub const fn pattern(&self) -> &Pattern {
        &self.pattern
    }

    /// Read the unwrapped observation index, independently of payload indices.
    #[must_use]
    pub const fn index(&self) -> &BigUint {
        &self.index
    }

    /// Compare exact Native generator records and index, not extensional equivalence.
    #[must_use]
    pub fn same_selection(&self, other: &Self) -> bool {
        self.index == other.index
            && (self.pattern.shares_generator(&other.pattern)
                || self.pattern.native().project() == other.pattern.native().project())
    }

    /// Observe step^k(seed) as a canonical Native state with bounded replay.
    ///
    /// Zero returns the seed's canonical value. The repetition index is not
    /// injected into payload directions, so it cannot collide with power depth.
    /// The generator remains available even when this readout cancels to zero.
    ///
    /// # Errors
    /// Rejects indices above the caller's repetition budget, failing steps,
    /// non-state results, and the graph evaluator's ordinary execution limits.
    pub fn project(&self, maximum_steps: u64) -> Result<NativeState, LanguageError> {
        Ok(self.evaluate(maximum_steps)?.project().clone())
    }

    /// Evaluate step^k(seed), retaining information for subsequent reflective readouts.
    ///
    /// This explicit evaluation may retain prior inputs inside the result. It
    /// does not add a history cache to the finite generator or observation.
    /// Use `project` when only the canonical value is needed.
    ///
    /// # Errors
    /// Rejects indices above the repetition budget, failing steps, non-state
    /// results, and the graph evaluator's ordinary execution limits.
    pub fn evaluate(&self, maximum_steps: u64) -> Result<State, LanguageError> {
        let count = repetition_count(&self.index, maximum_steps)?;
        repeat(&self.pattern, self.pattern.seed().clone(), count)
    }

    /// Serialize the generator and index, never an expanded observation history.
    ///
    /// Uses the ordinary retained-state wire format; no extra JSON metadata is needed.
    #[must_use]
    pub fn to_data(&self) -> Json {
        self.native.native_data()
    }
}
