// SPDX-License-Identifier: AGPL-3.0-or-later

import assert from "node:assert/strict";
import test from "node:test";

import { compareWaterStates, ParticleWater } from "./water-model.mjs";

const TEST_OPTIONS = Object.freeze({
  counts: [5, 5, 4],
  bounds: [3.2, 3.2, 3.2],
});

test("all-pairs, indexed, and contracted execution are numerically equivalent", () => {
  const initial = new ParticleWater(TEST_OPTIONS);
  initial.splash([1.6, 1.1, 1.6], { radius: 0.9, strength: 2.5 });
  const allPairs = initial.clone({ backend: "all-pairs" });
  const indexed = initial.clone({ backend: "indexed" });
  const contracted = initial.clone({ backend: "contracted-indexed" });

  for (let step = 0; step < 12; step += 1) {
    allPairs.step(1 / 240);
    indexed.step(1 / 240);
    contracted.step(1 / 240);
  }

  for (const error of [
    compareWaterStates(allPairs, indexed),
    compareWaterStates(allPairs, contracted),
  ]) {
    assert.ok(error.positions <= 1e-10);
    assert.ok(error.velocities <= 1e-10);
    assert.ok(error.densities <= 1e-9);
    assert.ok(error.pressures <= 1e-8);
  }
  assert.equal(allPairs.lastDiagnostics.supportedInteractions, indexed.lastDiagnostics.supportedInteractions);
  assert.equal(allPairs.lastDiagnostics.supportedInteractions, contracted.lastDiagnostics.supportedInteractions);
});

test("INDEX selection checks fewer candidates than all-pairs", () => {
  const initial = new ParticleWater({ ...TEST_OPTIONS, counts: [7, 7, 6] });
  const allPairs = initial.clone({ backend: "all-pairs" });
  const indexed = initial.clone({ backend: "indexed" });

  allPairs.step(1 / 240);
  indexed.step(1 / 240);

  assert.equal(allPairs.lastDiagnostics.candidateInteractions, initial.particleCount ** 2);
  assert.ok(
    indexed.lastDiagnostics.candidateInteractions < allPairs.lastDiagnostics.candidateInteractions,
    "the spatial INDEX must remove out-of-support candidates",
  );
  assert.equal(allPairs.lastDiagnostics.supportedInteractions, indexed.lastDiagnostics.supportedInteractions);
});

test("coefficient contraction stores each supported non-self pair once", () => {
  const initial = new ParticleWater({ ...TEST_OPTIONS, counts: [7, 7, 6] });
  const indexed = initial.clone({ backend: "indexed" });
  const contracted = initial.clone({ backend: "contracted-indexed" });

  indexed.step(1 / 240);
  contracted.step(1 / 240);

  assert.equal(indexed.lastDiagnostics.supportedInteractions, contracted.lastDiagnostics.supportedInteractions);
  assert.equal(
    contracted.lastDiagnostics.storedInteractions * 2 + initial.particleCount,
    contracted.lastDiagnostics.supportedInteractions,
  );
  assert.ok(
    contracted.lastDiagnostics.storedInteractions < indexed.lastDiagnostics.storedInteractions / 2,
  );
  assert.ok(
    contracted.lastDiagnostics.candidateInteractions < indexed.lastDiagnostics.candidateInteractions / 2,
  );
});

test("the finite state remains inside its declared tank", () => {
  const water = new ParticleWater(TEST_OPTIONS);
  water.splash([1.6, 1.2, 1.6], { radius: 1.2, strength: 7 });
  for (let step = 0; step < 180; step += 1) {
    water.step(1 / 240);
  }

  for (let particle = 0; particle < water.particleCount; particle += 1) {
    const offset = particle * 3;
    for (let axis = 0; axis < 3; axis += 1) {
      assert.ok(Number.isFinite(water.positions[offset + axis]));
      assert.ok(water.positions[offset + axis] >= water.particleRadius);
      assert.ok(water.positions[offset + axis] <= water.bounds[axis] - water.particleRadius);
    }
  }
  assert.ok(Number.isFinite(water.lastDiagnostics.averageDensity));
  assert.ok(water.lastDiagnostics.maximumSpeed <= water.maximumSpeed);
});

test("invalid integration and backend settings fail before execution", () => {
  assert.throws(() => new ParticleWater({ backend: "approximate" }), /Unknown interaction backend/);
  const water = new ParticleWater(TEST_OPTIONS);
  assert.throws(() => water.step(1 / 60), /at most/);
  assert.throws(() => water.setBackend("missing"), /Unknown interaction backend/);
});

test("drag stirring transfers velocity along a finite path", () => {
  const water = new ParticleWater(TEST_OPTIONS);
  water.stir([1.6, 1.1, 1.6], [0.2, 0, -0.1]);
  assert.ok(water.velocities.some((value) => value !== 0));
  assert.ok(water.velocities.every(Number.isFinite));
});
