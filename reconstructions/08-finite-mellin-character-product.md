<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# R-MELLIN-1: Finite Mellin/Dirichlet Character Product Law

**Status:** Reproduced  
**Known result:** evaluation of a finite monoid convolution by a multiplicative
character becomes ordinary multiplication  
**Dependencies:** L-MELLIN-1, L-MELLIN-2, T-MELLIN-1, T-MELLIN-2  
**Native invariant:** MULTIPLY adds directional depths, while the flow character
turns weighted depth ADD into coefficient MULTIPLY

## Conventional finite statement

Let $c_\alpha,d_\alpha$ have finite support on a free commutative monoid,
let

$$
(c*d)_\gamma
\mathrel{=}
\sum_{\alpha+\beta=\gamma}c_\alpha d_\beta,
$$

and let $U(\alpha+\beta)=U(\alpha)+U(\beta)$. Then for complex $s$,

$$
\sum_\gamma(c*d)_\gamma e^{-sU(\gamma)}
\mathrel{=}
\left(\sum_\alpha c_\alpha e^{-sU(\alpha)}\right)
\left(\sum_\beta d_\beta e^{-sU(\beta)}\right).
$$

All sums are finite.

## Native derivation

Native MULTIPLY is the finite monoid convolution on multi-indices. D-MELLIN-1
builds $U_u$ from directional depths, and L-MELLIN-1 proves its additivity.
D-MELLIN-2 then defines the flow character

$$
\chi_{u,s}(\alpha)=\mathcal E_s(-U_u(\alpha)).
$$

L-MELLIN-2 turns index composition into character MULTIPLY. T-MELLIN-1
therefore gives immediately

$$
\mathcal M_{u,s}(F\star G)
=\mathcal M_{u,s}(F)\boxtimes\mathcal M_{u,s}(G).
$$

## Back-translation

T-MELLIN-2 and T-CX-2 identify each native character with
$e^{-sU_u(\alpha)}$ in conventional complex coordinates. Expanding native
MULTIPLY and both finite camera sums gives exactly the conventional finite
product formula.

If positive bases $b_k=e^{u(a_k)}$ are introduced, each term is

$$
\left(\prod_k b_k^{d_k(\alpha)}\right)^{-s},
$$

which is the finite algebraic shape later used by Dirichlet and Mellin-type
expressions. No base is a prime at this stage.

## Information and comparison

**Classification:** operations-first and equivalent to finite character
evaluation.

The native form cleanly separates primitive identity, directional depth,
additive log-weight, and the external scalar character. It also shows that the
product law is already finite algebra; no transform integral is needed for
that law. This is the standard monoid-algebra character argument and is not a
new theorem.

The camera is generally noninjective. Formal states can collide after scalar
evaluation, so a successful product identity is not evidence that the camera
preserves the original representation.

## Executable boundary

No general exact-rational cross-check is added because flow characters usually
contain transcendental coordinates. The already tested exact C-EVAL runtime
implements the finite algebraic evaluation route; identifying arbitrary flow
values requires a future approximation contract.

## What is not established

- No infinite Dirichlet series or Mellin integral is defined by this finite
  reconstruction. R-ZETA-1 later adds the Dirichlet series and Euler product on
  $\mathrm{Re}(s)>1$, but no Mellin integral or continuation.
- This record itself establishes no pole, zero, or analytic continuation.
- No prime ordering or prime birth index has been introduced.
- No speedup or compression claim follows.
