<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Native Space 1.0: Finite Change and Derivative

**Document status:** Native Space 1.0 finite-calculus definition
**Research stage:** 4 -- Derive change and camera derivatives  
**Scope:** Finite coefficient strata only  
**Proof status:** Laws proved in `../proofs/03-native-derivatives.md`

## 1. Analytic substrate boundary

### A-AN-1 -- Finite real-analysis substrate [Axiom]

Assume the ordinary limit theory of finite-dimensional real normed spaces and
the standard real derivatives of polynomial, exponential, logarithmic,
trigonometric, and square-root functions on their usual domains.

This axiom is not part of the algebraic core and was not used to prove it. It is
introduced now because a derivative requires a notion of small change. No
infinite-dimensional Native Space, measure, integral, or series convergence is
assumed.

## 2. Real scaling and finite strata

### D-DER-1 -- Embedded real scalar action [Definition]

For $r\in\mathbb{R}$, define its oriented-scalar embedding

$$
\widehat r:=(r,0)\in\mathbb{O}.
$$

For $F\in\mathcal N_{\mathcal A}$, define

$$
r\odot F:=\eta(\widehat r)\star F.
$$

### D-DER-2 -- Finite coefficient stratum [Definition]

For a finite index set $S\subset\mathbb{I}_{\mathcal A}$, recall

$$
\mathcal N_S
\mathrel{=}
\{F\in\mathcal N_{\mathcal A}:\mathrm{supp}(F)\subseteq S\}.
$$

Perturbations within $\mathcal N_S$ do not create new support directions.
Different choices of $S$ provide compatible finite-dimensional strata of the
full finite-support carrier.

### D-DER-3 -- Native difference [Definition]

For native states $F,G$, define

$$
F\ominus G:=F\oplus(\ominus G).
$$

This is state difference under the proved additive group law.

## 3. Finite native size

### D-DER-4 -- Stratum norm candidate [Definition]

For $F\in\mathcal N_S$, define

$$
\|F\|_S
:=
\left(
\sum_{\alpha\in S}\nu(F_\alpha)
\right)^{1/2}.
$$

For an oriented scalar $z$, write

$$
\|z\|_{\mathbb{O}}:=\sqrt{\nu(z)}.
$$

The norm laws are theorem obligations, not definitions.

### D-DER-5 -- Product-stratum norm [Definition]

For $(F,G)\in\mathcal N_S\times\mathcal N_T$, define

$$
\|(F,G)\|_{S\times T}
:=
\sqrt{\|F\|_S^2+\|G\|_T^2}.
$$

## 4. Native real-linear maps

### D-DER-6 -- Real linearity [Definition]

A map $L:\mathcal N_S\to\mathcal N_T$ is native real-linear when, for all
$F,G\in\mathcal N_S$ and $r\in\mathbb{R}$,

$$
L(F\oplus G)=L(F)\oplus L(G),
\qquad
L(r\odot F)=r\odot L(F).
$$

The analogous definition applies to maps from product strata.

## 5. Native derivative

### D-DER-7 -- Fréchet derivative on a finite native stratum [Definition]

Let $U\subseteq\mathcal N_S$ be open in the metric induced by
$\|\cdot\|_S$, and let $f:U\to\mathcal N_T$. A native real-linear map
$L:\mathcal N_S\to\mathcal N_T$ is the derivative of $f$ at $F\in U$
when

$$
\lim_{\substack{H\to\mathsf0\\H\neq\mathsf0}}
\frac{
\|f(F\oplus H)\ominus f(F)\ominus L(H)\|_T
}{\|H\|_S}
=0.
$$

Write $Df_F:=L$. For conventional finite-dimensional camera codomains, use
the same definition with their stated Euclidean norm.

This definition is built from native ADD, additive opposite, real scaling, and
coefficient size. It does not require a complex or cone camera.

### D-DER-8 -- Native curve derivative [Definition]

For a curve $F:I\subseteq\mathbb{R}\to\mathcal N_S$, define

$$
\dot F(t)
:=
\lim_{h\to0}
\frac1h\odot\big(F(t+h)\ominus F(t)\big),
$$

when the limit exists in $\|\cdot\|_S$.

## 6. Required derivative laws

The following are not assumed; they are proved in
`../proofs/03-native-derivatives.md`:

- real vector-space and norm laws for every finite stratum;
- ADD derivative;
- MULTIPLY product rule;
- ORIENT and INDEX derivatives;
- native power and inverse rules;
- derivatives and rank statements for every differentiable camera;
- the finite assigned-generator evaluation derivative.

## 7. Scope and information boundary

The derivative is local to a chosen finite support stratum. A trajectory whose
support changes by adding a previously absent mode is not differentiable across
that event under one fixed stratum unless a larger finite stratum was declared
in advance. An unbounded or infinitely branching support requires an analytic
completion that this document does not define.

This limitation is intentional: “mode birth” may be a discrete selection event
rather than an infinitesimal change, and the theory must not hide that
difference inside notation.
