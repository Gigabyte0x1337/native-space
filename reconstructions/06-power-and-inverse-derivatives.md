<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# R-CALC-1: Power and Inverse Derivative Rules

**Status:** Reproduced  
**Known results:** derivative power rule and derivative of a reciprocal  
**Dependencies:** A-AN-1, L-BIN-1, L-OS-9, T-DER-POWER-1, T-DER-INV-1,
T-DER-CX-1, T-CX-2  
**Native invariant:** first-order change is native multiplication by the
linearized state action; all remaining power terms contain at least two
perturbation factors

## Conventional statements

For $n\in\mathbb{N}_0$, the complex power map $p_n(z)=z^n$ has directional
derivative

$$
Dp_n{}_z(h)=n z^{n-1}h
$$

for $n\geq1$, while $Dp_0=0$. On $z\neq0$, the reciprocal map has

$$
D(z\mapsto z^{-1})_z(h)=-z^{-2}h.
$$

The same formulas restricted to real $z,h$ are the ordinary one-variable
rules.

## Native power derivation

For the native scalar power $P_n(z)=z^{\boxtimes n}$, L-BIN-1 gives the
exact perturbation expansion

$$
(z\boxplus h)^{\boxtimes n}
=z^{\boxtimes n}
\boxplus
\widehat n\boxtimes z^{\boxtimes(n-1)}\boxtimes h
\boxplus R_n(z,h),
$$

where every term of $R_n$ contains at least two factors of $h$. L-OS-9
bounds that finite remainder by $C_z\lVert h\rVert^2$ on a bounded
neighborhood. Division by $\lVert h\rVert$ therefore sends the normalized
remainder to zero under A-AN-1. T-DER-POWER-1 concludes

$$
DP_n{}_z(h)
=\widehat n\boxtimes z^{\boxtimes(n-1)}\boxtimes h.
$$

For $n=0$, the map is constant $\mathbf1$ and its derivative is
$\mathbf0$.

## Native inverse derivation

For nonzero $z$ and sufficiently small $h$, the field laws give the exact
resolvent identity

$$
(z\boxplus h)^{[-1]}\boxminus z^{[-1]}
=
\boxminus\bigl((z\boxplus h)^{[-1]}
\boxtimes h\boxtimes z^{[-1]}\bigr).
$$

Subtracting the candidate linear term leaves a remainder with two factors of
$h$; inverse factors remain bounded near nonzero $z$. T-DER-INV-1 gives

$$
D(\mathrm{inv})_z(h)
=\boxminus\bigl(z^{[-1]}\boxtimes h\boxtimes z^{[-1]}\bigr).
$$

The zero scalar is excluded because no multiplicative inverse exists there.

## Back-translation

T-CX-2 preserves native ADD, MULTIPLY, inverses, and natural scalar
coefficients. T-DER-CX-1 identifies the derivative camera with the same
real-linear coordinate map. Applying $\kappa$ therefore yields

$$
Dp_n{}_z(h)=n z^{n-1}h
$$

and, using commutativity,

$$
D(\mathrm{inv})_z(h)=-z^{-1}hz^{-1}=-z^{-2}h.
$$

These are real Fréchet derivative statements on $\mathbb{C}\cong\mathbb{R}^2$.
Their complex-linear form is visible because each derivative acts by
multiplication with one fixed oriented scalar.

## Comparison

**Classification:** operations-first and equivalent in analytic strength.

The native proof makes the derivative a state action and isolates the exact
quadratic-or-higher remainder before taking a limit. This is the standard
commutative-field proof in typed native operations. It does not reduce the
required finite-dimensional limit theory or establish a shorter foundation.

## Executable boundary

Language 1.0 has no limit or derivative node. Exact evaluations at finitely
many rational perturbations cannot prove a derivative limit, so no numerical
cross-check is promoted as evidence here.

## What is not established

- The inverse rule has no value at zero.
- No fractional, irrational, or state-valued general exponent is defined.
- No infinite-support or unbounded-mode derivative is covered.
- No integration theorem follows from these results.
