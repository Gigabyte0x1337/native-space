<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Finite Mellin-Type Character Proofs

## Dependencies

These proofs use A-RF-1, the index monoid, the native flow laws, T-EVAL-1 and
T-EVAL-2, and the definitions in
`../theory/06-finite-mellin-character-camera.md`. Every sum and product is
finite.

### L-MELLIN-1 -- weighted depth is additive [Proved]

**Claim.** For all multi-indices $\alpha,\beta$,

$$
U_u(\alpha\oplus_I\beta)
=U_u(\alpha)+U_u(\beta).
$$

**Proof.** L-IDX-2 and D-IDX-6 make every directional depth add. Substitute
that coordinatewise rule into the finite sum D-MELLIN-1 and distribute real
addition. $\square$

### L-MELLIN-2 -- flow character is multiplicative [Proved]

**Claim.** For all multi-indices $\alpha,\beta$,

$$
\chi_{u,s}(\alpha\oplus_I\beta)
=
\chi_{u,s}(\alpha)\boxtimes\chi_{u,s}(\beta).
$$

Also,

$$
\chi_{u,s}(\alpha)
=
\mathop{\boxtimes}_{a_k\in\mathrm{supp}(\alpha)}
z_k(u,s)^{\boxtimes d_k(\alpha)}.
$$

**Proof.** L-MELLIN-1 makes the flow time on the left the sum
$-U_u(\alpha)-U_u(\beta)$. L-FLOW-1 turns time addition into native
MULTIPLY, proving the first formula.

For the second, L-FLOW-1 and L-FLOW-2 turn each directional power into
$\mathcal E_s(-u(a_k)d_k(\alpha))$. Multiplying the finite factors adds
their flow times and gives $\mathcal E_s(-U_u(\alpha))$. $\square$

### T-MELLIN-1 -- finite character camera is a ring homomorphism [Proved]

**Claim.** For finite native states $F,G$,

$$
\mathcal M_{u,s}(F\oplus G)
=\mathcal M_{u,s}(F)\boxplus\mathcal M_{u,s}(G),
$$

$$
\mathcal M_{u,s}(F\star G)
=\mathcal M_{u,s}(F)\boxtimes\mathcal M_{u,s}(G),
$$

and the camera preserves zero and one.

**Proof.** L-MELLIN-2 identifies C-MELLIN-1 exactly with C-EVAL-1 under the
assignment $a_k\mapsto z_k(u,s)$. T-EVAL-1 proves all four preservation
laws. $\square$

### T-MELLIN-2 -- coordinate form and information loss [Proved]

**Claim.** If $\kappa(s)=\sigma+i\tau$, then

$$
\kappa(\chi_{u,s}(\alpha))
=
e^{-(\sigma+i\tau)U_u(\alpha)}.
$$

The camera $\mathcal M_{u,s}$ is generally noninjective.

**Proof.** L-FLOW-2 writes

$$
\chi_{u,s}(\alpha)
=\mathcal E_{-U_u(\alpha)s}(1).
$$

T-FLOW-2, with generator coordinates
$(-U_u(\alpha)\sigma,-U_u(\alpha)\tau)$, gives the displayed conventional
complex exponential after T-CX-2.

For noninjectivity, T-EVAL-2 shows that the primitive state $\mathsf X_k$
and the embedded assigned scalar $\eta(z_k(u,s))$ are distinct native
states with the same evaluation. Flow values are nonzero by T-FLOW-1, but the
collision does not depend on zero. $\square$

## What these proofs do not establish

- A finite character sum is not an infinite Mellin transform or Dirichlet
  series.
- This finite theorem set supplies no convergence or analytic-continuation
  result. The later completed camera proves absolute zeta convergence on
  $\mathrm{Re}(s)>1$, without continuation.
- No prime semantics has entered the generic index labels.
- The scalar output cannot generally reconstruct the formal indexed input.
