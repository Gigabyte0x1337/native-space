<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Formal Completion and the Euler/Zeta Boundary

> **Boundary subsequently extended:** `10-analytic-zeta-camera.md` and
> `../proofs/12-analytic-zeta.md` now define the absolute completed camera and
> prove the zeta/Euler identity on $\mathrm{Re}(s)>1$. This document's
> formal-versus-analytic distinction remains in force outside that domain.

## Status and purpose

This Stage 7 extension separates two questions that must not be conflated:

1. Can infinite coefficient families be multiplied as formal native states?
2. Can an infinite native state be evaluated by a scalar camera?

The first question has an exact algebraic answer because the native index
monoid is locally finite. The second was intentionally left open in this
document; the versioned extension in `10-analytic-zeta-camera.md` now supplies
it on a restricted absolute-summability domain.

This extension does not change the finite Native Space 1.0 carrier or the
finite language runtime.

## Formal completed carrier

### D-COMP-1 -- coefficientwise formal Native Space [Definition]

For a countable primitive-label set $\mathcal A$, define

$$
\widehat{\mathcal N}_{\mathcal A}
:=
\mathbb{O}^{\mathbb{I}_{\mathcal A}},
$$

the set of all oriented-scalar coefficient functions on finite
multi-indices. Equality is coefficientwise. Unlike $\mathcal N_{\mathcal A}$,
a completed state need not have finite support.

The finite carrier embeds by extending every omitted coefficient by
$\mathbf0$.

### D-COMP-2 -- completed native operations [Definition]

For $F,G\in\widehat{\mathcal N}_{\mathcal A}$, define

$$
(F\widehat\oplus G)_\gamma:=F_\gamma\boxplus G_\gamma,
$$

Define completed additive opposite and subtraction coefficientwise by

$$
(\widehat\boxminus F)_\gamma:=\boxminus F_\gamma,
\qquad
F\widehat\boxminus G:=F\widehat\oplus(\widehat\boxminus G).
$$

$$
(F\widehat\star G)_\gamma
:=
\mathop{\boxplus}_{\alpha\oplus_I\beta=\gamma}
F_\alpha\boxtimes G_\beta.
$$

Define completed ORIENT coefficientwise, and completed INDEX by

$$
\widehat{\mathrm{INDEX}}_k(F)
:=\mathsf X_k\widehat\star F.
$$

The convolution sum is finite for each fixed $\gamma$; this is a theorem,
not a convergence assumption.

## All arithmetic functions

### D-ARITH-INF-1 -- unrestricted oriented arithmetic functions [Definition]

Let

$$
\mathcal F_{\mathrm{all}}
:=\mathbb{O}^{\mathbb{N}_{>0}}.
$$

Pointwise ADD and Dirichlet convolution use the same formulas as D-ARITH-2
and D-DIR-1. Each value of a Dirichlet convolution is a finite divisor sum,
even when both functions have infinite support.

### C-ARITH-INF-1 -- completed prime-index lift [Definition]

Define

$$
\widehat\Phi:\mathcal F_{\mathrm{all}}
\longrightarrow
\widehat{\mathcal N}_{\mathcal A_{\mathbb{P}}},
\qquad
(\widehat\Phi(f))_{\alpha_n}:=f(n).
$$

T-ENC-1 guarantees that every completed coefficient receives exactly one
positive-integer label.

## Formal prime factors

### D-GEO-INF-1 -- one-prime geometric state [Definition]

For prime direction $k$, define

$$
\mathsf G_k
:=
\sum_{d\geq0}^{\mathrm{formal}}
\mathbf1[d\varepsilon_k],
$$

meaning the completed state whose coefficient is $\mathbf1$ on the indices
$d\varepsilon_k$ and $\mathbf0$ elsewhere. No scalar infinite sum is
performed.

### D-ZETA-PATTERN-1 -- one generative multiplicative pattern [Definition]

Define

$$
\mathfrak Z_\mathrm{pattern}(\alpha):=\mathbf1
\qquad
(\alpha\in\mathbb{I}_{\mathcal A_{\mathbb{P}}}).
$$

This is one coefficient rule evaluated at a symbolic finite multiplicity input
$\alpha$. It does not construct or traverse a range. Under
$\widehat\Phi^{-1}$, it is the constant-one arithmetic-function camera. It
is the native zeta pattern, not yet a scalar Riemann zeta value.

### T-ZETA-PATTERN-1 -- the pattern is defined by one finite rule [Proved]

For any finite multi-index input, D-ZETA-PATTERN-1 returns the native unit.
Therefore the pattern is total on the completed coefficient carrier and is
self-represented by one function law rather than an infinite stored state.

## Coefficientwise infinite product

Let

$$
\mathfrak Z_K
:=
\mathsf G_1\widehat\star\cdots\widehat\star\mathsf G_K.
$$

Define the ordered formal product $\prod_{k\geq1}^{\mathrm{coeff}}\mathsf G_k$
when every coefficient of $\mathfrak Z_K$ eventually stabilizes, with the
stabilized values as its coefficients. The proof establishes

$$
\mathfrak Z_\mathrm{pattern}
\mathrel{=}
\prod_{k\geq1}^{\mathrm{coeff}}
(\mathsf1\widehat\boxminus\mathsf X_k)^{-1}
\mathrel{=}
\prod_{k\geq1}^{\mathrm{coeff}}\mathsf G_k.
$$

This is an exact formal Euler factorization. It is not a statement that a
numerical infinite product converges.

## Analytic camera boundary

C-MELLIN-1 evaluates only finite native states. Extending it to a completed
state would require the scalar series

$$
\mathop{\boxplus}_{\alpha\in\mathbb{I}}^{\infty}
F_\alpha\boxtimes\chi_{u,s}(\alpha).
$$

Before this expression can be used, the project must add and justify all of:

1. a topology and completeness theorem for the scalar codomain;
2. a precise ordering-independent summability domain;
3. closure of that domain under completed MULTIPLY;
4. a justified exchange/rearrangement theorem for the double sums;
5. for prime weights, the identification
   $U_u(\alpha_n)=\log n$;
6. convergence of the constant-one state on a stated parameter domain;
7. only after those steps, equality between scalar Dirichlet-series and Euler-
   product cameras.

A conventional absolute-summability candidate is

$$
\sum_\alpha
\left|\kappa(F_\alpha\boxtimes\chi_{u,s}(\alpha))\right|<\infty,
$$

but this line is a proposed analytic domain, not an accepted axiom or proved
Native Space theorem.

## Explicit exclusions

- The exact language and VM remain finite and cannot materialize a
  completed state.
- Coefficientwise stabilization is not norm, pointwise scalar, or uniform
  convergence.
- The formal Euler factorization proves no analytic continuation, functional
  equation, zero-free region, or statement about zeta zeros.
- Nothing here changes prime birth orientation or makes it injective.
- No novelty is claimed for the known formal power-series Euler mechanism.

The carrier choice and its non-analytic boundary are recorded in
The reason for this boundary is recorded directly here: local finiteness keeps
each coefficient computation finite while allowing formal infinite support.
It does not grant scalar convergence, which belongs to the later analytic
camera.
