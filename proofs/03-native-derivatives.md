<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Proofs of Finite Native Derivative Laws

**Proof status:** Complete paper proofs on finite coefficient strata  
**Research stage:** 4 -- Derivatives in the native system and its cameras  
**Machine-checked:** No

## 1. Dependencies and scope

These proofs use:

- the algebraic core proved in `01-core-algebra.md`;
- the camera classifications proved in `02-camera-equivalences.md`;
- A-AN-1 and definitions D-DER-1 through D-DER-8.

Every native domain below is finite-dimensional. No infinite-support derivative
or mode-birth derivative is claimed.

## 2. Finite native linear structure

### L-DER-1 -- Finite strata are real vector spaces [Proved]

**Claim.** Each $\mathcal N_S$, with $\oplus$ and $\odot$, is a real
vector space.

**Proof.** For $r\in\mathbb{R}$ and coefficient
$F_\alpha=(x_\alpha,y_\alpha)$, L-NS-4 and D-OS-3 give

$$
(r\odot F)_\alpha
=(r,0)\boxtimes(x_\alpha,y_\alpha)
=(r x_\alpha,r y_\alpha).
\tag{2.1}
$$

Thus real scaling acts coordinatewise and does not change support. L-NS-2
already gives the abelian group under $\oplus$. Every vector-space scalar law
now follows coefficientwise from the real-field laws. For example,

$$
((r+s)\odot F)_\alpha
=((r+s)x_\alpha,(r+s)y_\alpha)
=((r\odot F)\oplus(s\odot F))_\alpha,
$$

and

$$
(r\odot(s\odot F))_\alpha
=(rsx_\alpha,rsy_\alpha)
=((rs)\odot F)_\alpha.
$$

The laws involving $F\oplus G$, scalar one, and scalar zero are identical
coordinate calculations. State equality completes the proof. $\square$

### L-DER-2 -- Finite stratum norm [Proved]

**Claim.** D-DER-4 defines a norm on $\mathcal N_S$, and if
$S\subseteq T$, then $\|F\|_S=\|F\|_T$ for $F\in\mathcal N_S$.

**Proof.** Expanding all coefficients,

$$
\|F\|_S
\mathrel{=}
\left(
\sum_{\alpha\in S}
(x_\alpha^2+y_\alpha^2)
\right)^{1/2},
$$

which is the ordinary Euclidean norm on $2|S|$ real coordinates. A-AN-1
therefore supplies nonnegativity and the triangle inequality. L-OS-7 gives
$\|F\|_S=0$ exactly when every coefficient is $\mathbf0$, which is exactly
$F=\mathsf0$ by D-NS-2. Equation (2.1) gives

$$
\|r\odot F\|_S=|r|\|F\|_S.
$$

These are the norm laws. If $S\subseteq T$, every added coefficient is zero,
so the added squared-size terms vanish and the norm is unchanged. $\square$

### L-DER-3 -- Finite MULTIPLY bound [Proved]

**Claim.** For finite $S,T$, let

$$
S\oplus_I T
:=
\{\alpha\oplus_I\beta:\alpha\in S,\beta\in T\}.
$$

Then

$$
\|F\star G\|_{S\oplus_I T}
\leq
\sqrt{|S||T|}\,\|F\|_S\|G\|_T.
\tag{2.2}
$$

**Proof.** Write $|z|:=\sqrt{\nu(z)}$. L-OS-9 gives
$|z\boxtimes w|=|z||w|$, and the Euclidean triangle inequality gives
$|z\boxplus w|\leq|z|+|w|$. The Euclidean coefficient norm is at most the
sum of coefficient magnitudes, so D-NS-7 yields

$$
\begin{aligned}
\|F\star G\|_{S\oplus_I T}
&\leq\sum_{\alpha\in S}\sum_{\beta\in T}
|F_\alpha||G_\beta|\\
&=\left(\sum_{\alpha\in S}|F_\alpha|\right)
  \left(\sum_{\beta\in T}|G_\beta|\right).
\end{aligned}
$$

Finite Cauchy--Schwarz gives

$$
\sum_{\alpha\in S}|F_\alpha|
\leq\sqrt{|S|}\|F\|_S
$$

and the analogous bound for $G$. Multiplication proves (2.2). $\square$

This bound proves the native bilinear product is continuous on every finite pair
of strata.

## 3. Derivatives of native operations

### T-DER-ADD-1 -- Native ADD derivative [Proved]

**Claim.** For

$$
\mathsf{Add}(F,G):=F\oplus G,
$$

the derivative at every $(F,G)$ is

$$
D\mathsf{Add}_{(F,G)}(H,K)=H\oplus K.
$$

**Proof.** The candidate is real-linear by L-DER-1. Moreover,

$$
\begin{aligned}
&\mathsf{Add}(F\oplus H,G\oplus K)
\ominus\mathsf{Add}(F,G)
\ominus(H\oplus K)\\
&=(F\oplus H\oplus G\oplus K)
\ominus(F\oplus G)\ominus(H\oplus K)\\
&=\mathsf0
\end{aligned}
$$

by L-NS-2. The remainder quotient is identically zero. $\square$

### T-DER-MUL-1 -- Native MULTIPLY product rule [Proved]

**Claim.** For $\mathsf{Mul}(F,G):=F\star G$,

$$
D\mathsf{Mul}_{(F,G)}(H,K)
\mathrel{=}
(H\star G)\oplus(F\star K).
$$

**Proof.** The candidate is real-linear in $(H,K)$ by T-NS-1 and L-DER-1.
Distributivity gives the exact expansion

$$
(F\oplus H)\star(G\oplus K)
=F\star G
\oplus H\star G
\oplus F\star K
\oplus H\star K.
$$

After subtracting the value and candidate linear term, the remainder is
$H\star K$. By L-DER-3,

$$
\|H\star K\|
\leq C\|H\|\|K\|
\leq\frac C2(\|H\|^2+\|K\|^2)
=\frac C2\|(H,K)\|^2
$$

for fixed finite strata and $C=\sqrt{|S||T|}$. Dividing by
$\|(H,K)\|$ tends to zero with the perturbation. This is D-DER-7.
$\square$

### T-DER-ORIENT-1 -- ORIENT derivative [Proved]

**Claim.**

$$
D(\mathrm{ORIENT}_r)_F(H)=\mathrm{ORIENT}_r(H).
$$

**Proof.** L-SEP-3 expresses ORIENT as multiplication by a fixed state.
Distributivity makes it additive, and equation (2.1) plus commutativity makes it
real-homogeneous. Hence it is real-linear. For any real-linear map $L$,

$$
L(F\oplus H)\ominus L(F)\ominus L(H)=\mathsf0,
$$

so its derivative is itself. $\square$

L-OS-9 and L-OS-10 also give
$\|\mathrm{ORIENT}_r(H)\|=\|H\|$: quarter orientation is an isometry.

### T-DER-INDEX-1 -- INDEX derivative [Proved]

**Claim.** On a fixed finite input stratum,

$$
D(\mathrm{INDEX}_k)_F(H)=\mathrm{INDEX}_k(H).
$$

**Proof.** INDEX is multiplication by fixed state $\mathsf X_k$, so the same
linearity argument as T-DER-ORIENT-1 applies. L-SEP-2 shows the support shift is
injective and coefficients are unchanged, hence
$\|\mathrm{INDEX}_k(H)\|=\|H\|$ on the correspondingly shifted output
stratum. $\square$

### T-DER-POWER-1 -- Native scalar power rule [Proved]

**Claim.** For $P_n(z):=z^{\boxtimes n}$,

$$
DP_0{}_z(h)=\mathbf0,
$$

and for $n\geq1$,

$$
DP_n{}_z(h)
\mathrel{=}
\widehat n\boxtimes z^{\boxtimes(n-1)}\boxtimes h.
$$

**Proof.** The zero power is constant $\mathbf1$. For $n\geq1$, apply
the oriented-scalar clause of L-BIN-1:

$$
(z\boxplus h)^{\boxtimes n}
=z^{\boxtimes n}
\boxplus
\widehat n\boxtimes z^{\boxtimes(n-1)}\boxtimes h
\boxplus R_n(z,h),
$$

where every term in the finite remainder contains at least two factors of
$h$. On a bounded neighborhood of fixed $z$, L-OS-9 and the Euclidean
triangle inequality give $\|R_n(z,h)\|\leq C_z\|h\|^2$. Dividing by
$\|h\|$ gives a limit of zero. $\square$

### T-DER-INV-1 -- Native scalar inverse rule [Proved]

**Claim.** On $\mathbb{O}^\times$,

$$
D(\mathrm{inv})_z(h)
\mathrel{=}
\boxminus\big(z^{[-1]}\boxtimes h\boxtimes z^{[-1]}\big).
$$

**Proof.** For sufficiently small $h$, $z\boxplus h\neq\mathbf0$. The
field laws give the exact resolvent identity

$$
(z\boxplus h)^{[-1]}\boxminus z^{[-1]}
\mathrel{=}
\boxminus\big((z\boxplus h)^{[-1]}\boxtimes h\boxtimes z^{[-1]}\big).
$$

Subtracting the candidate linear term leaves

$$
\big(z^{[-1]}\boxminus(z\boxplus h)^{[-1]}\big)
\boxtimes h\boxtimes z^{[-1]}.
$$

Applying the same resolvent identity once more shows this remainder contains two
factors of $h$ and inverse factors that remain bounded in a sufficiently small
neighborhood of nonzero $z$. L-OS-9 therefore bounds it by
$C_z\|h\|^2$. The derivative limit follows. $\square$

## 4. Exact-camera derivatives

### T-DER-CX-1 -- Complex camera derivative [Proved]

**Claim.**

$$
D\kappa_z(h)=\kappa(h).
$$

**Proof.** T-CX-2 makes $\kappa$ additive, and its coordinate formula makes
it real-homogeneous. It is real-linear, so the derivative equals the map itself
with zero remainder. Its real rank is two. $\square$

### T-DER-MAT-1 -- Matrix camera derivative [Proved]

**Claim.**

$$
D\mu_z(h)=\mu(h).
$$

**Proof.** T-MAT-1 proves $\mu$ real-linear. The derivative of a linear map is
itself with zero remainder. Its image $\mathcal C_2$ has real dimension two,
so the rank is two. $\square$

## 5. Unwrapped-camera derivative

### T-DER-POLAR-1 -- Polar derivative and local rank [Proved]

**Claim.** For perturbation $(a,b)\in\mathbb{R}^2$,

$$
DW_{(\rho,\theta)}(a,b)
\mathrel{=}
e^\rho
\big(
a\cos\theta-b\sin\theta,
a\sin\theta+b\cos\theta
\big).
$$

Its rank is two at every finite $(\rho,\theta)$.

**Proof.** Differentiate the two conventional coordinate functions using
A-AN-1. The Jacobian is

$$
e^\rho
\begin{pmatrix}
\cos\theta&-\sin\theta\\
\sin\theta&\cos\theta
\end{pmatrix}.
$$

Its determinant is

$$
e^{2\rho}(\cos^2\theta+\sin^2\theta)=e^{2\rho}>0,
$$

so it has rank two everywhere. T-POLAR-1 already proves zero has no finite
preimage; this derivative does not remove that global singularity or the
$2\pi$-periodic fibers. $\square$

## 6. Quadratic-camera derivatives

Use the Frobenius norm on symmetric matrices and the Euclidean norm on
$\mathbb{R}^3$.

### T-DER-PSD-1 -- PSD camera derivative [Proved]

**Claim.** For oriented-scalar perturbation $h$,

$$
DH_z(h)=v_hv_z^T+v_zv_h^T.
$$

The derivative has rank zero at $z=\mathbf0$ and rank two for
$z\neq\mathbf0$.

**Proof.** Expand exactly:

$$
\begin{aligned}
H(z\boxplus h)
&=(v_z+v_h)(v_z+v_h)^T\\
&=v_zv_z^T+v_hv_z^T+v_zv_h^T+v_hv_h^T.
\end{aligned}
$$

The last term has Frobenius norm $O(\|h\|^2)$, proving the derivative. At
zero the displayed linear term vanishes. Suppose $z\neq0$ and
$v_hv_z^T+v_zv_h^T=0$. Multiplying this matrix by $v_z$ gives

$$
\|v_z\|^2v_h+(v_h^Tv_z)v_z=0.
$$

Taking the inner product with $v_z$ yields
$2\|v_z\|^2(v_h^Tv_z)=0$, so $v_h^Tv_z=0$. The previous vector equation
then reduces to $\|v_z\|^2v_h=0$, hence $v_h=0$. The derivative is
injective from a two-dimensional domain, so its rank is two. $\square$

### T-DER-CONE-1 -- Cone derivative and rank [Proved]

**Claim.** For $z=(x,y)$ and $h=(a,b)$,

$$
DQ_{(x,y)}(a,b)
\mathrel{=}
(2xa-2yb,\ 2ya+2xb,\ 2xa+2yb).
$$

The derivative has rank zero at the origin and rank two elsewhere.

**Proof.** Expanding each quadratic coordinate gives the displayed linear terms;
the remaining terms are

$$
(a^2-b^2,2ab,a^2+b^2)=Q(a,b),
$$

whose Euclidean norm is $O(a^2+b^2)$. This proves the derivative.

At the origin every linear term vanishes. Away from the origin, the first two
output coordinates have Jacobian minor

$$
\begin{pmatrix}
2x&-2y\\
2y&2x
\end{pmatrix}
$$

with determinant $4(x^2+y^2)>0$. Hence the full derivative has rank two.
$\square$

The rank loss at the cone apex is therefore local analytic evidence of the same
special point already visible in the global fiber theorem.

## 7. Readout-camera derivatives

### T-DER-READ-1 -- Coefficient, block, and self-selected derivatives [Proved]

**Claim.**

$$
D\pi_\alpha{}_F(H)=\pi_\alpha(H),
\qquad
D\Pi_S{}_F(H)=\Pi_S(H),
$$

and the state-selected coefficient camera has the same derivative.

**Proof.** T-COEF-1 and T-BLOCK-1 show the readouts are real-linear on their
finite domains, so their derivatives equal themselves. T-PAIR-1 identifies the
self-selected camera pointwise with $\pi_\alpha$, so their derivatives agree.
$\square$

## 8. Evaluation derivatives

Fix $F\in\mathcal N_{\mathcal A}$. Let

$$
K_F:=\{k\in K_{\mathcal A}:d_k(\alpha)>0
\text{ for some }\alpha\in\mathrm{supp}(F)\}.
$$

This set is finite. Regard the active generator assignment
$z=(z_k)_{k\in K_F}$ as a point of the finite product
$\mathbb{O}^{K_F}$.

### T-DER-EVAL-1 -- Assigned-generator evaluation derivative [Proved]

**Claim.** For perturbation $h=(h_k)_{k\in K_F}$,

$$
\begin{aligned}
D(\mathrm{ev}_{\bullet}(F))_z(h)
=\mathop{\boxplus}_{\alpha\in\mathrm{supp}(F)}
F_\alpha\boxtimes
\mathop{\boxplus}_{\substack{k\in K_F\\d_k(\alpha)>0}}
&\widehat{d_k(\alpha)}
\boxtimes z_k^{\boxtimes(d_k(\alpha)-1)}
\boxtimes h_k\\
&\boxtimes
\mathop{\boxtimes}_{\substack{j\in K_F\\j\neq k}}
z_j^{\boxtimes d_j(\alpha)}.
\end{aligned}
$$

Empty inner sums are $\mathbf0$, and empty products are $\mathbf1$.

**Proof.** Each $z^\alpha$ is a finite product of native scalar powers.
T-DER-POWER-1 differentiates each active factor, and repeated application of
T-DER-MUL-1 gives the sum over the one factor differentiated at a time. The
fixed coefficient $F_\alpha$ multiplies the result. Finally,
T-DER-ADD-1 sums the finitely many terms over $\alpha$. This produces exactly
the displayed formula. Because the map is a finite polynomial in real
coordinates, all omitted terms contain at least two perturbation factors and
are $O(\|h\|^2)$. $\square$

For fixed assignment $z$, T-EVAL-1 already makes
$F\mapsto\mathrm{ev}_z(F)$ real-linear, so its derivative with respect
to the native state is $H\mapsto\mathrm{ev}_z(H)$.

## 9. Stage 4 derivative conclusion

Finite Native Space now has an exact change calculus:

- ADD differentiates to ADD;
- MULTIPLY obeys the product rule;
- ORIENT and INDEX differentiate to themselves as linear actions;
- scalar powers and inverses obey their derived native rules;
- exact cameras have constant derivatives;
- wrapped polar coordinates are locally full rank but globally periodic;
- PSD and cone cameras lose derivative rank only at the origin;
- finite evaluation has an explicit generator-assignment derivative.

These results do not define differentiation through unbounded mode creation or
on an infinite spectral completion.
