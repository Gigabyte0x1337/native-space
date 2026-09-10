<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Native Space Language 1.0

## Retained execution update

Normal `run`, native function calls, and batch feedback retain primitive relationships. Bare output and
`as pattern` return the retained graph with its classical projection and exact
multiplicative-depth coordinates. `as number`, `as string`, `as boolean`, and
`as vector` explicitly request projected results. Zero proofs compare classical
projections, not retained history. Default compilation bypasses the old optimizer.
`view FILE --index-direction 1 --turns 0..3` displays depth–phase–index coordinates
without changing the retained state. Depth is natural-log magnitude.
`run` and `view` accept `--numeric exact|f64` (default: exact).
The f64 mode rounds every scalar arithmetic step, retains exact INDEX/graph
storage, and reports overflow/underflow-to-zero. It cannot be used by `check`
or for Boolean/string output. Source synthesis remains exact elaboration.
See [the core contract](README.md#retained-core-intent-and-contract).
The scalar/flat-stack laws below describe the projected arithmetic domain;
they do not permit erasing retained operands.

## Source format

`reflect(subject, pattern, replacement)` selects from the evaluated Native
state, with rule-local captures and precompiled replacement operations.
See [reflection](REFLECTION.md#value-reflection) for the supported pattern
grammar, multiplicity semantics, and distinction from host-side source rewriting.

Native Space 1.0 uses one `.ns` format. It has exact plane expressions,
ordinary `let` functions, operation-sequence functions, and finite Boolean logic.
There are no mandatory semicolons, function keywords, `end` markers, or
summary declarations.

`#` starts a comment outside a string and continues to the end of the line.
Blank lines and optional semicolons do not change meaning.

Rust contains no table of mathematical function names, analytic types, claim
names, theorem schemas, or RH-specific expression shapes.

## Document kinds

One `.ns` file is parsed as exactly one of these document kinds:

| Kind | Recognized form | Meaning |
|---|---|---|
| Exact state | function/operator/binding declarations followed by `output expression` | Evaluate one closed finite native state and apply an output camera |
| Zero proof | function/operator/binding declarations followed by `left = right` | Evaluate the native difference and accept only exact zero |
| Function library | optional `import`, then arrow-form `let name = (...) =>` declarations | Preserve or derive a finite source-function graph |
| Boolean proof | optional `parameter name: bool`, then `prove expression by truth_table` | Exhaustively check one finite propositional formula |

Exact-state and Boolean documents are self-contained. Only function libraries
may import other function libraries. A function library is derivable source;
it is not accepted as a mathematical proof merely because it parses.

## Values, functions and zero proofs

The five executable Native operations are:

```ns
let a = 2
let b = 3
output add(
    add(a, b),
    multiply(a, b),
    phase(1, a),
    index(7, a),
    reflect(index(7, a), index(7, value, depth), value)
)
```

Literals, references, calls, and finite argument packs are language structure,
not additional operations. The AST has one literal form (including zero and one)
and one call form (including named calls and chained partial calls).
INDEX depth names belong only to REFLECT templates.

Numbers are exact rationals; `zero`, `one`, and `scalar(real, imaginary)`
are literal spellings. UTF-8 strings lower to indexed byte literals.
PHASE accepts quarter-turn counts 0 through 3; repeated counts belong in INDEX.
INDEX directions and literal depths are positive unsigned integers. A template
depth name captures the full exact depth, including depths larger than u64.

```ns
let f = (a, b) => add(multiply(a, 2), b)
let g = f(3)
output g(4) as number
# 10
```

Functions share a compiled Native graph. Calls append bindings; bound zero is
distinct from an unbound slot. A function is directly available as Native data,
and reflected function data is callable through the same syntax.
Declarations may be interleaved; value bindings see only earlier value bindings.

```ns
let sum = (head, tail...) => add(head, tail...)
output sum(2, 3, 5)
# 10
```

A final variadic parameter is a finite argument pack. `items...` inserts its
arguments into a call, ADD, or MULTIPLY list. Bare `items` builds
`add(index(1, first), index(2, second), ...)`; positions start at one.
An empty bare pack is zero. Empty ADD/MULTIPLY spreads produce their identities.
There is no new pack operation or value type in Native state.

```ns
let first = (items...) => reflect(items, index(1, value), value)
output first(2, 3, 5)
# 2
```

This shorthand has exactly the semantics of handwritten ADD/INDEX: labels
combine with existing item labels, since INDEX composition is commutative.
It is not a collision-free nested array encoding. Zero inputs keep their retained
construction, but canonical reflection does not distinguish them from absent
contributions. Use an explicit presence encoding when that distinction matters.
Function arguments become their Native graph data and remain callable after
valid routing. Spreading still passes the original arguments, not that encoding.

Camera routing is ordinary reflection:

```ns
let route = (x) => reflect(x, index(7, value, depth), index(9, value, depth))
output route(multiply(3, index(7, 2, 17)))
# index(9, 6, 17)
```

There are no built-in `trace`, `untrace`, `camera`, `length`,
`rank_descent`, `rewrite`, `concat`, or `fold` expressions.
These names may be user-defined functions. Discovery and source rewriting remain
explicit host tools, not hidden REFLECT behavior. See [Reflection](REFLECTION.md).

`left = right` lowers to `add(left, phase(2, right))`.
The checker requires exact canonical zero and agreement after compiled artifact
serialization/reloading. Both executions use the shared executor; this is not
an independent formal verification of Rust or of infinite claims.

## Derived infix operators

Exact-state documents may define binary operators as ordinary functions:

```ns
let subtract = (left, right) => add(left, phase(2, right))

operator "*" = (left, right) => multiply(left, right)
operator "-" = (left, right) => subtract(left, right)

output 10 - 2 * 3
```

Definition order is precedence order: the first declared operator binds most
tightly, later declarations bind more loosely, repeated use at one level is
left-associative, and parentheses override the order. The example outputs `4`.

An operator name is quoted in its declaration, must be nonempty, must contain
no whitespace, and must lex as either one identifier or one punctuation
sequence. Operators lower immediately to ordinary binary function calls; there
is no operator AST node or extra primitive.

One central typed namespace list prevents collisions across functions,
operators, bindings, function parameters, and Boolean parameters:

| Namespace class | Language-owned names |
|---|---|
| Core operations | `add`, `multiply`, `phase`, `index`, `reflect`, `ADD`, `MULTIPLY`, `PHASE`, `INDEX` |
| Exact grammar | `zero`, `one`, `scalar`, `let`, `output`, `as`, `operator`, `import`, `string`, `number`, `vector`, `pattern`, `boolean`, `=>`, `=` |
| Function grammar | `...` |
| Boolean grammar | `parameter`, `bool`, `prove`, `by`, `truth_table`, `true`, `false`, `not`, `and`, `or`, `xor`, `implies`, `iff` |

No user declaration may use a name in that table. Function/operator/binding
collisions and duplicate operators are rejected during parsing; import merging
repeats the duplicate check across files.

## Output

`output expression` returns the result. Automatic output chooses an exact real
number, then a canonical UTF-8 string, otherwise a native pattern. A camera can
be requested explicitly:

```text
output value as string
output value as number
output value as vector
output value as pattern
output value as boolean
```

`as vector` returns the default depth–phase–index coordinates for one
unindexed scalar (k=0). For `3+4i` this is `[ln(25)/2,3/5,4/5]`,
serialized as exact rational/logarithmic/radical expressions. A literal zero
returns a zero-boundary tag for X and null Y/Z because no phase was supplied.
Use `view --index-direction D` for indexed states; other labels remain
retained. The quadratic cone is a separate derived camera, not this output.
Boolean output accepts only exact zero (`false`) or exact one (`true`).
Output cameras do not change the underlying state.

## Source-defined functions

A function library may begin with relative imports:

```ns
import "../language/functions.ns"
```

An import target must end in `.ns` and must itself be a function library.
Resolution is relative to the importing file. Each canonical file is loaded
once; cycles and duplicate function names are errors. Absolute paths are
rejected so a proof module remains portable. `check`, `inspect`, and `compile`
resolve imports from their input file. `derive --source FILE FUNCTION` expands
the merged library and preserves the original file location of every step.

[functions.ns](functions.ns) contains only generic source functions.
Domain-specific functions belong in user source modules, not the runtime.
Functions use the same `let`/arrow shape:

```ns
let axis_subtract = (left, right) =>
left()
right()
PHASE(2)
ADD()
```

The operation-derivation function-library document kind has its older, narrower
variadic form: it appends `...` to its one dynamic-function sequence parameter:

```ns
let derive_all = (functions...) =>
functions()
```

This `derive_all` pack invokes supplied function names to record their primitive
operation traces. It is distinct from the exact-state value pack defined
earlier, although both preserve finite source argument order and use the same
`...` spelling.

The function parser recognizes only generic calls and the four core operations:
ADD, MULTIPLY, PHASE, and INDEX. The next `let` or end-of-file ends the body.
Expansion erases ordinary calls. A reference to a function already active on
the current path closes a finite pattern graph and is recorded as a pattern
reference. It is not expanded again, rejected, or treated as a fifth
operation. Unknown calls and wrong arity remain located errors. There is no
proof-status instruction in the language. Loading, checking, inspecting, or
compiling a complete library validates every body, including bodies not reached
by a requested derivation.

```ns
let quarter_turn_pattern = () =>
PHASE(1)
quarter_turn_pattern()
```

Saving that source as `pattern.ns` and running
`derive --source pattern.ns quarter_turn_pattern` therefore
reports one finite source operation and one self-reference. Direct, mutual,
empty, and argument-carrying self-references use the same graph rule.
`primitive_steps` is the operation listing for one finite graph traversal in
source order; `pattern_references` says where that graph models its next
observation through itself. It is not a materialized unbounded execution log.

Operation-function libraries and exact-state expressions answer different
questions. A library may retain self-reference as pattern structure. An
exact-state output or zero proof must produce a result within the runtime budget.
An uncalled recursive function is a finite graph value; calling it may terminate
or exhaust that budget. There is no trace-only exception.

The current foundation is in `THEORY.md`; historical proof ledgers are outside
this repository. Executable zero checks end in `= 0` or use the finite Boolean checker.

## Finite Boolean logic

Finite Boolean proofs use `parameter name: bool` followed by
`prove expression by truth_table`. Semicolons are unnecessary. The generic
operators are `not`, `and`, `or`, `xor`, `implies`, and `iff`. Exhaustive
valuation accepts tautologies and returns a concrete counterexample otherwise.
This checker has no analytic or number-theory predicates.

## Compilation

The compiler emits a `native-space-program` artifact: Native strand records,
the source location, goal and output camera. It does not execute function
bodies, discovery or reflection while compiling.

Every function is encoded once with parameter slots, references and call
edges. Loading validates records and builds an address index over this same
graph. Function calls append Native bindings; a complete signature evaluates.
The graph stays shared across partial, repeated, nested and recursive calls.
See [Reflection](REFLECTION.md) for the binding schema and execution limits.

The source executor, CLI and exact CPU data host use this graph runtime.
Canonical REFLECT rules compile from the record view, not a reconstructed AST.
Arithmetic constructs retained states, with numerical projection deferred until
needed. Scalar leaves keep exact squared magnitude and any zero-boundary ray.

`bytecode::lower` is a separate, explicit operation: execute a document and lower
its closed retained result for stack replay. It preserves operation/scope edges
but is not the program compiler. The old `bytecode::compile` API is removed.

`Artifact::function(name)` selects a shared function from a loaded program for
repeated host input. `bytecode::lower_state` can replay each retained result
without recompiling the function or rerunning its body.
The optional GPU lowering remains a separate restricted backend.

A classical observation combines signed coefficients at the same INDEX
location and removes zero terms. The native graph retains those contributions.
`expand` exports elaborated source, not serialized native call-scope history.
Use native state JSON or binary output for full-state feedback.

## Complete-data execution host

The ordinary `run` command may pass every root item from one JSON or `NSBATCH`
file to one selected source function:

```text
native-space run model.ns --data observations.json --function train
```

Items are loaded once, retained as complete exact native states, and supplied
as function arguments in file order. A trailing variadic parameter receives
the complete finite pack. Rust does not choose a context width, model shape,
feature, update rule, selector, or output interpretation. Those decisions are
ordinary source functions. Array lowering uses the same exact coordinate rule
documented below.

The host does not chunk, independently map, or reset the data. Ordered state
transitions must be expressed by ordinary calls or supplied by the batch host. The source defines field layout, indexed coordinates, accumulation,
and output interpretation.

## Batch execution host

Batch execution is deliberately not language syntax. The CLI accepts an
exact-state source file, a unary source-function name, an ordered JSON or
version-2 binary data file, a nonnegative step count, and an explicit `cpu` or
`gpu` backend:

```text
native-space batch program.ns --function step --data data.json --steps 3 --backend gpu
```

Each data point starts with its own input value and applies the selected
function exactly `steps` times. Those applications are sequential for that
point. Backends may distribute only distinct points, and output order must
equal input order.

The next step receives the full native state, never just its classical answer.
Output contains `results` for readable camera observations and `states` for
native feedback. A saved batch output can be passed directly to `--data`, which
uses its `states` field. Passing only `results` is an explicit lossy input choice.

The CPU backend evaluates the ordinary exact Native Space state semantics on a
bounded number of workers. The GPU backend is a separate exact target for real
signed-32-bit scalar inputs and constants with ADD, MULTIPLY, and even PHASE
turns. Generated shader operations carry explicit overflow flags. Any overflow
or unsupported state or operation fails the complete batch; no wrapped,
floating-point, or CPU-fallback result is emitted. The GPU step limit is
1,000,000 per point to keep one dispatch explicitly bounded.
The host constructs retained graphs while the GPU computes those scalar
observations. REFLECT and indexed states are outside that restricted GPU target.

GPU support is an additive Cargo feature and is disabled by default. Building
with `--features gpu` enables the optional `wgpu` and `bytemuck` dependencies.
The Rust API and `gpu` backend name remain available in CPU-only builds; an
attempted GPU run returns `NSG001` and never falls back to CPU. Official release
binaries enable the feature. See [`GPU.md`](GPU.md) for the build invariant.

The JSON root is an ordered batch, a saved batch output with `states`, or one
native-state object. An item may be a serialized native graph, an exact real rational
string, a scalar object with exact `real` and `imag` strings, a canonical
`flat-stack-v1` state object, or a nonempty rectangular array of exact scalar
leaves with rank at most 64. For each leaf, the array axis is its INDEX
direction and the 1-based position on that axis is its INDEX nesting depth:

```text
[x, y]
-> add(index(1, x), index(1, index(1, y)))

[[a, b], [c, d]]
-> add(
     index(1, index(2, a)),
     index(1, index(2, index(2, b))),
     index(1, index(1, index(2, c))),
     index(1, index(1, index(2, index(2, d))))
   )
```

This mapping distinguishes transposed positions despite commutative INDEX
composition. Empty, ragged, and mixed-rank arrays are rejected. The array shape
is retained only so the host can print cancelled coordinates as zero instead
of losing the requested display shape. It is not part of the native state and
does not add a value kind or operation to Native Space 1.0. The CPU backend
accepts these indexed states. The current GPU target rejects them because its
proved exact domain is scalar.

`pack-data INPUT.json OUTPUT.nsb` validates and stores the full native graphs
and host shapes in versioned `NSBATCH` binary form. `batch --data` detects JSON
or binary from the content. The binary decoder is strict: unsupported versions,
malformed lengths, invalid exact coefficients, noncanonical graphs, shape/index
mismatches, and trailing bytes are errors. Its complete layout and design
invariants are recorded in [`ARRAY-DATA.md`](ARRAY-DATA.md).
Dense readable output is limited to one million elements; larger shapes keep
their metadata and exact state, using sparse/scalar classical output instead.

## Classical frequency synthesis host

`frequency` is a Rust host command, not Language 1.0 syntax:

```text
native-space frequency observations.ns --samples 16 --maximum-error 1e-12
```

The command evaluates the exact source and then applies the explicitly lossy
`classical-complex-f64` camera to a finite consecutive INDEX window. It selects
a deterministic projected-power prefix of finite frequency modes and accepts
the generated replay program only when every projected sample is within the
declared absolute error. The emitted schema records both the bound and observed
maximum error.

This contract is camera-relative finite agreement. It is not native equality,
symbolic function equivalence, or continuation beyond the sampled window. The
bounded algorithm and performance tradeoff are specified in `FREQUENCY.md`.
- `zero_fill_axes(parts...)` is an ordinary generic source-defined function,
  not a hard-coded perspective. A caller passes its own axis declaration and
  residual pattern immediately before comparison with zero. Every declared
  axis absent from that residual is exact zero. Axes are used separately inside
  their owner perspective. `classical_perspective` is the direct generic
  caller; `zeta_classical_pattern` and `re_classical_pattern` both call that
  source function for their local coefficient readout. This does not identify
  their geometric cameras. The separate 3D position and rotation transform is
  defined and proved outside the language kernel.
- Operation traces expand from `.ns` source, never a Rust catalog.
- Recursive source traces are finite graphs. Their pattern references retain
  the call location, argument flow, and complete closing function path.
- Perspective equivalence belongs to exact indexed states, not flattened
  primitive listings. A vertex path retains an outer INDEX for vertex order
  and inner INDEX coordinates for each vertex. Source-defined cameras act on
  those coordinates before projection; complementary cameras reconstruct each
  vertex at the same outer INDEX. A raw derivation trace may not be used to
  infer camera inequivalence merely because a projected path hides an INDEX.
- Boolean proofs compile to recomputable truth-table certificates.
- Parser and structural diagnostics retain a source file and location whenever
  the failing token has one. A failed final zero check currently reports the
  nonzero result at document level; primitive provenance for that failure is
  not yet implemented.

The explicit projection optimizer uses only these theorem-authorized rules.
Normal program compilation does not run them. Every emitted optimizer
event carries the listed dependency-ledger theorem ID, and the test suite
executes the full allowlist while comparing original and optimized states.

| Rule | Rewrite | Authority |
|---|---|---|
| `OPT-ADD-FLATTEN-1` | Flatten nested ADD | `L-NS-2` |
| `OPT-ADD-ZERO-1` | Remove additive zero | `L-NS-2` |
| `OPT-MUL-ZERO-1` | Replace a product containing zero by zero | `L-NS-8` |
| `OPT-MUL-FLATTEN-1` | Flatten nested MULTIPLY | `L-NS-5` |
| `OPT-MUL-ONE-1` | Remove multiplicative one | `L-NS-6` |
| `OPT-PHASE-NORMALIZE-1` | Reduce the sum of combined canonical turns modulo four | `L-SEP-5` |
| `OPT-PHASE-IDENTITY-1` | Remove a zero-turn PHASE | `L-SEP-5` |
| `OPT-PHASE-COMBINE-1` | Combine nested PHASE turns | `L-SEP-5` |

## Command line

All commands use the same Native Space 1.0 parser:

| Command | Input | Result |
|---|---|---|
| `native-space run FILE [--numeric exact\|f64]` | State document | Evaluate and print its selected output camera; exact by default |
| `native-space view FILE --index-direction 1 --turns 1 [--numeric exact\|f64]` | State document | Show cylindrical branch coordinates in the selected frame with the native source retained |
| `native-space run FILE --data DATA --function FUNCTION` | Exact-state document plus one complete ordered data file | Pass every root data item to the selected source function once and print its result |
| `native-space check FILE` | Any document kind | Validate it; execute zero/Boolean checks when present |
| `native-space inspect FILE` | Any document kind | Print its schema-1 parsed representation |
| `native-space expand FILE` | Exact-state document | Print generated pure source after calls, packs, concat, and reflective forms are lowered |
| `native-space compile FILE` | Any document kind | Emit a Native program, function-library artifact, or Boolean certificate |
| `native-space untrace [--input] FILE [--rank R]` | Exact ordered observations | Return an exact deterministic continuation or ranked relationship-frequency pattern; rank defaults to one |
| `native-space rank-descent [--input] FILE [--strategy adaptive\|linear]` | Exact ordered observations | Generate a data-sized rank-one reference and return the lowest fully matching candidate tested by the selected finite schedule |
| `native-space frequency FILE --samples N --maximum-error E` | Exact indexed state | Synthesize and verify one finite lossy classical-frequency replay program |
| `native-space derive FUNCTION [ARGS...]` | Bundled `language/functions.ns` | Expand one generic source function |
| `native-space derive --source FILE FUNCTION [ARGS...]` | Explicit function library and imports | Expand one function from that library |
| `native-space derive --json ...` | Either derive form | Emit the complete machine-readable derivation report |
| `native-space mcp` | Standard input/output | Serve operation derivation; the tool accepts an optional working-directory-confined relative `.ns` source path |

The no-`--source` derive form intentionally sees only the bundled generic
library. Functions in user libraries require the explicit `--source` argument.

Version 1.0 has no hidden analytic evaluator, privileged zeta/RH function,
specialized claim type, loop, mutation, floating-point value, materialized
infinite state, or automatic proof of a paper theorem.
