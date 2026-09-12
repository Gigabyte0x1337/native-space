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

The CLI exposes run/check/inspect and a stdio MCP run tool. Browser work runs in
a worker. The standalone NS1 GPU solvers and algorithm experiments were retired
from this foundation rather than relabelled as homogeneous execution.

## Verification

```sh
cargo fmt --manifest-path runtime/Cargo.toml --check
cargo test --manifest-path runtime/Cargo.toml --locked
cargo clippy --manifest-path runtime/Cargo.toml --all-targets -- -D warnings
```

The NS2 tests exercise arithmetic domains, retained scale, split recurrences,
camera inverses, currying, recursion limits, zero provenance, portable lazy
observations beyond u64, structural reflection and unseen-scale probe error.
Historical test counts are not claimed for the replacement implementation.
