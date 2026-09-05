<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Matrix, Tensor, and LLM Compression

**Status:** finite algebra and factor-known rank-one coefficient contraction
proved; automatic factor discovery and general compression claims open.

## Proposed native map

For a declared matrix/tensor camera $\mathcal C$, seek a sparse state

$$
W\approx\mathrm{Decode}_{\mathcal C}
\left(\bigoplus_{j=1}^{m}g_j[\alpha_j]\right),
$$

where coefficients hold gain/orientation, INDEX holds mode identity and depth,
and a separately typed selector names the consumed channels. Encode, Decode,
selector cost, approximation error, and any continuous rotation remain to be
defined. A speed claim additionally requires execution without rebuilding a
dense matrix.

## T-MAT-CORE-1 -- native operator action distributes [Proved]

For all finite native states $A,F,G$, define

$$
R=\mathrm{ADD}\!\left(
 \mathrm{MULTIPLY}(A,\mathrm{ADD}(F,G)),
 \mathrm{ORIENT}_2\!\left(\mathrm{ADD}(
   \mathrm{MULTIPLY}(A,F),
   \mathrm{MULTIPLY}(A,G))\right)
\right).
$$

Then $R=\mathsf0$.

**Proof.** L-NS-7 equates the first term with the ADD inside ORIENT. L-SEP-3
and $\mathbf J^{\boxtimes2}=\mathbf{-1}$ make `ORIENT(2, X)` the additive
inverse of $X$. L-NS-2 makes their ADD exactly $\mathsf0$. $\square$

Executable closed instance:
[matrix-distributivity.ns](../examples/applications/matrix-distributivity.ns).

**What this proves:** an already encoded native action is linear over native
ADD. **What it does not prove:** existence, brevity, accuracy, or speed of a
matrix encoding.

## T-MAT-RANK1-CONTRACTION-1 -- one shared multiplication coefficient [Proved]

Let two finite square matrices already be supplied by rank-one factors:

$$
A_{ik}=u_i v_k,
\qquad
B_{kj}=x_k y_j.
$$

Define the shared multiplication coefficient

$$
g=\sum_k v_k x_k.
$$

Then every output coordinate satisfies

$$
(AB)_{ij}
=\sum_k (u_i v_k)(x_k y_j)
=u_i\left(\sum_k v_k x_k\right)y_j
=u_i g y_j.
$$

**Proof.** Associate each finite product, distribute finite ADD over MULTIPLY,
and move the factors independent of $k$ outside the finite ADD. These are the
ordinary finite instances of L-NS-5 and L-NS-7. The remaining ADD is exactly
the coefficient $g$, so both constructions produce the same coefficient at
every matrix INDEX coordinate. Their oriented subtraction is therefore native
zero. $\square$

Executable exact instance:
[matrix-coefficient-contraction.ns](../examples/tryouts/matrix-coefficient-contraction.ns).
It checks a nontrivial integer example in which $g=146$ and all four matrix
coordinates cancel exactly. The source file is a closed witness; the displayed
finite algebra is the general proof.

When the four factor vectors are already available, ordinary dense
multiplication uses $n^3$ scalar multiplications and $n^2(n-1)$ additions. The
coefficient route used by the benchmark needs $n^2+2n$ multiplications and
$n-1$ additions. Both write $n^2$ output values. This proves a lower operation
count for the stated factor-known input representation; it does not prove that
an arbitrary dense matrix can be factored cheaply or exactly.

## E-MAT-COEFFICIENT-1 -- compiled CPU observation [Observed]

Run:

```powershell
cargo run --manifest-path language/runtime/Cargo.toml --release --example matrix_coefficient_benchmark
```

On one Intel Core i9-9900K CPU, two seven-round runs produced these ranges:

| Size | Dense multiplication | Coefficient contraction | Speedup |
|---:|---:|---:|---:|
| 64 | 63.4--68.6 microseconds | 0.795--0.802 microseconds | 79.0--86.3 times |
| 128 | 442--500 microseconds | 2.526--2.534 microseconds | 175--198 times |
| 256 | 4.01--4.27 milliseconds | 11.98--12.42 microseconds | 323--357 times |
| 512 | 32.7--33.0 milliseconds | 50.1--50.8 microseconds | 643--660 times |

The largest observed absolute output difference was below
$3.7\mathbin{\times}10^{-13}$ using `f64`. These are machine-specific elapsed
times for matrices constructed from known rank-one factors. They do not measure
factor discovery, arbitrary dense matrices, the exact-state VM, a tuned BLAS,
or GPU execution.

## H-MAT-1 / E-MAT-1 [Hypothesis / Planned experiment]

At equal total serialized bytes, test whether native atoms improve the error
frontier over truncated SVD, strong sparsity, tensor decomposition, and
quantization:

1. Freeze synthetic controls and at least 20 real operators from two sources.
2. Match bytes, precision, fitting budget, hardware, and decode cost.
3. Measure Frobenius, spectral, and held-out matvec error plus memory and
   end-to-end latency.
4. Require the universal pilot gate from
   [00-evaluation-protocol.md](00-evaluation-protocol.md).

Only after this passes should H-LLM-1 test a native linear layer in a frozen
open model against dense, quantized, low-rank, and sparse baselines. Report
quality, bytes, resident memory, latency, tokens/s, energy/token, and all dense
materialization.

**Refutation conditions:** no equal-byte Pareto gain; index/mask overhead
removes compression; dense reconstruction removes speed; or gains occur only
on synthetic structure used to design the representation.

## Primary sources

- [LoRA paper](https://arxiv.org/abs/2106.09685)
- [GPTQ paper](https://arxiv.org/abs/2210.17323)
- [QLoRA paper](https://arxiv.org/abs/2305.14314)
- [MLPerf Inference methodology](https://docs.mlcommons.org/inference/index_gh/)
- [EleutherAI evaluation harness](https://github.com/EleutherAI/lm-evaluation-harness)
