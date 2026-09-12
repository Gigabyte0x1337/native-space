<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# NS2 runtime design

## Why this is a breaking replacement

The previous implementation stored complex scalars using logarithmic depth and
a phase ray. Its function, binding and Pattern records were encoded using
arithmetic INDEX directions. Reusing that scalar algebra as NS2 metadata would
leave two incompatible meanings of state and retain the operations being removed.

NS2 therefore uses explicit typed records for graph structure and bindings,
and exact rational (L,A,M) for scalar values. This replaces the record codec and
scalar reducer together. The old sources, tests and examples are preserved in
the sibling pre-LAM archive and in Git history, not compiled as a compatibility
backend. Old parser extensions were not silently approximated.

## Preserved invariants

- One immutable shared function graph. Arc-backed expression nodes are reused
  across calls, partial bindings and Program observations.
- Bound zero is present. Binding lists distinguish it from a missing argument.
- Recursive references remain names in a finite graph; execution uses heap tasks.
- Scalar results retain input DAGs, including zero products and unused scalar
  call arguments. Their coordinates and history remain distinct.
- Programs contain seed and callable step. Observations add exact integer metadata.
- Selection/successor do not execute or expand a prefix.
- Portable records contain graphs and environments, not process addresses.
  Reload validates graph structure and matrix inverses without parsing source.
- Rendered numerical coordinates never replace rational runtime state.

## Deliberate boundaries and tradeoffs

NS2 uses real rational components. It does not preserve Gaussian-rational
scalar arithmetic as an implicit alternate domain. Exact irrational scalar
inputs and arbitrary nonlinear executable transforms need their own explicit
domain/evaluator; the current exact transform type is rational linear algebra.

Reflection now operates on typed structural fields rather than matching
monomials of an indexed complex polynomial. This is a semantic change, not
backwards compatibility. It is documented in SPEC and tested on functions,
programs, observations, state coordinates, and rebuilt callable records.

Raw equality, decoded equality and provenance equality are separate.
No optimizer applies projective distributivity as a raw rewrite.
The frame optimizer is an explicit API policy, not default normalization.
The four-sample scale probe reports held-out error and has no proof status.

## Frozen-core completion decisions

The correctness pass keeps every L/A/M equation unchanged. The finite algebra
modulo scale is Q x Q, not a field; inverse therefore needs both decoded
components nonzero. Cross-product projective comparison extends equality to
boundary points without allowing (0,0,0) to masquerade as a projective point.
Raw storage and provenance are not normalized to projective representatives.

**Local execution.** The previous decode/operate/encode route remains the exact
reference. ADD/MULTIPLY now compile their bilinear coefficient tensors once
per frame; negate/split compile linear maps. Basis evaluation reuses the frozen
equations instead of maintaining a second hand-coded algebra. Clones share a
lazy cache, and serialization omits it. Equality depends on validated matrices,
not whether a cache has been populated. Mixed-frame operands move explicitly
through T1*T2^-1. Inverse keeps its reference path because a separate quadratic
compiler would add complexity without a demonstrated benefit.

This changes the recorded computational route, not its raw canonical result.
The tests compare raw BigRational triples, not merely classical readouts.
Coefficient count or local execution alone is not a speedup claim.

**Scale.** Exact reciprocal balancing remains available. Dyadic balancing
avoids arbitrary rational frame coefficients and floating-point logarithms.
It retains the inverse frame and original raw scale. Explicit rescale_pow2,
unlike a camera change, intentionally changes the authoritative raw triple.

**Numerical layers.** A difference recurrence (Prony-style fit), real Schur
roots, and SVD amplitude solve provide progressively richer real exponential
models. nalgebra supplies the numerical decompositions rather than a custom
eigensolver. Decompositions use bounded iterations and relative rank checks.
This dependency is confined to the numerical probe; exact algebra uses
BigRational throughout. Jointly refitting the prefix at each rank prevents an
approximate first layer from permanently corrupting the next residual.
Subtraction stages, rejected attempts and residuals are retained. Held-out
samples select rank, never fit coefficients; fresh confirmation data would
still be needed to assess generalization. No convergence proof is claimed.
The probe bounds work at 4096 samples and 16 layers, rejects nonreal/growing
ratios, and reports ill-conditioning instead of fabricating a continuation.

The numerical implementation uses the documented fallible
[nalgebra decompositions](https://www.nalgebra.rs/docs/user_guide/decompositions_and_lapack/).

The CLI exposes run/check/inspect and a stdio MCP run tool. Browser work runs in
a worker. The standalone NS1 GPU solvers and algorithm experiments were retired
from this foundation rather than relabelled as homogeneous execution.

## Verification

```sh
cargo fmt --manifest-path runtime/Cargo.toml --check
cargo test --manifest-path runtime/Cargo.toml --locked
cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets --all-features -- -D warnings
cargo build --manifest-path runtime/Cargo.toml --locked --release
```

The NS2 tests exercise arithmetic domains, retained scale, split recurrences,
camera inverses, currying, recursion limits, zero provenance, portable lazy
observations beyond u64, structural reflection and unseen-scale probe error.
Historical test counts are not claimed for the replacement implementation.

properties.rs adds deterministic signed-rational generation with three fixed
seeds (6,144 algebra cases), 2,048 generated frame cases, 128 source-level
mixed-frame cases and 1,024 shared-generator cases. It covers boundaries,
zero divisors and exact scales at +/-10,000 binary exponents. Generation is
test-only SplitMix64; failing seed/case identifiers are reproducible and no
production pattern inference relies on that generator. These tests run in the
normal locked CI suite on every platform; existing tests are retained.
