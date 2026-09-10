<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Native Space

Native Space keeps a pattern and the operations that produced it, rather than
keeping only its numerical answer.

For example, `7 * 0` and `100 * 0` both display zero. Their retained patterns
are different: we can still inspect the original inputs.

## One pattern, repeated observations

A Pattern is a **seed and reusable step**. Observation zero is the seed;
observation k is the same step repeated k times. One shared graph represents
the generator, not a materialized list of its observations.

INDEX identifies the unwrapped repetition. PHASE wraps: a quarter-turn pattern
has the same phase after four more steps, but a different observation index.
Multiplicative depth and payload index directions remain distinct.

The default 3D camera displays these as:

```text
X = multiplicative depth = ln|z|
angle around X = phase
radius = observation index + 1
```

**INDEX does not mean radius; this camera stores it there.** At k=0 the
nonzero radius preserves phase. The logarithmic origin is magnitude one,
while exact zero has a separate negative-infinity depth boundary.

Multiplication adds depths and combines phases. Squaring a value doubles both.
Addition combines contributions; cancellation does not erase their source.
These are the roles of **ADD, MULTIPLY, PHASE, INDEX**.
**REFLECT** matches parts of an evaluated Native state and rebuilds them.
Together these are the five language operations; functions are directly
inspectable Native graphs, not a separate trace format.

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

Save the source above as `program.ns`, then run:

```sh
native-space run program.ns
native-space run program.ns --numeric f64
native-space view program.ns --index-direction 7
native-space view program.ns --index-direction 7 --numeric f64
```

Use `as number` for a real classical result, or bare output / `as pattern`
for the retained state.
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
