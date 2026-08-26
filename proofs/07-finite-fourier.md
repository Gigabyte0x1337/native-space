<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Finite Cyclic and Fourier Camera Proofs

## Dependencies

These proofs use the finite native ring, T-OS-1, the native flow results,
T-POLAR-1's exact orientation fibers, and the definitions in
`../theory/05-finite-convolution-fourier-camera.md`. All sums are finite.

### L-ROOT-1 -- exact order of the native Fourier root [Proved]

**Claim.** $\omega_N^{\boxtimes N}=\mathbf1$. For
$0<m<N$, $\omega_N^{\boxtimes m}\neq\mathbf1$.

**Proof.** L-FLOW-1 and L-FLOW-2 give

$$
\omega_N^{\boxtimes m}
=\mathcal E_{\mathbf J}\left(-\frac{2\pi m}{N}\right).
$$

T-FLOW-2 identifies this flow with unit orientation wrapping. T-POLAR-1 says
it equals $\mathbf1$ exactly when its angle is an integer multiple of
$2\pi$. This holds for $m=N$ and fails for $0<m<N$. $\square$

### L-ORTH-1 -- finite root orthogonality [Proved]

**Claim.** For every integer $m$,

$$
\mathop{\boxplus}_{r=0}^{N-1}
\omega_N^{\boxtimes rm}
=
\begin{cases}
\widehat N,&N\mid m,\\
\mathbf0,&N\nmid m.
\end{cases}
$$

**Proof.** If $N\mid m$, L-ROOT-1 makes every summand $\mathbf1$, so
the sum is $\widehat N$.

Otherwise let $q=\omega_N^{\boxtimes m}$. Reducing $m$ modulo $N$ and
using L-ROOT-1 shows $q\neq\mathbf1$, while $q^{\boxtimes N}=\mathbf1$.
The finite distributive telescoping identity gives

$$
(\mathbf1\boxplus(\boxminus q))
\boxtimes
\mathop{\boxplus}_{r=0}^{N-1}q^{\boxtimes r}
=\mathbf1\boxplus(\boxminus q^{\boxtimes N})
=\mathbf0.
$$

Because $q\neq\mathbf1$, the first factor is nonzero and invertible by
T-OS-1. The finite sum is therefore $\mathbf0$. $\square$

### T-CYCLIC-1 -- cyclic fold classification [Proved]

**Claim.** $\mathcal C_{N,k}$ is surjective, preserves ADD, and satisfies

$$
\mathcal C_{N,k}(F\star G)
=
(\mathcal C_{N,k}F)*_N(\mathcal C_{N,k}G).
$$

Its fibers are exactly the stated $\sim_{N,k}$ classes, and it is not
injective on $\mathcal N_{\langle k\rangle}$.

**Proof.** Given $f\in\mathbb{O}^N$, the finite state

$$
F=\mathop{\bigoplus}_{r=0}^{N-1}f_r[r\varepsilon_k]
$$

maps to $f$, proving surjectivity. ADD preservation follows by regrouping
finite coefficients within each residue class.

For MULTIPLY, D-NS-7 pairs depths $a,b$ at output depth $a+b$. Folding
that output into residue $r$ collects exactly the pairs whose residues obey
$a+b\equiv r\pmod N$. Grouping first by the residue of $a$ gives the
cyclic-convolution formula. Finite associativity and distributivity justify the
regrouping.

The fiber statement is the definition of $\sim_{N,k}$. Finally,
$\mathsf1\neq\mathsf X_k^{\star N}$ because their absolute depths differ,
but both fold to the vector with $\mathbf1$ at residue zero. $\square$

### T-DFT-1 -- Fourier camera is bijective [Proved]

**Claim.** The camera $\mathcal F_N$ is real-linear, and the candidate
$\mathcal F_N^{-1}$ is its two-sided inverse.

**Proof.** Substitute the forward definition into the inverse and use finite
distributivity:

$$
\begin{aligned}
(\mathcal F_N^{-1}\mathcal F_N f)_j
&=\widehat N^{[-1]}
  \boxtimes
  \mathop{\boxplus}_{r=0}^{N-1}
  \mathop{\boxplus}_{l=0}^{N-1}
  f_l\boxtimes\omega_N^{\boxtimes r(l-j)}\\
&=\widehat N^{[-1]}
  \boxtimes
  \mathop{\boxplus}_{l=0}^{N-1}
  f_l\boxtimes
  \left(
  \mathop{\boxplus}_{r=0}^{N-1}
  \omega_N^{\boxtimes r(l-j)}
  \right).
\end{aligned}
$$

L-ORTH-1 makes the inner sum $\widehat N$ only when $l=j$, and zero
otherwise. The result is $f_j$. The reverse composition has the same finite
calculation with the roles of indices exchanged. Finite coefficient ADD and
real homogeneity of $\boxtimes$ show directly from C-DFT-1 that
$\mathcal F_N$ is real-linear. $\square$

### T-DFT-2 -- finite convolution theorem [Proved]

**Claim.** For $f,g\in\mathbb{O}^N$,

$$
\mathcal F_N(f*_Ng)
=
\mathcal F_N(f)\boxtimes_{\mathrm{pt}}\mathcal F_N(g),
$$

where $\boxtimes_{\mathrm{pt}}$ is componentwise native MULTIPLY.

**Proof.** For output frequency $r$, substitute cyclic convolution and
reindex the finite pairs by $l=(s-j)\bmod N$:

$$
\begin{aligned}
(\mathcal F_N(f*_Ng))_r
&=\mathop{\boxplus}_{s=0}^{N-1}
  \mathop{\boxplus}_{j=0}^{N-1}
  f_j\boxtimes g_{(s-j)\bmod N}
  \boxtimes\omega_N^{\boxtimes rs}\\
&=\mathop{\boxplus}_{j=0}^{N-1}
  \mathop{\boxplus}_{l=0}^{N-1}
  f_j\boxtimes g_l
  \boxtimes\omega_N^{\boxtimes r(j+l)}\\
&=left(\mathop{\boxplus}_{j=0}^{N-1}
  f_j\boxtimes\omega_N^{\boxtimes rj}\right)
  \boxtimes
  \left(\mathop{\boxplus}_{l=0}^{N-1}
  g_l\boxtimes\omega_N^{\boxtimes rl}\right).
\end{aligned}
$$

The second equality uses $\omega_N^{\boxtimes N}=\mathbf1$, so replacing
$j+l$ by its residue does not change the power. The last equality is finite
distributivity. The two factors are the required DFT coefficients. $\square$

## Composite interpretation

T-CYCLIC-1 and T-DFT-2 give

$$
\mathcal F_N\mathcal C_{N,k}(F\star G)
=
\mathcal F_N\mathcal C_{N,k}(F)
\boxtimes_{\mathrm{pt}}
\mathcal F_N\mathcal C_{N,k}(G).
$$

T-DFT-1 is lossless on cyclic vectors. All information loss in the composite
is exactly the modulo-depth folding characterized by T-CYCLIC-1.

## What these proofs do not establish

- They do not identify absolute depths that differ by $N$; the camera loses
  that information deliberately.
- They do not prove an FFT complexity bound or implement an FFT.
- They do not define infinite Fourier series, integrals, or a topology on
  infinite-support states.
