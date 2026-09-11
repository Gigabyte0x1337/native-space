<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Native Space: the foundation

Start with a pattern. Keep its operations, inputs, and indices. A camera
observes that pattern without replacing it.

## Pattern and observation

A Pattern is a finite pair `(seed, reusable step)`. For one such generator:

```text
P = (S, G)
P(0) = S
P(k+1) = G(P(k))
P(k) = G^k(S)
```

The shared compiled graph represents G, including its curried bindings. A graph
address is not a Pattern or an observation index. Selecting k retains the same
generator and an exact unwrapped repetition coordinate; it does not construct
a prefix. Evaluation is a separate, budgeted readout.

PHASE is cyclic. For a quarter-turn generator, k and k+4 have the same phase
but remain different observations. Payload INDEX directions (for example prime
identity and power depth) are separate from the outer observation coordinate.
There is no automatic collision-prone insertion of k into payload direction 1.

The runtime `pattern::Pattern` API pairs a retained seed with an existing unary
FunctionValue. `observe(k)` is lazy. Explicit `project(maximum_steps)` reuses the
graph, stepping on exact retained Native states. No prefix is stored in the
Pattern or Observation. Temporary replay results may retain previous inputs;
canonicalizing those away would change reflective steps. The seed and generator
retain the recipe, including original zero provenance. Replay costs k calls;
large k can be represented even when replay exceeds a requested budget.
Pattern and Observation are authoritative Native records, not Rust-only
relationships. A retained seed is encoded with the existing addressed-state
codec; the step uses the existing callable graph and portable binding codec.
The exact nonnegative observation index is a scalar in its own record.
Rust may cache decoded fields, but deleting those caches loses no meaning:
`from_native` / `from_data` reconstruct the generator without compiling source.
REFLECT reads or replaces these fields just like other Native data.

No new NS primitive or source keyword is introduced.

The finite-generator invariant holds at every successor: `(P,k)` becomes
`(P,k+1)` with the identical shared seed/step graph. Only the arbitrary-precision
index changes. By induction, selecting any finite number of successors never
unrolls the generator. The index itself needs more digits as it grows; this is
not constant-size storage for an unbounded integer.

For the cyclic step `phase(1,x)` and seed `1`, the classical values repeat
`1, i, -1, -i`, while the indexed observations remain distinct. This recurrence
defines continued observations without a final index. Finite regression tests
check repeated cycles and indices beyond machine-integer limits; they do not
claim an infinite machine run. Evaluation remains resource-bounded, and an
arbitrary transition need not terminate. Retained evaluation history is distinct
from expansion of the compiled generator and is not silently discarded.

## The derived cylindrical camera

For one nonzero complex sample `z`, phase `phi`, and nonnegative integer
observation index `k`, the default cylindrical camera chooses:

```text
X = ln|z|
Y = (k+1) cos(phi)
Z = (k+1) sin(phi)

phase = atan2(Z,Y)                  modulo a full turn
index = sqrt(Y²+Z²)-1
classical value = exp(X) * (Y+iZ)/sqrt(Y²+Z²)
```

The inverse requires radius at least one and an integer radius minus one.
An arbitrary transformed 3D point need not satisfy these conditions.
A reversible frame change must retain its inverse; it is not permission to
reinterpret arbitrary coordinates as valid indices.

At index zero, `1` lies at `(0,1,0)`. Every unit phase has depth zero.
Index is not magnitude, graph address, or prime birth order unless the input
explicitly uses that meaning.

INDEX does not mean radius: this camera stores k in radius k+1 so phase remains
visible at k=0. The historical fixed-radius helix (cos(phi),sin(phi),k) is another
camera for cyclic patterns, but does not independently display magnitude depth.

For explicit array views, select one payload INDEX direction as `k`; keep all other labels
alongside the displayed point. A multi-term state is a collection of samples,
not one scalar or one magically complete 3D point.

## The operations

```text
multiply by a*exp(i theta), a > 0:
    depth += ln(a)
    phase += theta

square the value:
    depth *= 2
    phase *= 2

negative:       phase += half a turn
multiply by i:  phase += quarter of a turn
frequency:     repeat a constant phase step
select observation k: retain (P,k); evaluate G^k(S) only on request
cylindrical camera: store observation k as transverse radius k+1
```

A sample transform can preserve its index. Algebraic multiplication of two
INDEX-bearing states instead composes their indices by addition. Thus
`index(7, multiply(z,z), 30)` has index 30, whereas squaring
`index(7,z,30)` has index 60. These are different operations.

General addition is not linear in logarithmic depth. It combines the complex
contributions and lifts their result back. Equal-input addition changes depth
by `ln(2)`, not by one. Base-two depth remains a possible derived chart,
but is not the runtime's default.

A square root can be described by halving depth and a chosen lifted phase.
This needs a branch choice: wrapped phase alone does not select every root.
The existing source movement example demonstrates that coordinate rule;
it does not give the scalar runtime arbitrary irrational-number evaluation.

## Zero and retained information

Exact zero has depth `-infinity`, represented by a boundary tag rather than
a floating-point infinity. A supplied boundary phase and index can remain.
A literal classical zero has no specified phase; the runtime reports that
absence instead of inventing one.

```ns
let input = 7
let cancelled = multiply(input, 0)
output cancelled
```

The classical answer is zero. The graph still contains the input and operation.
Cancellation of an additive result does not determine one unique phase for its
zero; the operand branches retain the information that a single ray cannot.

A reciprocal of exact zero is undefined. Very small nonzero values are not
zero. Equal classical readouts do not imply equal retained histories.

Native → classical → native is exact for supported values **when the missing
index and any boundary provenance are supplied again**. Classical numbers
alone cannot reconstruct those discarded fields. Cartesian phase also wraps;
knowing an index only recovers winding when the phase progression is known.

## Derived cameras

For two amplitudes `a,b`, let `U=|a|²`, `V=|b|²`, and
`phi=arg(b)-arg(a)`:

```text
balance      = V-U
interference = 2*sqrt(UV)*cos(phi)
quadrature   = 2*sqrt(UV)*sin(phi)
total        = U+V

balance² + interference² + quadrature² = total²
total + interference = |a+b|²
```

This is the quadratic cone/sphere relationship. The last quantity is
intensity, not the complex sum itself. This camera loses common phase and
sample indices. It cannot replace the retained state.

A compact single-sample sphere camera is:

```text
tanh(X/2), sech(X/2)*cos(phi), sech(X/2)*sin(phi)
```

Its squared length is one for finite depth. Substituting
`X=2*sigma*u`, `phi=t*u` gives
`tanh(sigma*u), sech(sigma*u)*cos(t*u), sech(sigma*u)*sin(t*u)`.
That is a coordinate identity, not a zero-location theorem. Index is discarded,
and at the zero-boundary pole phase disappears from this display.

## Exact storage and numerical execution

The implementation stores `q=|z|²`, a rational phase ray, exact integer
indices, and the operation graph. It does not store rounded X/Y/Z as truth.
The exact readout uses `X=ln(q)/2` and
`(Y,Z)=(k+1)*ray/sqrt(ray.real²+ray.imag²)`.

This choice is deliberate: Gaussian-rational arithmetic stays closed and
exact, while its irrational logarithmic/radical coordinates need not be
numerically evaluated. An incompatible scalar coordinate request is rejected;
there is no silent conversion to floating point. Arbitrary irrational scalar
inputs and continuous fractional-phase evaluation need a larger exact carrier
and are not implemented by this storage choice.

`phase(0..3, value)` uses quarter-turn steps, with no old-name alias.
User-defined fractional phase coordinates are not silently reinterpreted as
primitive quarter-turn counts.

`run --numeric f64` explicitly rounds each scalar arithmetic step of the
elaborated graph. INDEX and graph storage remain exact. An indexed camera stays
in that graph, so it cannot silently evaluate a subtree exactly first.
Source synthesis/reflection decisions still occur during exact elaboration;
this flag does not change their algorithms.

Rounded zeros are observations, never proof certificates. Exact checking has
no f64 mode. The current numeric implementation rejects overflow and
underflow-to-zero; it does not claim arbitrary-range numerical evaluation.
Neither representation promises bounded memory for arbitrarily large graphs.

These decisions preserve the central invariant: the next native step can
receive the full state, while a user may explicitly request a cheaper numerical
observation. Tests check concrete domains, round trips, and independent
arithmetic comparisons. Wider claims require their own evidence.
