import assert from "node:assert/strict";
import test from "node:test";

import {
  BRAID_TRACK_COUNT,
  classicalPrimeRecords,
  firstPrimes,
  generatePrimeCameraData,
  indexedBraidWindow,
  orientationForBirthIndex,
  numberLineHitsFromOrientationCounts,
  projectCrossSection,
  DISPLAY_RADIUS,
} from "./model.mjs";

test("the native birth clock follows J, -1, -J, 1", () => {
  assert.deepEqual(
    [1, 2, 3, 4, 5].map((index) => orientationForBirthIndex(index).symbol),
    ["J", "−1", "−J", "1", "J"],
  );
});

test("the classical prime-value camera begins with the classical primes", () => {
  assert.deepEqual(Array.from(firstPrimes(10).primes), [2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
});

test("the explicit classical check contains the exact first thirty prime births", () => {
  const records = classicalPrimeRecords();
  assert.deepEqual(records.map(({ prime }) => prime), [
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29,
    31, 37, 41, 43, 47, 53, 59, 61, 67, 71,
    73, 79, 83, 89, 97, 101, 103, 107, 109, 113,
  ]);
  assert.deepEqual(records.slice(0, 8).map(({ index, classicalOrientation }) => [index, classicalOrientation]), [
    [1, "i"], [2, "−1"], [3, "−i"], [4, "1"],
    [5, "i"], [6, "−1"], [7, "−i"], [8, "1"],
  ]);
});

test("the four cameras preserve their exact cross-section laws", () => {
  for (const [x, z] of [[0, 4], [-4, 0], [0, -4], [4, 0]]) {
    const projected = projectCrossSection(x, z);
    assert.deepEqual(projected.classical, [x, z]);
    assert.deepEqual(projected.zeta, [(x + z) / 2, (x + z) / 2]);
    assert.deepEqual(projected.re, [(x - z) / 2, (-x + z) / 2]);
    assert.equal(projected.zeta[0] + projected.re[0], projected.classical[0]);
    assert.equal(projected.zeta[1] + projected.re[1], projected.classical[1]);
  }
});

test("the classical compass preserves four directions before the one-axis collisions", () => {
  const hits = numberLineHitsFromOrientationCounts([250_000, 250_000, 250_000, 250_000]);
  assert.deepEqual(hits.classical.map(({ coordinate, count }) => [coordinate, count]), [
    ["i", 250_000], ["1", 250_000], ["−i", 250_000], ["−1", 250_000],
  ]);
  assert.deepEqual(hits.zeta.map(({ coordinate, count }) => [coordinate, count]), [
    ["−1/2", 500_000], ["+1/2", 500_000],
  ]);
  assert.deepEqual(hits.re.map(({ coordinate, count }) => [coordinate, count]), [
    ["−1/2", 500_000], ["+1/2", 500_000],
  ]);
});

test("the native camera retains exact orientation strands and declares every braid vertex", () => {
  const { cameras, summary } = generatePrimeCameraData(400);
  assert.deepEqual(summary.nativeOrientationCounts, [100, 100, 100, 100]);
  assert.equal(summary.braidTrackCount, BRAID_TRACK_COUNT);
  assert.equal(summary.indexedBraidVertexCount, 400 * BRAID_TRACK_COUNT * 4);

  cameras.native.forEach((coordinates, remainder) => {
    const orientation = orientationForBirthIndex(remainder === 0 ? 4 : remainder);
    for (let offset = 0; offset < coordinates.length; offset += 3) {
      assert.equal(coordinates[offset], orientation.x * DISPLAY_RADIUS);
      assert.equal(coordinates[offset + 2], orientation.z * DISPLAY_RADIUS);
    }
  });
});

test("the lower readouts preserve INDEX and all three shifted braid tracks", () => {
  const classical = indexedBraidWindow("classical", 1, 4);
  const zeta = indexedBraidWindow("zeta", 1, 4);
  const re = indexedBraidWindow("re", 1, 4);

  assert.deepEqual(classical[0].map(({ index }) => index), [1, 2, 3, 4]);
  assert.deepEqual(classical[0].map(({ coordinate }) => coordinate), [0, 3, 2, 1]);
  assert.deepEqual(classical[1].map(({ coordinate }) => coordinate), [3, 2, 1, 0]);
  assert.deepEqual(classical[2].map(({ coordinate }) => coordinate), [2, 1, 0, 3]);
  assert.deepEqual(zeta[0].map(({ coordinate }) => coordinate), [0.5, -0.5, -0.5, 0.5]);
  assert.deepEqual(re[0].map(({ coordinate }) => coordinate), [-0.5, -0.5, 0.5, 0.5]);
});

test("the display height is the finite birth INDEX rather than prime magnitude", () => {
  const { cameras } = generatePrimeCameraData(5);
  const heights = Array.from({ length: 5 }, (_, index) => cameras.classical[index * 3 + 1]);
  assert.deepEqual(heights, [-8, -4, 0, 4, 8]);
});

test("the million-iteration experiment reaches the millionth prime in all four cameras", { timeout: 30_000 }, () => {
  const { cameras, summary } = generatePrimeCameraData(1_000_000);
  assert.equal(summary.iterationCount, 1_000_000);
  assert.equal(summary.projectedVertexCount, 4_000_000);
  assert.equal(summary.indexedBraidVertexCount, 12_000_000);
  assert.equal(summary.lastPrime, 15_485_863);
  assert.deepEqual(summary.nativeOrientationCounts, [250_000, 250_000, 250_000, 250_000]);
  assert.equal(cameras.native.reduce((total, values) => total + values.length / 3, 0), 1_000_000);
  assert.equal(cameras.classical.length / 3, 1_000_000);
  assert.equal(cameras.zeta.length / 3, 1_000_000);
  assert.equal(cameras.re.length / 3, 1_000_000);
});
