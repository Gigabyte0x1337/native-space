<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Navier–Stokes and Turbulence

**Status:** finite interaction associativity and one coefficient-contracted
Fourier-triad family proved. One manually compiled finite 3D SPH experiment
shows size-dependent gains from spatial INDEX selection and unordered
coefficient retention. A native Rust/Vulkan experiment shows a machine-specific
gain for the retained fixed-graph coefficient pass. A Native Space compiler
gain, full moving-fluid speedup, full fluid camera, turbulence result, and
regularity result remain open.

## Proposed native map

A declared spectral camera would map mode identity to INDEX, oriented gain to
coefficients, quadratic interaction to MULTIPLY, composition to ADD, and
incompressibility/triad compatibility to an explicit selector. This must add
value beyond standard Fourier triad structure. The existing finite cyclic DFT
proof is not a divergence-free continuum camera.

## T-NS-CORE-1 -- finite indexed interactions compose [Proved]

For finite native states $F,G,H$,

$$
\mathrm{MULTIPLY}(F,\mathrm{MULTIPLY}(G,H))
=\mathrm{MULTIPLY}(\mathrm{MULTIPLY}(F,G),H).
$$

**Proof.** A left term has index
$\alpha\oplus_I(\beta\oplus_I\gamma)$ and coefficient
$F_\alpha\boxtimes(G_\beta\boxtimes H_\gamma)$. L-IDX-2 and L-OS-3 turn
these into the right-associated index and coefficient. The same finite triples
contribute on both sides, and L-OS-1 equates their ADD folds. This is L-NS-5.
Their two-turn-ORIENT residual is zero by L-NS-2. $\square$

Executable instance:
[navier-stokes-index-composition.ns](../examples/applications/navier-stokes-index-composition.ns).

**Boundary:** this does not identify INDEX with signed wavevectors, encode
divergence-free selection, take a continuum limit, or control energy transfer
to infinite frequency. It proves no Navier–Stokes regularity result.

## T-NS-TRIAD-COEFFICIENT-1 -- one shared Fourier interaction coefficient [Proved]

Consider the finite two-dimensional wavevector family

$$
p=(1,0),
\qquad
q=(1,1),
\qquad
k=p+q=(2,1),
$$

with divergence-free source amplitudes

$$
u_p=(0,a),
\qquad
u_q=(b,-b).
$$

The two ordered quadratic interactions selected by $p+q=k$ share one
multiplication coefficient $g=ab$:

$$
N_k=(u_p\mathbin{\cdot}q)u_q+(u_q\mathbin{\cdot}p)u_p
=a(b,-b)+b(0,a)
=(g,0).
$$

The pressure/Leray projection at $k$ is

$$
P_kN_k
=N_k-k\frac{k\mathbin{\cdot}N_k}{k\mathbin{\cdot}k}
=(g,0)-(2,1)\frac{2g}{5}
=\left(\frac{g}{5},-\frac{2g}{5}\right).
$$

Therefore $k\mathbin{\cdot}P_kN_k=0$. If the target mode is
$u_k=(c,-2c)$, then both the viscous term $-\nu|k|^2u_k$ and the oriented
Fourier nonlinear term $-iP_kN_k$ also have zero dot product with $k$.
Their ADD is consequently divergence-free.

**What is proved:** the ordinary ordered interaction and its shared-coefficient
form are equal for this complete parameterized triad family; the explicit
pressure projection and viscosity preserve incompressibility.

Executable closed instance:
[navier-stokes-fourier-triad.ns](../examples/tryouts/navier-stokes-fourier-triad.ns).
It uses $a=2$, $b=3$, $c=4$, and $\nu=1/10$, tags seven separate residuals,
and checks them together as one exact native zero proof.

**Boundary:** this is one finite selected triad family. It does not yet define
signed wavevectors as a general INDEX camera, discover triads automatically,
advance a full spectral state, establish a CPU speedup, or address continuum
existence and regularity.

## E-NS-SPH-INDEX-1 -- finite candidate selection [Observed]

The executable experiment in [native-water](native-water/) implements one
finite weakly-compressible smoothed-particle hydrodynamics (WCSPH) update. It
is a particle approximation, not an exact solution of the continuum
Navier–Stokes equations.

All three execution paths use the same finite state and the same density kernel,
pressure force, viscosity force, gravity, boundary response, speed bound, and
semi-implicit integration. They differ only in how interaction work is retained:

- `all-pairs` tests every ordered particle pair;
- `indexed` places particles in smoothing-radius cells and tests only the 27
  adjacent 3D cells;
- `contracted-indexed` visits every unordered adjacent-cell pair once and
  retains one geometry/coefficient record for both particle directions.

The selected support lists then feed the same coefficient code. This isolates
the structural value of INDEX from changes to the physical approximation.

## E-NS-SPH-CONTRACT-1 -- unordered coefficient retention [Observed]

For one supported unordered pair `(left, right)`, the contracted path stores
the displacement and three gains:

```text
density_gain   = mass * poly6 * (h^2 - distance^2)^3
pressure_gain  = -mass * spiky_gradient * (h - distance)^2 / distance
viscosity_gain = viscosity * mass * viscosity_laplacian * (h - distance)
```

The density gain is applied to both particles. Pressure uses one gain with
opposite displacement signs. Viscosity reuses one gain while retaining each
particle's distinct density denominator. This reproduces both directed terms;
it does not replace the SPH equations with a symmetric approximation.

Run the benchmark from the repository root:

```powershell
node applications/native-water/benchmark.mjs
```

Observed on 2026-08-29 with Node 24.13.0 on one Intel Core i9-9900K. Each time
is the median of five rounds. Initialization, buffer allocation, warmup, and
state copying are outside the timed region. The fixed step is 1/240 second.

| Particles | Steps | Every pair | Spatial INDEX | Contracted INDEX | Contracted / INDEX | Total speedup | Checks removed | Maximum state error |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 64 | 20 | 1.35 ms | 3.19 ms | 3.33 ms | 0.96x | 0.41x | 68.0% | 2.3e-13 |
| 216 | 14 | 5.15 ms | 4.87 ms | 3.59 ms | 1.36x | 1.43x | 84.3% | 2.3e-13 |
| 512 | 10 | 15.76 ms | 9.66 ms | 6.52 ms | 1.48x | 2.42x | 90.2% | 3.2e-10 |
| 800 | 8 | 27.56 ms | 12.00 ms | 7.79 ms | 1.54x | 3.54x | 92.9% | 1.1e-9 |
| 1,200 | 6 | 41.85 ms | 14.32 ms | 9.06 ms | 1.58x | 4.62x | 95.0% | 1.3e-9 |
| 2,304 | 4 | 101.99 ms | 19.97 ms | 13.35 ms | 1.50x | 7.64x | 97.1% | 7.9e-10 |
| 3,990 | 3 | 209.70 ms | 25.46 ms | 15.98 ms | 1.59x | 13.12x | 98.4% | 8.0e-10 |

A second consecutive five-round run measured 1.50x through 1.66x additional
speed over ordinary INDEX at 216 through 3,990 particles, and 13.68x total at
3,990. The 64-particle case remains too small to amortize the grid and retained
coefficient buffers.

The maximum state error is the largest absolute difference across positions,
velocities, densities, and pressures after the stated steps. It comes from
floating-point summation order; the equivalent directed support interaction
count is identical in all paths. Contracted INDEX stores each non-self pair
once, while the first two paths retain both directed entries.

The [interactive view](native-water/index.html) runs the same model, exposes
all three paths and particle-count presets, and supports click impulses and
drag stirring. Contracted INDEX is the default.

**Boundary:** this is evidence for spatial candidate selection in manually
compiled JavaScript. Spatial hashing and pair-symmetric SPH evaluation are
established simulation techniques. The Native Space contribution tested here
is the explicit representation of an unordered interaction as one retained
coefficient strand, but the implementation is still handwritten. The
experiment does not show that Native Space discovered the contraction,
compiled the model, outperformed an optimized production SPH library, used a
GPU, or accelerated the Native Space VM. Those require separate baselines and
a native lowering path.

## E-NS-SPH-VULKAN-1 -- native CPU and Vulkan coefficient pass [Observed]

The [Rust benchmark](../language/runtime/examples/native_water_native.rs) and
its [Vulkan compute shader](../language/runtime/examples/native_water_native.wgsl)
remove JavaScript from the retained-coefficient hot path. Both evaluators
consume one identical finite unordered-pair graph, the same directed adjacency
view, the same initial velocities, and the same density, pressure, viscosity,
and gravity coefficients. The executable forces the Vulkan backend, requires
hardware timestamps, reads the GPU state back once, and fails when disagreement
exceeds its declared floating-point tolerances.

Run it from the repository root with the optional `gpu` feature:

```powershell
cargo run --manifest-path language/runtime/Cargo.toml --release --features gpu --example native-water-native -- --counts 19x15x14,48x32x32,64x48x48 --rounds 31
```

Two consecutive 31-round runs were recorded on 2026-08-29 with Rust 1.88.0 and
wgpu 30.0.1 on an Intel Core i9-9900K, NVIDIA GeForce RTX 4090, and Vulkan
1.4.341. The table below is the second run from the finished release binary.
Each time is a median.

| Particles | Retained pairs | Rust CPU pass | Vulkan kernel | Synchronized Vulkan | CPU / synchronized Vulkan | Maximum acceleration error |
|---:|---:|---:|---:|---:|---:|---:|
| 3,990 | 45,305 | 0.409 ms | 0.025 ms | 0.140 ms | 2.92x | 1.907e-6 |
| 49,152 | 602,780 | 11.295 ms | 0.068 ms | 0.222 ms | 50.81x | 1.907e-6 |
| 147,456 | 1,841,852 | 36.902 ms | 0.809 ms | 1.139 ms | 32.39x | 2.861e-6 |

Density and pressure agreed exactly in both recorded runs. `Vulkan kernel`
uses device timestamps spanning the density and force dispatches.
`Synchronized Vulkan` also includes command encoding, submission, timestamp
readback, and the CPU wait. The one-time Vulkan context and pipeline creation
took 591 ms in the second run. Graph construction and GPU resource upload are
reported separately in the [complete benchmark record](native-water/native-benchmark-2026-08-29.txt).

**Boundary:** this is a native execution result for an already-built retained
coefficient graph. It does not time graph/coefficient rebuilding after particle
motion, integration, collisions, or rendering. The CPU baseline is optimized
but single-threaded; it is not a production multicore SPH solver. The Rust and
WGSL are handwritten translations, not output from the Native Space compiler.
The result therefore separates JavaScript overhead from this kernel, but does
not yet establish a full-fluid or compiler-generated speedup.

### Native visual runner [Implemented, not benchmarked]

The separate [native visual application](../language/runtime/examples/native_water_visual/main.rs)
provides CPU-indexed, Vulkan-indexed, and Vulkan all-pairs implementations of
the same finite WCSPH approximation. Every substep first derives membership and
density/pressure from the complete previous state, then writes integration
results into a separate next-state buffer. No output particle can affect another
output until the following substep. The CPU mode uploads only its completed
state for Vulkan presentation. JavaScript is absent from every frame path.

```powershell
cargo run --manifest-path language/runtime/Cargo.toml --release --features visual --example native-water-visual
```

The window renders a three-axis marker and the complete simulation box. Its
default water view derives nearest surface depth and accumulated thickness from
the current particle buffer, smooths only nearby depths in two screen-space
passes, reconstructs surface normals, and shades the result as one continuous
body. Press `W` to switch to the direct particle view. This is a reversible
camera choice: both views consume the same current GPU state, and neither view
feeds screen coordinates, depth, thickness, or color back into the simulation.

Drag with the left mouse button to rotate the native 3D state and use the wheel
to zoom. Up/Down (or `+`/`-`) selects power-of-two loads from 512 through
1,048,576 particles and reconstructs a deterministic density-compatible
lattice for that load. `F` or Home restores the fitted camera, Space pauses,
`R` restores the selected lattice, and Escape exits. Press `I` to cycle through
CPU-indexed, GPU-indexed, and all-pairs WCSPH at supported loads. The fitted
camera uses the rotated box extents and current window aspect ratio, so resizing
or rotating does not clip the box at the default zoom.

Indexed WCSPH keeps the finite relation between every grid
cell and its 27 neighboring cells fixed. For each step Vulkan clears the prior
membership, maps every input point to one cell from its native coordinates,
evaluates only those neighboring cells, writes the translated position and
velocity to the opposite state buffer, and swaps the buffers. No rendered or
classical screen coordinate is fed back into the state. Changing mode resets
every path to the same deterministic input lattice. The quadratic baseline is
disabled above 16,384 particles; larger presets cycle between CPU-indexed and
GPU-indexed WCSPH.

The physical clock remains 240 steps per second. Presets above the 16,384-point
reference divide each physical step into two through four solver substeps as
their smoothing radius shrinks. Each CPU and GPU substep rebuilds membership
and performs one old-state-to-new-state translation. The window title exposes
the active multiplier as `solver Nx` and reports CPU milliseconds per physical
step when the CPU backend is selected.

The title reports particle count, rendered FPS, achieved simulation steps per
second against the 240-step target, dropped steps, and physical adapter. A load
has stopped running in real time when its simulation-step rate remains below
240 or dropped steps remain nonzero; rendered FPS alone is not the completion
criterion.

**Design boundary:** screen-space surface reconstruction changes presentation,
not the WCSPH approximation or its stability. Thickness includes overlapping
particle spheres along the view ray, and the normal is reconstructed from a
finite depth image; neither is a new simulated physical quantity. The visual
runner has equivalent CPU/GPU spatial INDEX implementations, an all-pairs
baseline through 16,384 particles, and indexed loads through 1,048,576. The
cell-neighbor graph is retained, but dynamic point membership is rebuilt each
step and interaction coefficients are recomputed from current coordinates. The
CPU implementation is a handwritten Rust mirror of the WGSL equations, not
Native Space compiler output. The language cannot yet express the required
dynamic neighbor selection, comparisons, division, or square root. A prior
ordered cable experiment was removed because it connected particles according
to array serialization and violated the full-state transition invariant.
The optional `cpu-simd` build substitutes one safe SIMD vector type into the
same solver source and the `benchmark-cpu` command measures it without Vulkan.
It is not the default: SIMD across the three coordinate components produced
load-sensitive results and does not vectorize the data-dependent neighbor walk.
Useful SIMD here requires a future contiguous neighbor-lane layout and a new
controlled A/B measurement, not unsafe gathers around the linked-list index.
Controlled multi-run timing and a CPU/GPU state-error table remain required
before claiming a visual-runner speedup.

## H-NS-1 / E-NS-1 [Hypothesis / Planned turbulence experiment]

On frozen JHTDB time blocks, compare native transfer operators against
spectral/Galerkin, POD/DMD, sparse operator-inference, and matched learned
baselines. Measure coefficient and shell-transfer error, spectrum, dissipation,
divergence residual, rollout stability, bytes, and latency. Refute the
candidate if sparsity is only ordinary triad selection, disappears with
resolution, or violates divergence/dissipation.

Regularity work starts only if a dataset-independent invariant controls
arbitrarily high frequency and survives refinement. Until proved, it remains a
conjecture.

## Primary sources

- [Clay Navier–Stokes problem statement](https://www.claymath.org/wp-content/uploads/2022/02/MPPc.pdf)
- [Johns Hopkins Turbulence Database](https://turbulence.pha.jhu.edu/)
- [Forced isotropic turbulence data](https://turbulence.pha.jhu.edu/Forced_isotropic_turbulence.aspx)
- [JHTDB citation guidance](https://turbulence.pha.jhu.edu/citing.aspx)
