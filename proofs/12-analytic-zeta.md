<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Analytic Zeta Camera Proofs

## Dependencies

These proofs use A-CNS-1, A-LOG-1, A-HOL-1, the finite flow character laws, the prime
factorization theorem T-FTA-1, the completed formal Euler factorization, and
the definitions in `../theory/10-analytic-zeta-camera.md`.

A-CNS-1 is used only for absolute sums, limits, and justified rearrangements.
A-HOL-1 is used for compact-uniform convergence, holomorphic sums, and
uniqueness of analytic continuation.
The zeta convergence estimate and Euler factorization are derived below; they
are not substrate assumptions.

### T-ZETA-FLAT-1 -- the zeta stack is lossless before aggregation [Proved]

**Claim.** On every domain where C-AFLAT-1 is defined, the weighted indexed
stack $\mathrm{Flat}_{\chi_s}(F)$ retains every weighted coefficient.
For the zeta character, it also determines every original coefficient of
$F$. C-COMP-EVAL-1 is exactly its separate final aggregation projection.

**Proof.** The stack records the complete multi-index $\alpha$ and the two
coordinates of $F_\alpha\boxtimes\chi_s(\alpha)$ at every nonzero weighted
coefficient. T-FLOW-1 says every flow-character value $\chi_s(\alpha)$ is
nonzero and has a native multiplicative inverse. Therefore

$$
F_\alpha
=
\big(F_\alpha\boxtimes\chi_s(\alpha)\big)
\boxtimes\chi_s(\alpha)^{[-1]}.
$$

If a weighted coefficient is absent, it is zero; invertibility then forces
$F_\alpha=\mathbf0$. Hence the weighted stack determines $F$
coefficientwise. C-COMP-EVAL-1 applies
$\mathrm{Agg}_{\mathrm{abs}}$ only afterward, by definition.
$\square$

**Boundary.** Aggregation can have nontrivial zero fibers: distinct nonzero
indexed terms may cancel. This theorem does not infer coefficientwise zero
from an aggregated scalar zero.

### T-CAM-COMP-1 -- the absolute completed camera is multiplicative [Proved]

**Claim.** Fix a multiplicative character $\chi$. If
$(F,\chi),(G,\chi)\in\mathrm{Dom}_{\mathrm{abs}}$, then

$$
(F\widehat\star G,\chi)\in\mathrm{Dom}_{\mathrm{abs}}
$$

and

$$
\widehat{\mathcal M}_{\chi}(F\widehat\star G)
=
\widehat{\mathcal M}_{\chi}(F)
\boxtimes
\widehat{\mathcal M}_{\chi}(G).
$$

The domain is also closed under completed ADD when both inputs are summable,
and the camera preserves ADD, zero, and one there.

**Proof.** Put

$$
x_\alpha=F_\alpha\boxtimes\chi(\alpha),
\qquad
y_\beta=G_\beta\boxtimes\chi(\beta).
$$

Multiplicativity of $\chi$, D-COMP-2, and the triangle inequality give

$$
\begin{aligned}
&\sum_\gamma
\left|\kappa\left(
  (F\widehat\star G)_\gamma\boxtimes\chi(\gamma)
\right)\right|\\
&\quad\leq
\sum_{\alpha,\beta}|\kappa(x_\alpha)|\,|\kappa(y_\beta)|\\
&\quad=
\left(\sum_\alpha|\kappa(x_\alpha)|\right)
\left(\sum_\beta|\kappa(y_\beta)|\right)<\infty.
\end{aligned}
$$

The local coefficient sums are finite by L-COMP-LOC-1. A-CNS-1 now permits
reindexing the absolutely summable double family by
$\gamma=\alpha\oplus_I\beta$, yielding

$$
\begin{aligned}
\widehat{\mathcal M}_{\chi}(F\widehat\star G)
&=\mathop{\boxplus}_{\alpha,\beta}^{\mathrm{abs}}
  x_\alpha\boxtimes y_\beta\\
&=\left(\mathop{\boxplus}_\alpha^{\mathrm{abs}}x_\alpha\right)
  \boxtimes
  \left(\mathop{\boxplus}_\beta^{\mathrm{abs}}y_\beta\right).
\end{aligned}
$$

The ADD claim follows from the triangle inequality and linearity of absolute
sums. Zero and one are immediate. $\square$

### L-ZETA-WEIGHT-1 -- native prime-log depth equals integer log [Proved]

**Claim.** For every positive integer $n$,

$$
U_{u_{\log}}(\alpha_n)=\log n.
$$

**Proof.** T-FTA-1 and D-ENC-1 give the finite factorization

$$
n=\prod_k p(k)^{v_k(n)}
$$

and identify $d_k(\alpha_n)=v_k(n)$. D-PLOG-1, D-MELLIN-1, and the
finite product law in A-LOG-1 therefore give

$$
U_{u_{\log}}(\alpha_n)
=\sum_k v_k(n)\log p(k)
=\log\!\left(\prod_kp(k)^{v_k(n)}\right)
=\log n.
$$

Only finitely many terms are nonzero. $\square$

### L-PLOG-HELIX-1 -- prime-log helix remains injective [Proved]

**Claim.** C-PLOG-HELIX-1 is an injective monotone axial deformation of the
flat indexed birth helix.

**Proof.** A-LOG-1 makes $\log$ strictly increasing and injective on the
positive reals. D-PRIME-2 gives $p(j)<p(k)$ whenever $j<k$, hence
$\log p(j)<\log p(k)$. Therefore equality of the third coordinates in
$H_{\log p}(j)$ and $H_{\log p}(k)$ implies $j=k$. The first two
coordinates retain the same quarter-turn orientation cycle as
C-BIRTH-HELIX-1. $\square$

**Boundary.** Logarithmic height is selected because multiplication becomes
additive in that camera. It does not change which primes exist or predict
their spacing.

### L-ZETA-COORD-1 -- zeta-character coordinate [Proved]

**Claim.** Write $z=\kappa(s)=\sigma+i\tau$. For every $n\geq1$,

$$
\kappa(\chi_s(\alpha_n))=n^{-z},
\qquad
|\kappa(\chi_s(\alpha_n))|=n^{-\sigma}.
$$

**Proof.** T-MELLIN-2 and L-ZETA-WEIGHT-1 give

$$
\kappa(\chi_s(\alpha_n))
=\exp(-z\log n)
=n^{-z}
$$

by A-LOG-1. The modulus law in A-LOG-1 gives the second formula. $\square$

### L-P-SERIES-1 -- the required real majorant converges [Proved]

**Claim.** If $\sigma>1$, then

$$
\sum_{n=1}^{\infty}n^{-\sigma}<\infty.
$$

**Proof.** For every $j\geq0$, the dyadic block
$2^j\leq n<2^{j+1}$ contains $2^j$ integers, and every term in it is at
most $2^{-j\sigma}$. Hence its total is at most

$$
2^j2^{-j\sigma}=2^{-j(\sigma-1)}.
$$

Because $\sigma-1>0$, the last quantities form a convergent geometric
series with ratio $2^{-(\sigma-1)}<1$. Comparison under A-CNS-1 proves the
claim. $\square$

### T-ZETA-CONV-1 -- the universal state is summable for Re(s) > 1 [Proved]

**Claim.** If $\mathrm{Re}\kappa(s)>1$, then

$$
(\mathfrak Z_{\mathrm{pattern}},\chi_s)
\in\mathrm{Dom}_{\mathrm{abs}},
$$

and

$$
\kappa(\mathcal Z_N(s))
=\sum_{n=1}^{\infty}n^{-\kappa(s)}.
$$

**Proof.** T-ENC-1 bijects positive integers and prime multi-indices. Every
coefficient of $\mathfrak Z_{\mathrm{pattern}}$ is $\mathbf1$. Under this
bijection, L-ZETA-COORD-1 makes the family of coefficient norms exactly
$n^{-\sigma}$, which is summable by L-P-SERIES-1. D-SUM-1 therefore admits
the completed camera. A-CNS-1 permits the bijective reindexing
$\alpha_n\leftrightarrow n$, and L-ZETA-COORD-1 gives the displayed
series. $\square$

### L-GEO-AN-1 -- each analytic prime factor is a geometric camera [Proved]

**Claim.** If $\sigma=\mathrm{Re}\kappa(s)>1$, then for symbolic
positive observation input $k$, $(\mathsf G_k,\chi_s)$ is absolutely summable and

$$
\widehat{\mathcal M}_{\chi_s}(\mathsf G_k)
=
\left(\mathbf1\boxminus\chi_s(\varepsilon_k)\right)^{\boxtimes-1}.
$$

**Proof.** L-ZETA-COORD-1 with $n=p(k)$ gives

$$
|\kappa(\chi_s(\varepsilon_k))|=p(k)^{-\sigma}<1.
$$

By L-MELLIN-2, the depth-$d$ coefficient evaluates to
$\chi_s(\varepsilon_k)^{\boxtimes d}$. Its norms form a convergent
geometric series, proving absolute summability. The finite geometric identity
from R-GEO-1, followed by the A-CNS-1 limit laws, gives the inverse formula.
$\square$

### T-ZETA-EULER-1 -- native zeta series equals the Euler camera [Reproduced]

**Claim.** If $\mathrm{Re}\kappa(s)>1$, then the ordered analytic
Euler camera exists and

$$
\mathcal Z_N(s)=\mathcal P_N(s).
$$

Equivalently, in conventional coordinates,

$$
\sum_{n=1}^{\infty}n^{-\kappa(s)}
=
\prod_{k=1}^{\infty}(1-p(k)^{-\kappa(s)})^{-1}.
$$

**Proof.** Fix $K$. L-GEO-AN-1 and repeated application of
T-CAM-COMP-1 give

$$
P_K(s)
=\widehat{\mathcal M}_{\chi_s}
  (\mathsf G_1\widehat\star\cdots\widehat\star\mathsf G_K).
$$

By the coefficient calculation in T-EULER-F-1, the state inside the camera has
coefficient $\mathbf1$ exactly on multi-indices supported in the first
$K$ prime directions. Under T-ENC-1 these are precisely the positive
integers whose prime factors all lie among $p_1,\ldots,p_K$. Call this set
$S_K$. Thus

$$
\kappa(P_K(s))=\sum_{n\in S_K}n^{-\kappa(s)}.
$$

The sets $S_K$ increase with $K$. T-FTA-1 says every positive integer has
finite prime support, so their union is all of $\mathbb{N}_{>0}$. The full
family is absolutely summable by T-ZETA-CONV-1. A-CNS-1 therefore makes the
sums over the increasing exhausting sets $S_K$ converge to the full sum.
Consequently $P_K(s)$ converges and its limit is $\mathcal Z_N(s)$.
T-ZETA-CONV-1 supplies the conventional series coordinate, while
L-ZETA-COORD-1 supplies each conventional prime factor. $\square$

### T-ZETA-PAIR-CONV-1 -- paired zeta observations converge for Re(s) > 0 [Proved]

**Claim.** Let

$$
b_m(s)=(2m-1)^{-s}-(2m)^{-s}.
$$

The series $\sum_{m\geq1}b_m(s)$ converges absolutely and locally uniformly
on $\mathrm{Re}(s)>0$, and its sum is holomorphic there. Consequently
the native paired camera $\mathcal E_N(s)$ is defined on that half-plane.

**Proof.** Fix a compact set $K\subset\{\mathrm{Re}(s)>0\}$. Choose
$\delta>0$ and $M>0$ such that
$\mathrm{Re}(s)\geq\delta$ and $|s|\leq M$ on $K$. For real
$x>0$, set $f_s(x)=x^{-s}$. Then

$$
f_s'(x)=-s x^{-s-1}.
$$

The real-path integral formula and A-LOG-1 give

$$
\begin{aligned}
|b_m(s)|
&=\left|\int_{2m-1}^{2m}s x^{-s-1}\,dx\right|\\
&\leq M(2m-1)^{-\delta-1}.
\end{aligned}
$$

The majorant is summable because $\delta+1>1$. The comparison and
compact-uniform convergence clauses of A-CNS-1 and A-HOL-1 therefore prove
absolute convergence at every point, local uniform convergence, and
holomorphicity of the sum. C-ZETA-PAIR-1 identifies that sum with
$\kappa(\mathcal E_N(s))$. $\square$

### T-ZETA-PAIR-IDENTITY-1 -- paired and direct cameras agree on Re(s) > 1 [Proved]

**Claim.** If $\mathrm{Re}(s)>1$, then

$$
\mathcal E_N(s)
=
\left(
\mathbf1\boxminus\underline2\boxtimes\chi_s(\alpha_2)
\right)
\boxtimes\mathcal Z_N(s).
$$

Equivalently,

$$
\sum_{m\geq1}\left((2m-1)^{-s}-(2m)^{-s}\right)
=(1-2^{1-s})\sum_{n\geq1}n^{-s}.
$$

**Proof.** T-ZETA-CONV-1 gives absolute convergence, so A-CNS-1 permits
separation into odd and even observations and bijective reindexing. Thus

$$
\begin{aligned}
\sum_{m\geq1}\left((2m-1)^{-s}-(2m)^{-s}\right)
&=\sum_{n\geq1}n^{-s}-2\sum_{m\geq1}(2m)^{-s}\\
&=\sum_{n\geq1}n^{-s}-2^{1-s}\sum_{m\geq1}m^{-s}\\
&=(1-2^{1-s})\sum_{n\geq1}n^{-s}.
\end{aligned}
$$

L-ZETA-COORD-1 and the field isomorphism $\kappa$ translate this coordinate
identity back to the displayed native identity. $\square$

### L-ZETA-PAIR-DENOM-1 -- the paired denominator is nonzero in the critical strip [Proved]

**Claim.** If $0<\mathrm{Re}(s)<1$, then

$$
1-2^{1-s}\neq0.
$$

**Proof.** Write $\sigma=\mathrm{Re}(s)$. A-LOG-1 gives

$$
|2^{1-s}|=2^{1-\sigma}>1.
$$

It therefore cannot equal $1$, whose modulus is $1$. $\square$

### T-ZETA-STRIP-1 -- the paired camera is the direct zeta coordinate in the critical strip [Reproduced]

**Claim.** On $0<\mathrm{Re}(s)<1$, the operations-first coordinate

$$
\mathcal Z_N^{\mathrm{pair}}(s)
=
\mathcal E_N(s)\boxtimes
\left(
\mathbf1\boxminus\underline2\boxtimes\chi_s(\alpha_2)
\right)^{\boxtimes-1}
$$

is holomorphic and is the analytic continuation of the direct zeta camera
from $\mathrm{Re}(s)>1$.

**Proof.** T-ZETA-PAIR-CONV-1 makes the numerator holomorphic for
$\mathrm{Re}(s)>0$. L-ZETA-PAIR-DENOM-1 makes the denominator
invertible throughout the open critical strip, so A-HOL-1 makes the quotient
holomorphic there. T-ZETA-PAIR-IDENTITY-1 proves on the nonempty open
half-plane $\mathrm{Re}(s)>1$, away from the discrete denominator
zeros on $\mathrm{Re}(s)=1$, that the quotient equals
$\mathcal Z_N(s)$. Every critical-strip point can be joined to that
half-plane by a path in $\mathrm{Re}(s)>0$ avoiding those discrete
points. Uniqueness in A-HOL-1 therefore identifies the paired quotient with
the analytic continuation of the direct camera along that domain. $\square$

**Boundary.** This theorem constructs zeta in the critical strip. It does not
say that a zero of its scalar coordinate makes the retained indexed prime
pattern coefficientwise zero.

### T-ZETA-ZERO-PAIR-1 -- critical-strip zeta zero is paired ADD zero [Proved]

**Claim.** If $0<\mathrm{Re}(s)<1$, then

$$
\mathcal Z_N^{\mathrm{pair}}(s)=\mathbf0
\quad\Longleftrightarrow\quad
\mathcal E_N(s)=\mathbf0.
$$

**Proof.** By L-ZETA-PAIR-DENOM-1 the denominator in C-ZETA-STRIP-1 is a
nonzero field element. MULTIPLY by its inverse is therefore a bijection and
preserves and reflects zero. $\square$

**Boundary.** The right side is the scalar ADD of all odd/even observation
pairs. It does not assert that each pair, each INDEX coefficient, or the
separate reflected-prime mismatch pattern is zero.

### T-ZETA-OUTPUT-RESIDUAL-1 -- automatic zeta residual detects zeta zero [Proved]

The executable derivation entry is valid Native Space source:

```ns
let zeta_classical_pattern = () =>
classical_perspective(zeta_strip_coordinate)
```

Run
`native-space derive --source examples/math-functions.ns zeta_classical_pattern`
from the repository root to inspect its expanded operation trace. Without
`--source`, derivation sees only the generic `language/functions.ns` library.
The trace cites the theorem boundaries; the analytic proof below remains a
paper proof because Native Space 1.0 has no quantified complex analysis kernel.

**Claim.** In the open critical strip,

$$
\mathcal Z_N^{\mathrm{pair}}(s)=\mathbf0
\quad\Longleftrightarrow\quad
\mathrm{Residual}_{Z}(s)=\varnothing.
$$

**Proof.** D-CLASSIC-P-1 writes every oriented scalar uniquely as
$x\mathbf1+y\mathbf J$, and T-CLASSIC-P-1 identifies its two coefficients
with its two owned coordinates. T-CAMERA-RESIDUAL-1 combines those coordinates
automatically and returns the empty residual exactly when both vanish. Apply
this to the paired zeta value. $\square$

**Boundary.** C-ZETA-CLASSICAL-COORDS-1 places this residual on the shared
classical axes. This theorem does not yet relate its coordinate values to the
RE pattern's coordinate values.

### T-ZETA-OUTPUT-FRAME-1 -- total zeta frame detects zeta zero [Proved]

**Claim.** The two zeta coordinate maps exist throughout the open critical
strip, and

$$
\mathcal Z_N^{\mathrm{pair}}(s)=\mathbf0
\quad\Longleftrightarrow\quad
\mathrm{Frame}_{\mathrm{cl}}(\mathcal Z_N^{\mathrm{pair}}(s))
=(0,0)=0.
$$

**Proof.** C-ZETA-CLASSICAL-COORDS-1 and T-CLASSIC-P-1 define the two unique
coordinates $Z_R,Z_I$, proving that both declared axes exist on the domain.
T-ZETA-OUTPUT-RESIDUAL-1 identifies zeta zero with the empty sparse residual.
T-CAMERA-ZERO-FILL-1 uniquely extends that residual over
$\mathrm{Ax}(C_{\mathrm{cl}})=\{\mathbf1,\mathbf J\}$ and is the zero frame exactly when the
residual is empty. $\square$

**Boundary.** The source proof explicitly calls `classical_perspective` before
the local zero comparison. Zeta and RE use the same coefficient convention,
but T-ZETA-RE-ROTATION-1 gives them separate perpendicular quadratic wrappers.
Their zeros remain separate until a wrapped relation is proved.

## Conventional verification references

These sources are checks, not dependencies of the native proof:

- [NIST DLMF, Riemann zeta definition and Euler product](https://dlmf.nist.gov/25.2)
  records the Dirichlet series and Euler product in the half-plane
  $\mathrm{Re}(s)>1$.
- [NIST DLMF, Euler products](https://dlmf.nist.gov/27.4) records the general
  absolute-convergence mechanism.
- [mathlib `EulerProduct.Basic`](https://leanprover-community.github.io/mathlib4_docs/Mathlib/NumberTheory/EulerProduct/Basic.html)
  is an independent formal-library reference for Euler products from
  multiplicativity and norm summability.

## What this proof does not establish

- It supplies the paired analytic continuation in the open critical strip but
  proves no zero-location statement or Riemann Hypothesis.
- It does not prove that scalar zeta cancellation is equivalent to
  coefficientwise cancellation of a retained INDEX stack.
- It reproduces a known theorem; no novelty claim is made for the identity.
- The proof is written and dependency-audited, but not checked by a foundational
  proof assistant.
