<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# NS is the tool

The foundation is in [THEORY.md](../THEORY.md). This directory contains the
compiler, runtime, CLI, MCP server, and optional GPU backend. Language version
is 1.0. Syntax is documented in [SPEC.md](SPEC.md).

## Retained core: intent and contract

The requirement is to keep what a numerical answer loses. A shared immutable
operation graph therefore owns scalar inputs, ADD, MULTIPLY, PHASE, INDEX,
REFLECT, and call-scope dependencies. Inputs survive cancellation
and unused arguments survive calls. Classical equality does not authorize
deleting this retained structure.

Scalar storage uses exact squared magnitude and a rational phase ray.
Depth is `ln|z| = ln(real²+imag²)/2`; logarithms are not rounded in storage.
The derived cylindrical camera stores a sample index k in transverse coordinates
`(k+1)*ray/length(ray)`; this is not the meaning of INDEX.
Native `as vector` requests this cylindrical view,
not the old quadratic cone. Use an explicitly selected index direction when
viewing arrays; graph addresses are not physical or index coordinates.

This exact carrier covers Gaussian-rational scalar values. Logarithms and
radicals in their coordinate readouts remain symbolic. Arbitrary irrational
scalar inputs require a larger carrier and are rejected here. A zero scalar
can retain a supplied phase ray; a classical zero alone supplies no such ray.
Cancellation operands remain in the graph even when the resulting phase is
undefined.

`phase(0..3, value)` names quarter-turn steps. There is no `orient` alias.
The spelling is shared by source, reflection, and bytecode.
General frame transformations remain separate from cyclic phase.

## Evaluation and cameras

`reflect(subject, pattern, replacement)` matches canonical indexed values,
not construction history. Its retained node contains a precompiled rule and
the original subject. Scaling and phase before the node therefore remain
visible to the matcher. See [the reflection contract](REFLECTION.md) for
capture scope, supported patterns, and shared function-graph execution.

Constructing a graph does not evaluate its arithmetic. Selection is retained as
REFLECT with its subject and rule, and replayed after its inputs. Host coordinate
routing uses this same rule; there is no separate Camera opcode.
Canonical reflection does not match zero-valued terms. Their boundary phase and
history remain in the retained subject, not in the selected result.

`retained::interpret` and `compiled::execute_retained` run the shared function
graph and return full states. `compiled::compile` does not run the program.
`bytecode::lower` separately lowers an executed state for stack replay.
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

## Shared functions

```ns
let f = (a, b) => add(multiply(a, 2), b)
let g = f(3)
output g(4) as number
# 10
```

Calls share one compiled Native graph and keep independent binding environments.
Zero is explicitly present when bound. Function values are Native data, so
`reflect(f, value, value)` can be called normally too; there is no `apply`
primitive. Recursive calls use fresh bindings and bounded heap frames.
[Reflection](REFLECTION.md) explains the design and remaining separate tools.

## Saving and feedback

JSON and [NSBATCH](ARRAY-DATA.md) retain full native graphs, including ordered
operands, scoped inputs, zero leaves, and deferred reflection. Codecs share one
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

## Verification

The standalone NS example collection was removed to keep the foundation small.
Git history preserves it. Example-specific tests were removed with that code;
core runtime tests use focused inline sources and import fixtures instead.
Fixtures test language behavior, not historical mathematical claims.
The separate Rust application demos remain in the Cargo package.

Tests cover exact coordinate/serialization round trips, complex operations,
phase landmarks and wrapping, multi-channel Fourier readouts, derived cameras,
zero provenance, deferred selection, rounded arithmetic, source/VM agreement,
malformed input, and the compiled ALU. These finite checks are not a formal
verification of Rust or a proof of universal numerical/scientific claims.

## Finite Pattern semantics

`pattern::Pattern::new(seed, step)` pairs the retained seed with one existing
fixed-arity FunctionValue having one unbound parameter. Currying and REFLECT
continue to use the same graph and environment; there is no second evaluator.

`pattern.observe(k)` is a lazy selection of `step^k(seed)`. Its representation
contains only the shared generator and exact BigUint index. Observation zero is
the seed; repeated or out-of-order requests do not mutate a cursor. Index is
not reduced modulo phase and is not an instruction address.

Explicit `observation.project(maximum_steps)` calls the shared step k times on
retained Native states, then projects the final result. It does not canonicalize
between calls: a step can encode a partially bound function and inspect the
retained argument structure with REFLECT. Canonical replacement would change G.
The Observation continues to retain only the original seed, step and index;
it never caches a prefix. Temporary replay results can retain earlier inputs
and therefore consume growing memory. This is not a bounded-memory evaluator or
a constant-time jump to k. Step errors leave other selections intact.

A step must return a state, not an unresolved function or argument pack. Generic
source INDEX still shifts a payload direction; observation INDEX is held outside
that payload to avoid collisions with power depth, arrays, and graph metadata.

`retained::coordinates::pattern_projection` derives the current cylindrical
camera from an Observation and retains all payload indices alongside the points.
The separate local playground uses this API for Pattern playback. Its first
frame is the seed, and seeking selects an observation rather than advancing a
hidden cursor. The graph-record inspector remains separate: graph addresses are
not repetition coordinates. No historical camera is restored and no new language
keyword is added.

`observation.evaluate(maximum_steps)` exposes the retained result for ordinary
Native readout functions. It does not canonicalize the input to those functions:
doing so could change reflective readouts. `project` is the explicit canonical
readout of that same evaluation.

Tests in `runtime/tests/pattern.rs` cover zero/first observations, phase cycles,
shared graph identity, curried zero bindings, independent payload depth, large
lazy indices, bounded replay, failed steps, and generator size after repeated
projection.

Sequential consumers may use `pattern.cursor()`. This disposable evaluator holds
only the current retained result and index, outside Pattern. Forward seeks reuse
that result; backward seeks replay the seed. Failed evaluation leaves the cursor
unchanged. It must agree with independent observation evaluation, including retained
provenance, and never becomes the serialized Pattern representation.

`observation.successor()` selects `(P,k+1)` without executing the step or copying
the graph; it works beyond machine-integer indices. `cursor.observation()` returns
the observation actually reached, initially `(P,0)`. A failed seek leaves both
its state and observation unchanged. The cursor's execution budget does not
restrict which indices can be represented lazily. Pattern remains a semantic
pair over the shared graph, not a sixth opcode or a replacement for REFLECT.
