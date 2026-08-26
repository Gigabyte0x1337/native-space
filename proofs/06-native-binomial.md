<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Native Binomial Theorem

## Definitions

### D-BIN-1 -- natural coefficients in both carriers [Definition]

For $m\in\mathbb{N}_0$, define

$$
\widehat m:=(m,0)\in\mathbb{O},
\qquad
\underline m:=\eta(\widehat m)\in\mathcal N_{\mathcal A}.
$$

Here $m$ in the real coordinate denotes the canonical repeated-one numeral
of the real field.

Coordinate ADD proves by induction that $\widehat m$ is the scalar ADD-fold
of $m$ copies of $\mathbf1$, and $\underline m$ is the native ADD-fold
of $m$ copies of $\mathsf1$. In particular,
$\underline0=\mathsf0$.

### D-BIN-2 -- binomial coefficients [Definition]

Define natural binomial coefficients recursively by

$$
{0\choose0}=1,
$$

with ${n\choose j}=0$ outside $0\leq j\leq n$, and Pascal recursion

$$
{n+1\choose j}={n\choose j}+{n\choose j-1}.
$$

For a native state $F$, define $F^{\star0}=\mathsf1$ and
$F^{\star(n+1)}=F\star F^{\star n}$.
For an oriented scalar $z$, define $z^{\boxtimes0}=\mathbf1$ and
$z^{\boxtimes(n+1)}=z\boxtimes z^{\boxtimes n}$.

### L-BIN-1 -- native binomial theorem [Proved]

**Claim.** For all $F,G\in\mathcal N_{\mathcal A}$ and
$n\in\mathbb{N}_0$,

$$
(F\oplus G)^{\star n}
=
\mathop{\bigoplus}_{j=0}^{n}
\underline{{n\choose j}}
\star F^{\star(n-j)}
\star G^{\star j}.
$$

For all $z,w\in\mathbb{O}$, the corresponding scalar identity is

$$
(z\boxplus w)^{\boxtimes n}
=
\mathop{\boxplus}_{j=0}^{n}
\widehat{{n\choose j}}
\boxtimes z^{\boxtimes(n-j)}
\boxtimes w^{\boxtimes j}.
$$

**Dependencies.** A-N-1, D-BIN-1, D-BIN-2, T-OS-1, T-NS-1.

**Proof.** For $n=0$, the left side is $\mathsf1$. The sum on the right
has one term,

$$
\underline1\star\mathsf1\star\mathsf1=\mathsf1.
$$

Assume the formula at $n$. By the power recursion and distributivity,

$$
\begin{aligned}
(F\oplus G)^{\star(n+1)}
&=(F\oplus G)\star(F\oplus G)^{\star n}\\
&=\mathop{\bigoplus}_{j=0}^{n}
  \underline{{n\choose j}}
  \star F^{\star(n+1-j)}\star G^{\star j}\\
&\quad\oplus
  \mathop{\bigoplus}_{j=0}^{n}
  \underline{{n\choose j}}
  \star F^{\star(n-j)}\star G^{\star(j+1)}.
\end{aligned}
$$

Reindex the second finite sum by $k=j+1$. For each exponent split
$0\leq k\leq n+1$, the common state monomial
$F^{\star(n+1-k)}\star G^{\star k}$ receives coefficient

$$
\underline{{n\choose k}}\oplus
\underline{{n\choose k-1}}
=
\underline{{n+1\choose k}}
$$

by coordinate ADD and D-BIN-2. Associativity and commutativity permit the
finite regrouping. This is the claimed formula at $n+1$, completing the
induction. The same induction with $\boxplus,\boxtimes,\mathbf0,\mathbf1$
and $\widehat m$ proves the oriented-scalar identity under T-OS-1.
$\square$

## Boundary

The proof uses only finite sums and natural coefficients. It does not define a
binomial series, fractional exponent, or convergence rule.
