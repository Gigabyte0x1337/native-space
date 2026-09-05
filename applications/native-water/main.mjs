// SPDX-License-Identifier: AGPL-3.0-or-later

import * as THREE from "https://cdn.jsdelivr.net/npm/three@0.180.0/build/three.module.js";

import { ParticleWater } from "./water-model.mjs";

const FIXED_STEP = 1 / 240;
const MAXIMUM_STEPS_PER_FRAME = 5;
const AUTO_SPLASH_SECONDS = 2.8;

const stage = document.querySelector("#water-stage");
const canvas = document.querySelector("#water-canvas");
const backendInput = document.querySelector("#interaction-backend");
const particleCountInput = document.querySelector("#particle-count");
const pauseButton = document.querySelector("#pause-water");
const splashButton = document.querySelector("#splash-water");
const resetButton = document.querySelector("#reset-water");
const autoSplashInput = document.querySelector("#auto-splash");
const fpsOutput = document.querySelector("#fps-output");
const stepOutput = document.querySelector("#step-output");
const candidateOutput = document.querySelector("#candidate-output");
const densityOutput = document.querySelector("#density-output");
const statusOutput = document.querySelector("#simulation-status");
const errorOutput = document.querySelector("#water-error");
const particleNote = document.querySelector("#particle-note");

const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
const PARTICLE_PRESETS = Object.freeze({
  800: Object.freeze([10, 10, 8]),
  1200: Object.freeze([12, 10, 10]),
  2304: Object.freeze([16, 12, 12]),
  3990: Object.freeze([19, 15, 14]),
});
let water = new ParticleWater();
const scene = new THREE.Scene();
scene.background = new THREE.Color(0x041119);
scene.fog = new THREE.FogExp2(0x041119, 0.052);

const renderer = new THREE.WebGLRenderer({
  canvas,
  antialias: true,
  powerPreference: "high-performance",
});
renderer.outputColorSpace = THREE.SRGBColorSpace;
renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 1.5));

const camera = new THREE.PerspectiveCamera(42, 1, 0.1, 60);
camera.position.set(7.8, 5.6, 8.7);
camera.lookAt(0, 1.45, 0);

scene.add(new THREE.HemisphereLight(0xb9edff, 0x0a1720, 2.5));
const keyLight = new THREE.DirectionalLight(0xffffff, 3.8);
keyLight.position.set(-5, 9, 4);
scene.add(keyLight);
const fillLight = new THREE.PointLight(0x33bde8, 24, 18);
fillLight.position.set(4, 2.8, -3);
scene.add(fillLight);

const centerOffset = new THREE.Vector3(-water.bounds[0] / 2, 0, -water.bounds[2] / 2);
const tankGeometry = new THREE.BoxGeometry(...water.bounds);
const tankEdges = new THREE.LineSegments(
  new THREE.EdgesGeometry(tankGeometry),
  new THREE.LineBasicMaterial({ color: 0x5ab7cd, transparent: true, opacity: 0.38 }),
);
tankEdges.position.set(0, water.bounds[1] / 2, 0);
scene.add(tankEdges);

const floor = new THREE.Mesh(
  new THREE.PlaneGeometry(water.bounds[0], water.bounds[2]),
  new THREE.MeshStandardMaterial({
    color: 0x082a37,
    roughness: 0.38,
    metalness: 0.2,
    transparent: true,
    opacity: 0.82,
  }),
);
floor.rotation.x = -Math.PI / 2;
scene.add(floor);

const particleGeometry = new THREE.IcosahedronGeometry(water.particleRadius * 0.92, 1);
const particleMaterial = new THREE.MeshPhysicalMaterial({
  color: 0xffffff,
  vertexColors: true,
  roughness: 0.12,
  metalness: 0.04,
  transmission: 0.16,
  thickness: 0.25,
  clearcoat: 0.65,
});
let particleMesh;

function createParticleMesh() {
  if (particleMesh) {
    scene.remove(particleMesh);
    particleMesh.dispose();
  }
  particleMesh = new THREE.InstancedMesh(
    particleGeometry,
    particleMaterial,
    water.particleCount,
  );
  particleMesh.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
  particleMesh.frustumCulled = false;
  scene.add(particleMesh);
}

createParticleMesh();

const matrix = new THREE.Matrix4();
const color = new THREE.Color();
function updateParticleInstances() {
  for (let particle = 0; particle < water.particleCount; particle += 1) {
    const offset = particle * 3;
    matrix.makeTranslation(
      water.positions[offset] + centerOffset.x,
      water.positions[offset + 1],
      water.positions[offset + 2] + centerOffset.z,
    );
    particleMesh.setMatrixAt(particle, matrix);
    const speed = Math.sqrt(
      water.velocities[offset] ** 2
      + water.velocities[offset + 1] ** 2
      + water.velocities[offset + 2] ** 2,
    );
    color.setHSL(0.54 - Math.min(0.12, speed * 0.012), 0.82, 0.54 + Math.min(0.2, speed * 0.025));
    particleMesh.setColorAt(particle, color);
  }
  particleMesh.instanceMatrix.needsUpdate = true;
  particleMesh.instanceColor.needsUpdate = true;
}

function resizeRenderer() {
  const width = Math.max(1, stage.clientWidth);
  const height = Math.max(1, stage.clientHeight);
  renderer.setSize(width, height, false);
  camera.aspect = width / height;
  camera.updateProjectionMatrix();
}
new ResizeObserver(resizeRenderer).observe(stage);
resizeRenderer();

function splashAt(x = water.bounds[0] / 2, z = water.bounds[2] / 2) {
  water.splash([x, water.bounds[1] * 0.34, z], { radius: 1.15, strength: 3.8 });
}

const raycaster = new THREE.Raycaster();
const pointer = new THREE.Vector2();
const splashPlane = new THREE.Plane(new THREE.Vector3(0, 1, 0), -water.bounds[1] * 0.3);
const hit = new THREE.Vector3();
function projectPointer(event) {
  const bounds = canvas.getBoundingClientRect();
  pointer.x = ((event.clientX - bounds.left) / bounds.width) * 2 - 1;
  pointer.y = -((event.clientY - bounds.top) / bounds.height) * 2 + 1;
  raycaster.setFromCamera(pointer, camera);
  if (raycaster.ray.intersectPlane(splashPlane, hit)) {
    return {
      x: Math.min(water.bounds[0], Math.max(0, hit.x - centerOffset.x)),
      z: Math.min(water.bounds[2], Math.max(0, hit.z - centerOffset.z)),
    };
  }
  return undefined;
}

let dragPoint;
let dragTravel = 0;
canvas.addEventListener("pointerdown", (event) => {
  dragPoint = projectPointer(event);
  dragTravel = 0;
  canvas.setPointerCapture(event.pointerId);
});
canvas.addEventListener("pointermove", (event) => {
  if (!canvas.hasPointerCapture(event.pointerId) || !dragPoint) {
    return;
  }
  const nextPoint = projectPointer(event);
  if (!nextPoint) {
    return;
  }
  const dx = nextPoint.x - dragPoint.x;
  const dz = nextPoint.z - dragPoint.z;
  const distance = Math.hypot(dx, dz);
  if (distance > 0.003) {
    water.stir(
      [nextPoint.x, water.bounds[1] * 0.3, nextPoint.z],
      [dx, 0, dz],
    );
    dragTravel += distance;
    dragPoint = nextPoint;
  }
});

function finishDrag(event) {
  if (dragPoint && dragTravel < 0.035) {
    splashAt(dragPoint.x, dragPoint.z);
  }
  dragPoint = undefined;
  dragTravel = 0;
  if (canvas.hasPointerCapture(event.pointerId)) {
    canvas.releasePointerCapture(event.pointerId);
  }
}
canvas.addEventListener("pointerup", finishDrag);
canvas.addEventListener("pointercancel", finishDrag);

let paused = reducedMotion.matches;
let accumulator = 0;
let previousTime = performance.now();
let autoSplashElapsed = 0;
let frames = 0;
let frameElapsed = 0;
let smoothedStepMilliseconds = 0;

function syncPauseUi() {
  pauseButton.textContent = paused ? "Resume" : "Pause";
  pauseButton.setAttribute("aria-pressed", String(paused));
  statusOutput.textContent = paused ? "Paused" : `Running · ${water.backend}`;
}

backendInput.addEventListener("change", () => {
  water.setBackend(backendInput.value);
  syncPauseUi();
});
particleCountInput.addEventListener("change", () => {
  const counts = PARTICLE_PRESETS[particleCountInput.value];
  if (!counts) {
    throw new RangeError(`Unknown particle preset: ${particleCountInput.value}.`);
  }
  water = new ParticleWater({ counts: [...counts], backend: backendInput.value });
  createParticleMesh();
  accumulator = 0;
  autoSplashElapsed = 0;
  smoothedStepMilliseconds = 0;
  particleNote.textContent = `${water.particleCount.toLocaleString()} particles · deterministic finite state · manually compiled JavaScript, not Native VM code generation.`;
  updateParticleInstances();
  syncPauseUi();
});
pauseButton.addEventListener("click", () => {
  paused = !paused;
  accumulator = 0;
  syncPauseUi();
});
splashButton.addEventListener("click", () => splashAt());
resetButton.addEventListener("click", () => {
  water.reset();
  accumulator = 0;
  autoSplashElapsed = 0;
  updateParticleInstances();
});
reducedMotion.addEventListener("change", (event) => {
  if (event.matches) {
    paused = true;
    syncPauseUi();
  }
});

function updateMetrics(frameSeconds) {
  frames += 1;
  frameElapsed += frameSeconds;
  if (frameElapsed < 0.35) {
    return;
  }
  fpsOutput.textContent = `${Math.round(frames / frameElapsed)} fps`;
  stepOutput.textContent = `${smoothedStepMilliseconds.toFixed(2)} ms`;
  const diagnostics = water.lastDiagnostics;
  const retained = diagnostics.allPairInteractions === 0
    ? 0
    : diagnostics.candidateInteractions / diagnostics.allPairInteractions;
  candidateOutput.textContent = `${diagnostics.candidateInteractions.toLocaleString()} · ${(retained * 100).toFixed(1)}%`;
  densityOutput.textContent = diagnostics.averageDensity.toFixed(1);
  frames = 0;
  frameElapsed = 0;
}

function animate(now) {
  try {
    const frameSeconds = Math.min((now - previousTime) / 1_000, 0.05);
    previousTime = now;
    if (!paused) {
      accumulator = Math.min(accumulator + frameSeconds, FIXED_STEP * MAXIMUM_STEPS_PER_FRAME);
      autoSplashElapsed += frameSeconds;
      if (autoSplashInput.checked && autoSplashElapsed >= AUTO_SPLASH_SECONDS) {
        const phase = now * 0.00037;
        splashAt(
          water.bounds[0] * (0.5 + Math.sin(phase) * 0.16),
          water.bounds[2] * (0.5 + Math.cos(phase * 1.31) * 0.16),
        );
        autoSplashElapsed = 0;
      }

      let measuredSteps = 0;
      const stepStart = performance.now();
      while (accumulator >= FIXED_STEP && measuredSteps < MAXIMUM_STEPS_PER_FRAME) {
        water.step(FIXED_STEP);
        accumulator -= FIXED_STEP;
        measuredSteps += 1;
      }
      if (measuredSteps > 0) {
        const current = (performance.now() - stepStart) / measuredSteps;
        smoothedStepMilliseconds = smoothedStepMilliseconds === 0
          ? current
          : smoothedStepMilliseconds * 0.88 + current * 0.12;
      }
      updateParticleInstances();
    }
    updateMetrics(frameSeconds);
    renderer.render(scene, camera);
    requestAnimationFrame(animate);
  } catch (error) {
    errorOutput.hidden = false;
    errorOutput.textContent = error instanceof Error ? error.message : String(error);
    statusOutput.textContent = "Stopped";
    throw error;
  }
}

backendInput.value = water.backend;
particleCountInput.value = String(water.particleCount);
particleNote.textContent = `${water.particleCount.toLocaleString()} particles · deterministic finite state · manually compiled JavaScript, not Native VM code generation.`;
syncPauseUi();
updateParticleInstances();
requestAnimationFrame(animate);
