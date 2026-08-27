export const ITERATION_COUNT = 1_000_000;
export const DISPLAY_HALF_HEIGHT = 8;
export const DISPLAY_RADIUS = 4;
export const BRAID_TRACK_COUNT = 3;
export const INDEX_READOUT_COUNT = 16;

// The native birth index is one-based: k = 1 starts at J^1 = J.
export const NATIVE_ORIENTATIONS = Object.freeze([
  Object.freeze({ remainder: 0, symbol: "1", x: 1, z: 0 }),
  Object.freeze({ remainder: 1, symbol: "J", x: 0, z: 1 }),
  Object.freeze({ remainder: 2, symbol: "−1", x: -1, z: 0 }),
  Object.freeze({ remainder: 3, symbol: "−J", x: 0, z: -1 }),
]);

export function nthPrimeUpperBound(count) {
  if (!Number.isSafeInteger(count) || count < 1) {
    throw new RangeError("Prime count must be a positive safe integer.");
  }

  if (count < 6) {
    return 15;
  }

  const logarithm = Math.log(count);
  return Math.ceil(count * (logarithm + Math.log(logarithm))) + 3;
}

export function firstPrimes(count) {
  const limit = nthPrimeUpperBound(count);
  const squareRootLimit = Math.floor(Math.sqrt(limit));
  const composite = new Uint8Array(limit + 1);
  const primes = new Uint32Array(count);
  let found = 0;

  for (let candidate = 2; candidate <= limit && found < count; candidate += 1) {
    if (composite[candidate] !== 0) {
      continue;
    }

    primes[found] = candidate;
    found += 1;

    if (candidate <= squareRootLimit) {
      for (let multiple = candidate * candidate; multiple <= limit; multiple += candidate) {
        composite[multiple] = 1;
      }
    }
  }

  if (found !== count) {
    throw new Error(`Prime bound ${limit} produced ${found} of ${count} requested observations.`);
  }

  return { primes, limit };
}

export function orientationForBirthIndex(index) {
  if (!Number.isSafeInteger(index) || index < 1) {
    throw new RangeError("Birth index must be a positive safe integer.");
  }

  return NATIVE_ORIENTATIONS[index % 4];
}

const CLASSICAL_SYMBOL_BY_REMAINDER = Object.freeze(["1", "i", "−1", "−i"]);

export function classicalPrimeRecords(count = 30) {
  const { primes } = firstPrimes(count);
  return Object.freeze(Array.from(primes, (prime, zeroBased) => {
    const index = zeroBased + 1;
    const orientation = orientationForBirthIndex(index);
    return Object.freeze({
      index,
      prime,
      remainder: orientation.remainder,
      nativeOrientation: orientation.symbol,
      classicalOrientation: CLASSICAL_SYMBOL_BY_REMAINDER[orientation.remainder],
    });
  }));
}

export function projectCrossSection(x, z) {
  return Object.freeze({
    native: Object.freeze([x, z]),
    classical: Object.freeze([x, z]),
    zeta: Object.freeze([(x + z) / 2, (x + z) / 2]),
    re: Object.freeze([(x - z) / 2, (-x + z) / 2]),
  });
}

export function numberLineHitsFromOrientationCounts(orientationCounts) {
  if (!Array.isArray(orientationCounts) || orientationCounts.length !== 4) {
    throw new TypeError("Four native orientation counts are required.");
  }

  const [one, j, minusOne, minusJ] = orientationCounts;
  return Object.freeze({
    classical: Object.freeze([
      Object.freeze({ coordinate: "i", count: j, orientations: "J → i" }),
      Object.freeze({ coordinate: "1", count: one, orientations: "1 → 1" }),
      Object.freeze({ coordinate: "−i", count: minusJ, orientations: "−J → −i" }),
      Object.freeze({ coordinate: "−1", count: minusOne, orientations: "−1 → −1" }),
    ]),
    zeta: Object.freeze([
      Object.freeze({ coordinate: "−1/2", count: minusOne + minusJ, orientations: "−1, −i" }),
      Object.freeze({ coordinate: "+1/2", count: one + j, orientations: "1, i" }),
    ]),
    re: Object.freeze([
      Object.freeze({ coordinate: "−1/2", count: j + minusOne, orientations: "i, −1" }),
      Object.freeze({ coordinate: "+1/2", count: one + minusJ, orientations: "1, −i" }),
    ]),
  });
}

const CLASSICAL_LANE_BY_REMAINDER = Object.freeze([1, 0, 3, 2]);

export function indexedBraidWindow(camera, startIndex = 1, count = INDEX_READOUT_COUNT) {
  if (!new Set(["classical", "zeta", "re"]).has(camera)) {
    throw new RangeError(`Unknown indexed braid camera ${camera}.`);
  }
  if (!Number.isSafeInteger(startIndex) || startIndex < 1) {
    throw new RangeError("The indexed braid window must start at a positive safe integer.");
  }
  if (!Number.isSafeInteger(count) || count < 2) {
    throw new RangeError("The indexed braid window must contain at least two vertices.");
  }

  return Object.freeze(Array.from({ length: BRAID_TRACK_COUNT }, (_, track) =>
    Object.freeze(Array.from({ length: count }, (_, offset) => {
      const index = startIndex + offset;
      const orientation = orientationForBirthIndex(index + track);
      let coordinate;

      if (camera === "classical") {
        coordinate = CLASSICAL_LANE_BY_REMAINDER[orientation.remainder];
      } else if (camera === "zeta") {
        coordinate = (orientation.x + orientation.z) / 2;
      } else {
        coordinate = (orientation.x - orientation.z) / 2;
      }

      return Object.freeze({ index, coordinate, orientation: orientation.symbol });
    })),
  ));
}

function writeVertex(target, offset, x, height, z) {
  target[offset] = x;
  target[offset + 1] = height;
  target[offset + 2] = z;
}

function normalizedIndexHeight(index, count) {
  if (count === 1) {
    return 0;
  }

  return ((index - 1) / (count - 1)) * DISPLAY_HALF_HEIGHT * 2 - DISPLAY_HALF_HEIGHT;
}

export function generatePrimeCameraData(count = ITERATION_COUNT) {
  const startedAt = performance.now();
  const { primes, limit } = firstPrimes(count);
  const orientationCounts = NATIVE_ORIENTATIONS.map(({ remainder }) =>
    Math.floor((count + ((4 - remainder) % 4)) / 4),
  );
  const native = orientationCounts.map(
    (orientationCount) => new Float32Array(orientationCount * 3),
  );
  const classical = new Float32Array(count * 3);
  const zeta = new Float32Array(count * 3);
  const re = new Float32Array(count * 3);
  const nativeOffsets = new Uint32Array(4);
  const lastPrime = primes[count - 1];
  let checksum32 = 0;

  for (let zeroBased = 0; zeroBased < count; zeroBased += 1) {
    const index = zeroBased + 1;
    const prime = primes[zeroBased];
    const orientation = orientationForBirthIndex(index);
    const nativeOffset = nativeOffsets[orientation.remainder] * 3;
    const cameraOffset = zeroBased * 3;
    const height = normalizedIndexHeight(index, count);
    const sourceX = orientation.x * DISPLAY_RADIUS;
    const sourceZ = orientation.z * DISPLAY_RADIUS;
    const zetaCoordinate = (sourceX + sourceZ) / 2;
    const reX = (sourceX - sourceZ) / 2;
    const reZ = (-sourceX + sourceZ) / 2;

    writeVertex(
      native[orientation.remainder],
      nativeOffset,
      sourceX,
      height,
      sourceZ,
    );
    writeVertex(classical, cameraOffset, sourceX, height, sourceZ);
    writeVertex(zeta, cameraOffset, zetaCoordinate, height, zetaCoordinate);
    writeVertex(re, cameraOffset, reX, height, reZ);

    nativeOffsets[orientation.remainder] += 1;
    checksum32 = (checksum32 + prime) >>> 0;
  }

  const numberLineHits = numberLineHitsFromOrientationCounts(Array.from(nativeOffsets));

  return {
    cameras: { native, classical, zeta, re },
    summary: Object.freeze({
      iterationCount: count,
      projectedVertexCount: count * 4,
      indexedBraidVertexCount: count * BRAID_TRACK_COUNT * 4,
      braidTrackCount: BRAID_TRACK_COUNT,
      firstPrime: primes[0],
      lastPrime,
      sieveLimit: limit,
      nativeOrientationCounts: Array.from(nativeOffsets),
      numberLineHits,
      checksum32,
      elapsedMilliseconds: performance.now() - startedAt,
    }),
  };
}
