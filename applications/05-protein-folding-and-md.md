<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Protein Folding and Molecular Dynamics

**Status:** finite superposition proved; molecular camera and scientific gains
open.

## Proposed native map

Use an SE(3)-aware camera to separate global translation/rotation, map residue
or interaction identity to INDEX, gain/orientation to coefficients, and
contacts to an explicit selector. A decoder must preserve chirality, chain
connectivity, and declared constraints. Static structure, thermodynamics,
kinetics, and trajectory prediction remain distinct tasks.

## T-PROT-CORE-1 -- finite interactions superpose independent of order [Proved]

For finite $F_1,\ldots,F_n$ and any permutation $\sigma$,

$$
\mathrm{ADD}(F_1,\ldots,F_n)
=\mathrm{ADD}(F_{\sigma(1)},\ldots,F_{\sigma(n)}).
$$

**Proof.** L-NS-2 proves native ADD associative and commutative. Every finite
permutation is a finite sequence of adjacent swaps. Each swap preserves the
ADD, and associativity permits regrouping. The folds are equal; adding the
two-turn ORIENT of one to the other gives $\mathsf0$. $\square$

Executable instance:
[molecular-interaction-order.ns](../examples/applications/molecular-interaction-order.ns).

**Boundary:** this proves superposition only after interactions are encoded. It
does not prove molecular forces have this form, define a force field, preserve
chirality, predict folding, or accelerate sampling.

## H-PROT-1 / E-PROT-1 [Hypothesis / Planned experiment]

On alanine dipeptide and one licensed fast-folding protein, compare against
Cartesian/internal-coordinate compression, PCA, TICA/kinetic models, and
matched latent transitions. Split contiguous blocks and independent runs.
Measure rate-distortion, bond/chirality violations, free-energy error, implied
timescales, transition rates, populations, rollout stability, and total
compute. Refute the candidate if RMSD improves while kinetics, populations, or
constraints worsen.

Only a passed pilot justifies larger proteins, binding systems, enhanced
sampling, design, enzyme, or drug-discovery claims.

## Primary sources

- [AlphaFold paper](https://www.nature.com/articles/s41586-021-03819-2)
- [RCSB Protein Data Bank API](https://data.rcsb.org/)
- [OpenMM guide](https://docs.openmm.org/latest/userguide/)
- [MDShare trajectories](https://markovmodel.github.io/mdshare/)
- [Anton trajectory portal](https://data.anton.psc.edu/)
