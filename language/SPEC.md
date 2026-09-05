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
| Exact state | function/operator/binding declarations followed by `output expression` | Evaluate one closed finite native state and apply an output camera |
| Zero proof | function/operator/binding declarations followed by `left = right` | Evaluate the native difference and accept only exact zero |
| Function library | optional `import`, then arrow-form `let name = (...) =>` declarations | Preserve or derive a finite source-function graph |
| Boolean proof | optional `parameter name: bool`, then `prove expression by truth_table` | Exhaustively check one finite propositional formula |

Exact-state and Boolean documents are self-contained. Only function libraries
may import other function libraries. A function library is derivable source;
it is not accepted as a mathematical proof merely because it parses.

## Values and zero proofs

The exact expression forms are numbers, UTF-8 strings, references, calls,
`trace(function)`, `length(operation_strand)`,
`untrace(value[, rank])`, `rank_descent(value[, target_rank[, minimum_agreement]])`, `apply(rank_descent(...), position)`, derived `concat`, finite `fold`, the
indexed `camera`, and the four operations:

```text
add(a, b, ...)
multiply(a, b, ...)
orient(turns, value)
index(direction, value)
```

`orient` accepts only the canonical turns `0`, `1`, `2`, and `3`. Four turns
are derived from MULTIPLY as the identity; the source interface never silently
reduces a larger or negative count. A repeated count belongs in INDEX while
ORIENT retains only its position in the four-state cycle.

`zero`, `one`, and `scalar(real, imaginary)` remain readable exact constants.
Strings lower to ADD/INDEX byte-position patterns. Numbers are exact integers
or rationals; there is no floating-point equality.

Functions are ordinary `let` values:

```ns
let Re = (x) => x
output Re(1)
```

Function, operator, and state-binding declarations may be interleaved before
the final output or zero equality. Binding values still see only bindings
declared earlier in source order.

The final function parameter may be a variadic source pack. The pack is not a
runtime array or native value. Writing `values...` inside an argument or
operand list forwards its finite source expressions:

```ns
let parameters = (head, tail...) =>
concat(9, head, tail...)

output parameters(2, 3, 5) as pattern
```

`concat(direction, values...)` is a transparent derived form. After pack
substitution, value $v_j$ at one-based source position $j$ becomes
$\mathrm{INDEX}_{direction}^j(v_j)$, and ADD joins the generated terms. The
direction is explicit because callers must choose an axis that is fresh when
they require position-preserving storage. Using an axis already present in a
value invokes ordinary INDEX-depth addition and may merge terms.

`concat` requires at least one value after expansion. A spread may also supply
operands to ADD or MULTIPLY; an empty pack then lowers to their existing zero
or one identity. Packs may be forwarded through variadic source calls, but a
pack cannot be used as an ordinary value or spread under ORIENT or INDEX.

The compiler erases calls and packs and expands concat to ADD/INDEX before
bytecode. `native-space expand FILE` prints that generated pure `.ns` source.
No pack, spread, or concat opcode exists.

`fold(function, initial, values...)` applies one ordinary two-parameter source
function from left to right over a finite expression pack. It is exactly the
nested source expression
`function(function(initial, first), second)` and returns `initial` when a
spread supplies no values. The selected function must be nonvariadic and have
exactly two parameters. Static folds are expanded before bytecode, so no fold
opcode exists.

`camera(from_direction, to_direction, value)` is the explicit indexed-state
camera needed to unpack model fields and move coordinates between perspectives.
For each term containing `from_direction`, it removes that direction and moves
its complete depth to `to_direction`. Destination zero unwraps the direction.
Terms without the source direction are discarded; collisions after remapping
combine by ordinary ADD. The source direction must be positive and the
destination is a nonnegative literal.

The camera is generally lossy because it can discard or merge coordinates. A
closed finite camera expression is evaluated and lowered back to exact
constants plus ADD/INDEX before bytecode. A source function receiving host data
uses the same exact finite camera directly. `trace(function)` records both fold
and camera source nodes, so a model's transformation remains inspectable.

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

`length(operation_strand)` moves the finite continuation extent of a canonical
trace onto INDEX direction 1. If the strand has $n$ instruction coordinates,
including its trace-start coordinate, the result is

$$
L(S)=\mathrm{INDEX}_1^n(\mathrm{ONE}).
$$

It counts trace coordinates: function boundaries, parameters, constants,
references, calls, and expression nodes. Source names and spans decorate those
coordinates but do not create additional instruction positions. The camera is
lowered before bytecode, so its result contains only ONE and INDEX. It rejects
ordinary states and malformed strands rather than treating arbitrary sparse
extent as program length.

MULTIPLY composes these unary lengths because INDEX depths add. Therefore a
strictly positive witness $k$ proves one traced program shorter than another
without adding an ordering primitive:

$$
L(P)\mathbin{\mathrm{MULTIPLY}}\mathrm{INDEX}_1^k(\mathrm{ONE})=L(Q),
\qquad k\geq1.
$$

This is instruction-coordinate length, not byte size, runtime, semantic
complexity, or a proof that a program is globally minimal.

`untrace(value)` discovers one of two pattern modes. It first searches
homogeneous constant-coefficient linear recurrences over complete native
states. For order $r$, the first $r$ states are seeds, positions $r$ through
$2r-1$ determine exact native-scalar coefficients shared by every coordinate,
and at least one later supplied state remains held out. A deterministic
candidate is accepted only when it recursively regenerates every held-out
state exactly.

If no supported exact recurrence exists, relationship mode assigns one exact
identifier to each distinct complete observation state. For every earlier and
later input pair $(i,j)$ with $i<j$, it counts the channel

$$
(s(X_i),s(X_j),j-i).
$$

The positive distance keeps local and long-range relationships separate. The
result also carries the complete exact counted symbol dictionary. `untrace(value)` uses
rank one and retains every distinct channel. `untrace(value, rank)` accepts an
exact number from zero through one and retains
$\lceil rank\cdot m\rceil$ of the $m$ channels, ordered by descending exact
count and then channel coordinates. Rank does not relax deterministic equality.

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

One ranked relationship view is written explicitly:

```ns
let observations = () =>
add(index(1, 1), index(2, 1), index(3, 2), index(4, 3), index(5, 5), index(6, 8), index(7, 13), index(8, 21), index(9, 35))

output untrace(observations(), 1/5) as pattern
```

The first example returns an operation strand for the exact order-two
continuation `next = add(previous_1, previous_2)`. The second has no supported
exact continuation and returns the strongest one fifth of its relationship
channels.

An operation strand selects a separate exact instruction mode. At rank one,
`untrace(trace(function), 1)` decodes the complete nested instruction and call
graph, applies every theorem-authorized optimizer occurrence, and reconstructs
the graph as ordinary native coordinates. The candidate replaces the original
only when its complete instruction-coordinate length is lower. The decoder
preserves parameters, dependency edges, function calls, recursion edges,
source locations, and operation arguments; it does not treat opcodes as an
unordered histogram.

Instruction ranks below one currently fail with `NSI002`. Removing a fraction
of instruction relationships could produce a malformed or behaviorally
different program, and no exact reconstruction theorem for that operation is
implemented. Rank-one instruction optimization is minimal only under the
finite optimizer-rule allowlist below; it is not a claim of global program
minimality. Repeating `untrace` on a generated continuation with no applicable
rewrite remains a fixed point.

The CLI may read scalar observations from CSV columns `index,value` or
`index,real,imag`. `--output pattern-csv` emits a compact table for scalar
models only. JSON and version-1 `NSBATCH` input instead treat every root item as
one complete ordered observation state. The runtime reads the entire file into
memory and preserves the sequence as one synthesis state: it does not chunk,
reset, flatten, or project the observations through the lossy frequency camera.
These are host interchange forms; they add no syntax or operation to the
language.

All supplied file observations participate. Recurrence order is bounded to 32
to bound exact elimination. Relationship construction is quadratic because
distance is retained and therefore has an explicit two-million-pair budget.
Rank filters only after exact counts are constructed. Deterministic predictions
beyond supplied positions remain experiments. The `untrace` result alone makes
no correctness claim about an unseen next value; the canonical finite replay
rule below only defines what this retained pattern itself generates.

Relationship replay is a separate deterministic host rule over the emitted
native pattern. The first observed symbol starts the replay. At each later
position, every retained channel whose left symbol occurs at its recorded
distance votes for its right symbol with its exact frequency. Maximum vote
wins, ascending first-occurrence symbol id breaks ties, and the most frequent
observed symbol is the fallback when no channel applies. Dictionary-entry
coordinates carry exact symbol frequencies, so the fallback is part of the
native pattern rather than hidden training data.

`rank-descent` generates a fixed rank-one reference whose row count defaults
to the number of supplied observations. A lower candidate is selected only if
its replay equals that complete reference row by row. Its longest exact prefix
and first mismatch are still reported when it fails. Adaptive search first
tests rank `1/2`; complete success moves the tested interval downward and a
mismatch moves it upward. It stops when the retained-channel bounds are
adjacent. Linear search tests every positive exact decrement. Both return the
lowest fully matching candidate encountered by that finite schedule. Adaptive
search does not infer untested success or claim global rank minimality because
replay success has not been proved monotone under channel removal.

The staged source forms return the selected pattern directly:

```ns
output rank_descent(observations()) as pattern
```

```ns
output rank_descent(observations(), 1/4) as pattern
```

The first form is adaptive and exact. The second tests one static quarter-rank
candidate and requires exact agreement. This explicitly lossy form retains the
same rank but accepts a candidate meeting a lower finite threshold:

```ns
output rank_descent(observations(), 1/4, 99/100) as pattern
```

Target rank is an exact number greater than zero through one. Minimum agreement
is an exact number from zero through one and defaults to one. Rank selection
uses an exact ceiling; agreement uses exact integer cross multiplication. Every
form lowers to ordinary native coordinates before bytecode generation.

One selected pattern can be replayed at a positive one-based position:

```ns
output apply(rank_descent(observations()), 13) as number
```

`apply` currently requires a direct `rank_descent(...)` first argument. It
returns the complete generated native state at that position. Positions within
the data-sized rank-one reference inherit exact finite equality only when
minimum agreement is one. A lower threshold is a measured lossy replay. Later
positions are deterministic extrapolations and carry no new correctness claim.
Pattern application is staged and leaves no bytecode opcode.

The same generated state may be viewed through the exact 3D cone output camera:

```ns
output apply(rank_descent(observations()), 13) as vector
```

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
| Exact grammar | `zero`, `one`, `scalar`, `trace`, `length`, `untrace`, `rank_descent`, `apply`, `concat`, `fold`, `camera`, `let`, `output`, `as`, `operator`, `import`, `string`, `number`, `vector`, `pattern`, `boolean`, `=>`, `=` |
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

`as vector` is the existing exact quadratic cone camera for one unindexed
oriented scalar `x + iy`. It returns the three exact rational coordinates
`[x*x - y*y, 2*x*y, x*x + y*y]`. Zero returns `[0, 0, 0]`; a multi-term or
indexed state is rejected rather than silently flattened. This camera is
two-to-one away from zero because a scalar and its negative have the same
vector. Boolean output accepts only exact zero (`false`) or exact one (`true`).
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
- `length(operation_strand)` is lowered to one unary INDEX-depth expression.
- Variadic packs are substituted into operand and argument lists, then
  `concat(direction, values...)` lowers to one ADD of position-depth INDEX
  terms. Neither construct reaches bytecode.
- Finite `fold` lowers to nested calls of its named binary source function.
  Closed camera expressions lower to their exact resulting ADD/INDEX state.
  Neither construct adds bytecode for a model-specific operation.
  The ordinary compiler and VM then process only exact constants and core
  operations; bytecode has no hidden trace opcode.
- `untrace(value[, rank])`, `rank_descent(value[, target_rank[, minimum_agreement]])`, and pattern `apply` are staged after source calls and traces are lowered.
  Exact deterministic discovery returns a recursive operation strand;
  relationship discovery returns an exact ranked coordinate state. Both lower
  to ordinary coordinates, so bytecode has no hidden untrace, rank-descent, or pattern-application opcode.
- Bytecode retains the source goal and selected output camera. One `INDEX`
  instruction carries both the positive direction and literal multiplicity;
  composing instructions accumulates depth as an arbitrary-size exact natural
  number.
- Exact results use the canonical finite flat-stack state. ADD automatically
  combines signed coordinates at each retained INDEX location, removes every
  zero term, and therefore emits only the nonzero residual. It never combines
  distinct INDEX locations or different camera perspectives implicitly.

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

The host does not chunk, independently map, or reset the data. `fold` provides
the explicit source-defined state chain when observations must share one model
state. [`../examples/data-frequency-model.ns`](../examples/data-frequency-model.ns)
is a complete zero proof and data-run example: the source defines field layout,
ordered pair coordinates, accumulation, and the resulting transition counts.

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
| `OPT-ORIENT-NORMALIZE-1` | Reduce the sum of combined canonical turns modulo four | `L-SEP-5` |
| `OPT-ORIENT-IDENTITY-1` | Remove a zero-turn ORIENT | `L-SEP-5` |
| `OPT-ORIENT-COMBINE-1` | Combine nested ORIENT turns | `L-SEP-5` |

## Command line

All commands use the same Native Space 1.0 parser:

| Command | Input | Result |
|---|---|---|
| `native-space run FILE` | Exact-state document | Evaluate and print its selected output camera |
| `native-space run FILE --data DATA --function FUNCTION` | Exact-state document plus one complete ordered data file | Pass every root data item to the selected source function once and print its result |
| `native-space check FILE` | Any document kind | Validate it; execute zero/Boolean checks when present |
| `native-space inspect FILE` | Any document kind | Print its schema-1 parsed representation |
| `native-space expand FILE` | Exact-state document | Print generated pure source after calls, packs, concat, and reflective forms are lowered |
| `native-space compile FILE` | Any document kind | Emit bytecode, a function-library artifact, or a Boolean certificate |
| `native-space untrace [--input] FILE [--rank R]` | Exact ordered observations | Return an exact deterministic continuation or ranked relationship-frequency pattern; rank defaults to one |
| `native-space rank-descent [--input] FILE [--strategy adaptive\|linear]` | Exact ordered observations | Generate a data-sized rank-one reference and return the lowest fully matching candidate tested by the selected finite schedule |
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
