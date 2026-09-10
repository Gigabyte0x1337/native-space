# Reflection

## Value reflection

`reflect(subject, pattern, replacement)` always matches the evaluated Native
value it receives. It does not walk the operations which constructed that value.
To inspect a program, pass its function value as Native data instead.

```ns
let from7 = (x) => reflect(x, index(7, value), value)
output from7(multiply(3, index(7, 2))) as number
```

This returns `6`, not `2`. ADD combines equal indexed coordinates before
matching. Cancelled contributions have no value match; their history remains
on the retained subject edge. Each distinct nonzero indexed contribution is
matched once, and the replacement results are added. No match returns zero.

The initial deterministic pattern grammar is a nonzero scalar factor, PHASE,
and INDEX around exactly one capture occurrence. Capture names are local to
the rule and shadow surrounding names. Replacement expressions use that
capture, exact scalar literals, ADD, MULTIPLY, PHASE, and INDEX; the capture
may be reused. Unbound captures, function calls in templates, additive
splitting, and multiple capture occurrences in a pattern are diagnosed.
For example, `add(a,b)` does not specify a unique decomposition of `6`.
This is a deliberately restricted matcher, not a general equation solver.

INDEX matching with a literal multiplicity subtracts that multiplicity from a
canonical term. Other directions and remaining depth stay in the value capture.
With a name in the third position, INDEX instead binds and removes the entire
depth of that direction. Depth bindings are exact natural numbers, not floats
or 64-bit counters. They can be returned as values or reused as INDEX depths.

```ns
let from7 = (x) => reflect(x, index(7, value, depth), value)
let move7to9 = (x) => reflect(x, index(7, value, depth), index(9, value, depth))
output move7to9(multiply(3, index(7, 2, 17)))
# index(9, 6, 17)
```

These implement complete-depth removal and routing for the named directions.
Unrelated directions remain in `value`; an existing destination depth is added
to the moved depth, exactly as ordinary INDEX composition requires. Missing
directions do not match. Reusing a depth name on different directions requires
equal depths. Value and depth names cannot collide. Capturing a direction twice,
or both subtracting and capturing that direction in one pattern, is ambiguous
and rejected. Depth names are local to REFLECT templates and invalid elsewhere.

Directions are still literal labels, not runtime arguments or direction captures.
There is no built-in camera. Write a rule for the labels you want to route.

```ns
let rotate = (p) => add(
    reflect(p, index(1, v), index(2, v)),
    reflect(p, index(2, v), index(1, phase(2, v))),
    reflect(p, index(3, v), index(3, v))
)
let transform = (p) => add(
    multiply(3, rotate(p)),
    index(1, 5), index(2, -1), index(3, 2)
)
output transform(add(index(1, 2), index(2, 3), index(3, 4)))
```

The resulting Cartesian fields are `(-4,5,14)`. These three labelled fields
are a source-defined Cartesian representation, not the default logarithmic
depth/phase/index readout of a single complex scalar.

### Compile-once boundary

The compiler stores a finite replacement instruction list and a canonical
pattern factor in a retained REFLECT node. The bytecode VM preserves this
node instead of substituting an observed answer. Evaluation binds values and
executes the stored list; it never constructs an AST or compiles new code
from runtime observations. JSON and NSBATCH retain the rule and its subject.
Decoded rules are validated before execution, including their backward edges.
Unbounded integer pattern depths use the standard big-integer serde codec.

## Shared executable graph

Functions now compile once into the existing Native strand records. Parameters,
references, function boundaries and calls remain in those records. The runtime
indexes their addresses once; it does not substitute source or rebuild an AST
when calling a function. REFLECT templates compile from borrowed record views.

```ns
let f = (a, b) => add(multiply(a, 2), b)
let g = f(3)
output g(4) as number
# 10
```

A function value is a shared graph reference plus its own Native binding
environment. A call appends arguments in parameter order. Too few arguments
return a partial function; a closed signature evaluates. Zero-argument functions
run with `()`; a variadic call closes once its fixed parameters are supplied.
Too many arguments for a fixed signature are errors.

Binding records contain an explicit presence marker and a reference to the
original argument graph. A bound zero is not an empty slot. Parameter selection
uses the same canonical REFLECT matcher; cached value handles avoid projecting
arguments or copying the program. Independent partial calls never modify each
other's bindings.

```ns
let f = (x) => multiply(2, x)
let p = reflect(f, value, value)
output p(7) as number
# 14
```

A function can be supplied directly as Native data. Calling transformed data
validates its graph and caches a reusable graph reference for the run. A malformed graph
is an error, not an implicit repair. There is no `apply` primitive or alias.

Completed calls retain the original argument states as scope edges, even when
the body ignores them. Binding metadata is not a replacement for those inputs.
The shared executor owns this retention for both source and host calls, so
feedback adds one call scope per step rather than a second host wrapper. The
call location is retained for replay diagnostics.

Recursion reuses the function graph with fresh environments. Heap continuation
frames avoid host-stack recursion. A shared budget bounds nested calls,
including calls through reflected data: 128 active calls and 1,000,000 evaluated
records per run. Compilation/loading accepts finite recursive references; an
unbounded run stops with a located diagnostic. This is not a termination proof.

Portable partial functions encode argument operations and scope edges as
Native records, rather than saving only their canonical result. This preserves
zero rays and indexed provenance. Slot payloads escape arbitrary user indices
so they cannot collide with presence metadata. A tagged function payload stores
its existing program records directly, including nested bindings; ordinary
state payloads store their retained operation edges. Export/loading bounds
nested function bindings to 128 levels. Explicit export/loading costs
more than in-memory currying; normal calls retain shared references.
Record data is encoded canonically, with balanced ADD trees when lifted into
retained storage. This avoids caching a growing prefix at every record during
replay, without changing the represented program or observing its arguments.

## Five-operation boundary

The executable AST is Literal, Reference, Spread, Call, Add, Multiply, Phase,
Index, Reflect, plus IndexCapture for template syntax only. Zero and one use
Literal; all calls use Call with a callee edge. Removed operation kinds are
rejected on artifact load. No compatibility decoder restores them.

The older AST mixed discovery, source rewriting, graph inspection, and execution.
The shared graph makes function values inspectable without a trace operation.
Camera routing is a full-depth REFLECT rule. This removes duplicate semantics
from the parser, graph executor, retained operation carrier, and stack VM.

Canonical reflection excludes zero terms. A zero-boundary phase remains on
the subject provenance edge; it is not selected as a nonzero value.

## Explicit host tools

- `reflection::rewrite` performs one bottom-up source rewrite on an unbound
  function graph. It returns an expression that builds Native graph data; callers can evaluate
  that expression and call the resulting function state.
  Inserted replacements are not revisited in the same pass.
- `strand::optimize_operation_strand` uses the existing exact rewrite rules
  and returns a shorter candidate or no candidate. Rank below one is rejected.
- `discovery::discover`, the rank-descent CLI, and frequency tooling remain
  outside the language. Their generated programs use the reduced AST.

Source tools reject partially bound functions rather than silently discarding
bindings. They explicitly reconstruct source for analysis; normal function
execution never does. Explicit rewriting does not prove equivalence or speedup.
The source reconstruction limit is 1,024 nodes deep; rewriting also has
finite node, depth and work limits.

## Argument-pack shorthand

Packs provide calling convenience, not a sixth operation. A bare pack reference
builds ADD of its arguments wrapped in one-based INDEX directions; spread inserts
the original arguments. Both execution and explicit source expansion follow this
same rule. The shared function graph is not copied or recompiled.

An empty pack builds zero. Zero operands and their locations remain in retained
construction; canonical REFLECT still skips zero contributions. Existing item
labels compose normally with positional labels, so this is intentionally not a
collision-free array or presence encoding. Functions are embedded as Native data.
This tradeoff preserves ordinary ADD/INDEX semantics instead of hiding a container
interpreter behind the syntax.

Pack tests cover expansion, saved artifacts, stack replay, empty/zero inputs,
mixed spread and bare uses, existing labels, and callable function contents.

Tests cover direct/reflected calls, currying and zero bindings, saved partials,
recursive graph data and bounded execution, complete-depth routing, malformed
records, exact host optimization, and explicit non-equivalent source rewrites.
