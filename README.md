<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Native Space

Native Space keeps a pattern and the operations that produced it, rather than
keeping only its numerical answer.

For example, `7 * 0` and `100 * 0` both display zero. Their retained patterns
are different: we can still inspect the original inputs.

## One sample, three coordinates

Write a complex value as magnitude and phase, and give it a retained index
`k >= 0`. Its native coordinates are:

```text
X = ln|z|
Y = (k + 1) cos(phase)
Z = (k + 1) sin(phase)
```

Depth runs along X. Phase turns around X. Index sets the transverse radius.
The value `1`, at index zero, is at `(0, 1, 0)`: depth zero, not the
Cartesian origin. Exact zero is a separate negative-infinity depth boundary.

Multiplication adds depths and combines phases. Squaring a value doubles both.
Addition combines contributions; cancellation does not erase their source.
These are the roles of **ADD, MULTIPLY, PHASE, INDEX**.

The classical value is a readout of this pattern. Cone and sphere coordinates
are derived views, not the core. A numerical readout alone does not retain
index or history.

## Try it

Build with Rust 1.88 or newer:

```sh
cargo build --release --locked --manifest-path language/runtime/Cargo.toml
```

NS source stays simple:

```ns
# phase takes quarter-turn steps: 1 means i.
let z = add(3, phase(1, 4))
output z as vector
```

The exact vector is `[ln(25)/2, 3/5, 4/5]`, serialized as exact expressions.
It is not converted to decimals unless requested.

```sh
native-space run examples/depth-phase-index.ns
native-space run examples/depth-phase-index.ns --numeric f64
native-space view examples/depth-phase-index.ns --index-direction 7
native-space view examples/depth-phase-index.ns --index-direction 7 --numeric f64
```

That example squares `3+4i` and displays the result. Use `as number` for a
real classical result, or bare output / `as pattern` for the retained state.
The branch view includes exact INDEX labels and source connections.

Exact execution is the default. In f64 mode each arithmetic step rounds;
the original graph and integer indices remain exact. Rounded output is not a
proof and is never substituted into the retained graph. Overflow and
underflow-to-zero produce an error.

The current exact scalar domain is rational real/imaginary components.
Logarithms and radicals in its 3D readout stay symbolic. This is not yet a
general symbolic evaluator for arbitrary irrational scalar inputs.

Read [the theory](THEORY.md) for the model, identities, and deliberate limits;
[the language specification](language/SPEC.md) for syntax; and
[the implementation contract](language/README.md) for runtime decisions.

Standalone applications and historical proof documents live outside this
foundation. No universal optimizer, scientific theorem, or performance gain
is claimed by these coordinate identities.

```sh
cargo test --locked --manifest-path language/runtime/Cargo.toml
```

Code uses strong copyleft; theory and documentation are share-alike.
See [licensing](LICENSE.md).
