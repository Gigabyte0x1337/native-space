<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# R-PYT-1: Euclid's Pythagorean Identity through the Cone Camera

**Status:** Reproduced  
**Known result:** forward Euclid parameterization of Pythagorean triples  
**Dependencies:** A-RF-1, D-OS-3, D-OS-5, L-OS-9, C-PSD-1,
C-CONE-1, T-CONE-1, T-CONE-2  
**Native invariant:** the squared size of a native square is the square of the
original size

## Conventional statement

For all $x,y\in\mathbb{R}$, define

$$
X=x^2-y^2,
\qquad
Y=2xy,
\qquad
Z=x^2+y^2.
$$

Then

$$
X^2+Y^2=Z^2,
\qquad Z\geq0.
$$

For integer $x,y$, this produces an integer Pythagorean triple. This forward
statement alone is not the classification of all primitive integer triples.

## Native construction

Let $z=(x,y)$ be an oriented scalar. Native squaring gives

$$
z\boxtimes z=(x^2-y^2,2xy)=(X,Y),
$$

while its squared size is

$$
\nu(z)=x^2+y^2=Z.
$$

Consequently the cone camera can be written operations-first as

$$
Q(z)=\bigl(\mathrm{first}(z\boxtimes z),
           \mathrm{second}(z\boxtimes z),
           \nu(z)\bigr).
$$

Apply L-OS-9 to $z\boxtimes z$:

$$
\nu(z\boxtimes z)=\nu(z)\nu(z)=\nu(z)^2.
$$

The left side is $X^2+Y^2$, and the right side is $Z^2$. This proves the
cone equation. The nonnegativity $Z\geq0$ follows from the real-field square
order used in L-OS-7.

## PSD factorization and information loss

C-PSD-1 first forms the rank-one positive-semidefinite matrix

$$
H(z)=
\begin{pmatrix}
x^2&xy\\
xy&y^2
\end{pmatrix}.
$$

The linear coordinate camera $\Lambda$ then gives

$$
\Lambda(H(z))=(x^2-y^2,2xy,x^2+y^2)=Q(z).
$$

Thus the Pythagorean cone is not the native carrier: it is the linear image of
a quadratic PSD camera. T-CONE-1 proves that every real point on the upper
cone is reached, while both $z$ and $\boxminus z$ produce the same point.
The camera therefore loses global sign and is two-to-one away from the apex.

## Back-translation

Substituting D-OS-3 and D-OS-5 into
$\nu(z\boxtimes z)=\nu(z)^2$ gives

$$
(x^2-y^2)^2+(2xy)^2=(x^2+y^2)^2,
$$

which is the conventional forward Euclid identity.

## Comparison

**Classification:** invariant-revealing and equivalent in total algebraic
content.

The native derivation identifies the first two cone coordinates as one native
square and the third as its pre-square size. That makes the cone equation an
instance of one already reusable invariant, L-OS-9. A conventional complex-
number proof makes the same observation through $|z^2|=|z|^2$, so this does
not establish novelty or a shorter foundation. It does clarify the project's
camera order:

$$
\text{oriented scalar}
\longrightarrow
\text{quadratic rank-one PSD state}
\longrightarrow
\text{Pythagorean cone coordinates}.
$$

## Executable cross-check

[`../language/runtime/tests/reconstructions.rs`](../language/runtime/tests/reconstructions.rs) checks
the cone equation on 200 deterministic exact-rational oriented scalars. This
checks implementation agreement, not the proof.

## What is not established

- This does not classify primitive integer Pythagorean triples or their unique
  parameters.
- It does not recover the sign discarded by the PSD/cone camera.
- It does not show that cone coordinates are computationally cheaper than the
  flat native coefficient.
