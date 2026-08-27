<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Formal Completion and Euler Product Proofs

## Dependencies

These proofs use the finite multi-index monoid, oriented-scalar ring laws,
native convolution definitions, and T-ENC-1. They do not use infinite scalar
sums or analytic convergence.

### L-COMP-LOC-1 -- the index monoid is locally finite [Proved]

**Claim.** For every $\gamma\in\mathbb{I}_{\mathcal A}$, the set

$$
\{(\alpha,\beta):\alpha\oplus_I\beta=\gamma\}
$$

is finite, with size

$$
\prod_{a_k\in\mathrm{supp}(\gamma)}(d_k(\gamma)+1).
$$

**Proof.** If $d_k(\gamma)=0$, nonnegativity forces both input depths to
be zero. If $d_k(\gamma)>0$, choosing $d_k(\alpha)$ from
$0,\ldots,d_k(\gamma)$ uniquely forces
$d_k(\beta)=d_k(\gamma)-d_k(\alpha)$. The support of $\gamma$ is finite,
so multiplying these finite independent choice counts gives the formula.
$\square$

### T-COMP-1 -- the formal carrier is a commutative ring [Proved]

**Claim.** D-COMP-2 is well defined and makes

$$
(\widehat{\mathcal N}_{\mathcal A},
\widehat\oplus,\widehat\star,\mathsf0,\mathsf1)
$$

a commutative ring. The finite Native Space carrier embeds as a subring.

**Proof.** L-COMP-LOC-1 makes every output coefficient of completed MULTIPLY
a finite oriented-scalar sum, so the operation is total. ADD is pointwise and
inherits its abelian-group laws.

For a fixed output index, commutativity swaps the two members of every finite
split. Associativity re-indexes both bracketings by the same finite set of
triples $(\alpha,\beta,\delta)$ satisfying
$\alpha\oplus_I\beta\oplus_I\delta=\gamma$. Distributivity expands and
recombines finite oriented-scalar sums. The state supported only at the zero
multi-index with coefficient $\mathbf1$ is the multiplicative identity.
Thus all ring laws follow from the oriented-scalar ring laws.

For finite-support inputs, D-COMP-2 has exactly D-NS-6 and D-NS-7's
coefficients, and its output support remains finite by T-NS-1. Extension by
zero therefore preserves ADD, MULTIPLY, zero, and one and is injective.
$\square$

### T-DIR-INF-1 -- all arithmetic functions are completed native states [Proved]

**Claim.** The completed lift is a commutative-ring isomorphism

$$
(\mathcal F_{\mathrm{all}},\boxplus_{\!A},\star_D,0_A,\delta_1)
\cong
(\widehat{\mathcal N}_{\mathcal A_{\mathbb{P}}},
\widehat\oplus,\widehat\star,\mathsf0,\mathsf1).
$$

**Proof.** T-ENC-1 bijects positive integers with prime multi-indices, so
$\widehat\Phi$ is a coefficient relabeling and therefore a bijection.
Pointwise ADD, zero, and the unit correspond exactly as in T-ARITH-1.

For multiplication, fix $r$. L-COMP-LOC-1 makes the coefficient sum at
$\alpha_r$ finite. T-ENC-1 bijects its index splits with factorizations
$r=d(r/d)$, so the coefficient is

$$
\mathop{\boxplus}_{d\mid r} f(d)\boxtimes g(r/d)
=(f\star_D g)(r).
$$

Hence $\widehat\Phi(f\star_D g)
=\widehat\Phi(f)\widehat\star\widehat\Phi(g)$. T-COMP-1 supplies the ring
laws on the completed side, so the displayed bijection is a ring isomorphism.
$\square$

### L-GEO-INF-1 -- each formal geometric state is an inverse [Proved]

**Claim.** For symbolic positive pattern observation $k$,

$$
(\mathsf1\widehat\boxminus\mathsf X_k)
\widehat\star\mathsf G_k
=\mathsf1.
$$

**Proof.** At the zero multi-index, only
$\mathsf1\widehat\star\mathsf G_k$ contributes, with coefficient
$\mathbf1$. At $d\varepsilon_k$ for $d\geq1$, the coefficient is
$\mathbf1\boxminus\mathbf1=\mathbf0$, pairing depth $d$ from the first
term with depth $d-1$ shifted by $\mathsf X_k$. At every index involving
another direction, both contributions are zero. Equality is coefficientwise.
$\square$

### T-EULER-F-1 -- coefficientwise formal Euler factorization [Proved]

**Claim.** The finite products $\mathfrak Z_K$ stabilize coefficientwise and

$$
\mathfrak Z_\mathrm{pattern}
\mathrel{=}
\prod_{k\geq1}^{\mathrm{coeff}}\mathsf G_k
\mathrel{=}
\prod_{k\geq1}^{\mathrm{coeff}}
(\mathsf1\widehat\boxminus\mathsf X_k)^{-1}.
$$

**Proof.** Fix a finite prime multi-index $\alpha$. If
$\mathrm{supp}(\alpha)\subseteq\{a_1,\ldots,a_K\}$, there is exactly
one way to select from each factor $\mathsf G_k$ the depth required by
$\alpha$; all unused factors contribute depth zero. Its coefficient in
$\mathfrak Z_K$ is therefore $\mathbf1$. If $\alpha$ uses a direction
beyond $K$, its coefficient is $\mathbf0$.

Because $\alpha$ has finite support, its coefficient is permanently
$\mathbf1$ once $K$ reaches its largest direction. Thus every coefficient
stabilizes to D-ZETA-PATTERN-1's value. L-GEO-INF-1 identifies each
$\mathsf G_k$ as the displayed formal inverse, proving both products.
$\square$

## What these proofs do not establish

- No infinite scalar sum or product was evaluated.
- The result is coefficientwise formal algebra, not complex convergence.
- Applying a Mellin/Dirichlet camera to $\mathfrak Z_\mathrm{pattern}$ remains
  undefined until the analytic obligations in the theory file are discharged.
- No statement about the zeros or continuation of a scalar zeta function
  follows.
