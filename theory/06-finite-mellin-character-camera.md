<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Finite Mellin-Type Character Camera

This Stage 6 camera extracts the finite algebraic mechanism behind
Mellin/Dirichlet character evaluation without defining an infinite series or
integral transform. It uses generic primitive directions and contains no prime
assignment.

Fix a real additive weight map

$$
u:\mathcal A\to\mathbb{R}.
$$

The map may have infinite support because every native multi-index has finite
support, so every weighted depth sum below is finite.

## Weighted depth

### D-MELLIN-1 -- additive depth weight [Definition]

For $\alpha\in\mathbb{I}_{\mathcal A}$, define

$$
U_u(\alpha)
:=
\sum_{a_k\in\mathrm{supp}(\alpha)}
u(a_k)d_k(\alpha).
$$

This records a real weight of depth. It does not replace primitive identity
$k$ or directional depth $d_k$.

## Flow character

### D-MELLIN-2 -- native exponential character [Definition]

For parameter $s\in\mathbb{O}$, define

$$
\chi_{u,s}(\alpha)
:=
\mathcal E_s\bigl(-U_u(\alpha)\bigr).
$$

Equivalently, assign each primitive generator the nonzero scalar

$$
z_k(u,s):=\mathcal E_s(-u(a_k))
$$

and multiply its directional powers. The equivalence is a proof obligation,
not part of the definition.

## Finite Mellin-type camera

### C-MELLIN-1 -- weighted character evaluation [Definition]

Define

$$
\mathcal M_{u,s}:\mathcal N_{\mathcal A}\to\mathbb{O}
$$

by the finite sum

$$
\mathcal M_{u,s}(F)
:=
\mathop{\boxplus}_{\alpha\in\mathrm{supp}(F)}
F_\alpha\boxtimes\chi_{u,s}(\alpha).
$$

This is C-EVAL-1 with the flow-derived assignment $z_k(u,s)$. The camera is
total on finite native states, has no convergence condition, and is generally
noninjective because it collapses formal index structure to one scalar.

## Conventional coordinate candidate

If $\kappa(s)=\sigma+i\tau$, T-FLOW-2 suggests

$$
\kappa(\chi_{u,s}(\alpha))
\mathrel{=}
\exp\bigl(- (\sigma+i\tau)U_u(\alpha)\bigr).
$$

If positive bases are introduced by $b_k=e^{u(a_k)}$, this becomes

$$
\left(\prod_k b_k^{d_k(\alpha)}\right)^{-(\sigma+i\tau)}.
$$

The first formula is the primary coordinate statement; the second uses
positive real bases and their conventional logarithmic parameterization.

## Explicit exclusions

- No primitive direction is assigned a prime value in Stage 6.
- No infinite Dirichlet series, Mellin integral, Euler product, analytic
  continuation, or convergence half-plane is defined.
- The camera is not injective and cannot recover the formal state in general.
- No transform algorithm or complexity advantage is claimed.

The character and homomorphism properties are proved in
`../proofs/08-finite-mellin-character.md`.
