<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Example Index

Native Space uses one `.ns` extension for several document kinds. The command
matters: an output evaluates a state, a zero or Boolean proof checks one closed
finite statement, and a function library only exposes a derivation graph.

The complete [reflection matrix example](examples/reflection-matrix.ns) defines
its own rewrite rule, transforms a trace, and executes the rebuilt graph:
`run examples/reflection-matrix.ns`. Its
[symbolic zero proof](examples/reflection-matrix-proof.ns) is checked with
`check examples/reflection-matrix-proof.ns`. The output is the matrix
`[[20, 24], [32, 36]]`; the rule reduces arithmetic, but no runtime speedup
is claimed. See the [reflection contract](language/REFLECTION.md).

Prepend each command below with:

```powershell
cargo run --manifest-path language/runtime/Cargo.toml --release --bin native-space --
```

| File | Command | Kind | What success establishes |
|---|---|---|---|
| `basic.ns` | `run examples/basic.ns` | Exact output | Evaluates one indexed oriented state |
| `axis_residual.ns` | `run examples/axis_residual.ns` | Exact output | Shows the remaining finite axis residual |
| `function_output.ns` | `run examples/function_output.ns` | Exact output | Shows that an ordinary source function can return a value |
| `trace.ns` | `run examples/trace.ns` | Exact output | Returns one complete exact-state source graph as a nested native operation strand |
| `variadic-concat.ns` | `check examples/variadic-concat.ns` and `expand examples/variadic-concat.ns` | Zero proof / generated source | Forwards a finite source pack, proves concat equals its manual ADD/INDEX form, and prints the generated pure source |
| `data-frequency-model.ns` + `data-frequency-observations.json` | `run examples/data-frequency-model.ns --data examples/data-frequency-observations.json --function train` | Exact output | Loads one complete ordered data pack and lets source-defined `fold` and `camera` functions produce exact transition-frequency coordinates |
| `continuation-observations.ns` | `untrace examples/continuation-observations.ns` | Exact output | Supplies raw Fibonacci observations and prints the synthesized runnable continuation source |
| `continuation-observations.csv` | `untrace --input examples/continuation-observations.csv --output pattern-csv` | Exact output | Reads exact indexed CSV observations and prints deterministic seeds, coefficients, and prediction rows |
| `relationship-observations.ns` | `untrace --rank 1/5 examples/relationship-observations.ns` | Exact output | Rejects an inexact continuation and retains the strongest one fifth of all exact ordered relationship channels |
| `rank-descent.ns` | `run examples/rank-descent.ns` | Exact output | Runs adaptive rank descent inside Native Space source and emits the selected pattern after staged lowering to ordinary coordinates |
| `rank-descent-static.ns` | `run examples/rank-descent-static.ns` | Declared lossy output | Retains one static quarter-rank candidate under an explicit zero agreement threshold; it demonstrates syntax, not equality with rank one |
| `rank-descent-apply.ns` | `run examples/rank-descent-apply.ns` | Exact output | Applies the selected pattern at one positive one-based position; position 13 deterministically returns scalar 2 and prints its exact cone vector `[4, 0, 4]` |
| `rank-descent-observations.json` | `rank-descent --input examples/rank-descent-observations.json` | Verified finite search | Uses the 40 supplied rows as the reference size, branches from one half by exact success, and selects the lowest fully matching candidate actually tested |
| `untrace-array-data.json` | `untrace --input examples/untrace-array-data.json` | Exact output | Loads one complete seven-state vector sequence, discovers one shared recurrence, and predicts the next complete state without flattening coordinates |
| `untrace-array-model.ns` | `run examples/untrace-array-model.ns` | Exact output | Runs the complete native-state continuation source generated from `untrace-array-data.json` |
| `untrace.ns` | `run examples/untrace.ns` | Exact output | Runs the in-language synthesizer and returns its recursive operation strand |
| `untrace-relationships.ns` | `run examples/untrace-relationships.ns` | Exact output | Uses in-language rank one fifth to return a relationship-frequency pattern when no exact continuation exists |
| `batch-program.ns` + `batch-data.json` | `batch examples/batch-program.ns --function step --data examples/batch-data.json --steps 3 --backend cpu` | Exact output | Runs three sequential steps per point while CPU workers distribute independent points |
| `batch-program.ns` + `batch-data.json` | `batch examples/batch-program.ns --function step --data examples/batch-data.json --steps 3 --backend gpu` | Exact output | With the `gpu` Cargo feature, runs the supported exact signed-32-bit scalar subset with one GPU invocation per point |
| `batch-vector-program.ns` + `batch-vector-data.json` | `batch examples/batch-vector-program.ns --function step --data examples/batch-vector-data.json --steps 2 --backend cpu` | Exact output | Lowers each rank-1 row by axis and position to nested INDEX coordinates and preserves its requested shape |
| `batch-array-program.ns` + `batch-array-data.json` | `batch examples/batch-array-program.ns --function step --data examples/batch-array-data.json --steps 1 --backend cpu` | Exact output | Round-trips independent rank-1, rank-2, and rank-3 arrays through ordinary ADD and INDEX states |
| `batch-array-data.json` | `pack-data examples/batch-array-data.json data.nsb` | Binary data | Validates and packs the exact sparse states and host shapes into versioned binary input |
| `frequency-observations.ns` | `frequency examples/frequency-observations.ns --samples 16 --maximum-error 1e-12` | Verified lossy output | Projects sixteen exact quarter-turn samples to classical complex coordinates, retains one mode, and verifies finite replay within the declared error |
| `operators.ns` | `run examples/operators.ns` | Exact output | Checks definition-ordered operator lowering for this expression |
| `classic_identities.ns` | `run examples/classic_identities.ns` | Exact output | Evaluates three tagged finite residuals; this file is not a zero-proof document |
| `orientation_zero.ns` | `run examples/orientation_zero.ns` | Exact output | Evaluates the four-orientation sum; this file is not a zero-proof document |
| `primes.ns` | `run examples/primes.ns` | Exact output | Displays one finite prime-pattern observation |
| `utf8.ns` | `check examples/utf8.ns` | Zero proof | The two shown UTF-8 spellings lower to one exact state |
| `prime_pattern.ns` | `check examples/prime_pattern.ns` | Zero proof | The two shown finite observations are equal |
| `projection-zero-fiber-counterexample.ns` | `check examples/projection-zero-fiber-counterexample.ns` | Zero proof | Checks the two tagged finite residual statements |
| `rh_two_interpretations.ns` | `check examples/rh_two_interpretations.ns` | Zero proof | Runs the proved shared-origin RH reading and the counterexample to equal complete zero fibers together |
| `zeta_re_perspective_rotation.ns` | `check examples/zeta_re_perspective_rotation.ns` | Zero proof | Proves that the zeta and RE projectors are idempotent, perpendicular, and complete |
| `zeta_re_perspective_position.ns` | `check examples/zeta_re_perspective_position.ns` | Zero proof | Proves that both perpendicular quadratic cameras place the multiplicative identity at exactly one half and reconstruct one together |
| `zeta_re_perspective_wrappers.ns` | `check examples/zeta_re_perspective_wrappers.ns` | Zero proof | Proves that the separate zeta and RE quadratic wrappers reconstruct one shared native state and its squared size |
| `zeta_re_perspective_cancellation.ns` | `check examples/zeta_re_perspective_cancellation.ns` | Zero proof | Derives both half-identity camera positions, reverses the RE comparison orientation, and proves their ADD residual is zero |
| `zeta_re_vertex_path.ns` | `check examples/zeta_re_vertex_path.ns` | Zero proof | Applies both perspectives to the same indexed vertices and reconstructs every coordinate without flattening the path |
| `zeta_re_classical_axis_rotation.ns` | `check examples/zeta_re_classical_axis_rotation.ns` | Zero proof | Proves that on the classical source axis the RE camera is exactly a 90-degree rotation of the zeta camera |
| `reflection-center.ns` | `check examples/reflection-center.ns` | Zero proof | Proves that half of the multiplicative identity is fixed by reflection and that centering makes an exact reflected pair cancel |
| `boolean_logic.ns` | `check examples/boolean_logic.ns` | Boolean proof | Exhaustively proves the displayed finite tautology |
| `math-functions.ns` | `derive --source examples/math-functions.ns FUNCTION` | Function library | Expands a selected mathematical source graph; it does not prove the function's paper theorem |
| `tryouts/matrix-coefficient-contraction.ns` | `check examples/tryouts/matrix-coefficient-contraction.ns` | Zero proof | Checks one exact nontrivial instance of rank-one matrix multiplication against its shared-coefficient contraction |
| `tryouts/navier-stokes-fourier-triad.ns` | `check examples/tryouts/navier-stokes-fourier-triad.ns` | Zero proof | Checks coefficient contraction, pressure projection, viscosity, and divergence preservation for one exact finite Fourier triad |
| `prime_count.ns` | `derive --source examples/prime_count.ns prime_count_example` | Function library | Expands the prime-counting camera example |
| `dual_alignment.ns` | `derive --source examples/dual_alignment.ns dual_alignment_example` | Function library | Expands the two reflected axes whose midpoint is half of the multiplicative identity |
| `re_critical_line.ns` | `derive --source examples/re_critical_line.ns re_critical_line_example` | Function library | Expands the centered RE height, including the half-identity construction; it is not a quantified critical-line proof |
| `zeta.ns` | `derive --source examples/zeta.ns zeta_example` | Function library | Expands the zeta graph centered at ADD zero; it does not shift zeta by one half or execute analytic continuation |
| `recursive-pattern.ns` | `derive --source examples/recursive-pattern.ns quarter_turn_pattern` | Recursive function library | Reports one finite operation and one self-reference edge |

The six files under [`examples/applications/`](examples/applications/) are
closed finite zero witnesses. They test the algebraic identity written in each
file, not the application hypothesis or an empirical performance gain.

The optional native SPH coefficient benchmark is a Rust example rather than a
`.ns` document:

```powershell
cargo run --manifest-path language/runtime/Cargo.toml --release --features gpu --example native-water-native -- --counts 19x15x14,48x32x32,64x48x48 --rounds 31
```

It compares one optimized Rust CPU thread with Vulkan on the same retained pair
graph. It does not benchmark a complete moving-particle frame or Native Space
compiler output; see the [recorded result](applications/native-water/native-benchmark-2026-08-29.txt).

The visual native application compares CPU-indexed, Vulkan-indexed, and Vulkan
all-pairs WCSPH, then renders the current state with Vulkan:

```powershell
cargo run --manifest-path language/runtime/Cargo.toml --release --features visual --example native-water-visual
```

The headless CPU benchmark excludes Vulkan and rendering:

```powershell
cargo run --manifest-path language/runtime/Cargo.toml --release --features visual --example native-water-visual -- benchmark-cpu --particles 2048 --steps 240
```

For an A/B measurement, compile the identical source with
`--features visual,cpu-simd`. This diagnostic feature substitutes a safe target
SIMD vector type; state traversal and solver equations stay identical. SIMD is
not the default because the current linked-cell layout does not provide
contiguous neighbor lanes and measured results depend on particle count.

Its default mode rebuilds spatial membership and evaluates the complete WCSPH
state transition on one CPU thread, then uploads one completed state per frame.
Every output reads only the complete previous state. Press `I` to compare the
GPU-indexed or finite all-pairs GPU implementations. CPU-indexed and GPU-indexed
modes support presets through 1,048,576 particles. Both are handwritten
application implementations; this is not Native Space compiler output.
Use Up/Down to select 512 through 1,048,576 particles, drag to rotate, and
scroll to zoom. Press `W` to switch between the derived screen-space water
surface and direct particles. Press `I` to cycle CPU-indexed, GPU-indexed, and
all-pairs WCSPH, and `F` or Home to restore the
rotation-aware fitted view. All-pairs mode is deliberately
disabled above 16,384 to prevent accidental quadratic workloads. Switching
mode resets every path to the same deterministic input state. The title reports
rendered FPS, simulation steps per second, CPU milliseconds per step in CPU
mode, and dropped steps so the real-time
failure point is explicit. The `solver Nx` field reports resolution-dependent
substeps: one physical `1/240`-second step is divided into smaller native-state
translations as the smoothing radius shrinks, preventing high-density pressure
and boundary corrections from overshooting.
