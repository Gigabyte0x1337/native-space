<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Navier–Stokes and Turbulence

**Status:** finite interaction associativity proved; fluid camera, empirical
gain, and regularity remain open.

## Proposed native map

A declared spectral camera would map mode identity to INDEX, oriented gain to
coefficients, quadratic interaction to MULTIPLY, composition to ADD, and
incompressibility/triad compatibility to an explicit selector. This must add
value beyond standard Fourier triad structure. The existing finite cyclic DFT
proof is not a divergence-free continuum camera.

## T-NS-CORE-1 -- finite indexed interactions compose [Proved]

For finite native states $F,G,H$,

$$
\mathrm{MULTIPLY}(F,\mathrm{MULTIPLY}(G,H))
=\mathrm{MULTIPLY}(\mathrm{MULTIPLY}(F,G),H).
$$

**Proof.** A left term has index
$\alpha\oplus_I(\beta\oplus_I\gamma)$ and coefficient
$F_\alpha\boxtimes(G_\beta\boxtimes H_\gamma)$. L-IDX-2 and L-OS-3 turn
these into the right-associated index and coefficient. The same finite triples
contribute on both sides, and L-OS-1 equates their ADD folds. This is L-NS-5.
Their two-turn-ORIENT residual is zero by L-NS-2. $\square$

Executable instance:
[navier-stokes-index-composition.ns](../examples/applications/navier-stokes-index-composition.ns).

**Boundary:** this does not identify INDEX with signed wavevectors, encode
divergence-free selection, take a continuum limit, or control energy transfer
to infinite frequency. It proves no Navier–Stokes regularity result.

## H-NS-1 / E-NS-1 [Hypothesis / Planned experiment]

On frozen JHTDB time blocks, compare native transfer operators against
spectral/Galerkin, POD/DMD, sparse operator-inference, and matched learned
baselines. Measure coefficient and shell-transfer error, spectrum, dissipation,
divergence residual, rollout stability, bytes, and latency. Refute the
candidate if sparsity is only ordinary triad selection, disappears with
resolution, or violates divergence/dissipation.

Regularity work starts only if a dataset-independent invariant controls
arbitrarily high frequency and survives refinement. Until proved, it remains a
conjecture.

## Primary sources

- [Clay Navier–Stokes problem statement](https://www.claymath.org/wp-content/uploads/2022/02/MPPc.pdf)
- [Johns Hopkins Turbulence Database](https://turbulence.pha.jhu.edu/)
- [Forced isotropic turbulence data](https://turbulence.pha.jhu.edu/Forced_isotropic_turbulence.aspx)
- [JHTDB citation guidance](https://turbulence.pha.jhu.edu/citing.aspx)
