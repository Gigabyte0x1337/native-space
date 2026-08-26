<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Native Perspective and Zeta Zero-Fiber Audit

## Purpose

This audit stays entirely inside the native algebra. It tests whether viewing
all prime orientations relative to each other supplies the missing implication
from a scalar zeta zero to zero on every reflected-prime strand.

### T-OPPOSITE-PAIR-1 -- opposite birth orientations and their gains [Proved]

Let $O_k:=\mathbf J^{\boxtimes k}$. For every birth index $k$,

$$
O_{k+2}=-O_k.
$$

Therefore, for native real-line gains $a_k,a_{k+2}$,

$$
a_kO_k\boxplus a_{k+2}O_{k+2}
=(a_k-a_{k+2})O_k,
$$

and this pair is zero exactly when $a_k=a_{k+2}$.

**Proof.** T-BIRTH-1 and $\mathbf J^{\boxtimes2}=\mathbf{-1}$ give
$O_{k+2}=O_k\boxtimes\mathbf{-1}=-O_k$. Distributivity gives the
weighted identity. Because $O_k$ is a unit, its product with
$a_k-a_{k+2}$ is zero exactly when the gain difference is zero.
$\square$

With birth indices beginning at one, the first cycle is

$$
p_1:\mathbf J,\qquad p_2:\mathbf{-1},\qquad
p_3:\mathbf{-J},\qquad p_4:\mathbf1.
$$

Thus $p_1/p_3$ form one opposite pair and $p_2/p_4$ the other. Their
positions are opposite; their weighted contributions cancel only when the
corresponding gains agree.

### T-ALL-PERSPECTIVES-1 -- all native observers give one rotated zero equation [Proved]

For a finite birth-oriented family define

$$
S:=\mathop{\boxplus}_k a_kO_k.
$$

From observer orientation $O_r$, every term is seen relative to that same
observer:

$$
S_r
:=
\mathop{\boxplus}_k a_k\mathrm{Rel}(r,[k]_4)
=
\mathop{\boxplus}_k a_k(O_r^{-1}\boxtimes O_k)
=O_r^{-1}\boxtimes S.
$$

Consequently

$$
S_r=\mathbf0\Longleftrightarrow S=\mathbf0
$$

for every $r$, and requiring zero from all four native perspectives is
equivalent to requiring it from any one perspective.

**Proof.** The displayed identity is distributivity of one common inverse
orientation over ADD. T-PERSPECTIVE-ZERO-1 proves both directions of the zero
equivalence. Since every equation $S_r=0$ is obtained by multiplying the
same equation $S=0$ by a unit, the four equations are equivalent rather
than four independent coefficient equations. $\square$

For example, four equal nonzero gains satisfy

$$
\mathbf1\boxplus\mathbf J\boxplus\mathbf{-1}\boxplus\mathbf{-J}
=\mathbf0
$$

from every native perspective, although none of the four gains is zero. The
perspective theorem therefore proves cancellation invariance, not
coefficientwise vanishing.

There are two different operations that must not be conflated:

- one native perspective multiplies **every** term by the same
  $O_r^{-1}$, so it preserves zero;
- making every term individually face $\mathbf1$ multiplies term $k$ by
  its own $O_k^{-1}$, so it is not one perspective transformation.

## Indexed strandwise dephasing

### D-DEPHASE-1 -- strandwise native dephasing [Definition]

Let a finite indexed flat stack be

$$
F=\mathop{\boxplus}_{j\in S}c_j[\alpha_j],
$$

where the INDEX locations $\alpha_j$ are distinct. For nonzero oriented
units $u_j$, define

$$
D_u(F):=
\mathop{\boxplus}_{j\in S}
(u_j^{-1}\boxtimes c_j)[\alpha_j].
$$

This is coefficientwise inverse MULTIPLY while every INDEX label is retained.

### T-DEPHASE-STACK-1 -- dephasing preserves full-stack zero [Proved]

**Claim.** $D_u$ is a bijection with inverse $D_{u^{-1}}$. Hence

$$
D_u(F)=\mathbf0\Longleftrightarrow F=\mathbf0.
$$

**Proof.** At each retained INDEX location, multiplication by $u_j^{-1}$
has inverse multiplication by $u_j$. Distinct locations remain distinct, so
the coefficientwise inverses compose to the identity stack. $\square$

### T-DEPHASE-AGG-NONCOMMUTE-1 -- generic projected zero is not preserved [Proved negative result]

Let $A(F):=\boxplus_j c_j$ be the scalar projection of an arbitrary finite
indexed state. There is no general law

$$
A(F)=\mathbf0\Longrightarrow A(D_u(F))=\mathbf0.
$$

**Proof by finite native counterexample.** At two distinct INDEX locations,
take

$$
F=\mathbf1[\alpha_1]\boxplus\mathbf{-1}[\alpha_2].
$$

The lossless stack is nonzero, but $A(F)=0$. Choose
$u_1=\mathbf1$ and $u_2=\mathbf{-1}$. Then

$$
D_u(F)=\mathbf1[\alpha_1]\boxplus\mathbf1[\alpha_2]
$$

and $A(D_u(F))=\mathbf2\neq\mathbf0$. $\square$

The closed executable witness
[projection-zero-fiber-counterexample.ns](../examples/projection-zero-fiber-counterexample.ns)
proves both finite equations in the 1.0 language: the original aggregate is
zero, while the separately reoriented aggregate is exactly two.

This counterexample concerns arbitrary finite states. It does not disprove a
stronger zero-fiber theorem restricted to the image of the single generative
prime pattern; such a restricted theorem would need its own proof.

The four-term version is equally native: the zero orientation cycle becomes
$\mathbf4$ if four different inverse orientations are applied termwise to
make every term face $\mathbf1$.

### T-ZERO-FIBER-FACTOR-1 -- classification of diagonal zero-preserving transforms [Proved]

Let $J$ be any finite INDEX set with at least two locations, let

$$
A(c)=\sum_{j\in J}c_j,
\qquad
T_v(c)=\sum_{j\in J}v_jc_j,
$$

and let every $v_j$ be an oriented scalar. Then

$$
\ker A\subseteq\ker T_v
\quad\Longleftrightarrow\quad
v_j=v_\ell\text{ for all }j,\ell\in J.
$$

**Proof.** If the zero-fiber inclusion holds, choose the finite state having
coefficient $\mathbf1$ at $j$, coefficient $\mathbf{-1}$ at $\ell$,
and zero elsewhere. It lies in $\ker A$, so

$$
\mathbf0=T_v(c)=v_j-v_\ell.
$$

Hence all multipliers agree. Conversely, if every multiplier is one common
$v$, distributivity gives $T_v(c)=vA(c)$, so every zero of $A$ is a zero
of $T_v$. $\square$

Thus a strand-dependent orientation or gain transform cannot preserve every
aggregate zero unless it is actually one common perspective. The theorem
rules out deriving O-ZETA-RE-CLASSICAL-PATTERN-1 from the generic ADD/MULTIPLY laws
alone. It does not rule out a separate identity restricted to zeta's special
one-parameter image.

### T-TWO-PROJECTION-FIBER-1 -- two scalar zeros do not identify their indexed states [Proved]

For indexed states $F$ and $G$,

$$
A(F)=A(G)=\mathbf0
\quad\Longrightarrow\quad
A(F\boxminus G)=\mathbf0,
$$

but this does not imply $F\boxminus G=\mathbf0$.

**Proof.** Linearity proves the displayed aggregate equation. For the converse,
take two INDEX locations and

$$
F=(\mathbf1,\mathbf{-1}),
\qquad
G=(\mathbf2,\mathbf{-2}).
$$

Both scalar projections are zero, while
$F\boxminus G=(\mathbf{-1},\mathbf1)\neq\mathbf0$. $\square$

Therefore observing zero in both a zeta camera and its reflected scalar camera
still yields only a scalar zero for their difference. Turning that fact into
the coefficientwise reflected-prime equality is precisely the additional
zeta-specific content demanded by O-ZETA-RE-CLASSICAL-PATTERN-1.

## What each native zero proves

| Native readout | Meaning of zero |
|---|---|
| indexed flat stack $F$ | every retained INDEX coefficient is zero |
| scalar aggregate $A(F)$ | the total cancels; indexed coefficients may be nonzero |
| full two-axis birth aggregate | the two opposite lane totals cancel separately |
| real-axis projection only | only the $\mathbf1/\mathbf{-1}$ lane balance is visible |

The rotated second projection in D-AXIS-PROJ-1 exposes the
$\mathbf J/\mathbf{-J}$ lane on a signed real line. Both projections recover
the two-dimensional aggregate, but an aggregate still does not recover every
indexed coefficient.

## Exact zeta consequence

For the reflected prime mismatch at $s=\sigma+it$, the one pattern observed
at symbolic input $k$ has mismatch

$$
\Delta(s;k)=
\left(p(k)^{-\sigma}-p(k)^{-(1-\sigma)}\right)e^{-it\log p(k)}.
$$

Native inverse MULTIPLY by $e^{-it\log p(k)}$ leaves

$$
g(\sigma;k)=p(k)^{-\sigma}-p(k)^{-(1-\sigma)}.
$$

T-NATIVE-RE-1 proves symbolically that $g(\sigma;k)=0$ exactly when
$\sigma=\tfrac{1}{2}$, independently of the selected positive observation.
Here the half is the unique midpoint of the reflected exponent pair
$\sigma$ and $1-\sigma$; the one is the multiplicative identity in
D-RH-REFL-1. It centers the RE input coordinate, while the zeta value remains
tested against the ADD identity zero.
T-ALL-PERSPECTIVES-1 and T-DEPHASE-AGG-NONCOMMUTE-1 show only that the required
pattern implication is not a law of arbitrary projected states.

O-ZETA-RE-CLASSICAL-PATTERN-1 must prove the zeta-specific classical-pattern law

$$
\mathrm{Residual}_{\mathrm{cl}}(\mathcal Z_N^{\mathrm{pair}}(s))=\varnothing
\Longrightarrow
\mathrm{Residual}_{\mathrm{cl}}(\Delta(s;k))=\varnothing,
$$

where both sides use the same local coefficient convention and are then placed
in their separate quadratic wrappers by T-ZETA-RE-ROTATION-1 and
T-ZETA-RE-POSITION-1. The missing construction is the native identity relating
the two wrapped zero patterns.
By T-DUAL-ALIGN-1 this implication is already equivalent to the critical-line
conclusion once the direct zeta coordinate exists. The generic counterexample
neither proves nor refutes this zeta-restricted implication.

Adding the wrapped birth orientation $\mathbf J^k$ is a new camera because
the proved native prime-log character assigns $p^{-s}$, while birth phase is
a separate annotation by D-BIRTH-1. Its two-axis cancellation is proved, but
preservation of the zeta zero set under that camera would itself require a
proved identity.

## Analytic domain audit

The repository proves the direct native Dirichlet-series/Euler-product camera
on $\mathrm{Re}(s)>1$, where absolute aggregation is available. The
paired camera continues its zeta coordinate into the open critical strip by
T-ZETA-STRIP-1. The zeta-specific zero-fiber law remains
O-ZETA-RE-CLASSICAL-PATTERN-1.

These conventional sources verify that boundary; they are not dependencies of
the native finite counterexamples:

- [NIST DLMF, zeta zeros](https://dlmf.nist.gov/25.10)
- [NIST DLMF, Euler products and Dirichlet series](https://dlmf.nist.gov/27.4)

## Audit result

- **Proved:** opposite-pair geometry and its equal-gain condition.
- **Proved:** changing one common native perspective preserves and reflects
  zero, but all observer equations are equivalent rotations of one equation.
- **Proved:** coefficientwise dephasing preserves zero while INDEX is retained,
  but does not preserve a generic scalar camera's zero fiber.
- **Scope:** that generic counterexample does not settle a theorem restricted
  to the single generative prime pattern.
- **Still open:** the zeta-specific zero-fiber identity.
- **Not claimed:** a proof of the Riemann Hypothesis.
