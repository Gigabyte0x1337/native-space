// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Retains native operation relationships independently of their classical projection.
//!
//! A state is an immutable, shared operation graph, not just its numerical
//! answer. ADD and MULTIPLY retain both inputs even when the answer cancels.
//! PHASE and INDEX retain their input and parameters. Classical projections
//! are lazy camera observations, never required to construct operations. Equality of
//! projections is deliberately separate from equality of retained structure.
//!
//! Magnitude depth is exactly `ln(|z|)`, represented as `ln(|z|²)/2`.
//! Rational squared magnitudes avoid floating-point logarithms. Zero has a
//! boundary coordinate and may retain a supplied phase ray. INDEX is independent.
//! This is an explicit retained-information model, not a claim that complex
//! multiplication alone stores history or that the graph is already minimal.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, OnceLock};

use num_traits::{One, Signed, Zero};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::core::{
    LanguageError, NativeScalar, NativeState, OutputKind, Program, Rational, Span,
    is_canonical_phase, rational_text,
};

pub mod coordinates;
pub(crate) mod evaluation;
mod field;
pub mod numeric;
mod scalar;
#[doc(inline)]
pub use scalar::Scalar;
pub use scalar::{RayData, ScalarData};

/// Exact radial multiplicative position, with one at depth zero.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Depth {
    squared_magnitude: Rational,
}

impl Depth {
    /// Measure a scalar without rounding its logarithm.
    #[must_use]
    pub fn of(value: &NativeScalar) -> Self {
        Self {
            squared_magnitude: &value.real * &value.real + &value.imag * &value.imag,
        }
    }

    /// Report whether this is the zero boundary, not a finite depth.
    #[must_use]
    pub fn is_zero_boundary(&self) -> bool {
        self.squared_magnitude.is_zero()
    }

    /// Report depth zero, the unit-magnitude multiplicative origin.
    #[must_use]
    pub fn is_origin(&self) -> bool {
        self.squared_magnitude.is_one()
    }

    /// Add logarithmic depths, corresponding to multiplication of magnitudes.
    #[must_use]
    pub fn compose(&self, other: &Self) -> Self {
        Self {
            squared_magnitude: &self.squared_magnitude * &other.squared_magnitude,
        }
    }

    /// Read the exact argument of twice the natural-log depth.
    #[must_use]
    pub fn squared_magnitude(&self) -> &Rational {
        &self.squared_magnitude
    }

    /// Describe the exact coordinate without materializing an irrational logarithm.
    #[must_use]
    pub fn to_data(&self) -> Value {
        if self.is_zero_boundary() {
            json!({"kind": "zero_boundary"})
        } else if self.is_origin() {
            json!({"kind": "finite", "value": "0"})
        } else {
            json!({"kind": "logarithmic", "base": "e", "factor": "1/2",
                "argument": rational_text(&self.squared_magnitude)})
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Operation {
    Scalar { coordinates: ScalarData },
    Add,
    Multiply,
    Phase { turns: i64 },
    Index { direction: u64, depth: u64 },
    Camera { from: u64, to: u64 },
}

#[derive(Debug)]
struct Node {
    operation: Operation,
    inputs: Vec<State>,
    retained: Vec<State>,
    projection: OnceLock<NativeState>,
    span: Option<Span>,
}

// Shared feedback chains can be much deeper than the host stack. Release the
// uniquely owned portion iteratively; shared nodes stay alive for their owners.
impl Drop for Node {
    fn drop(&mut self) {
        let mut pending = std::mem::take(&mut self.inputs);
        pending.append(&mut self.retained);
        while let Some(State(node)) = pending.pop() {
            if let Ok(mut unique) = Arc::try_unwrap(node) {
                pending.append(&mut unique.inputs);
                pending.append(&mut unique.retained);
            }
        }
    }
}

/// A native state retaining its inputs, operations, and exact projection.
#[derive(Clone)]
pub struct State(Arc<Node>);

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("operation", &self.0.operation)
            .field("inputs", &self.0.inputs.len())
            .field("retained", &self.0.retained.len())
            .field("projection_observed", &self.0.projection.get().is_some())
            .finish()
    }
}

impl State {
    fn new(operation: Operation, inputs: Vec<Self>) -> Self {
        Self(Arc::new(Node {
            operation,
            inputs,
            retained: Vec::new(),
            projection: OnceLock::new(),
            span: None,
        }))
    }

    /// Construct an exact scalar input, including literal zero.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "owning input constructor matches the shared evaluator carrier contract"
    )]
    #[must_use]
    pub fn scalar(value: NativeScalar) -> Self {
        Self::new(
            Operation::Scalar {
                coordinates: Scalar::from_classical(&value).to_data(),
            },
            Vec::new(),
        )
    }

    /// Construct a scalar input already expressed in native coordinates.
    #[must_use]
    pub fn native_scalar(value: &Scalar) -> Self {
        Self::new(
            Operation::Scalar {
                coordinates: value.to_data(),
            },
            Vec::new(),
        )
    }

    /// Construct the additive-zero input, without inventing prior history.
    #[must_use]
    pub fn zero() -> Self {
        Self::scalar(NativeScalar::zero())
    }

    /// Construct one, the positive multiplicative origin.
    #[must_use]
    pub fn one() -> Self {
        Self::scalar(NativeScalar::one())
    }

    /// Combine two states without discarding cancelled contributions.
    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        Self::new(Operation::Add, vec![self.clone(), other.clone()])
    }

    /// Multiply two states while retaining both sides of a zero product.
    #[must_use]
    pub fn multiply(&self, other: &Self) -> Self {
        Self::new(Operation::Multiply, vec![self.clone(), other.clone()])
    }

    /// Retain a phase operation, including phase of a projected zero.
    ///
    /// # Panics
    /// Panics for turns outside `0..=3`, matching the core's validated contract.
    #[must_use]
    pub fn phase(&self, turns: i64) -> Self {
        assert!(
            is_canonical_phase(turns),
            "phase turns must be from 0 through 3"
        );
        Self::new(Operation::Phase { turns }, vec![self.clone()])
    }

    /// Retain an index operation independently of radial multiplicative depth.
    ///
    /// # Errors
    /// Returns an error when the direction is zero.
    pub fn index_power(&self, direction: u64, depth: u64) -> Result<Self, String> {
        if direction == 0 {
            return Err("index direction must be positive".into());
        }
        Ok(Self::new(
            Operation::Index { direction, depth },
            vec![self.clone()],
        ))
    }

    /// Read the operation's retained inputs, even when its projection is zero.
    #[must_use]
    pub fn inputs(&self) -> &[Self] {
        &self.0.inputs
    }

    /// Read scope inputs preserved by a call or explicit camera observation.
    #[must_use]
    pub fn retained_inputs(&self) -> &[Self] {
        &self.0.retained
    }

    /// Keep read dependencies without pretending they contribute numerical zero.
    ///
    /// The result's arithmetic graph is unchanged. These separate scope edges
    /// preserve the source of an observation or an unused function argument.
    #[must_use]
    pub fn retaining(&self, inputs: &[Self]) -> Self {
        if inputs.is_empty() {
            return self.clone();
        }
        // A second scope keeps the previous scope as a shared branch instead
        // of copying its entire dependency list. Identity feedback must grow
        // linearly, not duplicate every earlier scope at every clock step.
        let mut retained = Vec::with_capacity(inputs.len() + 1);
        if !self.retained_inputs().is_empty()
            && !inputs.iter().any(|input| Arc::ptr_eq(&input.0, &self.0))
        {
            retained.push(self.clone());
        }
        retained.extend_from_slice(inputs);
        Self(Arc::new(Node {
            operation: self.0.operation.clone(),
            inputs: self.0.inputs.clone(),
            retained,
            projection: self.0.projection.clone(),
            span: self.0.span,
        }))
    }

    pub(crate) fn at_span(self, span: Option<Span>) -> Self {
        if self.0.span == span {
            return self;
        }
        Self(Arc::new(Node {
            operation: self.0.operation.clone(),
            inputs: self.0.inputs.clone(),
            retained: self.0.retained.clone(),
            projection: self.0.projection.clone(),
            span,
        }))
    }

    /// Lift a supplied classical state without inventing its earlier history.
    ///
    /// # Panics
    /// Panics if the supplied classical state contains an invalid zero INDEX direction.
    #[must_use]
    pub fn from_projection(value: &NativeState) -> Self {
        let mut terms = value.0.iter().map(|(index, coefficient)| {
            let mut state = Self::scalar(coefficient.clone());
            // MultiIndex depths are arbitrary-size integers. Build their exact
            // binary sum of u64-sized INDEX steps instead of narrowing them.
            for (direction, depth) in &index.0 {
                let digits = depth.to_u64_digits();
                let mut place = Self::one().index_power(*direction, 1).expect("valid index");
                let mut factor = Self::one();
                for bit in 0..depth.bits() {
                    if digits[usize::try_from(bit / 64).expect("allocated index fits usize")]
                        & (1_u64 << (bit % 64))
                        != 0
                    {
                        factor = factor.multiply(&place);
                    }
                    if bit + 1 < depth.bits() {
                        place = place.multiply(&place);
                    }
                }
                state = state.multiply(&factor);
            }
            state
        });
        let Some(first) = terms.next() else {
            return Self::zero();
        };
        terms.fold(first, |sum, term| sum.add(&term))
    }

    /// Select indexed coordinates while keeping the complete source state.
    #[must_use]
    pub fn camera(&self, from: u64, to: u64) -> Self {
        Self::new(Operation::Camera { from, to }, vec![self.clone()])
    }

    /// Read a branch by its ordered operand path, including cancelled branches.
    #[must_use]
    pub fn branch(&self, path: &[usize]) -> Option<&Self> {
        path.iter()
            .try_fold(self, |state, &index| state.inputs().get(index))
    }

    /// Observe the ordinary flat-stack result without altering this state.
    ///
    /// # Panics
    /// Panics only if an internal graph invariant is violated after construction.
    #[must_use]
    pub fn project(&self) -> &NativeState {
        let mut pending = vec![(self, false)];
        while let Some((state, ready)) = pending.pop() {
            if state.0.projection.get().is_some() {
                continue;
            }
            if !ready {
                pending.push((state, true));
                pending.extend(state.inputs().iter().rev().map(|input| (input, false)));
                continue;
            }
            let inputs = state.inputs();
            let observed =
                |index: usize| inputs[index].0.projection.get().expect("postorder camera");
            let value = match &state.0.operation {
                Operation::Scalar { coordinates } => NativeState::scalar(
                    Scalar::from_data(coordinates)
                        .expect("validated native scalar")
                        .project(),
                ),
                Operation::Add => observed(0).add(observed(1)),
                Operation::Multiply => {
                    NativeState::from_terms(observed(0).0.iter().flat_map(|(li, lc)| {
                        observed(1).0.iter().map(move |(ri, rc)| {
                            let coefficient =
                                Scalar::from_classical(lc).multiply(&Scalar::from_classical(rc));
                            (li.compose(ri), coefficient.project())
                        })
                    }))
                }
                Operation::Phase { turns } => observed(0).phase(*turns),
                Operation::Camera { from, to } => observed(0).camera(*from, *to),
                Operation::Index { direction, depth } => observed(0)
                    .index_power(*direction, *depth)
                    .expect("validated positive index direction"),
            };
            // Another reader may have observed this immutable node concurrently.
            // Both computations use the same exact operator and inputs.
            let _ = state.0.projection.set(value);
        }
        self.0.projection.get().expect("camera observed its root")
    }

    /// Compare numerical projections without asserting retained-state equality.
    #[must_use]
    pub fn same_projection(&self, other: &Self) -> bool {
        self.project() == other.project()
    }

    /// Compare complete ordered operation structure, independent of graph sharing.
    #[must_use]
    pub fn same_structure(&self, other: &Self) -> bool {
        let mut pending = vec![(self, other)];
        let mut visited = HashSet::new();
        while let Some((left, right)) = pending.pop() {
            if !visited.insert((left.key(), right.key())) {
                continue;
            }
            if left.0.operation != right.0.operation
                || left.inputs().len() != right.inputs().len()
                || left.retained_inputs().len() != right.retained_inputs().len()
            {
                return false;
            }
            pending.extend(left.inputs().iter().zip(right.inputs()));
            pending.extend(left.retained_inputs().iter().zip(right.retained_inputs()));
        }
        true
    }

    fn key(&self) -> *const Node {
        Arc::as_ptr(&self.0)
    }

    /// Serialize a finite graph, including branches hidden by the projection.
    ///
    /// References point backward, so even deeply nested states have flat JSON.
    /// The projection and depth coordinates are checked observations, not sources
    /// of authority when decoding.
    #[must_use]
    pub(crate) fn plan(&self) -> Vec<Step> {
        let mut ids = HashMap::new();
        let mut nodes = Vec::new();
        let mut pending = vec![(self, false)];
        while let Some((state, ready)) = pending.pop() {
            if ids.contains_key(&state.key()) {
                continue;
            }
            if !ready {
                pending.push((state, true));
                pending.extend(
                    state
                        .retained_inputs()
                        .iter()
                        .rev()
                        .map(|input| (input, false)),
                );
                pending.extend(state.inputs().iter().rev().map(|input| (input, false)));
                continue;
            }
            let inputs: Vec<_> = state
                .inputs()
                .iter()
                .map(|input| ids[&input.key()])
                .collect();
            ids.insert(state.key(), nodes.len());
            nodes.push(Step {
                operator: state.0.operation.clone(),
                inputs,
                retained: state
                    .retained_inputs()
                    .iter()
                    .map(|input| ids[&input.key()])
                    .collect(),
                span: state.0.span,
            });
        }
        nodes
    }

    /// Serialize native coordinates and relationships without opening a classical camera.
    #[must_use]
    pub fn native_data(&self) -> Value {
        let nodes = self.plan();
        json!({"schema": "native-space-retained-state", "version": 1,
            "root": nodes.len() - 1, "nodes": nodes})
    }

    /// Serialize the native state alongside checked classical camera observations.
    #[must_use]
    pub fn to_data(&self) -> Value {
        let nodes = self.plan();
        json!({"schema": "native-space-retained-state", "version": 1,
            "root": nodes.len() - 1, "nodes": nodes,
            "projection": self.project().to_data(), "coordinates": coordinates(self.project())})
    }

    /// Decode a graph and recompute every observation from its retained inputs.
    ///
    /// # Errors
    /// Rejects malformed operators, forward references, invalid parameters,
    /// unreachable nodes, and forged projection or coordinate observations.
    pub fn from_data(data: &Value) -> Result<Self, String> {
        let graph: GraphData =
            serde_json::from_value(data.clone()).map_err(|error| error.to_string())?;
        if graph.schema != "native-space-retained-state" || graph.version != 1 {
            return Err("unsupported retained-state schema or version".into());
        }
        let state = Self::from_steps(&graph.nodes, graph.root)?;
        let expected = if graph.projection.is_some() || graph.coordinates.is_some() {
            state.to_data()
        } else {
            state.native_data()
        };
        if expected != *data {
            return Err("retained graph is noncanonical or its observations do not match".into());
        }
        Ok(state)
    }

    // JSON and binary input share one validator. A wire codec cannot silently
    // accept graph shapes or scalar coordinates that the other codec rejects.
    pub(crate) fn from_steps(nodes: &[Step], root: usize) -> Result<Self, String> {
        let mut states: Vec<Self> = Vec::with_capacity(nodes.len());
        for node in nodes {
            let inputs =
                node.inputs
                    .iter()
                    .map(|index| {
                        states.get(*index).cloned().ok_or_else(|| {
                            "retained inputs must reference an earlier node".to_owned()
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
            let state = match (&node.operator, inputs.as_slice()) {
                (Operation::Scalar { coordinates }, []) => {
                    Self::native_scalar(&Scalar::from_data(coordinates)?)
                }
                (Operation::Add, [left, right]) => left.add(right),
                (Operation::Multiply, [left, right]) => left.multiply(right),
                (Operation::Phase { turns }, [value]) if is_canonical_phase(*turns) => {
                    value.phase(*turns)
                }
                (Operation::Index { direction, depth }, [value]) => {
                    value.index_power(*direction, *depth)?
                }
                (Operation::Camera { from, to }, [value]) => value.camera(*from, *to),
                _ => return Err("invalid retained operator parameters or input count".into()),
            };
            let retained =
                node.retained
                    .iter()
                    .map(|index| {
                        states.get(*index).cloned().ok_or_else(|| {
                            "retained scope must reference an earlier node".to_owned()
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
            states.push(state.retaining(&retained));
        }
        let state = states
            .get(root)
            .ok_or("invalid retained-state root")?
            .clone();
        let replay = state.plan();
        if root.checked_add(1) != Some(nodes.len())
            || replay.len() != nodes.len()
            || replay.iter().zip(nodes).any(|(left, right)| {
                left.operator != right.operator
                    || left.inputs != right.inputs
                    || left.retained != right.retained
            })
        {
            return Err("retained graph is noncanonical or contains unreachable nodes".into());
        }
        Ok(state)
    }

    /// Display retained structure unless an explicit classical output is requested.
    ///
    /// # Errors
    /// Returns the selected camera's diagnostic when the projection is unsuitable.
    ///
    /// # Panics
    /// Panics only if an internal graph invariant omits its required root.
    pub fn output_data(&self, kind: OutputKind) -> Result<Value, String> {
        match kind {
            OutputKind::Auto | OutputKind::Pattern => {
                Ok(json!({"kind": "pattern", "value": self.to_data()}))
            }
            OutputKind::Vector => {
                let fields = field::evaluate::<Scalar>(self, false)?;
                let field = fields.last().expect("native graph has a root");
                let index = crate::core::MultiIndex::default();
                if field.keys().any(|key| *key != index) {
                    return Err(
                        "vector output requires one unindexed scalar; use view for indexed states"
                            .into(),
                    );
                }
                let scalar = field
                    .get(&index)
                    .cloned()
                    .unwrap_or_else(|| Scalar::from_classical(&NativeScalar::zero()));
                let point = coordinates::Point::new(scalar, 0_u8.into());
                Ok(json!({"kind":"vector", "value":point.vector()}))
            }
            _ => crate::core::output_data(self.project(), kind),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Step {
    pub(crate) operator: Operation,
    pub(crate) inputs: Vec<usize>,
    pub(crate) retained: Vec<usize>,
    #[serde(skip)]
    pub(crate) span: Option<Span>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GraphData {
    schema: String,
    version: u64,
    nodes: Vec<Step>,
    root: usize,
    // Deserialized to reject missing/unknown fields; replay verifies the values.
    projection: Option<Value>,
    coordinates: Option<Value>,
}

fn coordinates(projection: &NativeState) -> Value {
    if projection.is_zero() {
        return json!({"depth": Depth::of(&NativeScalar::zero()).to_data(), "phase": null, "terms": []});
    }
    let terms: Vec<_> = projection
        .0
        .iter()
        .map(|(index, value)| {
            // A rational ray gives exact phase without approximating an angle.
            let scale = if value.real.is_zero() {
                value.imag.abs()
            } else {
                value.real.abs()
            };
            json!({"index": index.to_data(), "position": value.to_data(),
            "point": coordinates::Point::new(Scalar::from_classical(value), index.depth(1)).to_data(),
            "depth": Depth::of(value).to_data(),
            "phase": {"real": rational_text(&(&value.real / &scale)),
                "imag": rational_text(&(&value.imag / &scale))}})
        })
        .collect();
    json!({"model":"depth-phase-index", "index_direction":1, "terms": terms})
}

/// Interpret source operations and calls without replacing native states with camera values.
///
/// # Errors
/// Returns a located semantic, expansion, or unsupported-expression diagnostic.
pub fn interpret(program: &Program) -> Result<State, LanguageError> {
    crate::core::interpret_retained(program)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_unobserved(state: &State) {
        let mut pending = vec![state];
        let mut visited = HashSet::new();
        while let Some(state) = pending.pop() {
            if visited.insert(state.key()) {
                assert!(state.0.projection.get().is_none());
                pending.extend(state.inputs());
                pending.extend(state.retained_inputs());
            }
        }
    }

    #[test]
    fn indexed_camera_construction_and_vm_replay_remain_lazy() {
        let program = crate::core::parse(
            "let select = (x) => camera(7, 0, index(7, phase(1, x)))\noutput select(3)",
            "camera.ns",
        )
        .unwrap();
        let direct = interpret(&program).unwrap();
        assert_unobserved(&direct);
        let code = crate::bytecode::compile(&program).unwrap();
        let replay = crate::bytecode::execute_retained(&code).unwrap();
        assert_unobserved(&replay);
        assert!(direct.same_structure(&replay));
        assert!(replay.same_projection(&State::scalar(NativeScalar::from_text("0", "3").unwrap())));
    }

    #[test]
    fn native_construction_serialization_and_vm_replay_do_not_open_a_classical_camera() {
        let program = crate::core::parse(
            "let step = (x) => add(phase(1, multiply(x, 0)), index(4, 1))\noutput step(7)",
            "lazy.ns",
        )
        .unwrap();
        let direct = interpret(&program).unwrap();
        assert_unobserved(&direct);
        let saved = direct.native_data();
        assert_unobserved(&direct);
        let restored = State::from_data(&saved).unwrap();
        assert_unobserved(&restored);
        let vm = crate::bytecode::execute_retained(&crate::bytecode::compile(&program).unwrap())
            .unwrap();
        assert_unobserved(&vm);
        assert!(direct.same_structure(&vm));
        assert_unobserved(&direct);
        assert_unobserved(&vm);
        assert_eq!(direct.project(), &crate::core::interpret(&program).unwrap());
        assert_eq!(direct.native_data(), saved);
    }

    #[test]
    fn reflected_apply_observes_the_program_but_not_its_unused_native_argument() {
        let program = crate::core::parse(
            "let discard = (x) => 1\nlet invoke = (x) => apply(trace(discard), x)\noutput 0",
            "apply.ns",
        )
        .unwrap();
        let input =
            State::scalar(NativeScalar::from_text("7", "0").unwrap()).multiply(&State::zero());
        let result = crate::core::exact_function(&program, "invoke")
            .unwrap()
            .apply_retained(std::slice::from_ref(&input))
            .unwrap();
        assert_unobserved(&input);
        assert!(result.same_projection(&State::one()));
        assert_unobserved(&input);
        assert!(
            result
                .retained_inputs()
                .iter()
                .any(|source| source.same_structure(&input))
        );
    }

    #[test]
    fn repeated_identity_feedback_keeps_scopes_shared_and_linear() {
        let mut state = State::one();
        for _ in 0..10_000 {
            state = state.retaining(std::slice::from_ref(&state));
        }
        let plan = state.plan();
        assert_eq!(plan.len(), 10_001);
        assert_eq!(
            plan.iter().map(|step| step.retained.len()).sum::<usize>(),
            10_000
        );
        let restored = State::from_data(&state.native_data()).unwrap();
        assert!(restored.same_structure(&state));
        assert_unobserved(&restored);
    }
}
