<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Native Space Language 1.0

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
| Exact state | bindings/operators followed by `output expression` | Evaluate one closed finite native state and apply an output camera |
| Zero proof | bindings/operators followed by `left = right` | Evaluate the native difference and accept only exact zero |
| Function library | optional `import`, then arrow-form `let name = (...) =>` declarations | Preserve or derive a finite source-function graph |
| Boolean proof | optional `parameter name: bool`, then `prove expression by truth_table` | Exhaustively check one finite propositional formula |

Exact-state and Boolean documents are self-contained. Only function libraries
may import other function libraries. A function library is derivable source;
it is not accepted as a mathematical proof merely because it parses.

## Values and zero proofs

The exact expression forms are numbers, UTF-8 strings, references, calls,
`trace(function)`, `untrace(value[, maximum_error_ratio])`, and the four operations:

```text
add(a, b, ...)
multiply(a, b, ...)
orient(turns, value)
index(direction, value)
```

`zero`, `one`, and `scalar(real, imaginary)` remain readable exact constants.
Strings lower to ADD/INDEX byte-position patterns. Numbers are exact integers
or rationals; there is no floating-point equality.

Functions are ordinary `let` values:

```ns
let Re = (x) => x
output Re(1)
```

`trace` observes source structure. It receives a source-function name and
returns that function's complete reachable source graph as an ordinary native
state:

```ns
let quarter_step = (value) => add(index(7, value), orient(1, value))
output trace(quarter_step) as pattern
```

The returned value is a nested **operation strand**, not a flat instruction
array. If $h$ is one instruction coordinate and $t$ is the remaining strand,
one link is

$$
\mathrm{Node}(h,t)=
\mathrm{ADD}
\left(
\mathrm{INDEX}_{H}(h),
\mathrm{INDEX}_{C}(t)
\right).
$$

Repeated continuation indexing records exact chain position. Every instruction
coordinate retains its kind, arguments, source span, function name, and call
edges. The four operation identities are encoded by the four orientations in
the opcode coordinate: ADD at turn 0, MULTIPLY at turn 1, ORIENT at turn 2,
and INDEX at turn 3. Constants, parameters, and calls remain explicitly tagged
coordinates because deleting them would make reconstruction impossible.

Each transitively called function is encoded once. A direct or mutual recursive
call therefore remains a finite call edge to an already encoded function. Such
a function may be observed with `trace`; executing the recursive call as a
closed exact state remains invalid because Language 1.0 does not perform
unbounded unfolding. `trace` is deterministic and immutable. It is a
reflective camera whose result lowers entirely to exact constants and the four
core operations, not a fifth algebra operation.

`untrace(value)` synthesizes an exact continuation program from indexed
observations. `untrace(value, maximum_error_ratio)` permits that exact fraction
of held-out observation positions to differ from the generated continuation.
The ratio is an exact rational number from zero through one; zero is the
default.

Language 1.0 searches homogeneous constant-coefficient linear recurrences over
complete native states. For order $r$, the first $r$ states are seeds, positions
$r$ through $2r-1$ determine exact native-scalar coefficients shared by every
coordinate, and at least one later supplied state remains held out. The
candidate recursively generates its own complete states from the seeds through
the last supplied position. An error is one held-out position where those
states differ. Candidates outside the declared error ratio are rejected.
Selection first minimizes the exact error ratio, then seed nodes plus
recurrence-expression nodes; source length, order, and source text are
deterministic tie-breakers.

The compact scalar source layout remains valid: each one-depth INDEX direction
is an observation position, and missing directions inside the retained span are
zero observations. For structured source values, INDEX direction 1 is the
sequence axis and its depth is the 1-based observation position; all remaining
INDEX coordinates are retained as that observation's payload.

```ns
let observations = () =>
add(index(1, 1), index(2, 1), index(3, 2), index(4, 3), index(5, 5), index(6, 8), index(7, 13))

output untrace(observations()) as pattern
```

One tolerated mismatch is written explicitly:

```ns
let observations = () =>
add(index(1, 1), index(2, 1), index(3, 2), index(4, 3), index(5, 5), index(6, 8), index(7, 13), index(8, 21), index(9, 35))

output untrace(observations(), 1/5) as pattern
```

The result is the operation strand for an order-two continuation with
`next = add(previous_1, previous_2)`, a scalar position advanced by ADD one,
and one recursive call edge. Existing operation strands are fixed points, so
repeating `untrace` does not synthesize a program about program metadata.
Generated source reports the permitted and actual exact error ratios and every
mismatching index. It predicts from generated state, never by feeding an erroneous supplied
observation back into the continuation.

The CLI may read scalar observations from CSV columns `index,value` or
`index,real,imag`. `--output pattern-csv` emits a compact table for scalar
models only. JSON and version-1 `NSBATCH` input instead treat every root item as
one complete ordered observation state. The runtime reads the entire file into
memory and preserves the sequence as one synthesis state: it does not chunk,
reset, flatten, or project the observations through the lossy frequency camera.
These are host interchange forms; they add no syntax or operation to the
language.

All supplied file observations participate as training or held-out evidence.
The searched recurrence order is bounded to 32 to bound exact elimination; the
input length itself is not capped at 64. Failure to find a candidate inside the
declared error ratio is a normal diagnostic, not permission to emit an
overfitted lookup. The result is minimal only inside this documented recurrence
grammar. Predictions beyond supplied positions remain experiments.

The only mathematical proof form is equality to zero. The parser lowers

```text
left = right
```

to `add(left, orient(2, right))` and runs the same exact zero checker. It
accepts only when the direct evaluator and bytecode VM agree and the canonical
state is zero. There is no separate equality claim or theorem-name switch.

## Derived infix operators

Exact-state documents may define binary operators as ordinary functions:

```ns
let subtract = (left, right) => add(left, orient(2, right))

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
is no operator AST node or fifth primitive.

One central typed namespace list prevents collisions across functions,
operators, bindings, function parameters, and Boolean parameters:

| Namespace class | Language-owned names |
|---|---|
| Core operations | `add`, `multiply`, `orient`, `index`, `ADD`, `MULTIPLY`, `ORIENT`, `INDEX` |
| Exact grammar | `zero`, `one`, `scalar`, `trace`, `untrace`, `let`, `output`, `as`, `operator`, `import`, `string`, `number`, `pattern`, `boolean`, `=>`, `=` |
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
output value as pattern
output value as boolean
```

Boolean output accepts only exact zero (`false`) or exact one (`true`). Output
cameras do not change the underlying state.

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
Mathematical functions such as the prime-counting camera `pi` (meaning
$\pi(n)$, not the circle constant) and the zeta cameras belong in
`../examples/`. Functions use the same `let`/arrow shape:

```ns
let axis_subtract = (left, right) =>
left()
right()
ORIENT(2)
ADD()
```

A variadic function appends `...` to its one sequence parameter:

```ns
let apply = (functions...) =>
functions()
```

The function parser recognizes only generic calls and the four core operations:
ADD, MULTIPLY, ORIENT, and INDEX. The next `let` or end-of-file ends the body.
Expansion erases ordinary calls. A reference to a function already active on
the current path closes a finite pattern graph and is recorded as a pattern
reference. It is not expanded again, rejected, or treated as a fifth
operation. Unknown calls and wrong arity remain located errors. There is no
proof-status instruction in the language. Loading, checking, inspecting, or
compiling a complete library validates every body, including bodies not reached
by a requested derivation.

```ns
let quarter_turn_pattern = () =>
ORIENT(1)
quarter_turn_pattern()
```

`derive --source examples/recursive-pattern.ns quarter_turn_pattern` therefore
reports one finite source operation and one self-reference. Direct, mutual,
empty, and argument-carrying self-references use the same graph rule.
`primitive_steps` is the operation listing for one finite graph traversal in
source order; `pattern_references` says where that graph models its next
observation through itself. It is not a materialized unbounded execution log.

Operation-function libraries and exact-state expressions answer different
questions. A library may retain self-reference as pattern structure. An
exact-state `output` or zero proof must instead produce one closed finite
native state; every function path it executes must be acyclic. A function used
only as the target of `trace` may contain self-reference because `trace`
returns its finite source graph rather than unfolding it into a supposed final
state.
Definitions, theorems, conjectures, and open obligations live in the Markdown
dependency ledger. Executable mathematical proofs end in `= 0` or use the
finite Boolean checker.

## Finite Boolean logic

Finite Boolean proofs use `parameter name: bool` followed by
`prove expression by truth_table`. Semicolons are unnecessary. The generic
operators are `not`, `and`, `or`, `xor`, `implies`, and `iff`. Exhaustive
valuation accepts tautologies and returns a concrete counterexample otherwise.
This checker has no analytic or number-theory predicates.

## Compilation

- Exact functions are evaluated directly and independently erased before
  bytecode generation. Reflection and call erasure observe the original source
  graph before theorem-authorized optimizer rewrites.
- `trace(function)` is lowered first to its nested operation-strand expression.
  The ordinary compiler and VM then process only exact constants and core
  operations; bytecode has no hidden trace opcode.
- `untrace(value[, maximum_error_ratio])` is staged after source calls and traces are
  lowered. Error-bounded exact recurrence synthesis returns a recursive operation strand, which is then
  lowered to ordinary coordinates; bytecode has no hidden untrace opcode.
- Bytecode retains the source goal and selected output camera. One `INDEX`
  instruction carries both the positive direction and literal multiplicity;
  composing instructions accumulates depth as an arbitrary-size exact natural
  number.
- Exact results use the canonical finite flat-stack state. ADD automatically
  combines signed coordinates at each retained INDEX location, removes every
  zero term, and therefore emits only the nonzero residual. It never combines
  distinct INDEX locations or different camera perspectives implicitly.

## Batch execution host

Batch execution is deliberately not language syntax. The CLI accepts an
exact-state source file, a unary source-function name, an ordered JSON or
version-1 binary data file, a nonnegative step count, and an explicit `cpu` or
`gpu` backend:

```text
native-space batch program.ns --function step --data data.json --steps 3 --backend gpu
```

Each data point starts with its own input value and applies the selected
function exactly `steps` times. Those applications are sequential for that
point. Backends may distribute only distinct points, and output order must
equal input order.

The CPU backend evaluates the ordinary exact Native Space state semantics on a
bounded number of workers. The GPU backend is a separate exact target for real
signed-32-bit scalar inputs and constants with ADD, MULTIPLY, and even ORIENT
turns. Generated shader operations carry explicit overflow flags. Any overflow
or unsupported state or operation fails the complete batch; no wrapped,
floating-point, or CPU-fallback result is emitted. The GPU step limit is
1,000,000 per point to keep one dispatch explicitly bounded.

GPU support is an additive Cargo feature and is disabled by default. Building
with `--features gpu` enables the optional `wgpu` and `bytemuck` dependencies.
The Rust API and `gpu` backend name remain available in CPU-only builds; an
attempted GPU run returns `NSG001` and never falls back to CPU. Official release
binaries enable the feature. See [`GPU.md`](GPU.md) for the build invariant.

The JSON root is one ordered batch. An item may be an exact real rational
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

`pack-data INPUT.json OUTPUT.nsb` validates and stores the same sparse states
and host shapes in versioned `NSBATCH` binary form. `batch --data` detects JSON
or binary from the content. The binary decoder is strict: unsupported versions,
malformed lengths, invalid exact coefficients, noncanonical terms, shape/index
mismatches, and trailing bytes are errors. Its complete layout and design
invariants are recorded in [`ARRAY-DATA.md`](ARRAY-DATA.md).

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

The compiler runs only these theorem-authorized optimizer rules. Every emitted
event carries the listed dependency-ledger theorem ID, and the test suite
executes the full allowlist while comparing original and optimized states.

| Rule | Rewrite | Authority |
|---|---|---|
| `OPT-ADD-FLATTEN-1` | Flatten nested ADD | `L-NS-2` |
| `OPT-ADD-ZERO-1` | Remove additive zero | `L-NS-2` |
| `OPT-MUL-ZERO-1` | Replace a product containing zero by zero | `L-NS-8` |
| `OPT-MUL-FLATTEN-1` | Flatten nested MULTIPLY | `L-NS-5` |
| `OPT-MUL-ONE-1` | Remove multiplicative one | `L-NS-6` |
| `OPT-ORIENT-NORMALIZE-1` | Reduce turns modulo four | `L-SEP-5` |
| `OPT-ORIENT-IDENTITY-1` | Remove a zero-turn ORIENT | `L-SEP-5` |
| `OPT-ORIENT-COMBINE-1` | Combine nested ORIENT turns | `L-SEP-5` |

## Command line

All commands use the same Native Space 1.0 parser:

| Command | Input | Result |
|---|---|---|
| `native-space run FILE` | Exact-state document | Evaluate and print its selected output camera |
| `native-space check FILE` | Any document kind | Validate it; execute zero/Boolean checks when present |
| `native-space inspect FILE` | Any document kind | Print its schema-1 parsed representation |
| `native-space compile FILE` | Any document kind | Emit bytecode, a function-library artifact, or a Boolean certificate |
| `native-space frequency FILE --samples N --maximum-error E` | Exact indexed state | Synthesize and verify one finite lossy classical-frequency replay program |
| `native-space derive FUNCTION [ARGS...]` | Bundled `language/functions.ns` | Expand one generic source function |
| `native-space derive --source FILE FUNCTION [ARGS...]` | Explicit function library and imports | Expand one function from that library |
| `native-space derive --json ...` | Either derive form | Emit the complete machine-readable derivation report |
| `native-space mcp` | Standard input/output | Serve operation derivation; the tool accepts an optional working-directory-confined relative `.ns` source path |

The no-`--source` derive form intentionally sees only the bundled generic
library. Mathematical functions in `examples/math-functions.ns` always require
the explicit `--source` argument.

Version 1.0 has no hidden analytic evaluator, privileged zeta/RH function,
specialized claim type, loop, mutation, floating-point value, materialized
infinite state, or automatic proof of a paper theorem.
