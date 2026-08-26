<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Finite Cyclic Convolution and Fourier Cameras

This Stage 6 extension connects one native INDEX direction to finite cyclic
convolution and the discrete Fourier transform (DFT). The cyclic structure is
not smuggled into the 1.0 index carrier: it is introduced by an explicit
quotient camera that folds directional depth modulo $N$.

Fix a primitive direction $k$ and an integer $N\geq1$.

## One-direction native subring

### D-CYCLIC-1 -- one-direction carrier [Definition]

Define

$$
\mathcal N_{\langle k\rangle}
:=
\left\{
F\in\mathcal N_{\mathcal A}:
\mathrm{supp}(F)\subseteq
\{n\varepsilon_k:n\in\mathbb{N}_0\}
\right\}.
$$

It is closed under native ADD and MULTIPLY because depths on direction $k$
add and no other direction is introduced.

## Cyclic depth-fold camera

### C-CYCLIC-1 -- modulo-depth fold [Definition]

Define

$$
\mathcal C_{N,k}:\mathcal N_{\langle k\rangle}\to\mathbb{O}^N
$$

by

$$
(\mathcal C_{N,k}F)_r
:=
\mathop{\boxplus}_{\substack{n\geq0\\n\equiv r\pmod N}}
F_{n\varepsilon_k},
\qquad 0\leq r<N.
$$

Only finitely many summands are nonzero. On $\mathbb{O}^N$, define
componentwise ADD and cyclic convolution

$$
(f*_Ng)_r
:=
\mathop{\boxplus}_{j=0}^{N-1}
f_j\boxtimes g_{(r-j)\bmod N}.
$$

The camera is intended to preserve native ADD as componentwise ADD and native
MULTIPLY as cyclic convolution.

Its exact fibers are candidate equivalence classes

$$
F\sim_{N,k}G
\quad\Longleftrightarrow\quad
\mathcal C_{N,k}F=\mathcal C_{N,k}G.
$$

It is surjective by choosing representatives at depths $0,\ldots,N-1$, but
not injective on the full one-direction carrier: for example,
$\mathsf1$ and $\mathsf X_k^{\star N}$ have the same image. Thus it
forgets absolute depth beyond its residue class. It does not identify the
primitive direction $k$ with that depth.

## Native root of unity

### D-DFT-1 -- finite orientation root [Definition]

Using D-FLOW-1, define

$$
\omega_N:=\mathcal E_{\mathbf J}\left(-\frac{2\pi}{N}\right).
$$

Integer powers use native MULTIPLY; negative powers use the nonzero inverse
proved in T-FLOW-1.

## Fourier camera

### C-DFT-1 -- native-coefficient DFT [Definition]

Define

$$
\mathcal F_N:\mathbb{O}^N\to\mathbb{O}^N
$$

by

$$
(\mathcal F_N f)_r
:=
\mathop{\boxplus}_{j=0}^{N-1}
f_j\boxtimes\omega_N^{\boxtimes rj},
\qquad 0\leq r<N.
$$

The candidate inverse is

$$
(\mathcal F_N^{-1}h)_j
:=
\widehat N^{[-1]}
\boxtimes
\mathop{\boxplus}_{r=0}^{N-1}
h_r\boxtimes\omega_N^{\boxtimes(-rj)},
$$

where $\widehat N=(N,0)\in\mathbb{O}$.

## Required boundaries

- $\mathcal C_{N,k}$ is a quotient camera, not an equivalence on absolute
  native depth.
- $\mathcal F_N$ is intended to be a bijective linear camera on the cyclic
  coefficient vector; it introduces no further information loss.
- The composite $\mathcal F_N\circ\mathcal C_{N,k}$ remains lossy exactly
  because $\mathcal C_{N,k}$ is lossy.
- A DFT formula is not an FFT algorithm and carries no runtime claim.
- Infinite Fourier series and Fourier integrals remain outside this finite
  construction.

The classifications and convolution law are proved in
`../proofs/07-finite-fourier.md`.
