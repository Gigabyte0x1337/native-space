<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Stage 8 Application Program

Stage 8 asks whether Native Space is useful, not merely coherent. This directory
contains application dossiers, their evaluation protocol, and a portfolio
plan. The application claims are falsifiable research proposals; none is
currently an observed gain.

Each application separates three layers:

1. a universal finite theorem already proved in the native algebra;
2. a closed `.ns` zero equality checked by both exact evaluators; and
3. the missing application camera or empirical hypothesis.

The witness proves its exact finite instance. The linked paper theorem proves
the universal algebraic statement. Neither one proves that the proposed map to
matrices, learned latents, trajectories, fluid modes, molecules, or programs is
useful.

The common candidate is a sparse operational description built from:

- frequency or mode identity selected by a declared camera;
- gain stored in coefficients;
- quarter orientation from the exact core, or continuous rotation from an
  explicitly approximate/analytic extension;
- INDEX depth and multi-index interaction structure;
- selection as a declared sparse mask, gate, or interaction rule.

Selection and arbitrary continuous frequency/rotation are not silently added
to the 1.0 core. Each application must define how those quantities map to
ADD, MULTIPLY, ORIENT, INDEX, and a camera.

## Attack order

| Priority | Program | Why now | Stop/go gate |
|---:|---|---|---|
| 1 | Matrix/operator compression | Cheapest direct test of frequency + gain + rotation + selection | Beat the best equal-byte baseline on held-out operators |
| 2 | Minimal programming language | Exact strings make a cheap self-representation test possible | Prove typed data-camera round trips and evaluator/compiler agreement |
| 3 | Three-body/N-body strands | Exact simulator baselines and strong conservation checks | Improve the error/cost frontier without worse invariant drift |
| 4 | Protein molecular dynamics | Very high scientific value and public trajectories | Compress or predict held-out dynamics without losing kinetic observables |
| 5 | Turbulence/Navier–Stokes | Strong mode-interaction fit but high dimensionality | Demonstrate sparse nonlinear transfer on DNS before any regularity claim |
| 6 | JEPA/diffusion and LLM integration | Large potential impact but expensive and confounded | Proceed only after the operator representation wins smaller tests |

The ordering is experimental, not a judgment of ultimate impact. Matrix and
N-body work are designed to kill weak formulations quickly before expensive
training or simulation.

## Theory dependencies and missing application maps

| Program | Existing proved foundation | Definition still required before execution |
|---|---|---|
| Matrix/LLM | [finite native ring and self-action](../proofs/01-core-algebra.md), [typed language semantics](../proofs/04-language-semantics.md) | Matrix/tensor Encode and Decode camera; selection-mask type; approximate rotation policy |
| JEPA/diffusion | finite ring, [native flow](../proofs/05-native-flows.md), [finite derivative laws](../proofs/03-native-derivatives.md) | Latent-state camera; time/noise INDEX meaning; SELECT semantics |
| N-body | finite ring, native flow, finite derivatives, [cone/PSD camera classification](../proofs/02-camera-equivalences.md) | Symmetry-reduced Cartesian-to-strand camera and inverse/error rule |
| Navier–Stokes | finite ring and [finite Fourier reconstruction](../reconstructions/07-finite-convolution-theorem.md) | Divergence-free spatial spectral camera, triad selector, resolution/refinement map; the finite cyclic result is not that continuum camera |
| Protein MD | finite ring, native flow, cone/PSD cameras | SE(3)-aware molecular camera, chirality/constraint-preserving decoder, contact selector |
| Programming language | T-LANG-UTF8-1, finite ring, T-ACT-1, exact VM | Collision-free typed cameras for symbols/sequences/records/AST; scope, control flow, effects, and self-hosting proof |

## Executable native witnesses

- [matrix distributivity](../examples/applications/matrix-distributivity.ns)
- [dynamical ORIENT equivariance](../examples/applications/dynamics-orient-equivariance.ns)
- [N-body perspective-zero preservation](../examples/applications/nbody-perspective-zero.ns)
- [fluid-mode interaction composition](../examples/applications/navier-stokes-index-composition.ns)
- [molecular interaction ADD order](../examples/applications/molecular-interaction-order.ns)
- [programming-language strings and AST fields](../examples/applications/programming-language-data.ns)

From the repository root, run any witness with:

```powershell
cargo run --manifest-path language/runtime/Cargo.toml --release --bin native-space -- check <file>
```

A successful result says `Valid zero proof`; a nonzero residual fails the
check. The current final-zero diagnostic identifies the document but does not
yet include a primitive provenance trace.

An experiment cannot move from Planned to Observed until its missing map is
versioned and its round-trip or approximation error is tested.

## Documents

- [Evaluation and claim protocol](00-evaluation-protocol.md)
- [Matrix and LLM compression](01-matrix-and-llm-compression.md)
- [JEPA and diffusion dynamics](02-jepa-and-diffusion.md)
- [Three-body and N-body mode strands](03-nbody-mode-strands.md)
- [Navier–Stokes and turbulence](04-navier-stokes-turbulence.md)
- [Protein folding and molecular dynamics](05-protein-folding-and-md.md)
- [Impact ladder and portfolio gates](06-impact-and-priorities.md)
- [Programming-language path](07-programming-language.md)
- [One-million-prime four-perspective simulation](08-prime-four-perspective-simulation.md)
