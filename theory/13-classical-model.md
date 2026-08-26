<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Classical Model of the Four Native Operations

This is the default soundness perspective for the finite Native Space algebra.
It does not replace the native definitions. It gives them one exact classical
model in which all four operations can be checked simultaneously.

Fix the primitive INDEX set $\mathcal A$. Let

$$
M_{\mathcal A}:=\mathbb{N}_0^{(\mathcal A)}
$$

be the free commutative monoid of finite-support exponent maps, and let

$$
\mathbb{C}[M_{\mathcal A}]
$$

be its finite monoid algebra. Its elements are finite formal polynomials

$$
\sum_{\alpha\in M_{\mathcal A}}c_\alpha X^\alpha,
\qquad c_\alpha\in\mathbb{C}.
$$

This is a formal finite algebra. No numerical evaluation of the variables
$X_k$ is implied.

### C-CLASSICAL-MODEL-1 -- unified finite model [Definition]

Using the oriented-scalar field isomorphism $\kappa:\mathbb{O}\to\mathbb{C}$,
define

$$
\Phi(F):=
\sum_{\alpha\in\mathrm{supp}(F)}
\kappa(F_\alpha)X^\alpha.
$$

Interpret the four native operations classically by

$$
\begin{aligned}
\mathrm{ADD}&\longmapsto\text{polynomial addition},\\
\mathrm{MULTIPLY}&\longmapsto\text{monoid-algebra multiplication},\\
\mathrm{ORIENT}_r&\longmapsto\text{coefficient multiplication by }i^r,\\
\mathrm{INDEX}_k&\longmapsto\text{multiplication by }X_k.
\end{aligned}
$$

The proof that these mappings commute exactly and that $\Phi$ is bijective
is T-CLASSICAL-MODEL-1.

## Prime and perspective interpretation

The intrinsic native prime $\mathsf X_k$ maps to the indeterminate $X_k$.
Its prime birth metadata remain separate:

$$
q(p(k))=k,
\qquad
O_k=\mathbf J^k\longmapsto i^k.
$$

The native observer view maps to one common coefficient rotation:

$$
\Phi(\mathrm{View}_r F)=i^{-r}\Phi(F).
$$

Thus the classical model verifies T-PERSPECTIVE-ZERO-1 and
T-ALL-PERSPECTIVES-1 without changing their meaning. Applying different
rotations at different INDEX locations remains a separate diagonal transform,
not one observer perspective.

## Soundness boundary

The model proves relative consistency of the finite native equations: if the
classical monoid algebra is consistent, the isomorphic finite native algebra
cannot derive a contradictory equality.

It does not prove that every proposed camera preserves every zero fiber. In
particular, scalar aggregation is a lossy map out of the monoid algebra, and a
zeta zero does not become coefficientwise zero merely because the finite core
has a faithful classical model.

The strict A-IND-1 construction in `12-foundational-purity.md` remains an
optional deeper reduction of the classical arithmetic trust base.
