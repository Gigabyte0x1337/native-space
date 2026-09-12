<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Native Space 2

Native Space keeps an exact state and the operations that produced it.
The state is three rational components: **LOAD, ADD, MULTIPLY**, written
`(L,A,M)`. It is not a count of instructions.

The classical value is `M/L`. The independent order/frame readout is
`R=(A+M)/L`. Multiplying all three components by the same nonzero scale
preserves those two readouts, but Native Space retains that scale.

## A program and its projections

```ns
# One shared step, reused without copying its function graph.
let step = (s) => split(s, "add")

# Views are ordinary functions and do not replace the program's state.
let native = (s) => s
let classical = (s) =>
    reflect(s, state(l,a,m), multiply(m, inverse(l)))

let p = program(state(1,1,1), step)
output observe(p, 4) as state
```

The seed is `(1,1,1)`. One ADD split gives `(1/2,3/2,1)`.
The total stays 3, while its classical value changes from 1 to 2.
Four splits give `(1/16,31/16,1)`, whose classical value is 16.

A Program is a finite seed plus reusable function. An Observation retains
that Program and an exact nonnegative index. Selecting an index is lazy;
numerical/state output evaluates the requested steps. No output prefix is
stored in the Program. Evaluation retains actual operation inputs.

The playground displays either the full L/A/M state (or an explicitly
transformed frame) or its classical output. It does not use instruction
addresses as geometry. Optional retained-state points are actual intermediate
states, not a universal geometric encoding of an unbound function.

## Run

Rust 1.88 or newer:

```sh
cargo build --release --locked --manifest-path language/runtime/Cargo.toml
cargo test --locked --manifest-path language/runtime/Cargo.toml
native-space run example.ns
native-space inspect example.ns
native-space mcp
```

Use `as number` for classical output, `as vector` for exact L/A/M strings,
and `as program` for the portable finite Program/Observation representation.
The CLI's `check` executes the document; it is not a universal theorem prover.

[Theory](THEORY.md) · [Language](language/SPEC.md) ·
[Design and migration](language/README.md) · [Licensing](LICENSE.md)

This is a breaking replacement of the complex/depth/PHASE scalar model.
PHASE and arithmetic INDEX are removed. Complex arithmetic, old prime examples,
GPU solvers and NS1 artifacts are not silently interpreted as NS2.
Exact rational linear transforms are executable; nonlinear log/orthogonal
cameras are explicit numerical probes, not exact scalar storage.
No general optimizer, prime theorem, or scientific speedup is claimed.
