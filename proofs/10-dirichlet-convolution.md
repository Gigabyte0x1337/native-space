<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Finite Arithmetic Functions and Dirichlet Convolution Proofs

## Dependencies

These proofs use the oriented-scalar and native-state ring laws, T-ENC-1, and
the definitions in `../theory/08-finite-arithmetic-functions.md`. Every support,
sum, and product in the carrier is finite.

### L-DIR-1 -- finite-support closure [Proved]

**Claim.** If $f,g\in\mathcal F_{\mathrm{fin}}$, then
$f\boxplus_{\!A}g$, $\boxminus_{\!A}f$, and $f\star_D g$ have finite
support.

**Proof.** Pointwise ADD and negation have support contained in the finite
union of the input supports. Put

$$
S:=\{ab:a\in\mathrm{supp}(f),\ b\in\mathrm{supp}(g)\}.
$$

This is a finite image of a finite Cartesian product. If
$(f\star_D g)(n)\neq\mathbf0$, at least one term in its finite divisor sum is
nonzero, so some divisor $d$ lies in $\mathrm{supp}(f)$ and
$n/d$ lies in $\mathrm{supp}(g)$. Hence $n\in S$, proving finite
support. $\square$

### T-ARITH-1 -- the coefficient lift is an additive bijection [Proved]

**Claim.** C-ARITH-1 is a bijection and satisfies

$$
\Phi(0_A)=\mathsf0,
\qquad
\Phi(f\boxplus_{\!A}g)=\Phi(f)\oplus\Phi(g),
\qquad
\Phi(\boxminus_{\!A}f)=\boxminus\Phi(f).
$$

It also sends $\delta_1$ to $\mathsf1$.

**Proof.** T-ENC-1 gives a bijection $n\leftrightarrow\alpha_n$ between
positive integers and finite prime multi-indices. C-ARITH-1 therefore merely
relabels each nonzero coefficient from $n$ to $\alpha_n$. Finite support
is preserved in both directions, proving bijectivity.

D-NS-6 defines native ADD coefficientwise at each multi-index. Its coefficient
at $\alpha_n$ is therefore $f(n)\boxplus g(n)$, exactly the coefficient
of $\Phi(f\boxplus_{\!A}g)$. The zero and negation identities follow by the
same
coefficientwise argument. Finally, T-ENC-1 sends $1$ to the zero
multi-index, so the sole coefficient of $\Phi(\delta_1)$ is $\mathbf1$
at that index: this is $\mathsf1$. $\square$

### T-DIR-1 -- Dirichlet convolution is native MULTIPLY [Proved]

**Claim.** For all $f,g\in\mathcal F_{\mathrm{fin}}$,

$$
\Phi(f\star_D g)=\Phi(f)\star\Phi(g).
$$

Consequently,

$$
(\mathcal F_{\mathrm{fin}},\boxplus_{\!A},\star_D,0_A,\delta_1)
\cong
(\mathcal N_{\mathcal A_{\mathbb{P}}},\oplus,\star,\mathsf0,\mathsf1)
$$

as commutative rings.

**Proof.** Fix $r\in\mathbb{N}_{>0}$. By D-NS-7, the coefficient of the
native product at $\alpha_r$ is

$$
\mathop{\boxplus}_{\alpha_m\oplus_I\alpha_n=\alpha_r}
f(m)\boxtimes g(n).
$$

T-ENC-1 states
$\alpha_m\oplus_I\alpha_n=\alpha_{mn}$, and its injectivity states
$\alpha_{mn}=\alpha_r$ exactly when $mn=r$. Re-indexing by
$m=d$ and $n=r/d$ changes the displayed coefficient into

$$
\mathop{\boxplus}_{d\mid r}f(d)\boxtimes g(r/d)
=(f\star_D g)(r).
$$

This is the coefficient of $\Phi(f\star_D g)$ at $\alpha_r$. Every native
multi-index equals one $\alpha_r$, so the states are equal. Together with
T-ARITH-1, $\Phi$ is a bijection preserving ADD, MULTIPLY, zero, and one;
therefore it is a commutative-ring isomorphism. $\square$

### T-DIR-CHAR-1 -- finite character evaluation is multiplicative [Proved]

**Claim.** Define

$$
\mathcal D_{u,s}:=\mathcal M_{u,s}\circ\Phi.
$$

Then

$$
\mathcal D_{u,s}(f\boxplus_{\!A}g)
=\mathcal D_{u,s}(f)\boxplus\mathcal D_{u,s}(g),
$$

$$
\mathcal D_{u,s}(f\star_D g)
=\mathcal D_{u,s}(f)\boxtimes\mathcal D_{u,s}(g),
$$

and $\mathcal D_{u,s}$ preserves zero and one.

**Proof.** T-ARITH-1 and T-DIR-1 prove that $\Phi$ preserves the four
named structures. T-MELLIN-1 proves the same for $\mathcal M_{u,s}$.
Their composition therefore preserves them. Expanding C-MELLIN-1 at
$\Phi(f)$ gives the finite character polynomial displayed in the theory
file. $\square$

## What these proofs do not establish

- The ring isomorphism is for finite-support functions only.
- It transfers structure, not an asymptotic complexity improvement.
- Encoding arbitrary conventional integer labels can require factorization.
- T-DIR-CHAR-1 is a finite polynomial identity, not an infinite Dirichlet
  series or Euler-product theorem.
