<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Proofs of the Native Space Camera Classifications

**Proof status:** Complete paper proofs for the non-derivative Stage 4 camera obligations  
**Research stage:** 4 -- Prove each mapping and equivalence  
**Machine-checked:** No

## 1. Dependency boundary

These proofs may use the completed core theorems from
`01-core-algebra.md` and standard facts in the conventional codomains. A
conventional fact used to classify a camera does not become a premise of the
native core.

## 2. Complex-coordinate camera

### T-CX-1 -- Complex camera bijection [Proved]

**Claim.** The map

$$
\kappa(x,y)=x+i y
$$

is a bijection $\mathbb{O}\to\mathbb{C}$ with inverse
$\kappa^{-1}(x+i y)=(x,y)$.

**Proof.** Every conventional complex number has a unique real part $x$ and
imaginary part $y$, so the stated inverse is defined on all of $\mathbb{C}$.
For every $(x,y)\in\mathbb{O}$,

$$
\kappa^{-1}(\kappa(x,y))=(x,y).
$$

For every $x+i y\in\mathbb{C}$,

$$
\kappa(\kappa^{-1}(x+i y))=x+i y.
$$

The two-sided inverse proves bijectivity. $\square$

### T-CX-2 -- Complex camera field isomorphism [Proved]

**Claim.** $\kappa$ preserves ADD, MULTIPLY, zero, one, additive opposites,
and nonzero inverses.

**Proof.** Let $z=(x,y)$ and $w=(u,v)$. Then

$$
\begin{aligned}
\kappa(z\boxplus w)
&=\kappa(x+u,y+v)\\
&=(x+u)+i(y+v)\\
&=(x+i y)+(u+i v)\\
&=\kappa(z)+\kappa(w).
\end{aligned}
$$

For multiplication,

$$
\begin{aligned}
\kappa(z\boxtimes w)
&=(xu-yv)+i(xv+yu)\\
&=(x+i y)(u+i v)\\
&=\kappa(z)\kappa(w),
\end{aligned}
$$

using conventional $i^2=-1$ only in the target field. Directly,

$$
\kappa(\mathbf0)=0,
\qquad
\kappa(\mathbf1)=1,
\qquad
\kappa(\boxminus z)=-\kappa(z).
$$

For nonzero $z$, T-OS-1 and multiplication preservation give

$$
\kappa(z)\kappa(z^{[-1]})
=\kappa(z\boxtimes z^{[-1]})
=\kappa(\mathbf1)=1.
$$

Since $\kappa(z)\neq0$ by T-CX-1, uniqueness of inverse in $\mathbb{C}$
gives $\kappa(z^{[-1]})=\kappa(z)^{-1}$. Together with T-CX-1, these facts
make $\kappa$ a field isomorphism. $\square$

**Interpretation boundary.** This proves that the scalar core reproduces the
conventional complex field. It does not prove the larger indexed native ring is
novel or simpler.

## 3. Structured real-operator camera

### T-MAT-1 -- Matrix camera linear bijection [Proved]

**Claim.** $\mu$ is a real-linear bijection from $\mathbb{O}$ onto
$\mathcal C_2$.

**Proof.** By definition, every matrix in $\mathcal C_2$ has the form

$$
\begin{pmatrix}x&-y\\y&x\end{pmatrix}
$$

for real $x,y$, so it is $\mu(x,y)$. Thus $\mu$ is surjective onto its
declared image. Equality of two such matrices forces equality of their
$(1,1)$ and $(2,1)$ entries, hence equality of both source coordinates;
therefore $\mu$ is injective. The stated inverse reads those two entries.

For real $a,b$, coordinate expansion gives

$$
\mu(a z\boxplus b w)=a\mu(z)+b\mu(w),
$$

where real scalar multiplication on $\mathbb{O}$ means multiplying each
coordinate. Hence the bijection is real-linear. $\square$

### T-MAT-2 -- Matrix camera algebra and action laws [Proved]

**Claim.** $\mu$ preserves ADD and MULTIPLY, and

$$
\mathrm{col}(z\boxtimes w)=\mu(z)\mathrm{col}(w).
$$

**Proof.** ADD preservation follows entrywise from the matrix definition. For
MULTIPLY,

$$
\begin{aligned}
\mu(x,y)\mu(u,v)
&=
\begin{pmatrix}x&-y\\y&x\end{pmatrix}
\begin{pmatrix}u&-v\\v&u\end{pmatrix}\\
&=
\begin{pmatrix}
xu-yv&-(xv+yu)\\
xv+yu&xu-yv
\end{pmatrix}\\
&=\mu(xu-yv,xv+yu)\\
&=\mu(z\boxtimes w).
\end{aligned}
$$

Likewise,

$$
\mu(z)\mathrm{col}(w)
\mathrel{=}
\begin{pmatrix}xu-yv\\yu+xv\end{pmatrix}
=operatorname{col}(z\boxtimes w).
$$

Together with T-MAT-1, multiplication preservation makes $\mu$ an algebra
isomorphism onto $\mathcal C_2$, and the last identity realizes native
multiplicative self-action as a real linear operator. $\square$

## 4. Unwrapped scale-orientation camera

This section uses standard conventional facts about $\exp$, $\log$,
$\sin$, and $\cos$: positivity and injectivity of $\exp$, the identity
$\sin^2+\cos^2=1$, angle existence on the unit circle, periodicity by
$2\pi$, and the angle-addition formulas.

### T-POLAR-1 -- Polar camera covering and fibers [Proved]

**Claim.** $W:\mathbb{R}^2\to\mathbb{O}^\times$ is surjective, and every
fiber is exactly

$$
W^{-1}(z)
\mathrel{=}
\left\{
\left(\tfrac{1}{2}\log\nu(z),\theta_z+2\pi n\right):n\in\mathbb{Z}
\right\}.
$$

**Proof.** Let $z=(x,y)\neq\mathbf0$. L-OS-7 gives $\nu(z)>0$. Set

$$
\rho_z:=\tfrac{1}{2}\log\nu(z).
$$

Then $e^{\rho_z}=\sqrt{\nu(z)}$. The pair

$$
\frac{(x,y)}{\sqrt{\nu(z)}}
$$

lies on the conventional unit circle, so an angle $\theta_z$ exists with the
required sine and cosine. Substitution gives $W(\rho_z,\theta_z)=z$, proving
surjectivity.

Now suppose $W(\rho,\theta)=W(\sigma,\phi)$. Taking $\nu$ of both sides
using the displayed coordinates and $\sin^2+\cos^2=1$ gives

$$
e^{2\rho}=e^{2\sigma}.
$$

Injectivity of $\exp$ gives $\rho=\sigma$. Dividing the equal coordinate
pairs by the common positive radius gives

$$
(\cos\theta,\sin\theta)=(\cos\phi,\sin\phi).
$$

The conventional unit-circle parameterization has exactly the fibers
$\theta=\phi+2\pi n$, $n\in\mathbb{Z}$. Conversely, every such integer
shift leaves sine and cosine unchanged. This proves the complete fiber
description. $\square$

### T-POLAR-2 -- Quotient bijection and flat multiplication [Proved]

**Claim.** $\overline W$ is a bijection

$$
\mathbb{R}\times(\mathbb{R}/2\pi\mathbb{Z})
\longrightarrow
\mathbb{O}^\times,
$$

and wrapped MULTIPLY corresponds to coordinate addition.

**Proof.** T-POLAR-1 says two points of $\mathbb{R}^2$ have the same image
exactly when their first coordinates agree and their second coordinates differ
by an integer multiple of $2\pi$. Those are exactly the equivalence classes
in the quotient cylinder. Therefore the induced map is both injective and
surjective.

For the operation law, use D-OS-3 and the conventional angle-addition formulas:

$$
\begin{aligned}
&W(\rho,\theta)\boxtimes W(\sigma,\phi)\\
&=e^{\rho+\sigma}
\big(
\cos\theta\cos\phi-\sin\theta\sin\phi,
\cos\theta\sin\phi+\sin\theta\cos\phi
\big)\\
&=e^{\rho+\sigma}(\cos(\theta+\phi),\sin(\theta+\phi))\\
&=W(\rho+\sigma,\theta+\phi).
\end{aligned}
$$

Thus nonzero MULTIPLY is addition on the quotient coordinates. Zero remains
excluded because $e^\rho>0$ for every finite $\rho$. $\square$

**Interpretation boundary.** This theorem linearizes nonzero MULTIPLY only. It
does not linearize ADD, remove the zero singularity, or retain unwrapped turn
count after applying $W$.

## 5. PSD camera

### T-PSD-1 -- PSD image and fibers [Proved]

**Claim.** The image of $H(z)=v_zv_z^T$ is exactly
$\mathcal P_1$, with fiber $\{\mathbf0\}$ over zero and
$\{z,\boxminus z\}$ over every nonzero image.

**Proof.** For any real column $v$ and any real column $r$,

$$
r^T(vv^T)r=(v^Tr)^2\geq0,
$$

so $vv^T$ is positive semidefinite. Its image is contained in the span of
$v$, so its rank is at most one. Thus $H(\mathbb{O})\subseteq\mathcal P_1$.

Conversely, let $A\in\mathcal P_1$. If $A=0$, then
$A=H(\mathbf0)$. If $A\neq0$, the real symmetric PSD matrix has one
positive eigenvalue $\lambda$ and a unit eigenvector $u$, with

$$
A=\lambda uu^T.
$$

Setting $v=\sqrt\lambda\,u$ gives $A=vv^T=H(v)$. Hence the image is all
of $\mathcal P_1$.

If $vv^T=0$, then its trace is $v^Tv=0$, so $v=0$. This proves the zero
fiber. Suppose $vv^T=ww^T\neq0$. Both matrices have the same one-dimensional
range, so $w=c v$ for some nonzero real $c$. Substitution gives
$c^2vv^T=vv^T$, hence $c^2=1$ and $c=\pm1$. Conversely,
$(-v)(-v)^T=vv^T$. Thus the nonzero fiber is exactly $\{v,-v\}$.
$\square$

## 6. Cone camera

### T-CONE-1 -- Cone image, identity, and fibers [Proved]

**Claim.** $Q$ maps onto $\mathcal K$, satisfies the cone identity, and
has exactly the candidate fibers in C-CONE-1.

**Proof.** For $Q(x,y)=(X,Y,Z)$,

$$
\begin{aligned}
X^2+Y^2
&=(x^2-y^2)^2+(2xy)^2\\
&=x^4-2x^2y^2+y^4+4x^2y^2\\
&=(x^2+y^2)^2\\
&=Z^2.
\end{aligned}
$$

Also $Z=x^2+y^2\geq0$, so the image lies in $\mathcal K$.

Let $(X,Y,Z)\in\mathcal K$. If $Z=0$, then the cone equation gives
$X=Y=0$, and $(0,0)$ maps to the point.

Assume $Z>0$. The cone equation implies $-Z\leq X\leq Z$. If
$Z+X>0$, define

$$
x_0=\sqrt{\frac{Z+X}{2}},
\qquad
y_0=\frac{Y}{2x_0}.
$$

Then $x_0^2=(Z+X)/2$. Using
$Y^2=Z^2-X^2=(Z-X)(Z+X)$,

$$
y_0^2
=\frac{Y^2}{4x_0^2}
=\frac{Z-X}{2}.
$$

Therefore

$$
x_0^2-y_0^2=X,
\qquad
2x_0y_0=Y,
\qquad
x_0^2+y_0^2=Z.
$$

If $Z+X=0$, then $X=-Z$ and the cone equation gives $Y=0$. Taking
$x_0=0$, $y_0=\sqrt Z$ again yields the target point. Thus $Q$ is
surjective.

It remains to prove the fibers without using the later PSD/cone equivalence.
Suppose

$$
Q(x,y)=Q(u,v).
$$

Equality of $X$ and $Z$ gives

$$
x^2-y^2=u^2-v^2,
\qquad
x^2+y^2=u^2+v^2.
$$

Adding and subtracting these equations gives $x^2=u^2$ and $y^2=v^2$,
so $u=\pm x$ and $v=\pm y$. Equality of $Y$ gives $xy=uv$. If both
$x$ and $y$ are nonzero, the two signs must agree. If either coordinate is
zero, the corresponding equality of squares forces the matching target
coordinate to be zero, and the remaining coordinate still differs by only one
global sign. Hence $(u,v)=(x,y)$ or $(u,v)=(-x,-y)$. At the origin these
are the same point. Conversely, the formula directly gives
$Q(-x,-y)=Q(x,y)$. This proves the complete fibers. $\square$

### T-CONE-2 -- PSD/cone linear bijection [Proved]

**Claim.** $\Lambda$ bijects $\mathcal P_1$ and $\mathcal K$, and
$Q=\Lambda\circ H$.

**Proof.** On all symmetric $2\times2$ matrices, define

$$
\Lambda^{-1}(X,Y,Z)
:=
\begin{pmatrix}
(Z+X)/2&Y/2\\
Y/2&(Z-X)/2
\end{pmatrix}.
$$

Direct substitution proves
$\Lambda^{-1}\circ\Lambda$ and
$\Lambda\circ\Lambda^{-1}$ are identities. Hence $\Lambda$ is a linear
bijection between $\mathrm{Sym}_2(\mathbb{R})$ and $\mathbb{R}^3$.

For $(X,Y,Z)\in\mathcal K$, the inverse matrix has trace $Z\geq0$,
nonnegative diagonal entries because $|X|\leq Z$, and determinant

$$
\frac{Z^2-X^2-Y^2}{4}=0.
$$

It is therefore PSD with rank at most one, so it lies in $\mathcal P_1$.
Conversely, T-PSD-1 writes every member of $\mathcal P_1$ as $H(z)$, and
T-CONE-1 shows $\Lambda(H(z))=Q(z)\in\mathcal K$. Thus the restricted map is
a bijection. Finally,

$$
\Lambda
\begin{pmatrix}x^2&xy\\xy&y^2\end{pmatrix}
=(x^2-y^2,2xy,x^2+y^2)=Q(x,y),
$$

which proves the factorization. $\square$

**Interpretation boundary.** The PSD and cone cameras are exact descriptions of
the quotient by $z\sim -z$. They are not lossless descriptions of oriented
scalars.

## 7. Default flat-stack and coefficient cameras

### T-FLAT-STACK-1 -- the default flat-stack camera is lossless [Proved]

**Claim.** C-FLAT-STACK-1 defines a bijection

$$
\mathrm{Flat}:\mathcal N_{\mathcal A}
\longrightarrow\mathcal F_{\mathcal A}
$$

with inverse Unflat. Under this bijection, ADD is same-index collection and
MULTIPLY is finite convolution over composed multi-indices.

**Proof.** D-NS-1 gives every finite native state a finite support. D-NS-2
says that the state is determined exactly by its coefficient at every
multi-index. Sorting the nonzero support therefore gives one member of
$\mathcal F_{\mathcal A}$. Unflat places each listed nonzero coefficient
back at its recorded index, so

$$
\mathrm{Unflat}(\mathrm{Flat}(F))=F.
$$

Conversely, a valid flat stack has unique increasing indices and nonzero
coefficients. Constructing its state and reading the sorted nonzero support
returns the original stack, so

$$
\mathrm{Flat}(\mathrm{Unflat}(S))=S.
$$

D-NS-5 defines ADD coefficientwise, which is exactly collection at equal
stack indices with zero results omitted. D-NS-7 defines the coefficient of
MULTIPLY at $\gamma$ as the finite sum over
$\alpha\oplus_I\beta=\gamma$, which is exactly finite stack convolution.
Thus the camera is lossless and preserves the complete finite-state
semantics. $\square$

**Default boundary.** This theorem makes Flat the default readout, not a new
underlying carrier. The native state and its canonical flat stack are
bijective descriptions. PSD, cone, evaluation, and selected-coefficient
cameras remain explicit opt-in maps with their stated fibers.

### T-COEF-1 -- Coefficient projection law [Proved]

**Claim.** $\pi_\alpha$ is additive and has the fiber stated in C-COEF-1.

**Proof.** D-NS-5 gives

$$
\pi_\alpha(F\oplus G)
=(F\oplus G)_\alpha
=F_\alpha\boxplus G_\alpha
=\pi_\alpha(F)\boxplus\pi_\alpha(G).
$$

The preimage of $c$ consists, by definition, exactly of states whose
$\alpha$-coefficient is $c$. Coefficients at all other indices are
unconstrained, which is the stated fiber and information loss. $\square$

### T-BLOCK-1 -- Finite-block restricted bijection [Proved]

**Claim.** $\Pi_S|_{\mathcal N_S}$ is a bijection with the candidate inverse.

**Proof.** Given $(c_\alpha)_{\alpha\in S}$, the finite state

$$
F=\mathop{\bigoplus}_{\alpha\in S}c_\alpha[\alpha]
$$

has exactly coefficient $c_\alpha$ at each $\alpha\in S$, after zero
coefficients are omitted, and has zero coefficients outside $S$. Thus it lies
in $\mathcal N_S$ and maps to the tuple, proving surjectivity.

If two states in $\mathcal N_S$ map to the same tuple, their coefficients
agree on $S$ and both vanish outside $S$. D-NS-2 gives equality, proving
injectivity. The construction is therefore the two-sided inverse. $\square$

### T-PAIR-1 -- State-selected coefficient camera [Proved]

**Claim.**

$$
\pi_\alpha^{\mathrm{self}}(F)=\pi_\alpha(F).
$$

**Proof.** The selector $R_\alpha=\mathbf1[\alpha]$ has one nonzero
coefficient. D-NS-12 therefore reduces the pairing to

$$
\langle R_\alpha,F\rangle
=\mathbf1^\dagger\boxtimes F_\alpha.
$$

L-OS-12 gives $\mathbf1^\dagger=\mathbf1$, and L-OS-4 gives
$\mathbf1\boxtimes F_\alpha=F_\alpha$. This equals
$\pi_\alpha(F)$. $\square$

**Interpretation boundary.** Native states can select exact coefficient
cameras. This does not imply that nonlinear, quotient, or external-coordinate
cameras are native states.

## 8. Assigned-generator evaluation

### L-EVAL-1 -- Assigned-power composition [Proved]

For every $z\in\mathbb{O}$ and $m,n\in\mathbb{N}_0$, recursion and L-OS-3
associativity give

$$
z^{\boxtimes(m+n)}
=z^{\boxtimes m}\boxtimes z^{\boxtimes n}.
$$

The proof is induction on $n$: the zero case uses L-OS-4; the successor case
uses the recursive definition and associativity. Applying this independently at
each finitely supported direction and using L-OS-3 commutativity yields

$$
z^{\alpha\oplus_I\beta}
=z^\alpha\boxtimes z^\beta.
\tag{8.1}
$$

### T-EVAL-1 -- Evaluation homomorphism [Proved]

**Claim.** $\mathrm{ev}_z$ preserves ADD, MULTIPLY, zero, and one.

**Proof.** ADD preservation follows from D-NS-5, L-OS-5, and finite
regrouping:

$$
\begin{aligned}
\mathrm{ev}_z(F\oplus G)
&=\mathop{\boxplus}_\alpha
(F_\alpha\boxplus G_\alpha)\boxtimes z^\alpha\\
&=\mathrm{ev}_z(F)\boxplus\mathrm{ev}_z(G).
\end{aligned}
$$

For multiplication, expand the finite convolution and use L-EVAL-1:

$$
\begin{aligned}
\mathrm{ev}_z(F\star G)
&=\mathop{\boxplus}_{\alpha,\beta}
(F_\alpha\boxtimes G_\beta)
\boxtimes z^{\alpha\oplus_I\beta}\\
&=\mathop{\boxplus}_{\alpha,\beta}
(F_\alpha\boxtimes z^\alpha)
\boxtimes(G_\beta\boxtimes z^\beta)\\
&=\mathrm{ev}_z(F)\boxtimes\mathrm{ev}_z(G).
\end{aligned}
$$

All sums are finite, and L-OS-1, L-OS-3, and L-OS-5 justify the regrouping.
The zero state has an empty evaluation fold, so it maps to $\mathbf0$. The
one state contributes
$\mathbf1\boxtimes z^{\mathbf0_I}=\mathbf1$, so it maps to
$\mathbf1$. $\square$

### T-EVAL-2 -- Evaluation is generally noninjective [Proved]

**Claim.** If $\mathcal A$ is nonempty, evaluation collapses distinct formal
native states; specifically,

$$
\mathrm{ev}_z(\mathsf X_k)
\mathrel{=}
\mathrm{ev}_z(\eta(z_k)).
$$

**Proof.** By C-EVAL-1,

$$
\mathrm{ev}_z(\mathsf X_k)
=\mathbf1\boxtimes z^{\varepsilon_k}
=z_k.
$$

The embedded scalar is supported at $\mathbf0_I$, so

$$
\mathrm{ev}_z(\eta(z_k))
=z_k\boxtimes z^{\mathbf0_I}
=z_k.
$$

The states are distinct: $\mathsf X_k$ has coefficient $\mathbf1$ at
$\varepsilon_k$, whereas $\eta(z_k)$ has coefficient $\mathbf0$ there
because $\varepsilon_k\neq\mathbf0_I$. Therefore the camera is not
injective. If $\mathcal A$ is empty, this particular collision does not exist;
the theorem makes no noninjectivity claim for that degenerate case. $\square$

### T-CAMERA-RESIDUAL-1 -- automatic residual is canonical [Proved]

**Claim.** For every finite camera codomain with exact signed coordinates,
$\mathrm{Residual}_C(x)$ is unique and

$$
 a\notin\mathrm{Residual}_C(x)
 \Longleftrightarrow
 \mathrm{ADD}_C(x;a)=0,
$$

and consequently

$$
\mathrm{Residual}_C(x)=\varnothing
\quad\Longleftrightarrow\quad
C(x)=0_C.
$$

**Proof.** Each owned coordinate is an exact oriented scalar. Commutativity
and associativity make the finite ADD result independent of input order, while
uniqueness of the $\mathbf1,\mathbf J$ coefficients makes its reduced value
unique. By definition, the canonical sparse form retains an axis exactly when
that reduced coefficient is nonzero. Hence an axis is absent exactly when its
signed ADD total is zero. Omitting all and only those zero coordinates produces
one canonical residual. It is empty exactly when every coordinate is zero,
which is the codomain zero. Retained INDEX locations are distinct coordinates by
D-NS-2 and are never combined by this normalization. $\square$

### T-CAMERA-ZERO-FILL-1 -- zero-fill is the unique total extension [Proved]

**Claim.** Let $r=\mathrm{Residual}_C(x)$ be a canonical sparse
residual on the declared axis set $\mathrm{Ax}(C)$. Then
$\mathrm{Frame}_C(x)$ from C-CAMERA-ZERO-FILL-1 is the unique total
coordinate function that agrees with $r$ on its support and is zero off its
support. It satisfies

$$
\mathrm{Frame}_C(x)=0_C
\quad\Longleftrightarrow\quad
\mathrm{Residual}_C(x)=\varnothing.
$$

**Proof.** Every declared axis is either in the finite support of $r$ or is
not. Agreement with $r$ fixes the first case, and the required additive
identity fixes the second, so the extension exists and is unique. Restricting
the total frame to its nonzero support recovers $r$, hence zero-fill changes
no information. The total frame is zero on every declared axis exactly when
it has no nonzero support, which is exactly when $r$ is empty. Coordinatewise
ADD follows from the codomain's product operation, with every absent sparse
entry contributing the additive identity. $\square$

**Boundary.** This theorem is parameterized by one selected camera. It does
not equate an axis of $C$ with an axis of another camera $D$.

### T-CAMERA-TRANSITION-1 -- valid cross-perspective zero transfer [Proved]

**Claim.** Let $C:X\to Y$, $D:X\to Z$, and $T:Y\to Z$ satisfy
$D=T\circ C$ on a declared domain.

- If $T(0_Y)=0_Z$, then $C(x)=0_Y$ implies $D(x)=0_Z$.
- If $T^{-1}(\{0_Z\})=\{0_Y\}$, then $D(x)=0_Z$ implies
  $C(x)=0_Y$.
- If both hold, the two camera zeros are equivalent on that domain.

**Proof.** The first clause follows by substitution:
$D(x)=T(C(x))=T(0_Y)=0_Z$. For the second, $D(x)=0_Z$ gives
$T(C(x))=0_Z$, so the stated zero fiber forces $C(x)=0_Y$. Combining the
two implications proves the last clause. $\square$

**Boundary.** Matching coordinate labels or a shared source object does not
supply $T$, the commuting relation, or either zero-fiber property. Those are
separate proof obligations for every cross-perspective use.

## 9. Classification summary

The proofs establish:

| Camera | Proved classification |
|---|---|
| $\kappa$ | field isomorphism $\mathbb{O}\cong\mathbb{C}$ |
| $\mu$ | algebra isomorphism onto $\mathcal C_2$ |
| $W$ | surjective $2\pi$-periodic covering of $\mathbb{O}^\times$ |
| $\overline W$ | bijection from the quotient cylinder |
| $H$ | quotient camera with fibers $\{z,-z\}$ away from zero |
| $Q$ | quotient camera onto $\mathcal K$ with the same fibers |
| $\Lambda$ | linear bijection $\mathcal P_1\cong\mathcal K$ |
| $\pi_\alpha$ | additive projection |
| $\Pi_S|_{\mathcal N_S}$ | finite-coordinate bijection |
| state-selected pairing | exact coefficient camera |
| $\mathrm{ev}_z$ | generally noninjective algebra homomorphism |

These classifications close the non-derivative camera obligations. They do not
yet supply native derivative laws or an analytic completion.
