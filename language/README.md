<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# NS is the tool

The foundation is in [THEORY.md](../THEORY.md). This directory contains the
compiler, runtime, CLI, MCP server, and optional GPU backend. Language version
is 1.0. Syntax is documented in [SPEC.md](SPEC.md).

## Retained core: intent and contract

The requirement is to keep what a numerical answer loses. A shared immutable
operation graph therefore owns scalar inputs, ADD, MULTIPLY, PHASE, INDEX,
indexed camera reads, and call-scope dependencies. Inputs survive cancellation
and unused arguments survive calls. Classical equality does not authorize
deleting this retained structure.

Scalar storage uses exact squared magnitude and a rational phase ray.
Depth is `ln|z| = ln(real²+imag²)/2`; logarithms are not rounded in storage.
A sample at integer index k has transverse coordinates
`(k+1)*ray/length(ray)`. Native `as vector` is this cylindrical view,
not the old quadratic cone. Use an explicitly selected index direction when
viewing arrays; graph addresses are not physical or index coordinates.

This exact carrier covers Gaussian-rational scalar values. Logarithms and
radicals in their coordinate readouts remain symbolic. Arbitrary irrational
scalar inputs require a larger carrier and are rejected here. A zero scalar
can retain a supplied phase ray; a classical zero alone supplies no such ray.
Cancellation operands remain in the graph even when the resulting phase is
undefined.

`phase(0..3, value)` names quarter-turn steps. There is no `orient` alias.
The spelling changed across source, reflection, bytecode, and examples.
General frame transformations remain separate from cyclic phase.

## Evaluation and cameras

Constructing a graph does not evaluate its arithmetic. In particular,
`camera(from, to, state)` is retained as a read operation, not replaced by an
eagerly computed constant. The bytecode VM replays that read after its input
graph. This prevents a rounded run from secretly evaluating selected subtrees
exactly. Camera routing is not a new arithmetic primitive.

`retained::interpret` and `bytecode::execute_retained` return full states.
`bytecode::execute` explicitly returns the classical projection.
`same_structure` and `same_projection` intentionally ask different questions.

Normal CPU execution is exact. `run --numeric f64` and `view --numeric f64`
explicitly round each scalar arithmetic operation; their integer indices and
retained graph are unchanged. Rounded result wrappers are not loadable as exact
states. Their `state` member is the original exact graph, not rounded feedback.

Number output displays the requested numerical readout. Boolean/string output
and `check` require exact evaluation. Overflow and underflow-to-zero in f64
are diagnosed with the operation location. A view of every branch evaluates
every branch; an output evaluates only its arithmetic dependencies, not unused
scope records. Source synthesis, reflection decisions, and projected
`expand` export remain exact elaboration tools; the numeric option does not
replace their algorithms. Default compilation still bypasses the old optimizer.

This deliberately trades storage and exact-arithmetic cost for recoverable
structure. It is not a universal optimizer or an arbitrary-range float engine.

## Saving and feedback

JSON and [NSBATCH](ARRAY-DATA.md) retain full native graphs, including ordered
operands, scoped inputs, zero leaves, and deferred cameras. Codecs share one
validator and reject malformed, unreachable, or forged observations. There is
no compatibility decoder. Scalar bytecode stores native coordinates directly,
so a zero-boundary phase is not lost by converting it to a classical zero.

`native_data()` saves without numerical observation; `to_data()` includes
checked classical readouts. Batch separates `results` from reusable `states`.
Source calls and exact CPU batch feedback carry complete states. Deep graph
serialization, comparison, and release are iterative. Scopes share earlier
states instead of repeatedly copying their complete history.

Batch remains exact on CPU. The optional [GPU](GPU.md) remains the documented
signed-32-bit classical backend, with host-retained state. The new f64 option
does not select GPU execution or provide a GPU-resident native graph.

## Examples and verification

[depth-phase-index.ns](../examples/depth-phase-index.ns) demonstrates the default
model. [movement-patterns.ns](../examples/movement-patterns.ns) is a separate,
explicit base-two movement chart with full-cycle rational turns. Its root
example halves supplied coordinates; it is not a general irrational scalar
evaluator. [common-operations.ns](../examples/common-operations.ns) and
[operation-patterns.ns](../examples/operation-patterns.ns) retain source-defined
reference algorithms and their stated domains.

The older executable corpus is kept for import and regression coverage, not
as evidence for broader claims in historical names. Standalone applications
and narrative proof collections remain outside this foundation. Application
Rust examples still in the Cargo package need a separate package cleanup.

Tests cover exact coordinate/serialization round trips, complex operations,
phase landmarks and wrapping, multi-channel Fourier readouts, derived cameras,
zero provenance, deferred selection, rounded arithmetic, source/VM agreement,
malformed input, and the compiled ALU. These finite checks are not a formal
verification of Rust or a proof of universal numerical/scientific claims.
