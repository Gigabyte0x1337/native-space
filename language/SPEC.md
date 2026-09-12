<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# NS 2 source

One UTF-8 .ns file. Comments start with #. No semicolons or end keywords.
Numbers are exact integers/fractions; strings use double quotes. Decimals
must be supplied as fractions. Lists and named records are structural data,
not arithmetic INDEX encodings.

```ns
let f = (a,b) => add(multiply(a,2),b)
output f(3)(4) as number
```

Calling binds arguments to a shared function graph. A partly bound function
is a function value; zero is an explicit bound value, never a missing binding.
Recursive names point to the same graph. Calls use heap continuations and
an execution/call-depth budget, not unlimited host recursion.

## Constructs

- `state(l,a,m)`: construct raw coordinates from classical rational arguments.
- `add(...)`, `multiply(...)`: homogeneous arithmetic, retaining inputs.
- `negate(s)`, `inverse(s)`: exact laws with checked inverse domain.
- `split(s,"add")`, `split(s,"multiply")`: halve and redistribute LOAD.
- `transform(matrix)`: validate an invertible rational 3x3 matrix.
- `transform(s,t)`: enter a computational frame, preserving the decoded object.
- `decode(s)`: return the canonical raw state from a frame.
- `mutate(s,t)`: explicitly change raw state by the matrix.
- `program(seed,step)`: finite seed and unary or curried-to-unary function.
- `observe(p,k)`: lazy selection at an exact nonnegative arbitrary-precision index.
- `reflect(subject,pattern,replacement)`: match fields, bind, rebuild.

Transform matrices and records are data. Rotation is a matrix, not a PHASE
primitive. Observation index is metadata, not arithmetic INDEX.
Built-in names cannot be overridden. There are no mathematical theorem names,
automatic frequency fitting, or prime-specific instructions.

## Reflection

```ns
let swap = (s) => reflect(s,state(l,a,m),state(l,m,a))
let inspect = (p) => reflect(p,program(seed,step),{seed:seed,step:step})
output swap(state(1,2,3))
```

Patterns may use state/program/observe constructors, records, lists, exact
numbers and strings. A fresh name captures a value; _ ignores it.
An already-bound name must match. Record patterns select named fields;
list patterns require the same length. A non-match returns an empty list.
Reflection matches the supplied object's fields, not a recursively scanned
history. There is no implicit ADD of heterogeneous records.

Function fields are `graph`, `name`, `bindings`, `scope`. Rebuild those
fields into a record and call it to execute a validated transformed graph,
without source recompilation. Graph records are not geometric coordinates.

## Output and input

`as state` (default) resolves an observation and exports retained state.
`as number` reads x=M/L. `as vector` returns three exact coordinate strings.
`as program` exports the finite lazy value without replaying observations.

Runtime JSON is version-specific typed structural data with retained scalar
DAGs. Reloaded functions and transforms are validated. NS1 artifacts are
incompatible. Source imports, custom infix operators, old Boolean proof syntax
and GPU/batch entry points are not part of this NS2 grammar.

The browser accepts exact scalar values, fraction strings, lists, records,
or `{"l":"1","a":"2","m":"3"}` as an optional Program seed.
Every example defines ordinary `native(s)` and `classical(s)` projection
functions; these names are UI settings, not reserved language names.
Readout functions do not feed back into the Program.

## Projective states and computational frames

The scalar equations are unchanged. Raw equality compares L/A/M, finite
decoded equality compares both (R,x), projective equality compares nonzero
triples by cross-products, and provenance equality also compares retained
structure. None of these relations silently replaces another.

Library State APIs: is_projective_point(), projective_equivalent(other),
is_finite(), is_boundary(). The all-zero raw triple is stored and round-trips,
but projective comparison rejects it. L=0 never decodes to a finite value.
T=L+A+M=0 only invalidates the simplex camera. M=0 or A+M=0 invalidates full
inverse even when the raw state is nonzero.

Existing source arithmetic on framed values now uses cached rational tensors
for ADD/MULTIPLY and transported matrices for negate/split. Inverse uses the
exact canonical reference route. The result keeps the first explicit frame;
mixed local inputs are converted by T1*T2^-1. Exact raw canonical results are
unchanged. Retained execution history records the local route rather than
inventing canonical intermediate evaluations.

The library supplies Transform::identity(), inverse(), compose(first),
operations(), reframe(local,from), add_reference() and multiply_reference().
second.compose(first) means second * first. Serialized frames retain their
validated matrices; local-operation caches are rebuilt, not trusted on load.
No new arithmetic opcode or reserved language name is introduced for these APIs.

State::rescale_pow2(k) explicitly changes raw scale. In contrast,
optimize::balanced_frame_with(state, Balance::PowerOfTwo) changes only a
computational frame. It uses exact integer/rational comparisons.

optimize::scale::progressive(samples,options) is a numerical library probe, not
a source primitive. Options specify layer count, held-out suffix and absolute/
relative tolerances. Reports retain attempts, inferred limit/amplitudes/ratios/
signs/exponents, predictions, held-out errors, residual subtraction stages and
unexplained residual. selected is absent unless a fit validates; best can still
identify an unvalidated improving fit. Tolerance passes when either the
absolute error or the relative error meets its respective bound.
Layers are jointly refitted using only training samples; the suffix is used
for model selection, not fitting or independent confirmation. See THEORY.

PHASE and arithmetic INDEX remain absent. Observation index is metadata.
No Fourier, Laplace, zeta or prime builtin is introduced.

## Execution limits

Source: 128 KB. Expression nesting and function-call depth: 128.
Evaluation: 1,000,000 machine steps. A lazy index may be larger than this;
evaluating it requires an admissible replay budget.
Serialized values: 16 MB; retained history: 100,000 records.
The browser worker additionally has a ten-second timeout and display bounds.
These are execution limits, not mathematical claims about finite patterns.
They are not a hard memory sandbox: exact rational arithmetic can grow large
within one operation. Run untrusted workloads in an externally resource-limited
process. Retained history records describe execution; loading them validates
their structure, not a mathematical certificate of every recorded operation.
