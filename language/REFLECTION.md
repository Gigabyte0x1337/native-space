# Source-defined graph rewriting

The requirement is to move rewrite decisions into Native Space source, starting
with a complete matrix example. Existing traces already encode ordered source
graphs as native states; introducing a second graph file format or a
matrix-specific Rust optimizer would duplicate that representation.

The first reflection interface is declarative:

```ns
let before = (a, b, x) => add(multiply(a, x), multiply(b, x))
let after = (a, b, x) => multiply(add(a, b), x)
let program = (x) => add(multiply(2, x), multiply(3, x))
let candidate = rewrite(trace(program), trace(before), trace(after))
output apply(candidate, 7) as number
```

## Contract

- `rewrite(graph, pattern, replacement)` takes three canonical trace values.
  Pattern-root parameters are structural placeholders. Repeated occurrences
  must match the same ordered expression, ignoring source locations only.
- Replacement-root parameters must be bound by the pattern. Both rule roots
  are fixed-arity functions, not variadic matchers. Matching is syntactic:
  no implicit commutation, associativity, inlining, or numerical approximation.
- One call performs one bottom-up pass through each function body. Inserted
  replacements are not revisited during that pass. Explicit nested calls can
  request more passes. Source function cycles remain references, not unfolding.
- Helper definitions with the same name must agree structurally. Name collisions
  fail instead of silently changing a callee or capturing a parameter.
- `apply(graph, arguments...)` validates the graph and uses its root signature.
  It supports ordinary finite source computations; recursive execution and
  graph application nested inside applied graphs are rejected. This finite
  boundary prevents reflective self-application from bypassing existing
  execution-cycle checks. Tracing such source remains possible.
- Existing direct `apply(rank_descent(...), positive_position)` keeps its
  sequence-replay meaning. A graph value is never interpreted as observations.
- Reflection is staged into existing operations for bytecode execution. It is
  not a new algebra operation or a matrix backend.

## What a rewrite does not prove

Rewriting constructs a candidate. It does not certify equivalence or lower cost.
The caller supplies the rule, tests/proves the relevant algebraic identity, and
decides whether to use the candidate. Even a longer or non-equivalent replacement
is a valid structural transformation. This reflection operation contains no
optimization rule; the existing theorem-backed compiler optimizer is separate.

This is the first complete reflection path, not unrestricted node callbacks,
higher-order functions, an equivalence oracle, or universal optimization.
The tradeoff is a smaller auditable interface with rules written as ordinary
functions. Native numeric state semantics, ordered graph dependencies, source
locations, and strict malformed-graph diagnostics remain authoritative.

## Verification

Runtime tests cover structural rewriting, graph application, round trips,
and rejection of invalid or recursive executable graphs. The standalone
matrix demonstration has been removed; those tests do not claim a measured
matrix-multiplication speedup.

## Source round-trip requirements

Exact-state directions/depths now parse their actual unsigned 64-bit carrier.
The previous signed intermediate rejected the trace camera's own directions.
An explicit positive multiplicity can be written as
`index(direction, value, depth)`; the two-argument form still means depth one.
Export uses the counted form instead of manufacturing deeply nested calls.
This exposes an existing INDEX field, not an extra algebra operation.

Reflection bounds input graphs to 20,000 coordinates/nodes and expression
nesting and function catalogs to 128; a rewrite has a 100,000-unit
matching/construction work budget.
These limits return diagnostics and are implementation guardrails, not
mathematical limits. Ordinary computation after graph application retains
the existing evaluator's cost characteristics.

## Acceptance tests

Tests must exercise direct evaluation and compiled VM results; matrix
factorization; repeated-placeholder mismatch; nested matches; no-match
identity; invalid graphs; wrong argument counts; rule signature and callee
collisions; recursive execution; trace/AST round trips; and explicit
non-equivalent rewrites (which must never be presented as proved).
