<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# R-GEO-1: Finite Geometric Series as Index Telescoping

**Status:** Reproduced  
**Known result:** finite geometric-series identity  
**Dependencies:** D-NS-3, D-NS-9, D-NS-10, L-SEP-2, T-NS-1,
T-EVAL-1, T-OS-1, T-CX-2  
**Native invariant:** one INDEX step shifts every directional depth by one

## Conventional statement

For a commutative-ring element $x$ and $n\in\mathbb{N}_0$,

$$
(1-x)\sum_{j=0}^{n-1}x^j=1-x^n,
$$

where the empty sum for $n=0$ is zero. If $1-x$ is invertible, this yields

$$
\sum_{j=0}^{n-1}x^j=\frac{1-x^n}{1-x}.
$$

## Native construction

Fix primitive direction $k$ and write its generator state as

$$
X:=\mathsf X_k=\mathbf1[\varepsilon_k].
$$

Define native powers recursively by

$$
X^{\star0}:=\mathsf1,
\qquad
X^{\star(j+1)}:=X\star X^{\star j}.
$$

The base $j=0$ is $\mathsf1=\mathbf1[\mathbf0_I]$. If the formula holds
at depth $j$, D-NS-10 identifies multiplication by $X$ with
$\mathrm{INDEX}_k$, and L-SEP-2 increments only depth $k$. Induction
therefore gives

$$
X^{\star j}=\mathbf1[j\varepsilon_k].
$$

Thus the power does not confuse the primitive identity $k$ with its depth
$j$: $k$ selects the direction and $j$ counts its repeated use.

Let

$$
G_n:=\mathop{\bigoplus}_{j=0}^{n-1}X^{\star j}.
$$

By distributivity and the power recursion,

$$
X\star G_n
=\mathop{\bigoplus}_{j=0}^{n-1}X^{\star(j+1)}
=\mathop{\bigoplus}_{j=1}^{n}X^{\star j}.
$$

The ring laws also imply
$(\ominus X)\star G_n=\ominus(X\star G_n)$: distribute
$(X\oplus(\ominus X))\star G_n$, use zero absorption, and use uniqueness of
the additive inverse.

Therefore

$$
\begin{aligned}
(\mathsf1\oplus(\ominus X))\star G_n
&=G_n\oplus\bigl(\ominus(X\star G_n)\bigr)\\
&=\left(\mathsf1\oplus X\oplus\cdots\oplus X^{\star(n-1)}\right)\\
&\quad\oplus
\left(\ominus X\oplus\cdots\oplus\ominus X^{\star n}\right)\\
&=\mathsf1\oplus(\ominus X^{\star n}).
\end{aligned}
$$

Every interior directional depth cancels by the native ADD group law. For
$n=0$, both sides are $\mathsf0$, so the identity includes the empty case.

## Back-translation

Choose an oriented scalar $z$ for direction $k$ in the evaluation camera
of C-EVAL-1, with other directions irrelevant to this one-generator state.
T-EVAL-1 preserves ADD, MULTIPLY, zero, and one, so evaluation yields

$$
(\mathbf1\boxplus(\boxminus z))
\boxtimes
\mathop{\boxplus}_{j=0}^{n-1}z^{\boxtimes j}
\mathrel{=}
\mathbf1\boxplus(\boxminus z^{\boxtimes n}).
$$

Through the complex camera T-CX-2 this is exactly

$$
(1-z)\sum_{j=0}^{n-1}z^j=1-z^n.
$$

When $z\neq1$, T-OS-1 supplies the inverse of $1-z$, producing the usual
quotient form. At $z=1$, division is unavailable but the division-free
identity remains valid and the sum evaluates to the scalar $(n,0)$, written
conventionally as $n$.

## Comparison

**Classification:** invariant-revealing and equivalent as a ring proof.

The native view makes telescoping a cancellation between adjacent INDEX
depths. It also keeps primitive identity $k$ separate from exponent/depth
$j$, which is a central project invariant. Conventional polynomial rings
already provide the same structural proof, so no shorter total derivation or
new generality has been established.

## Executable cross-check

[`../language/runtime/tests/reconstructions.rs`](../language/runtime/tests/reconstructions.rs) checks
the division-free identity exactly for lengths zero through nine on primitive
direction three. This is an implementation cross-check, not the proof.

## What is not established

- No infinite geometric series is defined; 1.0 has only finite-support states.
- No convergence claim such as $\sum_{j\ge0}z^j=(1-z)^{-1}$ follows.
- The native state $\mathsf1\oplus(\ominus X)$ is not asserted to be
  invertible in the finite native ring.
- This identity alone gives no compression or runtime advantage.
