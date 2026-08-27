<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Finite Arithmetic Functions and Dirichlet Convolution

This Stage 7 layer translates finitely supported arithmetic functions into
prime-indexed native states. Its central claim is exact and finite:

> pointwise function addition becomes native ADD, and Dirichlet convolution
> becomes native MULTIPLY.

The statement is an algebraic isomorphism, not a speedup claim. Constructing a
native index from an unfactored integer still requires the valuations excluded
from the language's automatic operations.

## Function carrier

### D-ARITH-1 -- finitely supported oriented arithmetic functions [Definition]

Let

$$
\mathcal F_{\mathrm{fin}}
:=
\{f:\mathbb{N}_{>0}\to\mathbb{O}:
\mathrm{supp}(f)\text{ is finite}\},
$$

where

$$
\mathrm{supp}(f):=\{n:f(n)\neq\mathbf0\}.
$$

The coefficient carrier is the oriented-scalar ring $\mathbb{O}$. Ordinary
real- or rational-valued finite arithmetic functions enter through the scalar
embedding $r\mapsto(r,0)$.

### D-ARITH-2 -- pointwise additive structure [Definition]

For $f,g\in\mathcal F_{\mathrm{fin}}$, define

$$
(f\boxplus_{\!A}g)(n):=f(n)\boxplus g(n),
\qquad
(\boxminus_{\!A}f)(n):=\boxminus f(n).
$$

Let $0_A(n)=\mathbf0$, and define the unit point mass

$$
\delta_1(n):=
\begin{cases}
\mathbf1,&n=1,\\
\mathbf0,&n\neq1.
\end{cases}
$$

Finite support is preserved by these pointwise operations.

## Native-first multiplication

### D-DIR-1 -- finite Dirichlet convolution [Definition]

For $f,g\in\mathcal F_{\mathrm{fin}}$, define

$$
(f\star_D g)(n)
:=
\mathop{\boxplus}_{d\mid n}
f(d)\boxtimes g(n/d).
$$

Every coefficient sum is finite because a positive integer has finitely many
positive divisors. The resulting function also has finite support: a nonzero
coefficient can occur only at a product $mn$ with
$m\in\mathrm{supp}(f)$ and
$n\in\mathrm{supp}(g)$.

### C-ARITH-1 -- prime-index coefficient lift [Definition]

Define

$$
\Phi:\mathcal F_{\mathrm{fin}}
\longrightarrow
\mathcal N_{\mathcal A_{\mathbb{P}}}
$$

by

$$
\Phi(f)
:=
\mathop{\bigoplus}_{n\in\mathrm{supp}(f)}
f(n)[\alpha_n],
$$

where $\alpha_n$ is D-ENC-1's unique prime multi-index. Equivalently,
the coefficient of $\Phi(f)$ at $\alpha_n$ is $f(n)$.

This map changes coordinates from positive-integer labels to prime-depth
labels. It does not evaluate or collapse coefficients.

## Operations-first interpretation

The conventional divisor formula in D-DIR-1 is the integer-coordinate camera
of the simpler native rule

$$
(F\star G)_\gamma
\mathrel{=}
\mathop{\boxplus}_{\alpha\oplus_I\beta=\gamma}
F_\alpha\boxtimes G_\beta.
$$

T-ENC-1 identifies $mn$ with
$\alpha_m\oplus_I\alpha_n$. Therefore a divisor split $n=d(n/d)$
is precisely a multi-index split. The exact isomorphism is proved in
`../proofs/10-dirichlet-convolution.md`.

## Finite character camera

Composing $\Phi$ with C-MELLIN-1 gives the finite readout

$$
\mathcal D_{u,s}(f)
:=
\mathcal M_{u,s}(\Phi(f))
\mathrel{=}
\mathop{\boxplus}_{n\in\mathrm{supp}(f)}
f(n)\boxtimes\chi_{u,s}(\alpha_n).
$$

This is a finite character polynomial. Choosing prime-dependent weights can
produce familiar finite Dirichlet-polynomial coordinates, but no infinite
Dirichlet series is created by that choice.

## Explicit exclusions

- The constant-one function, Möbius function, divisor-counting function, and
  other globally supported arithmetic functions are not members of
  $\mathcal F_{\mathrm{fin}}$.
- Truncation to $n\leq N$ is a projection and is not closed under ordinary
  Dirichlet convolution unless an additional quotient or truncation rule is
  declared.
- No infinite sum, Euler product, zeta function, convergence half-plane, or
  analytic continuation is defined in this finite carrier. The completed
  extension later proves the absolute zeta camera on
  $\mathrm{Re}(s)>1$.
- The isomorphism does not provide fast factorization or fast convolution.
- Birth orientation remains a separate annotation and is not multiplied into
  arithmetic-function coefficients.
