<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Unified Classical-Model Proof

### T-CLASSICAL-MODEL-1 -- all four operations have one exact classical model [Proved]

**Claim.** C-CLASSICAL-MODEL-1 is a bijection

$$
\Phi:\mathcal N_{\mathcal A}\longrightarrow\mathbb{C}[M_{\mathcal A}]
$$

and for all finite native states $F,G$, orientations $r$, and primitive
directions $k$,

$$
\begin{aligned}
\Phi(F\oplus G)&=\Phi(F)+\Phi(G),\\
\Phi(F\star G)&=\Phi(F)\Phi(G),\\
\Phi(\mathrm{ORIENT}_rF)&=i^r\Phi(F),\\
\Phi(\mathrm{INDEX}_kF)&=X_k\Phi(F).
\end{aligned}
$$

It maps native zero and one to polynomial zero and one.

**Proof.** A finite native state and a finite monoid-algebra polynomial have
the same support type $M_{\mathcal A}$. T-CX-2 makes $\kappa$ a bijection
on every coefficient. Applying $\kappa$ coefficientwise therefore gives a
bijection $\Phi$, with inverse obtained by applying $\kappa^{-1}$ to every
nonzero polynomial coefficient.

Native ADD combines coefficients at equal multi-indices. Since $\kappa$
preserves oriented ADD, this is exactly polynomial coefficient addition.

For MULTIPLY, the coefficient of $X^\gamma$ in the classical product is

$$
\sum_{\alpha+\beta=\gamma}
\kappa(F_\alpha)\kappa(G_\beta).
$$

The native convolution coefficient at $\gamma$ is

$$
\mathop{\boxplus}_{\alpha+\beta=\gamma}
F_\alpha\boxtimes G_\beta.
$$

T-CX-2 preserves both coefficient operations, so the two coefficients agree.
All sums are finite.

ORIENT multiplies every native coefficient by
$\mathbf J^{\boxtimes r}$. T-CX-2 maps this unit to $i^r$, proving the
third identity. INDEX adds $\varepsilon_k$ to every native multi-index;
classical multiplication by $X_k$ performs the identical exponent shift,
proving the fourth identity. Empty support and the zero multi-index give zero
and one. $\square$

### T-CLASSICAL-PERSPECTIVE-1 -- the observer theorem commutes with the model [Proved]

**Claim.** For every native state $F$ and observer orientation $r$,

$$
\Phi(\mathrm{View}_rF)=i^{-r}\Phi(F).
$$

Therefore $\mathrm{View}_rF=0$ exactly when $\Phi(F)=0$.

**Proof.** D-REL-ORIENT-1 defines the view as multiplication by
$O_r^{-1}$. T-CLASSICAL-MODEL-1 maps that common coefficient operation to
multiplication by $i^{-r}$. The factor is nonzero, and $\Phi$ is
bijective, giving both zero implications. $\square$

### T-CLASSICAL-RELATIVE-SOUND-1 -- finite native equations are model-sound [Proved]

**Claim.** An equality between finite native expressions built from the four
operations holds exactly when the corresponding formal-polynomial equality
holds under C-CLASSICAL-MODEL-1.

**Proof.** Structural induction on an expression. Values use coefficientwise
bijectivity. Each of the four inductive constructors commutes with $\Phi$ by
T-CLASSICAL-MODEL-1. Applying $\Phi^{-1}$ proves reflection of equality.
$\square$

**Boundary.** This is a relative soundness and completeness theorem for finite
equations. It does not validate infinite limits, analytic continuation, lossy
aggregation inverses, or K-RH-1.
