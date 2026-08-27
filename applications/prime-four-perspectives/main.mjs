import * as THREE from "https://cdn.jsdelivr.net/npm/three@0.180.0/build/three.module.js";
import {
  BRAID_TRACK_COUNT,
  classicalPrimeRecords,
  DISPLAY_HALF_HEIGHT,
  DISPLAY_RADIUS,
  INDEX_READOUT_COUNT,
  ITERATION_COUNT,
  indexedBraidWindow,
} from "./model.mjs";

const stage = document.querySelector("#prime-stage");
const canvas = document.querySelector("#prime-canvas");
const loading = document.querySelector("#loading-state");
const errorOutput = document.querySelector("#error-output");
const iterationOutput = document.querySelector("#iteration-count");
const vertexOutput = document.querySelector("#vertex-count");
const braidVertexOutput = document.querySelector("#braid-vertex-count");
const lastPrimeOutput = document.querySelector("#last-prime");
const elapsedOutput = document.querySelector("#elapsed-time");
const indexStartInput = document.querySelector("#index-start");
const indexWindowOutput = document.querySelector("#index-window-output");
const fullIndexButton = document.querySelector("#full-index-view");
const showIndexReadoutsInput = document.querySelector("#show-index-readouts");
const indexedReadouts = document.querySelector("#indexed-readouts");
const visualStack = document.querySelector(".visual-stack");
const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: true });
renderer.outputColorSpace = THREE.SRGBColorSpace;
renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 1.5));
renderer.setScissorTest(true);

const cameraDefinitions = [
  { id: "native", color: 0x59d9ff, background: 0x07111d },
  { id: "classical", color: 0xe9f2fa, background: 0x091522 },
  { id: "zeta", color: 0x7ef7c6, background: 0x07131c },
  { id: "re", color: 0xff5ebc, background: 0x11101c },
];
const scenes = cameraDefinitions.map(() => {
  const scene = new THREE.Scene();
  scene.fog = new THREE.FogExp2(0x07101b, 0.014);
  return scene;
});
const dataGroups = scenes.map((scene) => {
  const group = new THREE.Group();
  scene.add(group);
  return group;
});
const cameras = cameraDefinitions.map(() => new THREE.PerspectiveCamera(34, 1, 0.1, 100));
const nativeColors = [0x7ef7c6, 0x59d9ff, 0xff5ebc, 0xffc95c];
// A side-on default keeps the +45-degree zeta and -45-degree RE outputs
// equally visible relative to the same classical axes.
let orbitAzimuth = Math.PI / 2;
let orbitElevation = 0.18;
let cameraDistance = 28;
let dragStart = null;
let ready = false;

function line(scene, points, color, opacity = 1) {
  const geometry = new THREE.BufferGeometry().setFromPoints(points);
  const material = new THREE.LineBasicMaterial({ color, transparent: opacity < 1, opacity });
  const result = new THREE.Line(geometry, material);
  scene.add(result);
  return result;
}

function addVerticalAxis(scene) {
  line(
    scene,
    [
      new THREE.Vector3(0, -DISPLAY_HALF_HEIGHT - 0.65, 0),
      new THREE.Vector3(0, DISPLAY_HALF_HEIGHT + 0.65, 0),
    ],
    0x8da2b6,
    0.5,
  );
}

function addCameraGuide(scene, directionX, directionZ, color) {
  addVerticalAxis(scene);
  for (let level = -DISPLAY_HALF_HEIGHT; level <= DISPLAY_HALF_HEIGHT; level += 2) {
    line(
      scene,
      [
        new THREE.Vector3(0, level, 0),
        new THREE.Vector3(directionX * DISPLAY_RADIUS, level, directionZ * DISPLAY_RADIUS),
      ],
      color,
      level === 0 ? 0.38 : 0.14,
    );
  }
}

function addProjectionPlane(scene, angle, color, opacity = 0.045) {
  const geometry = new THREE.PlaneGeometry(DISPLAY_RADIUS * 2.5, DISPLAY_HALF_HEIGHT * 2.05);
  const material = new THREE.MeshBasicMaterial({
    color,
    side: THREE.DoubleSide,
    transparent: true,
    opacity,
    depthWrite: false,
  });
  const plane = new THREE.Mesh(geometry, material);
  plane.rotation.y = -angle;
  scene.add(plane);
}

function addCircularCrossSections(scene, color = 0x5f7288) {
  addVerticalAxis(scene);
  for (let level = -DISPLAY_HALF_HEIGHT; level <= DISPLAY_HALF_HEIGHT; level += 2) {
    const ring = new THREE.EllipseCurve(0, 0, DISPLAY_RADIUS, DISPLAY_RADIUS).getPoints(96);
    const points = ring.map(({ x, y }) => new THREE.Vector3(x, level, y));
    points.push(points[0].clone());
    line(scene, points, color, level === 0 ? 0.35 : 0.13);
  }
}

function addClassicalAxes(scene) {
  line(
    scene,
    [
      new THREE.Vector3(-DISPLAY_RADIUS - 0.45, 0, 0),
      new THREE.Vector3(DISPLAY_RADIUS + 0.45, 0, 0),
    ],
    0xe9f2fa,
    0.55,
  );
  line(
    scene,
    [
      new THREE.Vector3(0, 0, -DISPLAY_RADIUS - 0.45),
      new THREE.Vector3(0, 0, DISPLAY_RADIUS + 0.45),
    ],
    0xe9f2fa,
    0.55,
  );
}

function buildReferenceGeometry() {
  const nativeScene = scenes[0];
  addCircularCrossSections(nativeScene);
  addProjectionPlane(nativeScene, Math.PI / 4, 0x7ef7c6, 0.018);
  addProjectionPlane(nativeScene, -Math.PI / 4, 0xff5ebc, 0.018);

  // The classical camera preserves the full complex cross-section and changes
  // notation from J to i. Zeta and RE are its complementary one-axis views.
  addCircularCrossSections(scenes[1], 0x6d8094);
  addClassicalAxes(scenes[1]);
  addProjectionPlane(scenes[2], Math.PI / 4, 0x7ef7c6);
  addProjectionPlane(scenes[3], -Math.PI / 4, 0xff5ebc);
  addCameraGuide(scenes[2], 0.5, 0.5, 0x7ef7c6);
  addCameraGuide(scenes[3], 0.5, -0.5, 0xff5ebc);
}

function pointsObject(positions, material) {
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
  geometry.computeBoundingSphere();
  return new THREE.Points(geometry, material);
}

function pointMaterial(color, opacity) {
  return new THREE.PointsMaterial({
    color,
    size: 0.048,
    sizeAttenuation: true,
    transparent: true,
    opacity,
    depthWrite: false,
    blending: THREE.AdditiveBlending,
  });
}

function addCameraPoints(cameraData) {
  cameraData.native.forEach((positions, remainder) => {
    dataGroups[0].add(pointsObject(positions, pointMaterial(nativeColors[remainder], 0.16)));
  });
  dataGroups[1].add(pointsObject(cameraData.classical, pointMaterial(cameraDefinitions[1].color, 0.2)));
  dataGroups[2].add(pointsObject(cameraData.zeta, pointMaterial(cameraDefinitions[2].color, 0.22)));
  dataGroups[3].add(pointsObject(cameraData.re, pointMaterial(cameraDefinitions[3].color, 0.22)));
}

const indexedBraidVertexShader = `
  uniform float cameraMode;
  uniform float quarterTurns;

  vec2 rotateQuarterTurns(vec2 source, float turns) {
    float angle = turns * 1.5707963267948966;
    float cosine = cos(angle);
    float sine = sin(angle);
    return vec2(
      cosine * source.x - sine * source.y,
      sine * source.x + cosine * source.y
    );
  }

  void main() {
    vec2 source = rotateQuarterTurns(position.xz, quarterTurns);
    vec2 target = source;

    if (cameraMode > 0.5 && cameraMode < 1.5) {
      float zeta = (source.x + source.y) * 0.5;
      target = vec2(zeta, zeta);
    } else if (cameraMode > 1.5) {
      float reflected = (source.x - source.y) * 0.5;
      target = vec2(reflected, -reflected);
    }

    gl_Position = projectionMatrix * modelViewMatrix * vec4(target.x, position.y, target.y, 1.0);
  }
`;

const indexedBraidFragmentShader = `
  uniform vec3 lineColor;

  void main() {
    gl_FragColor = vec4(lineColor, 0.82);
  }
`;

function indexedBraidMaterial(cameraMode, track, color) {
  return new THREE.ShaderMaterial({
    uniforms: {
      cameraMode: { value: cameraMode },
      quarterTurns: { value: track },
      lineColor: { value: new THREE.Color(color) },
    },
    vertexShader: indexedBraidVertexShader,
    fragmentShader: indexedBraidFragmentShader,
    transparent: true,
    depthWrite: false,
  });
}

function addIndexedBraidLines(cameraData) {
  const trackColors = [0x59d9ff, 0xff5ebc, 0xffc95c];
  const cameraModes = [0, 0, 1, 2];
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute("position", new THREE.BufferAttribute(cameraData.classical, 3));
  geometry.computeBoundingSphere();

  scenes.forEach((scene, cameraIndex) => {
    for (let track = 0; track < BRAID_TRACK_COUNT; track += 1) {
      const braid = new THREE.Line(
        geometry,
        indexedBraidMaterial(cameraModes[cameraIndex], track, trackColors[track]),
      );
      braid.frustumCulled = false;
      braid.renderOrder = 2;
      dataGroups[cameraIndex].add(braid);
    }
  });
}

function renderIndexedBraidReadouts(startIndex = 1) {
  const svgNamespace = "http://www.w3.org/2000/svg";
  const chartDefinitions = [
    { camera: "classical", group: "#classical-index-braids", minimum: 0, maximum: 3 },
    { camera: "zeta", group: "#zeta-index-braids", minimum: -0.5, maximum: 0.5 },
    { camera: "re", group: "#re-index-braids", minimum: -0.5, maximum: 0.5 },
  ];
  const left = 58;
  const right = 302;
  const top = 15;
  const bottom = 110;

  for (const definition of chartDefinitions) {
    const group = document.querySelector(definition.group);
    group.replaceChildren();
    const tracks = indexedBraidWindow(definition.camera, startIndex);
    const x = (coordinate) =>
      left + ((coordinate - definition.minimum) / (definition.maximum - definition.minimum)) * (right - left);
    const y = (index) =>
      bottom - ((index - startIndex) / (INDEX_READOUT_COUNT - 1)) * (bottom - top);

    tracks.forEach((vertices, track) => {
      const path = document.createElementNS(svgNamespace, "path");
      const data = vertices
        .map((vertex, offset) => `${offset === 0 ? "M" : "L"}${x(vertex.coordinate)} ${y(vertex.index)}`)
        .join(" ");
      path.setAttribute("class", `braid-path track-${track}`);
      path.setAttribute("d", data);
      path.setAttribute("vector-effect", "non-scaling-stroke");
      group.append(path);
    });
  }

  document.querySelectorAll("[data-index-window-start]").forEach((label) => {
    label.textContent = `k=${startIndex.toLocaleString("en-US")}`;
  });
  const endIndex = startIndex + INDEX_READOUT_COUNT - 1;
  document.querySelectorAll("[data-index-window-end]").forEach((label) => {
    label.textContent = `k=${endIndex.toLocaleString("en-US")}`;
  });
  indexWindowOutput.value = `k=${startIndex.toLocaleString("en-US")}…${endIndex.toLocaleString("en-US")}`;
}

function renderClassicalPrimeCheck() {
  const records = classicalPrimeRecords();
  const tracks = indexedBraidWindow("classical", 1, records.length);
  const svgNamespace = "http://www.w3.org/2000/svg";
  const grid = document.querySelector("#classical-thirty-grid");
  const braids = document.querySelector("#classical-thirty-braids");
  const tableBody = document.querySelector("#classical-thirty-table-body");
  const left = 72;
  const right = 1090;
  const laneTop = 52;
  const laneGap = 37;
  const x = (index) => left + ((index - 1) / (records.length - 1)) * (right - left);
  const y = (lane) => laneTop + lane * laneGap;

  records.forEach((record) => {
    const guide = document.createElementNS(svgNamespace, "line");
    guide.setAttribute("class", "prime-guide");
    guide.setAttribute("x1", x(record.index));
    guide.setAttribute("x2", x(record.index));
    guide.setAttribute("y1", "34");
    guide.setAttribute("y2", "184");
    grid.append(guide);

    const prime = document.createElementNS(svgNamespace, "text");
    prime.setAttribute("class", "plane-prime");
    prime.setAttribute("text-anchor", "middle");
    prime.setAttribute("x", x(record.index));
    prime.setAttribute("y", "20");
    prime.textContent = String(record.prime);
    grid.append(prime);

    const index = document.createElementNS(svgNamespace, "text");
    index.setAttribute("class", "plane-index");
    index.setAttribute("text-anchor", "middle");
    index.setAttribute("x", x(record.index));
    index.setAttribute("y", "207");
    index.textContent = String(record.index);
    grid.append(index);

    const row = document.createElement("tr");
    for (const value of [record.index, record.prime, record.classicalOrientation]) {
      const cell = document.createElement("td");
      cell.textContent = String(value);
      row.append(cell);
    }
    tableBody.append(row);
  });

  tracks.forEach((vertices, track) => {
    const path = document.createElementNS(svgNamespace, "path");
    path.setAttribute("class", `braid-path track-${track}`);
    path.setAttribute(
      "d",
      vertices.map((vertex, offset) =>
        `${offset === 0 ? "M" : "L"}${x(vertex.index)} ${y(vertex.coordinate)}`,
      ).join(" "),
    );
    path.setAttribute("vector-effect", "non-scaling-stroke");
    braids.append(path);

    vertices.forEach((vertex) => {
      const point = document.createElementNS(svgNamespace, "circle");
      point.setAttribute("class", `braid-point track-${track}`);
      point.setAttribute("cx", x(vertex.index));
      point.setAttribute("cy", y(vertex.coordinate));
      point.setAttribute("r", "2.3");
      braids.append(point);
    });
  });
}

function normalizedIndexHeight(index) {
  return ((index - 1) / (ITERATION_COUNT - 1)) * DISPLAY_HALF_HEIGHT * 2 - DISPLAY_HALF_HEIGHT;
}

function showIndexWindow(startIndex) {
  const maximumStart = ITERATION_COUNT - INDEX_READOUT_COUNT + 1;
  const start = THREE.MathUtils.clamp(Math.round(startIndex), 1, maximumStart);
  const end = start + INDEX_READOUT_COUNT - 1;
  const startHeight = normalizedIndexHeight(start);
  const endHeight = normalizedIndexHeight(end);
  const center = (startHeight + endHeight) / 2;
  const scale = (DISPLAY_HALF_HEIGHT * 2) / (endHeight - startHeight);

  dataGroups.forEach((group) => {
    group.scale.y = scale;
    group.position.y = -center * scale;
  });
  indexStartInput.value = String(start);
  fullIndexButton.setAttribute("aria-pressed", "false");
  fullIndexButton.textContent = "Show full million";
  stage.dataset.indexView = `${start}:${end}`;
  renderIndexedBraidReadouts(start);
  render();
}

function showFullIndexRange() {
  dataGroups.forEach((group) => {
    group.scale.y = 1;
    group.position.y = 0;
  });
  fullIndexButton.setAttribute("aria-pressed", "true");
  fullIndexButton.textContent = "Show INDEX window";
  stage.dataset.indexView = "full";
  render();
}

function updateIndexedReadoutVisibility() {
  const visible = showIndexReadoutsInput.checked;
  indexedReadouts.hidden = !visible;
  visualStack.classList.toggle("readouts-hidden", !visible);
  render();
}

function setCameraTransforms() {
  cameras.forEach((camera) => {
    const horizontalDistance = Math.cos(orbitElevation) * cameraDistance;
    camera.position.set(
      Math.cos(orbitAzimuth) * horizontalDistance,
      Math.sin(orbitElevation) * cameraDistance,
      Math.sin(orbitAzimuth) * horizontalDistance,
    );
    camera.up.set(0, 1, 0);
    camera.lookAt(0, 0, 0);
  });
}

function render() {
  const width = Math.max(1, stage.clientWidth);
  const height = Math.max(1, stage.clientHeight);
  const pixelRatio = renderer.getPixelRatio();
  const drawingWidth = Math.round(width * pixelRatio);
  const drawingHeight = Math.round(height * pixelRatio);

  if (canvas.width !== drawingWidth || canvas.height !== drawingHeight) {
    renderer.setSize(width, height, false);
  }

  setCameraTransforms();
  const halfWidth = Math.floor(width / 2);
  const halfHeight = Math.floor(height / 2);

  cameras.forEach((camera, index) => {
    const left = index % 2 === 0 ? 0 : halfWidth;
    const top = index < 2 ? 0 : halfHeight;
    const viewportWidth = index % 2 === 0 ? halfWidth : width - halfWidth;
    const viewportHeight = index < 2 ? halfHeight : height - halfHeight;
    const bottom = height - top - viewportHeight;

    camera.aspect = viewportWidth / viewportHeight;
    camera.updateProjectionMatrix();
    renderer.setViewport(left, bottom, viewportWidth, viewportHeight);
    renderer.setScissor(left, bottom, viewportWidth, viewportHeight);
    renderer.setClearColor(cameraDefinitions[index].background, 1);
    renderer.render(scenes[index], camera);
  });
}

function renderedVertexCount() {
  let count = 0;
  scenes.forEach((scene) => scene.traverse((child) => {
    if (child.isPoints) {
      count += child.geometry.getAttribute("position").count;
    }
  }));
  return count;
}

function announceReady(summary) {
  const detail = {
    ...summary,
    renderedVertexCount: renderedVertexCount(),
    cameraCount: cameras.length,
    cameras: cameraDefinitions.map(({ id }) => id),
  };

  stage.dataset.status = "ready";
  stage.dataset.renderedVertexCount = String(detail.renderedVertexCount);
  stage.dataset.cameraCount = String(detail.cameraCount);
  stage.dataset.cameras = detail.cameras.join(",");
  stage.dataset.braidTrackCount = String(detail.braidTrackCount);
  stage.dataset.indexedBraidVertexCount = String(detail.indexedBraidVertexCount);
  window.__nativeSpacePrimeSimulation = Object.freeze({ status: "ready", ...detail });
  window.dispatchEvent(new CustomEvent("prime-simulation-ready", { detail }));
}

function showFailure(message) {
  loading.hidden = true;
  errorOutput.hidden = false;
  errorOutput.textContent = `The prime cameras could not be built: ${message}`;
  stage.dataset.status = "error";
  window.__nativeSpacePrimeSimulation = Object.freeze({ status: "error", message });
}

buildReferenceGeometry();
renderIndexedBraidReadouts();
renderClassicalPrimeCheck();
updateIndexedReadoutVisibility();

indexStartInput.addEventListener("input", () => showIndexWindow(Number(indexStartInput.value)));
fullIndexButton.addEventListener("click", () => {
  if (fullIndexButton.getAttribute("aria-pressed") === "true") {
    showIndexWindow(Number(indexStartInput.value));
  } else {
    showFullIndexRange();
  }
});
showIndexReadoutsInput.addEventListener("change", updateIndexedReadoutVisibility);

const worker = new Worker(new URL("./prime-worker.mjs", import.meta.url), { type: "module" });
worker.addEventListener("message", ({ data }) => {
  if (data.error) {
    showFailure(data.error);
    worker.terminate();
    return;
  }

  addCameraPoints(data.cameras);
  addIndexedBraidLines(data.cameras);
  showIndexWindow(Number(indexStartInput.value));
  iterationOutput.textContent = data.summary.iterationCount.toLocaleString("en-US");
  vertexOutput.textContent = data.summary.projectedVertexCount.toLocaleString("en-US");
  braidVertexOutput.textContent = data.summary.indexedBraidVertexCount.toLocaleString("en-US");
  lastPrimeOutput.textContent = data.summary.lastPrime.toLocaleString("en-US");
  elapsedOutput.textContent = `${Math.round(data.summary.elapsedMilliseconds).toLocaleString("en-US")} ms`;
  loading.hidden = true;
  ready = true;
  render();
  announceReady(data.summary);
  worker.terminate();
});
worker.addEventListener("error", ({ message }) => showFailure(message));

stage.addEventListener("pointerdown", (event) => {
  dragStart = { x: event.clientX, y: event.clientY, orbitAzimuth, orbitElevation };
  stage.setPointerCapture(event.pointerId);
});
stage.addEventListener("pointermove", (event) => {
  if (!dragStart || !ready) {
    return;
  }

  orbitAzimuth = dragStart.orbitAzimuth - (event.clientX - dragStart.x) * 0.004;
  orbitElevation = THREE.MathUtils.clamp(
    dragStart.orbitElevation + (event.clientY - dragStart.y) * 0.003,
    -0.42,
    0.58,
  );
  render();
});
stage.addEventListener("pointerup", (event) => {
  dragStart = null;
  stage.releasePointerCapture(event.pointerId);
});
stage.addEventListener(
  "wheel",
  (event) => {
    event.preventDefault();
    cameraDistance = THREE.MathUtils.clamp(cameraDistance + event.deltaY * 0.012, 21, 39);
    render();
  },
  { passive: false },
);

new ResizeObserver(render).observe(stage);

if (ITERATION_COUNT !== 1_000_000) {
  showFailure("The experiment must retain its declared one-million-birth invariant.");
}
