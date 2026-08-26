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

The exact expression forms are numbers, UTF-8 strings, references, calls, and
the four operations:

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
| Exact grammar | `zero`, `one`, `scalar`, `let`, `output`, `as`, `operator`, `import`, `string`, `number`, `pattern`, `boolean`, `=>`, `=` |
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
native state; its function-call graph must be acyclic and it does not unfold a
pattern into a supposed final state.
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
  bytecode generation.
- Bytecode retains the source goal and selected output camera. One `INDEX`
  instruction carries both the positive direction and literal multiplicity;
  composing instructions accumulates depth as an arbitrary-size exact natural
  number.
- Exact results use the canonical finite flat-stack state. ADD automatically
  combines signed coordinates at each retained INDEX location, removes every
  zero term, and therefore emits only the nonzero residual. It never combines
  distinct INDEX locations or different camera perspectives implicitly.
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
