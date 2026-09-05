// SPDX-License-Identifier: AGPL-3.0-or-later

import { performance } from "node:perf_hooks";

import { compareWaterStates, ParticleWater } from "./water-model.mjs";

const CASES = Object.freeze([
  Object.freeze({ counts: [4, 4, 4], steps: 20 }),
  Object.freeze({ counts: [6, 6, 6], steps: 14 }),
  Object.freeze({ counts: [8, 8, 8], steps: 10 }),
  Object.freeze({ counts: [10, 10, 8], steps: 8 }),
  Object.freeze({ counts: [12, 10, 10], steps: 6 }),
  Object.freeze({ counts: [16, 12, 12], steps: 4 }),
  Object.freeze({ counts: [19, 15, 14], steps: 3 }),
]);
const ROUNDS = 5;
const FIXED_STEP = 1 / 240;

function median(values) {
  const ordered = [...values].sort((left, right) => left - right);
  const middle = Math.floor(ordered.length / 2);
  return ordered.length % 2 === 0
    ? (ordered[middle - 1] + ordered[middle]) / 2
    : ordered[middle];
}

function runTimed(state, steps) {
  const start = performance.now();
  for (let step = 0; step < steps; step += 1) {
    state.step(FIXED_STEP);
  }
  return performance.now() - start;
}

function benchmarkCase(testCase) {
  const initial = new ParticleWater({ counts: testCase.counts });
  initial.splash(
    [initial.bounds[0] / 2, initial.bounds[1] * 0.34, initial.bounds[2] / 2],
    { radius: 1.15, strength: 3.6 },
  );

  const backends = ["all-pairs", "indexed", "contracted-indexed"];
  const states = Object.fromEntries(
    backends.map((backend) => [backend, initial.clone({ backend })]),
  );
  // Allocate, grow buffers, and warm each path before measuring. State copies,
  // initialization, and cloning are excluded from the timed region.
  for (const state of Object.values(states)) {
    runTimed(state, 6);
    state.copyStateFrom(initial);
  }

  const times = Object.fromEntries(backends.map((backend) => [backend, []]));
  for (let round = 0; round < ROUNDS; round += 1) {
    // Rotate order so a consistently hotter later run does not favor one backend.
    const offset = round % backends.length;
    const order = [...backends.slice(offset), ...backends.slice(0, offset)];
    for (const backend of order) {
      const state = states[backend];
      state.copyStateFrom(initial);
      times[backend].push(runTimed(state, testCase.steps));
    }
  }

  const allPairsMilliseconds = median(times["all-pairs"]);
  const indexedMilliseconds = median(times.indexed);
  const contractedMilliseconds = median(times["contracted-indexed"]);
  const comparedAllPairs = states["all-pairs"];
  const comparedIndexed = states.indexed;
  const comparedContracted = states["contracted-indexed"];
  const allPairCandidates = comparedAllPairs.lastDiagnostics.candidateInteractions;
  const indexedCandidates = comparedIndexed.lastDiagnostics.candidateInteractions;
  const contractedCandidates = comparedContracted.lastDiagnostics.candidateInteractions;
  return Object.freeze({
    particles: initial.particleCount,
    steps: testCase.steps,
    allPairsMilliseconds,
    indexedMilliseconds,
    contractedMilliseconds,
    indexedSpeedup: allPairsMilliseconds / indexedMilliseconds,
    contractionSpeedup: indexedMilliseconds / contractedMilliseconds,
    totalSpeedup: allPairsMilliseconds / contractedMilliseconds,
    indexedCandidateReduction: 1 - indexedCandidates / allPairCandidates,
    contractedCandidateReduction: 1 - contractedCandidates / allPairCandidates,
    allPairCandidates,
    indexedCandidates,
    contractedCandidates,
    supportedInteractions: comparedIndexed.lastDiagnostics.supportedInteractions,
    contractedPairs: comparedContracted.lastDiagnostics.storedInteractions,
    indexedStateError: compareWaterStates(comparedAllPairs, comparedIndexed),
    contractedStateError: compareWaterStates(comparedAllPairs, comparedContracted),
  });
}

const results = CASES.map(benchmarkCase);
if (process.argv.includes("--json")) {
  console.log(JSON.stringify({ rounds: ROUNDS, fixedStep: FIXED_STEP, results }, null, 2));
} else {
  console.log("Finite 3D SPH: identical equations, indexed and coefficient-contracted execution");
  console.log(`Median of ${ROUNDS} rounds; initialization excluded; fixed dt ${FIXED_STEP.toFixed(6)} s`);
  console.table(results.map((result) => ({
    particles: result.particles,
    steps: result.steps,
    "all-pairs ms": result.allPairsMilliseconds.toFixed(2),
    "INDEX ms": result.indexedMilliseconds.toFixed(2),
    "contracted ms": result.contractedMilliseconds.toFixed(2),
    "contract / INDEX": `${result.contractionSpeedup.toFixed(2)}x`,
    "total speedup": `${result.totalSpeedup.toFixed(2)}x`,
    "checks removed": `${(result.contractedCandidateReduction * 100).toFixed(1)}%`,
    "max state error": Math.max(...Object.values(result.contractedStateError)).toExponential(1),
  })));
}
