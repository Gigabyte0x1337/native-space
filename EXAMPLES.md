<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Example Index

Native Space uses one `.ns` extension for several document kinds. The command
matters: an output evaluates a state, a zero or Boolean proof checks one closed
finite statement, and a function library only exposes a derivation graph.

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
| `continuation-observations.ns` | `untrace examples/continuation-observations.ns` | Exact output | Supplies raw Fibonacci observations and prints the synthesized runnable continuation source |
| `continuation-observations.csv` | `untrace --input examples/continuation-observations.csv --output pattern-csv` | Exact output | Reads exact indexed CSV observations and prints compact seeds, coefficients, mismatches, and prediction rows |
| `continuation-observations-with-error.ns` | `untrace --maximum-error-ratio 1/5 examples/continuation-observations-with-error.ns` | Exact output | Supplies one mismatching held-out index and prints the lowest-error-ratio continuation source |
| `untrace-array-data.json` | `untrace --input examples/untrace-array-data.json` | Exact output | Loads one complete seven-state vector sequence, discovers one shared recurrence, and predicts the next complete state without flattening coordinates |
| `untrace-array-model.ns` | `run examples/untrace-array-model.ns` | Exact output | Runs the complete native-state continuation source generated from `untrace-array-data.json` |
| `untrace.ns` | `run examples/untrace.ns` | Exact output | Runs the in-language synthesizer and returns its recursive operation strand |
| `untrace-with-errors.ns` | `run examples/untrace-with-errors.ns` | Exact output | Allows a held-out error ratio of one fifth and returns the most repeatable supported continuation |
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
| `prime_count.ns` | `derive --source examples/prime_count.ns prime_count_example` | Function library | Expands the prime-counting camera example |
| `dual_alignment.ns` | `derive --source examples/dual_alignment.ns dual_alignment_example` | Function library | Expands the two reflected axes whose midpoint is half of the multiplicative identity |
| `re_critical_line.ns` | `derive --source examples/re_critical_line.ns re_critical_line_example` | Function library | Expands the centered RE height, including the half-identity construction; it is not a quantified critical-line proof |
| `zeta.ns` | `derive --source examples/zeta.ns zeta_example` | Function library | Expands the zeta graph centered at ADD zero; it does not shift zeta by one half or execute analytic continuation |
| `recursive-pattern.ns` | `derive --source examples/recursive-pattern.ns quarter_turn_pattern` | Recursive function library | Reports one finite operation and one self-reference edge |

The six files under [`examples/applications/`](examples/applications/) are
closed finite zero witnesses. They test the algebraic identity written in each
file, not the application hypothesis or an empirical performance gain.
