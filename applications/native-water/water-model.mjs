// SPDX-License-Identifier: AGPL-3.0-or-later

export const INTERACTION_BACKENDS = Object.freeze([
  "all-pairs",
  "indexed",
  "contracted-indexed",
]);

export const DEFAULT_WATER_OPTIONS = Object.freeze({
  counts: Object.freeze([10, 10, 8]),
  bounds: Object.freeze([5.4, 4.8, 4.2]),
  spacing: 0.28,
  smoothingRadius: 0.52,
  restDensity: 1_000,
  pressureStiffness: 1_400,
  viscosity: 0.14,
  gravity: -9.81,
  restitution: 0.28,
  maximumSpeed: 12,
  backend: "contracted-indexed",
});

const MAXIMUM_STEP_SECONDS = 1 / 120;
const BOUNDARY_DAMPING = 0.997;
const INITIAL_PAIR_CAPACITY_PER_PARTICLE = 24;
const FORWARD_CELL_OFFSETS = Object.freeze((() => {
  const offsets = [];
  for (let z = -1; z <= 1; z += 1) {
    for (let y = -1; y <= 1; y += 1) {
      for (let x = -1; x <= 1; x += 1) {
        if (z > 0 || (z === 0 && y > 0) || (z === 0 && y === 0 && x > 0)) {
          offsets.push(Object.freeze([x, y, z]));
        }
      }
    }
  }
  return offsets;
})());

function positiveFinite(value, name) {
  if (!Number.isFinite(value) || value <= 0) {
    throw new RangeError(`${name} must be a positive finite number.`);
  }
  return value;
}

function validateTriplet(value, name, predicate) {
  if (!Array.isArray(value) || value.length !== 3 || !value.every(predicate)) {
    throw new RangeError(`${name} must contain exactly three valid values.`);
  }
  return Object.freeze([...value]);
}

function validateBackend(backend) {
  if (!INTERACTION_BACKENDS.includes(backend)) {
    throw new RangeError(`Unknown interaction backend: ${backend}.`);
  }
  return backend;
}

function maximumAbsoluteDifference(left, right) {
  if (left.length !== right.length) {
    throw new RangeError("Compared arrays must have equal length.");
  }

  let maximum = 0;
  for (let index = 0; index < left.length; index += 1) {
    maximum = Math.max(maximum, Math.abs(left[index] - right[index]));
  }
  return maximum;
}

/**
 * A finite weakly-compressible SPH state.
 *
 * Every backend below represents the same kernels, coefficients, forces, and
 * integration rule. The contracted backend changes evaluation order by
 * retaining one coefficient record per unordered supported pair. This is the
 * invariant that makes the benchmark an execution-structure experiment rather
 * than a comparison between different fluid approximations.
 */
export class ParticleWater {
  constructor(options = {}) {
    const settings = { ...DEFAULT_WATER_OPTIONS, ...options };
    this.counts = validateTriplet(
      settings.counts,
      "Particle counts",
      (value) => Number.isSafeInteger(value) && value > 0,
    );
    this.bounds = validateTriplet(
      settings.bounds,
      "Tank bounds",
      (value) => Number.isFinite(value) && value > 0,
    );
    this.spacing = positiveFinite(settings.spacing, "Particle spacing");
    this.smoothingRadius = positiveFinite(settings.smoothingRadius, "Smoothing radius");
    if (this.smoothingRadius <= this.spacing) {
      throw new RangeError("Smoothing radius must exceed particle spacing.");
    }
    this.restDensity = positiveFinite(settings.restDensity, "Rest density");
    this.pressureStiffness = positiveFinite(settings.pressureStiffness, "Pressure stiffness");
    this.viscosity = positiveFinite(settings.viscosity, "Viscosity");
    if (!Number.isFinite(settings.gravity)) {
      throw new RangeError("Gravity must be finite.");
    }
    if (!Number.isFinite(settings.restitution) || settings.restitution < 0 || settings.restitution > 1) {
      throw new RangeError("Restitution must be between zero and one.");
    }
    this.gravity = settings.gravity;
    this.restitution = settings.restitution;
    this.maximumSpeed = positiveFinite(settings.maximumSpeed, "Maximum speed");
    this.backend = validateBackend(settings.backend);
    for (let axis = 0; axis < 3; axis += 1) {
      const occupiedExtent = (this.counts[axis] - 1) * this.spacing + this.spacing * 0.86;
      if (occupiedExtent > this.bounds[axis]) {
        throw new RangeError(`Particle lattice does not fit tank axis ${axis}.`);
      }
    }

    this.particleCount = this.counts[0] * this.counts[1] * this.counts[2];
    this.mass = this.restDensity * this.spacing ** 3;
    this.particleRadius = this.spacing * 0.43;
    this.positions = new Float64Array(this.particleCount * 3);
    this.velocities = new Float64Array(this.particleCount * 3);
    this.accelerations = new Float64Array(this.particleCount * 3);
    this.densities = new Float64Array(this.particleCount);
    this.pressures = new Float64Array(this.particleCount);
    this.supportLists = Array.from({ length: this.particleCount }, () => []);
    this.gridCounts = this.bounds.map((extent) => Math.ceil(extent / this.smoothingRadius) + 1);
    this.gridBuckets = Array.from(
      { length: this.gridCounts[0] * this.gridCounts[1] * this.gridCounts[2] },
      () => [],
    );
    this.activeGridCells = [];
    this.pairCount = 0;
    this.pairCapacity = 0;
    this.pairLeft = new Int32Array(0);
    this.pairRight = new Int32Array(0);
    this.pairDx = new Float64Array(0);
    this.pairDy = new Float64Array(0);
    this.pairDz = new Float64Array(0);
    this.pairDensityCoefficient = new Float64Array(0);
    this.pairPressureCoefficient = new Float64Array(0);
    this.pairViscosityCoefficient = new Float64Array(0);
    this.lastDiagnostics = Object.freeze({
      backend: this.backend,
      particleCount: this.particleCount,
      candidateInteractions: 0,
      supportedInteractions: 0,
      storedInteractions: 0,
      allPairInteractions: this.particleCount ** 2,
      averageDensity: 0,
      maximumSpeed: 0,
    });

    const h = this.smoothingRadius;
    this.kernelRadiusSquared = h * h;
    this.poly6Coefficient = 315 / (64 * Math.PI * h ** 9);
    this.spikyGradientCoefficient = -45 / (Math.PI * h ** 6);
    this.viscosityLaplacianCoefficient = 45 / (Math.PI * h ** 6);
    this.densityCoefficientScale = this.mass * this.poly6Coefficient;
    this.pressureCoefficientScale = -this.mass * this.spikyGradientCoefficient;
    this.viscosityCoefficientScale = this.viscosity * this.mass * this.viscosityLaplacianCoefficient;
    this.selfDensityCoefficient = this.densityCoefficientScale * this.kernelRadiusSquared ** 3;
    if (this.backend === "contracted-indexed") {
      this.#ensurePairCapacity(this.particleCount * INITIAL_PAIR_CAPACITY_PER_PARTICLE);
    }
    this.reset();
  }

  setBackend(backend) {
    this.backend = validateBackend(backend);
    if (this.backend === "contracted-indexed" && this.pairCapacity === 0) {
      this.#ensurePairCapacity(this.particleCount * INITIAL_PAIR_CAPACITY_PER_PARTICLE);
    }
  }

  reset() {
    this.velocities.fill(0);
    this.accelerations.fill(0);
    this.densities.fill(0);
    this.pressures.fill(0);

    const [countX, countY, countZ] = this.counts;
    const startX = (this.bounds[0] - (countX - 1) * this.spacing) / 2;
    const startZ = (this.bounds[2] - (countZ - 1) * this.spacing) / 2;
    const startY = this.particleRadius + this.spacing * 0.65;
    let particle = 0;
    for (let y = 0; y < countY; y += 1) {
      for (let z = 0; z < countZ; z += 1) {
        for (let x = 0; x < countX; x += 1) {
          const offset = particle * 3;
          this.positions[offset] = startX + x * this.spacing;
          this.positions[offset + 1] = startY + y * this.spacing;
          this.positions[offset + 2] = startZ + z * this.spacing;
          particle += 1;
        }
      }
    }
  }

  copyStateFrom(source) {
    if (!(source instanceof ParticleWater) || source.particleCount !== this.particleCount) {
      throw new RangeError("Water states must have the same particle count.");
    }
    this.positions.set(source.positions);
    this.velocities.set(source.velocities);
    this.accelerations.set(source.accelerations);
    this.densities.set(source.densities);
    this.pressures.set(source.pressures);
  }

  clone({ backend = this.backend } = {}) {
    const clone = new ParticleWater({
      counts: [...this.counts],
      bounds: [...this.bounds],
      spacing: this.spacing,
      smoothingRadius: this.smoothingRadius,
      restDensity: this.restDensity,
      pressureStiffness: this.pressureStiffness,
      viscosity: this.viscosity,
      gravity: this.gravity,
      restitution: this.restitution,
      maximumSpeed: this.maximumSpeed,
      backend,
    });
    clone.copyStateFrom(this);
    return clone;
  }

  splash(center = [this.bounds[0] / 2, this.bounds[1] * 0.35, this.bounds[2] / 2], options = {}) {
    const radius = positiveFinite(options.radius ?? 1.15, "Splash radius");
    const strength = positiveFinite(options.strength ?? 3.6, "Splash strength");
    validateTriplet(center, "Splash center", Number.isFinite);

    const radiusSquared = radius * radius;
    for (let particle = 0; particle < this.particleCount; particle += 1) {
      const offset = particle * 3;
      const dx = this.positions[offset] - center[0];
      const dy = this.positions[offset + 1] - center[1];
      const dz = this.positions[offset + 2] - center[2];
      const distanceSquared = dx * dx + dy * dy + dz * dz;
      if (distanceSquared >= radiusSquared) {
        continue;
      }

      const distance = Math.sqrt(distanceSquared) || this.spacing;
      const gain = strength * (1 - distance / radius);
      this.velocities[offset] += (dx / distance) * gain * 0.42;
      this.velocities[offset + 1] += gain;
      this.velocities[offset + 2] += (dz / distance) * gain * 0.42;
    }
  }

  stir(center, displacement, options = {}) {
    const radius = positiveFinite(options.radius ?? 0.82, "Stir radius");
    const strength = positiveFinite(options.strength ?? 24, "Stir strength");
    validateTriplet(center, "Stir center", Number.isFinite);
    validateTriplet(displacement, "Stir displacement", Number.isFinite);

    const radiusSquared = radius * radius;
    const horizontalTravel = Math.hypot(displacement[0], displacement[2]);
    for (let particle = 0; particle < this.particleCount; particle += 1) {
      const offset = particle * 3;
      const dx = this.positions[offset] - center[0];
      const dy = this.positions[offset + 1] - center[1];
      const dz = this.positions[offset + 2] - center[2];
      const distanceSquared = dx * dx + dy * dy + dz * dz;
      if (distanceSquared >= radiusSquared) {
        continue;
      }

      const gain = strength * (1 - Math.sqrt(distanceSquared) / radius);
      this.velocities[offset] += displacement[0] * gain;
      this.velocities[offset + 1] += (displacement[1] + horizontalTravel * 0.18) * gain;
      this.velocities[offset + 2] += displacement[2] * gain;
    }
  }

  #ensurePairCapacity(requiredCapacity) {
    if (requiredCapacity <= this.pairCapacity) {
      return;
    }
    const capacity = Math.max(
      256,
      requiredCapacity,
      this.pairCapacity === 0 ? 0 : this.pairCapacity * 2,
    );
    const grow = (source, ArrayType) => {
      const target = new ArrayType(capacity);
      target.set(source.subarray(0, this.pairCount));
      return target;
    };
    this.pairLeft = grow(this.pairLeft, Int32Array);
    this.pairRight = grow(this.pairRight, Int32Array);
    this.pairDx = grow(this.pairDx, Float64Array);
    this.pairDy = grow(this.pairDy, Float64Array);
    this.pairDz = grow(this.pairDz, Float64Array);
    this.pairDensityCoefficient = grow(this.pairDensityCoefficient, Float64Array);
    this.pairPressureCoefficient = grow(this.pairPressureCoefficient, Float64Array);
    this.pairViscosityCoefficient = grow(this.pairViscosityCoefficient, Float64Array);
    this.pairCapacity = capacity;
  }

  #appendContractedPair(left, right) {
    const leftOffset = left * 3;
    const rightOffset = right * 3;
    const dx = this.positions[leftOffset] - this.positions[rightOffset];
    const dy = this.positions[leftOffset + 1] - this.positions[rightOffset + 1];
    const dz = this.positions[leftOffset + 2] - this.positions[rightOffset + 2];
    const distanceSquared = dx * dx + dy * dy + dz * dz;
    if (distanceSquared >= this.kernelRadiusSquared) {
      return false;
    }

    this.#ensurePairCapacity(this.pairCount + 1);
    const pair = this.pairCount;
    const densityRemainder = this.kernelRadiusSquared - distanceSquared;
    const distance = Math.sqrt(distanceSquared);
    this.pairLeft[pair] = left;
    this.pairRight[pair] = right;
    this.pairDx[pair] = dx;
    this.pairDy[pair] = dy;
    this.pairDz[pair] = dz;
    this.pairDensityCoefficient[pair] = this.densityCoefficientScale * densityRemainder ** 3;
    if (distance === 0) {
      // The directed reference skips all force terms at coincident positions.
      this.pairPressureCoefficient[pair] = 0;
      this.pairViscosityCoefficient[pair] = 0;
    } else {
      const forceRemainder = this.smoothingRadius - distance;
      this.pairPressureCoefficient[pair] = (
        this.pressureCoefficientScale * forceRemainder * forceRemainder / distance
      );
      this.pairViscosityCoefficient[pair] = this.viscosityCoefficientScale * forceRemainder;
    }
    this.pairCount += 1;
    return true;
  }

  #gridIndex(x, y, z) {
    return (z * this.gridCounts[1] + y) * this.gridCounts[0] + x;
  }

  #buildGrid() {
    for (const index of this.activeGridCells) {
      this.gridBuckets[index].length = 0;
    }
    this.activeGridCells.length = 0;
    const inverseCellSize = 1 / this.smoothingRadius;
    for (let particle = 0; particle < this.particleCount; particle += 1) {
      const offset = particle * 3;
      const x = Math.floor(this.positions[offset] * inverseCellSize);
      const y = Math.floor(this.positions[offset + 1] * inverseCellSize);
      const z = Math.floor(this.positions[offset + 2] * inverseCellSize);
      const index = this.#gridIndex(x, y, z);
      const bucket = this.gridBuckets[index];
      if (bucket.length === 0) {
        this.activeGridCells.push(index);
      }
      bucket.push(particle);
    }
  }

  #isSupported(left, right) {
    const leftOffset = left * 3;
    const rightOffset = right * 3;
    const dx = this.positions[leftOffset] - this.positions[rightOffset];
    const dy = this.positions[leftOffset + 1] - this.positions[rightOffset + 1];
    const dz = this.positions[leftOffset + 2] - this.positions[rightOffset + 2];
    return dx * dx + dy * dy + dz * dz < this.kernelRadiusSquared;
  }

  #buildSupportLists() {
    let candidateInteractions = 0;
    let supportedInteractions = 0;

    if (this.backend === "all-pairs") {
      for (let left = 0; left < this.particleCount; left += 1) {
        const support = this.supportLists[left];
        support.length = 0;
        for (let right = 0; right < this.particleCount; right += 1) {
          candidateInteractions += 1;
          if (this.#isSupported(left, right)) {
            support.push(right);
            supportedInteractions += 1;
          }
        }
      }
    } else {
      this.#buildGrid();
      const inverseCellSize = 1 / this.smoothingRadius;
      for (let left = 0; left < this.particleCount; left += 1) {
        const support = this.supportLists[left];
        support.length = 0;
        const offset = left * 3;
        const cellX = Math.floor(this.positions[offset] * inverseCellSize);
        const cellY = Math.floor(this.positions[offset + 1] * inverseCellSize);
        const cellZ = Math.floor(this.positions[offset + 2] * inverseCellSize);

        const minimumX = Math.max(0, cellX - 1);
        const maximumX = Math.min(this.gridCounts[0] - 1, cellX + 1);
        const minimumY = Math.max(0, cellY - 1);
        const maximumY = Math.min(this.gridCounts[1] - 1, cellY + 1);
        const minimumZ = Math.max(0, cellZ - 1);
        const maximumZ = Math.min(this.gridCounts[2] - 1, cellZ + 1);
        for (let z = minimumZ; z <= maximumZ; z += 1) {
          for (let y = minimumY; y <= maximumY; y += 1) {
            for (let x = minimumX; x <= maximumX; x += 1) {
              const bucket = this.gridBuckets[this.#gridIndex(x, y, z)];
              for (const right of bucket) {
                candidateInteractions += 1;
                if (this.#isSupported(left, right)) {
                  support.push(right);
                  supportedInteractions += 1;
                }
              }
            }
          }
        }

      }
    }

    return { candidateInteractions, supportedInteractions };
  }

  #buildContractedPairs() {
    this.#buildGrid();
    this.pairCount = 0;
    let candidateInteractions = 0;
    const [gridX, gridY, gridZ] = this.gridCounts;

    for (const sourceIndex of this.activeGridCells) {
      const x = sourceIndex % gridX;
      const yz = (sourceIndex - x) / gridX;
      const y = yz % gridY;
      const z = (yz - y) / gridY;
      const source = this.gridBuckets[sourceIndex];
      for (let leftIndex = 0; leftIndex < source.length; leftIndex += 1) {
        for (let rightIndex = leftIndex + 1; rightIndex < source.length; rightIndex += 1) {
          candidateInteractions += 1;
          this.#appendContractedPair(source[leftIndex], source[rightIndex]);
        }
      }

      for (const [offsetX, offsetY, offsetZ] of FORWARD_CELL_OFFSETS) {
        const neighborX = x + offsetX;
        const neighborY = y + offsetY;
        const neighborZ = z + offsetZ;
        if (
          neighborX < 0 || neighborX >= gridX
          || neighborY < 0 || neighborY >= gridY
          || neighborZ < 0 || neighborZ >= gridZ
        ) {
          continue;
        }
        const neighbor = this.gridBuckets[this.#gridIndex(neighborX, neighborY, neighborZ)];
        for (const left of source) {
          for (const right of neighbor) {
              candidateInteractions += 1;
            this.#appendContractedPair(left, right);
          }
        }
      }
    }

    return {
      candidateInteractions,
      supportedInteractions: this.particleCount + this.pairCount * 2,
      storedInteractions: this.pairCount,
    };
  }

  #computeDensityAndPressure() {
    let totalDensity = 0;
    for (let left = 0; left < this.particleCount; left += 1) {
      const leftOffset = left * 3;
      let density = 0;
      for (const right of this.supportLists[left]) {
        const rightOffset = right * 3;
        const dx = this.positions[leftOffset] - this.positions[rightOffset];
        const dy = this.positions[leftOffset + 1] - this.positions[rightOffset + 1];
        const dz = this.positions[leftOffset + 2] - this.positions[rightOffset + 2];
        const remainder = this.kernelRadiusSquared - (dx * dx + dy * dy + dz * dz);
        density += this.mass * this.poly6Coefficient * remainder ** 3;
      }
      density = Math.max(density, this.restDensity * 0.05);
      this.densities[left] = density;
      this.pressures[left] = this.pressureStiffness * Math.max(0, density - this.restDensity);
      totalDensity += density;
    }
    return totalDensity / this.particleCount;
  }

  #computeContractedDensityAndPressure() {
    this.densities.fill(this.selfDensityCoefficient);
    for (let pair = 0; pair < this.pairCount; pair += 1) {
      const coefficient = this.pairDensityCoefficient[pair];
      this.densities[this.pairLeft[pair]] += coefficient;
      this.densities[this.pairRight[pair]] += coefficient;
    }

    let totalDensity = 0;
    for (let particle = 0; particle < this.particleCount; particle += 1) {
      const density = Math.max(this.densities[particle], this.restDensity * 0.05);
      this.densities[particle] = density;
      this.pressures[particle] = this.pressureStiffness * Math.max(0, density - this.restDensity);
      totalDensity += density;
    }
    return totalDensity / this.particleCount;
  }

  #computeAccelerations() {
    for (let left = 0; left < this.particleCount; left += 1) {
      const leftOffset = left * 3;
      let accelerationX = 0;
      let accelerationY = this.gravity;
      let accelerationZ = 0;

      for (const right of this.supportLists[left]) {
        if (right === left) {
          continue;
        }
        const rightOffset = right * 3;
        const dx = this.positions[leftOffset] - this.positions[rightOffset];
        const dy = this.positions[leftOffset + 1] - this.positions[rightOffset + 1];
        const dz = this.positions[leftOffset + 2] - this.positions[rightOffset + 2];
        const distance = Math.sqrt(dx * dx + dy * dy + dz * dz);
        if (distance === 0 || distance >= this.smoothingRadius) {
          continue;
        }

        const remainder = this.smoothingRadius - distance;
        const gradient = this.spikyGradientCoefficient * remainder * remainder;
        const pressureScale = -this.mass * (
          this.pressures[left] / this.densities[left] ** 2
          + this.pressures[right] / this.densities[right] ** 2
        ) * gradient / distance;
        accelerationX += pressureScale * dx;
        accelerationY += pressureScale * dy;
        accelerationZ += pressureScale * dz;

        const laplacian = this.viscosityLaplacianCoefficient * remainder;
        const viscosityScale = this.viscosity * this.mass * laplacian / this.densities[right];
        accelerationX += viscosityScale * (this.velocities[rightOffset] - this.velocities[leftOffset]);
        accelerationY += viscosityScale * (this.velocities[rightOffset + 1] - this.velocities[leftOffset + 1]);
        accelerationZ += viscosityScale * (this.velocities[rightOffset + 2] - this.velocities[leftOffset + 2]);
      }

      this.accelerations[leftOffset] = accelerationX;
      this.accelerations[leftOffset + 1] = accelerationY;
      this.accelerations[leftOffset + 2] = accelerationZ;
    }
  }

  #computeContractedAccelerations() {
    this.accelerations.fill(0);
    for (let particle = 0; particle < this.particleCount; particle += 1) {
      this.accelerations[particle * 3 + 1] = this.gravity;
    }

    for (let pair = 0; pair < this.pairCount; pair += 1) {
      const left = this.pairLeft[pair];
      const right = this.pairRight[pair];
      const leftOffset = left * 3;
      const rightOffset = right * 3;
      const dx = this.pairDx[pair];
      const dy = this.pairDy[pair];
      const dz = this.pairDz[pair];
      const pressureScale = this.pairPressureCoefficient[pair] * (
        this.pressures[left] / this.densities[left] ** 2
        + this.pressures[right] / this.densities[right] ** 2
      );
      const pressureX = pressureScale * dx;
      const pressureY = pressureScale * dy;
      const pressureZ = pressureScale * dz;
      this.accelerations[leftOffset] += pressureX;
      this.accelerations[leftOffset + 1] += pressureY;
      this.accelerations[leftOffset + 2] += pressureZ;
      this.accelerations[rightOffset] -= pressureX;
      this.accelerations[rightOffset + 1] -= pressureY;
      this.accelerations[rightOffset + 2] -= pressureZ;

      const velocityDifferenceX = this.velocities[rightOffset] - this.velocities[leftOffset];
      const velocityDifferenceY = this.velocities[rightOffset + 1] - this.velocities[leftOffset + 1];
      const velocityDifferenceZ = this.velocities[rightOffset + 2] - this.velocities[leftOffset + 2];
      const viscosityCoefficient = this.pairViscosityCoefficient[pair];
      const leftViscosityScale = viscosityCoefficient / this.densities[right];
      const rightViscosityScale = viscosityCoefficient / this.densities[left];
      this.accelerations[leftOffset] += leftViscosityScale * velocityDifferenceX;
      this.accelerations[leftOffset + 1] += leftViscosityScale * velocityDifferenceY;
      this.accelerations[leftOffset + 2] += leftViscosityScale * velocityDifferenceZ;
      this.accelerations[rightOffset] -= rightViscosityScale * velocityDifferenceX;
      this.accelerations[rightOffset + 1] -= rightViscosityScale * velocityDifferenceY;
      this.accelerations[rightOffset + 2] -= rightViscosityScale * velocityDifferenceZ;
    }
  }

  #integrate(deltaSeconds) {
    let maximumSpeed = 0;
    const radius = this.particleRadius;
    for (let particle = 0; particle < this.particleCount; particle += 1) {
      const offset = particle * 3;
      let velocityX = this.velocities[offset] + this.accelerations[offset] * deltaSeconds;
      let velocityY = this.velocities[offset + 1] + this.accelerations[offset + 1] * deltaSeconds;
      let velocityZ = this.velocities[offset + 2] + this.accelerations[offset + 2] * deltaSeconds;
      const speed = Math.sqrt(velocityX ** 2 + velocityY ** 2 + velocityZ ** 2);
      if (speed > this.maximumSpeed) {
        const scale = this.maximumSpeed / speed;
        velocityX *= scale;
        velocityY *= scale;
        velocityZ *= scale;
      }

      let positionX = this.positions[offset] + velocityX * deltaSeconds;
      let positionY = this.positions[offset + 1] + velocityY * deltaSeconds;
      let positionZ = this.positions[offset + 2] + velocityZ * deltaSeconds;

      if (positionX < radius || positionX > this.bounds[0] - radius) {
        positionX = Math.min(this.bounds[0] - radius, Math.max(radius, positionX));
        velocityX *= -this.restitution;
        velocityY *= BOUNDARY_DAMPING;
        velocityZ *= BOUNDARY_DAMPING;
      }
      if (positionY < radius || positionY > this.bounds[1] - radius) {
        positionY = Math.min(this.bounds[1] - radius, Math.max(radius, positionY));
        velocityY *= -this.restitution;
        velocityX *= BOUNDARY_DAMPING;
        velocityZ *= BOUNDARY_DAMPING;
      }
      if (positionZ < radius || positionZ > this.bounds[2] - radius) {
        positionZ = Math.min(this.bounds[2] - radius, Math.max(radius, positionZ));
        velocityZ *= -this.restitution;
        velocityX *= BOUNDARY_DAMPING;
        velocityY *= BOUNDARY_DAMPING;
      }

      this.positions[offset] = positionX;
      this.positions[offset + 1] = positionY;
      this.positions[offset + 2] = positionZ;
      this.velocities[offset] = velocityX;
      this.velocities[offset + 1] = velocityY;
      this.velocities[offset + 2] = velocityZ;
      maximumSpeed = Math.max(maximumSpeed, Math.sqrt(velocityX ** 2 + velocityY ** 2 + velocityZ ** 2));
    }
    return maximumSpeed;
  }

  step(deltaSeconds) {
    positiveFinite(deltaSeconds, "Simulation step");
    if (deltaSeconds > MAXIMUM_STEP_SECONDS) {
      throw new RangeError(`Simulation step must be at most ${MAXIMUM_STEP_SECONDS} seconds.`);
    }

    const contracted = this.backend === "contracted-indexed";
    const interactions = contracted ? this.#buildContractedPairs() : this.#buildSupportLists();
    const averageDensity = contracted
      ? this.#computeContractedDensityAndPressure()
      : this.#computeDensityAndPressure();
    if (contracted) {
      this.#computeContractedAccelerations();
    } else {
      this.#computeAccelerations();
    }
    const maximumSpeed = this.#integrate(deltaSeconds);
    this.lastDiagnostics = Object.freeze({
      backend: this.backend,
      particleCount: this.particleCount,
      candidateInteractions: interactions.candidateInteractions,
      supportedInteractions: interactions.supportedInteractions,
      storedInteractions: interactions.storedInteractions ?? interactions.supportedInteractions,
      allPairInteractions: this.particleCount ** 2,
      averageDensity,
      maximumSpeed,
    });
    return this.lastDiagnostics;
  }
}

export function compareWaterStates(left, right) {
  if (!(left instanceof ParticleWater) || !(right instanceof ParticleWater)) {
    throw new TypeError("Both compared states must be ParticleWater instances.");
  }
  return Object.freeze({
    positions: maximumAbsoluteDifference(left.positions, right.positions),
    velocities: maximumAbsoluteDifference(left.velocities, right.velocities),
    densities: maximumAbsoluteDifference(left.densities, right.densities),
    pressures: maximumAbsoluteDifference(left.pressures, right.pressures),
  });
}
