<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Matrix, Tensor, and LLM Compression

**Status:** finite algebra proved; matrix camera and all compression claims open.

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
