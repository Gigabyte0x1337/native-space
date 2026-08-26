<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Stage 8 Evaluation and Claim Protocol

This document defines planned experiments. It contains no observed result.
The evidence rules in `../theory/00-method-and-status.md` remain authoritative.

## Common representation contract

Every prototype must publish a typed tuple

$$
R=(\mathcal C,\mathcal S,\Theta,\mathrm{Encode},
\mathrm{Evolve},\mathrm{Decode}),
$$

where:

- $\mathcal C$ is the source coordinate domain;
- $\mathcal S$ is the exact or approximate native-state domain;
- $\Theta$ contains all learned parameters, masks, codebooks, and metadata;
- Encode and Decode have measured costs and errors;
- Evolve is the native operation or learned interaction law;
- every lossy camera and non-core operation is named explicitly.

“Frequency,” “rotation,” or “selection” is not a sufficient definition. The
record must state its type, units, discrete/continuous domain, and how equality
is tested.

Every application dossier must also contain:

- the strongest native theorem that already applies, with its proof ID;
- a complete proof in ADD, MULTIPLY, ORIENT, and INDEX terms;
- a closed `.ns` zero equality whenever the exact 1.0 language can
  express one;
- the precise camera, selector, physical law, or approximation not supplied by
  that theorem; and
- an experiment capable of refuting the application hypothesis.

A finite witness is evidence for the implementation and that instance. It is
not evidence that a proposed application camera exists or compresses data.

## Equal-budget rule

Compare methods at matched:

- serialized bytes, including indices, masks, quantization scales, metadata,
  and model-specific decoder parameters;
- numerical precision;
- fitting/training data and update budget;
- hardware, batch shape, and warm-up;
- quality or error target;
- end-to-end path, including Encode, transforms, materialization, and Decode.

Report both amortized and one-instance costs. A fast inner operation is not a
speedup if conversion or memory movement erases the gain.

## Data and split rule

Every experiment must lock before fitting:

1. immutable train/calibration/validation/test identifiers;
2. dataset version, checksum, source, and license;
3. in-distribution and out-of-distribution splits;
4. all preprocessing and symmetry normalization;
5. random seeds and stopping rules.

Synthetic data may diagnose mechanisms but cannot establish practical value by
itself.

## Baselines

At least one baseline must be structurally strong for the target:

- matrices: truncated SVD, sparse, tensor, and quantized baselines where
  applicable;
- dynamics: the source numerical integrator plus Koopman/DMD, Fourier/modal,
  and generic learned-transition baselines;
- neural models: uncompressed, quantized, low-rank, and architecture-matched
  ablations;
- molecular systems: Cartesian/internal-coordinate compression and established
  kinetic dimensionality reduction.

Hyperparameter search budgets must be equal. A deliberately weak baseline
invalidates the comparison.

## Metrics and uncertainty

Every result reports:

- task error and at least one application-specific invariant;
- serialized size, peak memory, latency, throughput, and fitting cost;
- mean, median, dispersion, and confidence intervals across locked seeds or
  systems;
- the complete Pareto frontier rather than one chosen operating point;
- negative and failed runs.

## Universal pilot gate

A pilot advances only if the native candidate either:

1. reduces median task error by at least 10% at the same serialized-byte budget
   relative to the best admitted baseline; or
2. improves measured end-to-end throughput by at least 25% at a predeclared
   non-inferior task-error threshold.

The 95% bootstrap confidence interval for the improvement must exclude zero,
and the result must hold across at least three seeds and two non-synthetic
instances. Domain dossiers may impose stronger gates.

Failure to produce any Pareto improvement is a recorded refutation of that
representation and budget, not a reason to change the metric after inspection.

## Claim promotion

| Before execution | After valid execution |
|---|---|
| `H-*` Hypothesis | remains Hypothesis or becomes Refuted |
| `E-*` Planned experiment | becomes Observed only with immutable artifacts |
| “candidate mapping” | may become an approximation with measured error |
| “potential impact” | may become a scoped measured gain |

No experiment can promote an empirical result to a mathematical proof.
Likewise, an algebraic zero-equality result cannot promote an application
hypothesis to an observed gain.
