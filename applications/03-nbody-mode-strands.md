<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Three-Body and N-Body Mode Strands

**Status:** perspective-zero law proved; trajectory camera and gains open.

## Proposed native map

After removing translation and declaring rotation/scale cameras, represent each
body by a small finite ADD of indexed strands with evolving coefficients.
Encode and Decode must recover positions and velocities without future samples.
The missing camera must handle periodic, resonant, chaotic, near-encounter, and
escape regimes.

## T-NBODY-CORE-1 -- zero center is perspective-invariant [Proved]

For finite $F_1,\ldots,F_n$ and quarter orientation $r$,

$$
\mathrm{ADD}(F_1,\ldots,F_n)=\mathsf0
\quad\Longleftrightarrow\quad
\mathrm{ORIENT}_r(\mathrm{ADD}(F_1,\ldots,F_n))=\mathsf0.
$$

**Proof.** T-PERSPECTIVE-ZERO-1 applies. By L-SEP-3, ORIENT is multiplication
by $\eta(\mathbf J^{\boxtimes r})$. L-NS-8 sends zero to zero. Conversely,
L-OS-10 and L-NS-6 give inverse
$\eta(\mathbf J^{\boxtimes(4-r)})$; applying it recovers the original ADD.
$\square$

Executable instance:
[nbody-perspective-zero.ns](../examples/applications/nbody-perspective-zero.ns).

**Boundary:** calling this ADD a center-of-mass condition requires an unproved
mass/position camera. The result does not encode gravity, integrate motion,
preserve physical invariants, or compress chaos.

## H-NBODY-1 / E-NBODY-1 [Hypothesis / Planned experiment]

Compare a native strand model against Fourier, DMD/Koopman, recurrent, and
neural-ODE baselines at equal bytes and rollout compute on frozen REBOUND
trajectories. Hold out initial-condition families. Report position/velocity
error by horizon, energy and angular-momentum drift, event classification,
memory, and latency. Refute the candidate if interpolation wins but rollout or
invariant preservation loses.

## Primary sources

- [REBOUND integrator](https://rebound.hanno-rein.de/)
- [REBOUND paper](https://arxiv.org/abs/1110.4876)
- [REBOUND documentation](https://rebound.readthedocs.io/)
- [JPL Horizons manual](https://ssd.jpl.nasa.gov/horizons/manual.html)
- [Hamiltonian Neural Networks](https://arxiv.org/abs/1906.01563)
