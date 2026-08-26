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
