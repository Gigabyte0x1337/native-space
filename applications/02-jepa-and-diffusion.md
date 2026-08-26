<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# JEPA and Diffusion as Native Dynamical Systems

**Status:** finite equivariance proved; latent camera and model gains open.

## Proposed native map

Represent a latent step by

$$
F_{t+1}\approx\mathrm{SELECT}_{S_t}
\left(A_t\star F_t\oplus B_t\star C_t\oplus N_t\right).
$$

The candidate uses coefficients for gain/orientation, INDEX for shared mode and
time identity, MULTIPLY for interaction, and ADD for composition. SELECT,
latent Encode/Decode, continuous rotations, and noise/time semantics are not
core operations and remain undefined.

## T-DYN-CORE-1 -- affine steps are quarter-ORIENT equivariant [Proved]

For finite $A,F,C$ and $r\in\mathbb{Z}_4$,

$$
\mathrm{ORIENT}_r(\mathrm{ADD}(\mathrm{MULTIPLY}(A,F),C))
=\mathrm{ADD}(\mathrm{MULTIPLY}(A,\mathrm{ORIENT}_r(F)),
\mathrm{ORIENT}_r(C)).
$$

**Proof.** L-SEP-3 makes `ORIENT(r, X)` equal
$\eta(\mathbf J^{\boxtimes r})\star X$. L-NS-7 distributes this product over
ADD. L-NS-5 associates and commutes the first product until the orientation
factor multiplies $F$. L-SEP-3 in reverse gives the right side. Their native
difference is zero by L-NS-2. $\square$

Executable instance:
[dynamics-orient-equivariance.ns](../examples/applications/dynamics-orient-equivariance.ns).

**Boundary:** this proves one finite affine identity, not JEPA prediction,
diffusion denoising, SELECT, sample quality, or compression.

## H-DYN-1 / E-DYN-1 [Hypothesis / Planned experiment]

Freeze an official small JEPA or diffusion checkpoint and immutable latent
pairs. Fit native transition atoms and equal-byte dense, low-rank, sparse, MLP,
and time-conditioned baselines. Measure latent error, downstream/sample
quality, latency, memory, and cross-time atom reuse. Refute the candidate if
atom sets do not transfer across time, downstream quality falls, or selection
and decoding remove any measured saving.

## Primary sources

- [I-JEPA publication](https://ai.meta.com/research/publications/self-supervised-learning-from-images-with-a-joint-embedding-predictive-architecture/)
- [Official I-JEPA code](https://github.com/facebookresearch/ijepa)
- [Official V-JEPA code](https://github.com/facebookresearch/jepa)
- [Denoising Diffusion Probabilistic Models](https://arxiv.org/abs/2006.11239)
- [Score-based modeling through SDEs](https://arxiv.org/abs/2011.13456)
- [Denoising Diffusion Implicit Models](https://openreview.net/pdf?id=St1giarCHLP)
