import { generatePrimeCameraData, ITERATION_COUNT } from "./model.mjs";

try {
  const result = generatePrimeCameraData(ITERATION_COUNT);
  const transfer = [
    ...result.cameras.native.map((positions) => positions.buffer),
    result.cameras.classical.buffer,
    result.cameras.zeta.buffer,
    result.cameras.re.buffer,
  ];
  self.postMessage(result, transfer);
} catch (error) {
  self.postMessage({
    error: error instanceof Error ? error.message : String(error),
  });
}
