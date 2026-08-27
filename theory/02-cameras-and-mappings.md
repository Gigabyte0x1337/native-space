<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Native Space 1.0: Cameras and Mappings

**Document status:** Native Space 1.0 camera definitions
**Research stage:** 2 -- Define the mappings and cameras  
**Proof status:** Classifications proved in
`../proofs/02-camera-equivalences.md`; finite derivatives proved in
`../proofs/03-native-derivatives.md`

## 1. Camera rule

A camera is a typed map that exposes selected native information in another
coordinate system. A camera does not become fundamental merely because its
output is familiar or easy to visualize.

Every camera below records:

- its domain and codomain;
- its forward rule;
- an inverse, local inverse, or fiber description;
- singularities and branch choices;
- preserved and discarded information;
- the exact properties that remain to be proved.

The Native Space definitions in `01-native-system.md` do not depend on any
camera in this file.

### D-CAMERA-AXIS-1 -- perspective-owned axes [Definition]

An axis is identified by the pair

$$
(C,j),
$$

where $C:X\to Y$ is its camera and $j$ selects one coordinate in that
camera's codomain. Axis $(C,j)$ and axis $(D,j)$ are different when
$C\neq D$, even if both coordinates are informally called "real",
"imaginary", or "axis 1".

Two coordinates may be compared directly only inside one codomain. A
cross-perspective comparison must provide an explicit transition

$$
T:Y\to Z
$$

and prove the commuting relation $D=T\circ C$ on the stated domain. A zero
claim additionally records whether $T$ preserves zero, reflects zero, or
both. Without that transition theorem, coordinates from different
perspectives remain separate and imply nothing about each other's zero fibers.

This is a repository-wide invariant. It applies to flat, complex, matrix,
polar, cone, analytic, zeta, RE, and future application cameras.

### C-CAMERA-RESIDUAL-1 -- automatic perspective normal form [Definition]

Once a perspective $C$ is selected, combine opposite signed contributions
on each axis owned by $C$, and omit every resulting zero coordinate. The
remaining sparse coordinates are

$$
\mathrm{Residual}_C(x).
$$

This normalization is automatic camera behavior, not a list of cancellation
steps that every proof must repeat. It exposes exactly what did not cancel.
For every axis $a$ owned by $C$,

$$
a\notin\mathrm{Residual}_C(x)
\quad\Longleftrightarrow\quad
\text{the signed ADD total on }a\text{ is }0.
$$

Thus a cancelled axis does not exist in the canonical residual representation;
it is not retained with a zero coefficient.
Normalization never crosses camera ownership. It also never crosses retained
INDEX locations; a lossy camera must explicitly discard or aggregate INDEX
before those locations may contribute to one coordinate.

The exact runtime implements this invariant canonically: ADD combines
coefficients only at the same multi-index, combines their signed real and
imaginary coordinates, and removes the term exactly when both coordinates
become zero.

### C-CAMERA-ZERO-FILL-1 -- total perspective frame [Definition]

Every perspective $C$ declares an axis set $\mathrm{Ax}(C)$ as part of
its codomain. Each declared coordinate map must be well-defined on the stated
domain; this is the perspective's axis-existence obligation. Its residual is a
sparse partial representation supported on
$\mathrm{Ax}(C)$. When that perspective is used, extend the residual to
the total frame

$$
\mathrm{Frame}_C(x)(a):=
\begin{cases}
\mathrm{Residual}_C(x)(a),&
 a\in\mathrm{supp}(\mathrm{Residual}_C(x)),\\
0,&a\in\mathrm{Ax}(C)\setminus
\mathrm{supp}(\mathrm{Residual}_C(x)).
\end{cases}
$$

Thus every declared axis exists in the frame, while every cancelled or
otherwise zero axis is read as the additive identity without storing a zero
entry. ADD and equality use these axes separately and coordinatewise inside
the selected perspective. Zero-fill never identifies axes owned by different
perspectives and never creates a cross-perspective transition.

### C-FLAT-STACK-1 -- default lossless flat-stack camera [Definition]

For a finite native state $F$, list its nonzero coefficients in increasing
multi-index order:

$$
\mathrm{Flat}(F)
:=
\big((\alpha,x_\alpha,y_\alpha)
  : F_\alpha=(x_\alpha,y_\alpha)\neq\mathbf0\big).
$$

The zero state maps to the empty stack. The codomain $\mathcal F_{\mathcal A}$
is the set of finite stacks with strictly increasing multi-indices and nonzero
oriented coordinate pairs. Its candidate inverse is

$$
\mathrm{Unflat}
\big((\alpha_j,x_j,y_j)_{j=1}^{m}\big)
:=
\mathop{\bigoplus}_{j=1}^{m}(x_j,y_j)[\alpha_j].
$$

This is the **default camera** for native states. It is an exact flattened
stack of signed orientation planes over INDEX locations. On a one-coordinate
prime-birth slice it reads $(x,y,k)$, producing the flat prime helix. It
does not square coordinates, collapse sign, evaluate generators, or discard
unselected indices.

### T-FLAT-STACK-1 -- the default camera is lossless [Proved]

The proof in `../proofs/02-camera-equivalences.md` establishes that Flat and
Unflat are mutual inverses. It also shows that native ADD and MULTIPLY become,
respectively, exact same-index collection and finite index-convolution in the
flat stack. The runtime serialization name for this camera is
`flat-stack-v1`.

## 2. Oriented-scalar cameras

### C-CX-1 -- Conventional complex-coordinate camera [Definition]

Let $\mathbb{C}$ denote the conventional complex field with conventional
imaginary unit $i$. Define

$$
\kappa:\mathbb{O}\to\mathbb{C},
\qquad
\kappa(x,y):=x+i y.
$$

Candidate inverse:

$$
\kappa^{-1}(x+i y):=(x,y).
$$

| Property | Record |
|---|---|
| Domain | all of $\mathbb{O}$ |
| Codomain | all of $\mathbb{C}$ |
| Singularities | none |
| Branch choices | none |
| Intended status | bijective field isomorphism |
| Information loss | none, if the intended status is proved |

Stage 4 must prove that $\kappa$ and its stated inverse are mutually inverse
and that

$$
\kappa(z\boxplus w)=\kappa(z)+\kappa(w),
\qquad
\kappa(z\boxtimes w)=\kappa(z)\kappa(w).
$$

Complex arithmetic cannot be used to prove the Native Space laws in Stage 3;
that would reverse the dependency direction.

### C-MAT-1 -- Real operator camera [Definition]

Define

$$
\mu:\mathbb{O}\to M_2(\mathbb{R}),
\qquad
\mu(x,y):=
\begin{pmatrix}
x&-y\\
y&x
\end{pmatrix}.
$$

Its declared image is

$$
\mathcal C_2
:=
\left\{
\begin{pmatrix}
x&-y\\
y&x
\end{pmatrix}
:x,y\in\mathbb{R}
\right\}.
$$

Candidate inverse on $\mathcal C_2$:

$$
\mu^{-1}
\begin{pmatrix}
x&-y\\
y&x
\end{pmatrix}
:=(x,y).
$$

| Property | Record |
|---|---|
| Domain | all of $\mathbb{O}$ |
| Codomain | $\mathcal C_2\subset M_2(\mathbb{R})$ |
| Singularities | none as a coordinate map |
| Branch choices | none |
| Intended status | bijective algebra isomorphism onto $\mathcal C_2$ |
| Information loss | none inside the stated image |

For $w=(w_1,w_2)$, write
$\mathrm{col}(w):=(w_1,w_2)^T$. The operator camera should make
native self-action visible:

$$
\mathrm{col}(z\boxtimes w)
\mathrel{=}
\mu(z)
\mathrm{col}(w).
$$

Stage 4 must prove this action law and both additive and multiplicative
preservation.

### C-POLAR-1 -- Unwrapped scale-orientation covering [Definition]

Let

$$
\mathbb{P}:=\mathbb{R}\times\mathbb{R}.
$$

Define the conventional wrapping camera

$$
W:\mathbb{P}\to\mathbb{O}^{\times},
\qquad
W(\rho,\theta)
:=
\left(e^\rho\cos\theta,\ e^\rho\sin\theta\right).
$$

Here exponential and trigonometric functions belong to the camera. They are not
native primitives.

Candidate fiber over $z=(x,y)\in\mathbb{O}^\times$: choose any conventional
angle $\theta_z$ satisfying

$$
(\cos\theta_z,\sin\theta_z)
\mathrel{=}
\frac{(x,y)}{\sqrt{\nu(z)}}.
$$

Then the full candidate fiber is

$$
W^{-1}(z)
\mathrel{=}
\left\{
\left(\tfrac{1}{2}\log\nu(z),\ \theta_z+2\pi n\right)
:n\in\mathbb{Z}
\right\}.
$$

| Property | Record |
|---|---|
| Domain | $\mathbb{R}^2$ in unwrapped coordinates |
| Codomain | $\mathbb{O}^\times$, excluding zero |
| Singularities | $\mathbf0$ has no finite $\rho$ preimage |
| Branch choices | every single-valued angle inverse requires a branch |
| Intended status | surjective covering with period $2\pi$ |
| Information loss | wrapping forgets the integer turn count |

The quotient cylinder

$$
\overline{\mathbb{P}}
:=
\mathbb{R}\times(\mathbb{R}/2\pi\mathbb{Z})
$$

has the candidate induced bijection

$$
\overline W:\overline{\mathbb{P}}\to\mathbb{O}^\times.
$$

Stage 4 must prove the fiber statement, quotient bijection, and multiplication
law

$$
W(\rho,\theta)\boxtimes W(\sigma,\phi)
\mathrel{=}
W(\rho+\sigma,\theta+\phi).
$$

This is the precise sense in which nonzero multiplication may become flat
translation. It says nothing similar about native ADD.

## 3. Quadratic PSD and cone cameras

### C-PSD-1 -- Rank-one PSD camera [Definition]

Write an oriented scalar $z=(x,y)$ as the real column
$v_z=(x,y)^T$. Define

$$
H:\mathbb{O}\to\mathrm{Sym}_2(\mathbb{R}),
\qquad
H(z):=v_zv_z^T
\mathrel{=}
\begin{pmatrix}
x^2&xy\\
xy&y^2
\end{pmatrix}.
$$

Its declared image is

$$
\mathcal P_1
:=
\{A\in\mathrm{Sym}_2(\mathbb{R}):A\succeq0,\ \mathrm{rank}A\leq1\}.
$$

Candidate fibers:

$$
H^{-1}(0)=\{\mathbf0\},
\qquad
H^{-1}(H(z))=\{z,\boxminus z\}
\quad(z\neq\mathbf0).
$$

| Property | Record |
|---|---|
| Domain | all of $\mathbb{O}$ |
| Codomain | $\mathcal P_1$ |
| Singularities | rank drops to zero at $\mathbf0$ |
| Branch choices | recovering a nonzero state requires a sign choice |
| Intended status | surjective two-to-one quadratic camera away from zero |
| Information loss | global sign / half-turn orientation |

This is a positive second-moment camera, not the foundational native state.

### C-CONE-1 -- Three-dimensional cone camera [Definition]

Define the linear coordinate map on symmetric matrices

$$
\Lambda
\begin{pmatrix}
a&b\\
b&d
\end{pmatrix}
:=
(a-d,\ 2b,\ a+d).
$$

Define the cone camera by composition:

$$
Q:=\Lambda\circ H,
$$

so explicitly

$$
Q(x,y)
\mathrel{=}
(X,Y,Z)
:=
(x^2-y^2,\ 2xy,\ x^2+y^2).
$$

Its declared codomain is the future quadratic cone

$$
\mathcal K
:=
\{(X,Y,Z)\in\mathbb{R}^3:Z\geq0,\ X^2+Y^2=Z^2\}.
$$

Candidate inverse fiber construction for $(X,Y,Z)\in\mathcal K$:

- if $Z=0$, the only candidate preimage is $(0,0)$;
- if $Z+X>0$, set
  $$
  x_0=\sqrt{\frac{Z+X}{2}},
  \qquad
  y_0=\frac{Y}{2x_0};
  $$
- otherwise set $x_0=0$ and $y_0=\sqrt Z$.

The complete nonzero candidate fiber is

$$
Q^{-1}(X,Y,Z)=\{(x_0,y_0),(-x_0,-y_0)\}.
$$

| Property | Record |
|---|---|
| Domain | all of $\mathbb{O}$ |
| Codomain | $\mathcal K\subset\mathbb{R}^3$ |
| Singularities | cone apex at $\mathbf0$ |
| Branch choices | one of two opposite square roots away from the apex |
| Intended status | surjective two-to-one quadratic camera away from zero |
| Information loss | $Q(z)=Q(\boxminus z)$ |

Stage 4 must prove:

$$
X^2+Y^2=Z^2,
\qquad Z\geq0,
$$

surjectivity onto $\mathcal K$, the complete fiber statement, and the linear
bijection between $\mathcal P_1$ and $\mathcal K$ induced by $\Lambda$.

The cone is therefore retained as an important exact quadratic camera while the
flatter oriented state remains underneath it.

## 4. Native-state readout cameras

### C-COEF-1 -- Single-coefficient camera [Definition]

For $\alpha\in\mathbb{I}_{\mathcal A}$, define

$$
\pi_\alpha:\mathcal N_{\mathcal A}\to\mathbb{O},
\qquad
\pi_\alpha(F):=F_\alpha.
$$

The fiber over $c\in\mathbb{O}$ is exactly

$$
\pi_\alpha^{-1}(c)
\mathrel{=}
\{F\in\mathcal N_{\mathcal A}:F_\alpha=c\}.
$$

This camera discards every coefficient outside $\alpha$ and is not injective
when any other index is available. It is total and has no singularity or branch
choice.

### C-BLOCK-1 -- Finite coefficient-block camera [Definition]

For finite $S\subset\mathbb{I}_{\mathcal A}$, define

$$
\Pi_S:\mathcal N_{\mathcal A}\to\mathbb{O}^S,
\qquad
\Pi_S(F):=(F_\alpha)_{\alpha\in S}.
$$

Let

$$
\mathcal N_S
:=
\{F\in\mathcal N_{\mathcal A}:\mathrm{supp}(F)\subseteq S\}.
$$

The restriction $\Pi_S|_{\mathcal N_S}$ has candidate inverse

$$
(c_\alpha)_{\alpha\in S}
\longmapsto
\mathop{\bigoplus}_{\alpha\in S}c_\alpha[\alpha],
$$

using a fixed finite fold until state ADD is proved order-independent. On the
full native space, $\Pi_S$ discards all coefficients outside $S$.
The finite readout is total and has no branch choice.

### C-PAIR-1 -- State-selected coefficient camera [Definition]

For $\alpha\in\mathbb{I}_{\mathcal A}$, define the selector state

$$
R_\alpha:=\mathbf1[\alpha].
$$

Using D-NS-12, define

$$
\pi_\alpha^{\mathrm{self}}(F)
:=
\langle R_\alpha,F\rangle.
$$

Stage 4 must prove

$$
\pi_\alpha^{\mathrm{self}}(F)=\pi_\alpha(F).
$$

If proved, this establishes a limited and exact form of self-observation:
coefficient cameras can be selected by states from the same carrier. It does
not show that every useful nonlinear or quotient camera lives in the carrier.
The pairing rule is total on finite states and has no branch choice.

### C-EVAL-1 -- Assigned-generator evaluation camera [Definition]

Let $z:\mathcal A\to\mathbb{O}$ assign an oriented scalar $z_k:=z(a_k)$
to each primitive label. For $n\in\mathbb{N}_0$, define powers recursively by

$$
z_k^{\boxtimes0}:=\mathbf1,
\qquad
z_k^{\boxtimes(n+1)}:=z_k\boxtimes z_k^{\boxtimes n}.
$$

For a multi-index $\alpha$, define its assigned scalar by a canonical finite
fold:

$$
z^\alpha
:=
\mathop{\boxtimes}_{k\in K_{\mathcal A}:d_k(\alpha)>0}
z_k^{\boxtimes d_k(\alpha)}.
$$

The empty product is $\mathbf1$. Define

$$
\mathrm{ev}_z:\mathcal N_{\mathcal A}\to\mathbb{O},
\qquad
\mathrm{ev}_z(F)
:=
\mathop{\boxplus}_{\alpha\in\mathrm{supp}(F)}
F_\alpha\boxtimes z^\alpha.
$$

| Property | Record |
|---|---|
| Domain | finite native states |
| Codomain | one oriented scalar |
| Singularities | none as a finite rule; assignments may create degeneracy |
| Branch choices | none |
| Intended status | algebra homomorphism after Stage 3 laws are proved |
| Information loss | generally severe; formal index structure is collapsed |

In particular, $\mathsf X_k$ and $\eta(z_k)$ are distinct formal native
states but are intended to have the same evaluation under
$\mathrm{ev}_z$. This is a designed quotient behavior, not an
equivalence.

## 5. Camera ledger

| ID | Camera | Exact target | Reversibility target | Known loss or singularity |
|---|---|---|---|---|
| C-FLAT-STACK-1 | $\mathrm{Flat}$ | finite indexed $(x,y)$ stack | bijection | none |
| C-CX-1 | $\kappa$ | conventional complex scalar | bijection | none |
| C-MAT-1 | $\mu$ | structured real $2\times2$ operator | bijection onto $\mathcal C_2$ | none inside image |
| C-POLAR-1 | $W$ | nonzero scale/orientation state | covering; quotient bijection | turn count; zero excluded |
| C-PSD-1 | $H$ | rank-one PSD matrix | two-to-one away from zero | global sign |
| C-CONE-1 | $Q$ | future 3D quadratic cone | two-to-one away from apex | global sign |
| C-COEF-1 | $\pi_\alpha$ | one native coefficient | projection | all other coefficients |
| C-BLOCK-1 | $\Pi_S$ | finite coefficient vector | bijection only on $\mathcal N_S$ | coefficients outside $S$ |
| C-PAIR-1 | state-selected pairing | one native coefficient | projection | all other coefficients |
| C-EVAL-1 | $\mathrm{ev}_z$ | assigned scalar realization | quotient-like, generally noninjective | formal index structure |

The classifications in this ledger are proved in
`../proofs/02-camera-equivalences.md`. Derivative properties remain separate.

## 6. Derivative targets

Calculus is not part of the 1.0 core. Once native differentiation is defined,
Stage 4 should derive rather than assume:

1. $D\kappa$, expected to be the constant identity coordinate map;
2. $D\mu$, expected to be constant and linear;
3. $DW$, including its local rank and the absence of a finite preimage for
   zero;
4. $DH_z$, expected to send a perturbation $h$ to
   $h z^T+z h^T$ in column coordinates;
5. $DQ_{(x,y)}$, including rank loss at the origin;
6. derivatives of evaluation only after a topology and differentiable
   parameterization of the assignment $z$ are stated.

These expected formulas are proof targets, not current dependencies.

## 7. Stage 2 acceptance checklist

- [x] Every camera has a domain and codomain.
- [x] Every camera has a forward rule.
- [x] Candidate inverses or complete intended fibers are stated where relevant.
- [x] Singularities and branch choices are explicit.
- [x] Preserved and discarded information is explicit.
- [x] The cone is a quadratic camera, not the native foundation.
- [x] The lossless flat stack is the default; quotient cameras are explicit.
- [x] Flat multiplicative coordinates do not claim to linearize ADD.
- [x] State-selected cameras are limited to what is exactly formulated.
- [x] Non-derivative mapping obligations are proved.
- [x] Derivative obligations are proved on finite strata in
  `../proofs/03-native-derivatives.md`.

This completes the stated Stage 4 camera and finite-derivative obligations.
